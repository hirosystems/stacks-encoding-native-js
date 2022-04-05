use super::types::*;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::{Cursor, Read};

pub struct DeserializeError(pub String);

impl DeserializeError {
    pub fn as_string(self) -> String {
        self.0
    }
}

impl From<std::io::Error> for DeserializeError {
    fn from(err: std::io::Error) -> Self {
        format!("Serialization error: {:?}", err).into()
    }
}

impl From<String> for DeserializeError {
    fn from(err: String) -> Self {
        DeserializeError(err)
    }
}

impl From<&str> for DeserializeError {
    fn from(err: &str) -> Self {
        DeserializeError(err.to_string())
    }
}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! define_u8_enum {
    ($Name:ident { $($Variant:ident = $Val:literal),+ }) =>
    {
        #[derive(Debug, Clone, PartialEq)]
        #[repr(u8)]
        pub enum $Name {
            $($Variant = $Val),*,
        }
        impl $Name {
            // pub const ALL: &'static [$Name] = &[$($Name::$Variant),*];

            pub fn to_u8(&self) -> u8 {
                match self {
                    $(
                        $Name::$Variant => $Val,
                    )*
                }
            }

            pub fn from_u8(v: u8) -> Option<Self> {
                match v {
                    $(
                        v if v == $Name::$Variant as u8 => Some($Name::$Variant),
                    )*
                    _ => None
                }
            }
        }
    }
}

define_u8_enum!(TypePrefix {
    Int = 0,
    UInt = 1,
    Buffer = 2,
    BoolTrue = 3,
    BoolFalse = 4,
    PrincipalStandard = 5,
    PrincipalContract = 6,
    ResponseOk = 7,
    ResponseErr = 8,
    OptionalNone = 9,
    OptionalSome = 10,
    List = 11,
    Tuple = 12,
    StringASCII = 13,
    StringUTF8 = 14
});

macro_rules! serialize_guarded_string {
    ($Name:ident) => {
        impl $Name {
            /*
            fn serialize_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
                w.write_all(&self.len().to_be_bytes())?;
                w.write_all(self.as_str().as_bytes())
            }
            */

            fn deserialize_read<R: Read>(r: &mut R) -> Result<Self, DeserializeError> {
                let mut len = [0; 1];
                r.read_exact(&mut len)?;
                let len = u8::from_be_bytes(len);
                if len > MAX_STRING_LEN {
                    return Err("String too long".into());
                }

                let mut data = vec![0; len as usize];
                r.read_exact(&mut data)?;

                String::from_utf8(data)
                    .map_err(|_| "Non-UTF8 string data".into())
                    .and_then(|x| $Name::try_from(x).map_err(|_| "Illegal Clarity string".into()))
            }
        }
    };
}

serialize_guarded_string!(ClarityName);
serialize_guarded_string!(ContractName);

impl StandardPrincipalData {
    /*
    fn serialize_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&[self.0])?;
        w.write_all(&self.1)
    }
    */

    fn deserialize_read<R: Read>(r: &mut R) -> Result<Self, DeserializeError> {
        let mut version = [0; 1];
        let mut data = [0; 20];
        r.read_exact(&mut version)?;
        r.read_exact(&mut data)?;
        Ok(StandardPrincipalData(version[0], data))
    }
}

#[allow(dead_code)]
fn get_cursor_slice<'a>(start_position: usize, cursor: &'a Cursor<&[u8]>) -> &'a [u8] {
    let cur_position = cursor.position() as usize;
    let inner = cursor.get_ref();
    &inner[start_position..cur_position]
}

impl Value {
    pub fn deserialize_read(r: &[u8]) -> Result<ClarityValue, DeserializeError> {
        // let mut cur = Cursor::new(r);
        // Value::inner_deserialize_read(&mut cur, 0)
        Value::inner_deserialize_read(r, 0)
    }

