use blockstack_lib::address::c32::c32_address;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use std::ops::Deref;

use crate::hex::{encode_hex, encode_hex_no_prefix};

use super::serder::TypePrefix;

pub const MAX_STRING_LEN: u8 = 128;
pub const MAX_VALUE_SIZE: u32 = 1024 * 1024; // 1MB
                                             // this is the charged size for wrapped values, i.e., response or optionals

// #[derive(Clone)]
#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Int(i128),
    UInt(u128),
    Bool(bool),
    Buffer(Vec<u8>),
    List(Vec<Value>),
    StringUTF8(Vec<Vec<u8>>),
    StringASCII(Vec<u8>),
    PrincipalStandard(StandardPrincipalData),
    PrincipalContract(QualifiedContractIdentifier),
    Tuple(BTreeMap<ClarityName, Value>),
    OptionalSome(Box<Value>),
    OptionalNone,
    ResponseOk(Box<Value>),
    ResponseErr(Box<Value>),
}

impl Value {
    pub fn buff(buff_data: Vec<u8>) -> Value {
        Value::Buffer(buff_data)
    }

    pub fn okay(data: Value) -> Value {
        Value::ResponseOk(Box::new(data))
    }

    pub fn error(data: Value) -> Value {
        Value::ResponseErr(Box::new(data))
    }

    pub fn some(data: Value) -> Value {
        Value::OptionalSome(Box::new(data))
    }

    pub fn none() -> Value {
        Value::OptionalNone
    }

    pub fn list(list_data: Vec<Value>) -> Value {
        Value::List(list_data)
    }

    pub fn tuple(data: BTreeMap<ClarityName, Value>) -> Value {
        Value::Tuple(data)
    }

    pub fn string_ascii(bytes: Vec<u8>) -> Value {
        Value::StringASCII(bytes)
    }

    pub fn string_utf8(bytes: Vec<u8>) -> Value {
        let validated_utf8_str = String::from_utf8_lossy(&bytes);
        let mut data = vec![];
        for char in validated_utf8_str.chars() {
            let mut encoded_char: Vec<u8> = vec![0; char.len_utf8()];
            char.encode_utf8(&mut encoded_char[..]);
            data.push(encoded_char);
        }
        Value::StringUTF8(data)
    }

    pub fn type_prefix(&self) -> TypePrefix {
        use Value::*;
        match self {
            Int(_) => TypePrefix::Int,
            UInt(_) => TypePrefix::UInt,
            Bool(true) => TypePrefix::BoolTrue,
            Bool(false) => TypePrefix::BoolFalse,
            PrincipalStandard(_) => TypePrefix::PrincipalStandard,
            PrincipalContract(_) => TypePrefix::PrincipalContract,
            ResponseOk(_) => TypePrefix::ResponseOk,
            ResponseErr(_) => TypePrefix::ResponseErr,
            OptionalSome(_) => TypePrefix::OptionalSome,
            OptionalNone => TypePrefix::OptionalNone,
            Tuple(_) => TypePrefix::Tuple,
            Buffer(_) => TypePrefix::Buffer,
            List(_) => TypePrefix::List,
            StringASCII(_) => TypePrefix::StringASCII,
            StringUTF8(_) => TypePrefix::StringUTF8,
        }
    }

    pub fn repr_string(&self) -> String {
        let mut w: Vec<u8> = Vec::new();
        Value::repr_string_to_buffer(self, &mut w).unwrap();
        let string_result = unsafe { String::from_utf8_unchecked(w) };
        string_result
    }

