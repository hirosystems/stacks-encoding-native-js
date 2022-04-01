use super::types::*;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::Read;

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

impl Value {
    pub fn deserialize_read<R: Read>(r: &mut R) -> Result<Value, DeserializeError> {
        Value::inner_deserialize_read(r, 0)
    }

    fn inner_deserialize_read<R: Read>(r: &mut R, depth: u8) -> Result<Value, DeserializeError> {
        use super::types::Value::*;

        if depth >= 16 {
            return Err(format!("TypeSignatureTooDeep: {}", depth).into());
        }

        let mut header = [0];
        r.read_exact(&mut header)?;

        let prefix = TypePrefix::from_u8(header[0]).ok_or_else(|| "Bad type prefix")?;

        match prefix {
            TypePrefix::Int => {
                let mut buffer = [0; 16];
                r.read_exact(&mut buffer)?;
                Ok(Int(i128::from_be_bytes(buffer)))
            }
            TypePrefix::UInt => {
                let mut buffer = [0; 16];
                r.read_exact(&mut buffer)?;
                Ok(UInt(u128::from_be_bytes(buffer)))
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
                Ok(Value::buff(data))
            }
            TypePrefix::BoolTrue => Ok(Bool(true)),
            TypePrefix::BoolFalse => Ok(Bool(false)),
            TypePrefix::PrincipalStandard => {
                let principal = StandardPrincipalData::deserialize_read(r)?;
                Ok(Value::PrincipalStandard(principal))
            }
            TypePrefix::PrincipalContract => {
                let issuer = StandardPrincipalData::deserialize_read(r)?;
                let name = ContractName::deserialize_read(r)?;
                Ok(Value::PrincipalContract(QualifiedContractIdentifier {
                    issuer,
                    name,
                }))
            }
            TypePrefix::ResponseOk | TypePrefix::ResponseErr => {
                let committed = prefix == TypePrefix::ResponseOk;
                let data = Value::inner_deserialize_read(r, depth + 1)?;
                let value = if committed {
                    Value::okay(data)
                } else {
                    Value::error(data)
                };
                Ok(value)
            }
            TypePrefix::OptionalNone => Ok(Value::none()),
            TypePrefix::OptionalSome => {
                let value = Value::some(Value::inner_deserialize_read(r, depth + 1)?);
                Ok(value)
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
                    items.push(Value::inner_deserialize_read(r, depth + 1)?);
                }
                Ok(Value::list(items))
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
                    let key = ClarityName::deserialize_read(r)?;
                    let value = Value::inner_deserialize_read(r, depth + 1)?;
                    data.insert(key, value);
                }
                Ok(Value::tuple(data))
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
                Ok(Value::string_ascii(data))
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
                Ok(Value::string_utf8(data))
            }
        }
    }
}