    fn inner_deserialize_read(
        // r: &'a mut Cursor<&[u8]>,
        // r: &mut Cursor<&[u8]>,
        buffer: &[u8],
        depth: u8,
    ) -> Result<ClarityValue, DeserializeError> {
        use super::types::Value::*;

        if depth >= 16 {
            return Err(format!("TypeSignatureTooDeep: {}", depth).into());
        }

        let mut r = Cursor::new(buffer);

        let cursor_start = r.position() as usize;

        let mut header = [0];
        r.read_exact(&mut header)?;

        let prefix = TypePrefix::from_u8(header[0]).ok_or_else(|| "Bad type prefix")?;

        match prefix {
            TypePrefix::Int => {
                let mut int_buffer = [0; 16];
                r.read_exact(&mut int_buffer)?;
                let val = Int(i128::from_be_bytes(int_buffer));
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, val))
            }
            TypePrefix::UInt => {
                let mut int_buffer = [0; 16];
                r.read_exact(&mut int_buffer)?;
                let val = UInt(u128::from_be_bytes(int_buffer));
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, val))
            }
            TypePrefix::Buffer => {
                let mut buffer_len = [0; 4];
                r.read_exact(&mut buffer_len)?;
                let buffer_len = u32::from_be_bytes(buffer_len);
                if buffer_len > MAX_VALUE_SIZE {
                    return Err("Illegal buffer type size".into());
                }
                let mut data = vec![0; buffer_len as usize];
                r.read_exact(&mut data[..])?;
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::buff(data)))
            }
            TypePrefix::BoolTrue => {
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Bool(true)))
            }
            TypePrefix::BoolFalse => {
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Bool(false)))
            }
            TypePrefix::PrincipalStandard => {
                let principal = StandardPrincipalData::deserialize_read(&mut r)?;
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::PrincipalStandard(principal)))
            }
            TypePrefix::PrincipalContract => {
                let issuer = StandardPrincipalData::deserialize_read(&mut r)?;
                let name = ContractName::deserialize_read(&mut r)?;
                let val = Value::PrincipalContract(QualifiedContractIdentifier { issuer, name });
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, val))
            }
            TypePrefix::ResponseOk => {
                let remaining = &r.get_ref()[r.position() as usize..];
                let value = Value::inner_deserialize_read(remaining, depth + 1)?;
                r.set_position(r.position() + value.serialized_bytes.len() as u64);
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::okay(value)))
            }
            TypePrefix::ResponseErr => {
                let remaining = &r.get_ref()[r.position() as usize..];
                let value = Value::inner_deserialize_read(remaining, depth + 1)?;
                r.set_position(r.position() + value.serialized_bytes.len() as u64);
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::error(value)))
            }
            TypePrefix::OptionalNone => {
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::none()))
            }
            TypePrefix::OptionalSome => {
                let remaining = &r.get_ref()[r.position() as usize..];
                let value = Value::inner_deserialize_read(remaining, depth + 1)?;
                r.set_position(r.position() + value.serialized_bytes.len() as u64);
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::some(value)))
            }
            TypePrefix::List => {
                let mut len = [0; 4];
                r.read_exact(&mut len)?;
                let len = u32::from_be_bytes(len);
                if len > MAX_VALUE_SIZE {
                    return Err("Illegal list type size".into());
                }
                let mut items = Vec::with_capacity(len as usize);
                for _i in 0..len {
                    let remaining = &r.get_ref()[r.position() as usize..];
                    let value = Value::inner_deserialize_read(remaining, depth + 1)?;
                    r.set_position(r.position() + value.serialized_bytes.len() as u64);
                    items.push(value);
                }
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::list(items)))
            }
            TypePrefix::Tuple => {
                let mut len = [0; 4];
                r.read_exact(&mut len)?;
                let len = u32::from_be_bytes(len);
                if len > MAX_VALUE_SIZE {
                    return Err("Illegal tuple type size".into());
                }
                let mut data = BTreeMap::new();
                for _i in 0..len {
                    let key = ClarityName::deserialize_read(&mut r)?;
                    let remaining = &r.get_ref()[r.position() as usize..];
                    let value = Value::inner_deserialize_read(remaining, depth + 1)?;
                    r.set_position(r.position() + value.serialized_bytes.len() as u64);
                    data.insert(key, value);
                }
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::tuple(data)))
            }
            TypePrefix::StringASCII => {
                let mut buffer_len = [0; 4];
                r.read_exact(&mut buffer_len)?;
                let buffer_len = u32::from_be_bytes(buffer_len);
                if buffer_len > MAX_VALUE_SIZE {
                    return Err("Illegal string-ascii type size".into());
                }
                let mut data = vec![0; buffer_len as usize];
                r.read_exact(&mut data[..])?;
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::string_ascii(data)))
            }
            TypePrefix::StringUTF8 => {
                let mut total_len = [0; 4];
                r.read_exact(&mut total_len)?;
                let total_len = u32::from_be_bytes(total_len);
                if total_len > MAX_VALUE_SIZE {
                    return Err("Illegal string-utf8 type size".into());
                }
                let mut data: Vec<u8> = vec![0; total_len as usize];
                r.read_exact(&mut data[..])?;
                let bytes = &buffer[cursor_start..r.position() as usize];
                Ok(ClarityValue::new(bytes, Value::string_utf8(data)))
            }
        }
    }
}