    fn repr_string_to_buffer(val: &Value, w: &mut Vec<u8>) -> std::io::Result<()> {
        use Value::*;
        match val {
            Int(data) => write!(w, "{}", data),
            UInt(data) => write!(w, "u{}", data),
            Bool(data) => write!(w, "{}", data),
            OptionalSome(value) => {
                write!(w, "(some ")?;
                Value::repr_string_to_buffer(value, w)?;
                write!(w, ")")
            }
            OptionalNone => write!(w, "none"),
            ResponseOk(data) => {
                write!(w, "(ok ")?;
                Value::repr_string_to_buffer(data, w)?;
                write!(w, ")")
            }
            ResponseErr(data) => {
                write!(w, "(err ")?;
                Value::repr_string_to_buffer(data, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.iter() {
                    write!(w, " ({} ", name)?;
                    Value::repr_string_to_buffer(value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            PrincipalStandard(data) => {
                write!(w, "'{}", c32_address(data.0, &data.1).unwrap())
            }
            PrincipalContract(data) => {
                write!(
                    w,
                    "'{}.{}",
                    c32_address(data.issuer.0, &data.issuer.1).unwrap(),
                    data.name
                )
            }
            Buffer(value) => {
                write!(w, "{}", encode_hex(value))
            }
            List(value) => {
                write!(w, "(list")?;
                for val in value {
                    write!(w, " ")?;
                    Value::repr_string_to_buffer(val, w)?;
                }
                write!(w, ")")
            }
            StringASCII(data) => {
                write!(w, "\"")?;
                for c in data.iter() {
                    write!(w, "{}", std::ascii::escape_default(*c))?;
                }
                write!(w, "\"")
            }
            StringUTF8(data) => {
                write!(w, "u\"")?;
                for c in data.iter() {
                    if c.len() > 1 {
                        // We escape extended charset
                        write!(w, "\\u{{{}}}", encode_hex_no_prefix(c))?;
                    } else {
                        // We render an ASCII char, escaped
                        write!(w, "{}", std::ascii::escape_default(c[0]))?;
                    }
                }
                write!(w, "\"")
            }
        }
    }

    pub fn type_signature(&self) -> String {
        let mut w: Vec<u8> = Vec::new();
        Value::type_signature_to_buffer(self, &mut w).unwrap();
        let string_result = unsafe { String::from_utf8_unchecked(w) };
        string_result
    }

    fn type_signature_to_buffer(val: &Value, w: &mut Vec<u8>) -> std::io::Result<()> {
        use Value::*;
        match val {
            Int(_) => write!(w, "int"),
            UInt(_) => write!(w, "uint"),
            Bool(_) => write!(w, "bool"),
            OptionalSome(value) => {
                write!(w, "(optional ")?;
                Value::type_signature_to_buffer(value, w)?;
                write!(w, ")")
            }
            OptionalNone => write!(w, "(optional UnknownType)"),
            ResponseOk(data) => {
                write!(w, "(response ")?;
                Value::type_signature_to_buffer(data, w)?;
                write!(w, " UnknownType)")
            }
            ResponseErr(data) => {
                write!(w, "(response UnknownType ")?;
                Value::type_signature_to_buffer(data, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.iter() {
                    write!(w, " ({} ", name)?;
                    Value::type_signature_to_buffer(value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            PrincipalStandard(_) | PrincipalContract(_) => write!(w, "principal"),
            Buffer(value) => write!(w, "(buff {})", value.len()),
            List(value) => {
                write!(w, "(list {} ", value.len())?;
                if value.len() > 0 {
                    // TODO: this should use the least common supertype
                    Value::type_signature_to_buffer(&value[0], w)?;
                } else {
                    write!(w, "UnknownType")?;
                }
                write!(w, ")")
            }
            StringASCII(data) => {
                write!(w, "(string-ascii {})", data.len())
            }
            StringUTF8(data) => {
                write!(w, "(string-utf8 {})", data.len() * 4)
            }
        }
    }
}

/*
impl ListData {
    pub fn list_type(&self) -> Result<Option<&Value>, String> {
        let children_len = self.data.len();
        if let Some((first, rest)) = self.data.split_first() {
            let mut current_entry_type = Some(first);
            for next_entry in rest {
                if let Some(current_value) = current_entry_type {
                    current_entry_type = Self::least_supertype(current_value, next_entry)?;
                }
            }
            Ok(current_entry_type)
        } else {
            Ok(None)
        }
    }

    fn least_supertype<'a>(a: &'a Value, b: &'a Value) -> Result<Cow<'a, Value>, String> {
        use Value::*;
        match (a, b) {
            (
                a @ Tuple(TupleData { data_map: types_a }),
                b @ Tuple(TupleData { data_map: types_b }),
            ) => {
                if !types_a.keys().eq(types_b.keys()) {
                    return Err(format!("Could not find least supertype with mismatched tuple keys: {} vs {}", types_a.keys().collect(), types_b.keys().collect()));
                }
                if a == b {
                    return Ok(Cow::Borrowed(a));
                }
                let mut type_map_out: BTreeMap<ClarityName, Value> = BTreeMap::new();
                for (name, entry_a) in types_a.into_iter() {
                    let entry_b = types_b.get(name).unwrap();
                    let tuple_least_supertype = Self::least_supertype(entry_a, entry_b)?;
                    type_map_out.insert(name.clone(), tuple_least_supertype.into_owned());
                }
                Ok(Cow::Owned(Tuple(TupleData { data_map: type_map_out })))
            }
            (
                a @ Sequence(SequenceData::List(ListData { data: data_a })),
                b @ Sequence(SequenceData::List(ListData { data: data_b })),
            ) => {
                if data_a.len() == 0 && data_b.len() == 0 {
                    Ok(Cow::Borrowed(a))
                } else if data_a.len() == 0 {
                    // TODO: return least_supertype(data_b)
                    Ok(Some(data_b[0]))
                } else if data_b.len() == 0 {
                    // TODO: return least_supertype(data_a)
                    Ok(Some(data_a[0]))
                } else {
                    Self::least_supertype(*data_a, *data_b)
                }
            }
            (Response(resp_a), Response(resp_b)) => {
                let ok_type = {
                    if !resp_a.committed && resp_b.committed {
                        Ok(Some(&(*resp_b.data)))
                    } else if !resp_a.committed && resp_b.committed {
                        Ok(Some(&(*resp_a.data)))
                    } else if resp_a.committed && resp_b.committed {
                        Self::least_supertype(&resp_a.data, &resp_b.data)
                    } else {
                        Ok(None)
                    }
                }?;

                let err_type = {
                    if resp_a.committed && !resp_b.committed {
                        Ok(Some(&(*resp_b.data)))
                    } else if resp_a.committed && !resp_b.committed {
                        Ok(Some(&(*resp_a.data)))
                    } else if !resp_a.committed && !resp_b.committed {
                        Self::least_supertype(&resp_a.data, &resp_b.data)
                    } else {
                        Ok(None)
                    }
                }?;

                Ok(match (ok_type, err_type) {
                    (None, None) => None,
                    (None, Some(err)) => Some(err),
                    (Some(val), None) => Some(val),
                    (Some(val), Some(_)) => Some(val),
                })
            }
            (a @ Optional(some_a), b @ Optional(some_b)) => {
                match (some_a.data, some_b.data) {
                    (None, None) => Ok(None),
                    (None, Some(_)) => Ok(Some(b)),
                    (Some(_), None) => Ok(Some(a)),
                    (Some(_), Some(_)) => Self::least_supertype(&a, &b)
                }
            },
            (a @ Sequence(SequenceData::Buffer(buff_a)), b @ Sequence(SequenceData::Buffer(buff_b))) => {
                if buff_a.data.len() > buff_b.data.len() {
                    Ok(Some(a))
                } else {
                    Ok(Some(b))
                }
            },
            (a @ Sequence(SequenceData::String(CharType::ASCII(string_a))), b @ Sequence(SequenceData::String(CharType::ASCII(string_b)))) => {
                if string_a.data.len() > string_b.data.len() {
                    Ok(Some(a))
                } else {
                    Ok(Some(b))
                }
            }
            (a @ Sequence(SequenceData::String(CharType::UTF8(string_a))), b @ Sequence(SequenceData::String(CharType::UTF8(string_b)))) => {
                if string_a.data.len() > string_b.data.len() {
                    Ok(Some(a))
                } else {
                    Ok(Some(b))
                }
            },
            (NoType, x) | (x, NoType) => Ok(Some(x)),
            (x, y) => {
                if x == y {
                    Ok(Some(x))
                } else {
                    Err("Type mismatch".to_string())
                }
            }
        }
    }
}
*/

#[derive(Clone, Eq, PartialEq)]
pub struct StandardPrincipalData(pub u8, pub [u8; 20]);

#[derive(Clone, Eq, PartialEq)]
pub struct QualifiedContractIdentifier {
    pub issuer: StandardPrincipalData,
    pub name: ContractName,
}

#[macro_export]
macro_rules! guarded_string {
    ($Name:ident, $Label:literal, $Regex:expr) => {
        #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $Name(String);
        impl TryFrom<String> for $Name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.len() > (MAX_STRING_LEN as usize) {
                    return Err(format!("Bad name value {}, {}", $Label, value));
                }
                if $Regex.is_match(&value) {
                    Ok(Self(value))
                } else {
                    Err(format!("Bad name value {}, {}", $Label, value))
                }
            }
        }

        impl Clone for $Name {
            fn clone(&self) -> Self {
                Self(self.0.to_string())
            }
        }

        impl $Name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Deref for $Name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Borrow<str> for $Name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl Into<String> for $Name {
            fn into(self) -> String {
                self.0
            }
        }

        impl From<&'_ str> for $Name {
            fn from(value: &str) -> Self {
                Self::try_from(value.to_string()).unwrap()
            }
        }

        impl fmt::Display for $Name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

lazy_static! {
    pub static ref CLARITY_NAME_REGEX: Regex =
        Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_!?+<>=/*])*$|^[-+=/*]$|^[<>]=?$").unwrap();
    pub static ref CONTRACT_NAME_REGEX: Regex =
        Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_])*$|^__transient$").unwrap();
}

guarded_string!(ClarityName, "ClarityName", CLARITY_NAME_REGEX);
guarded_string!(ContractName, "ContractName", CONTRACT_NAME_REGEX);
