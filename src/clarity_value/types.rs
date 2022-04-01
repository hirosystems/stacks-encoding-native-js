use blockstack_lib::address::c32::c32_address;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::{borrow::Cow, collections::BTreeMap};

use crate::hex::{encode_hex, encode_hex_no_prefix};

use super::serder::TypePrefix;
use super::signatures::{BufferLength, TypeSignature};

pub const MAX_STRING_LEN: u8 = 128;
pub const MAX_VALUE_SIZE: u32 = 1024 * 1024; // 1MB
                                             // this is the charged size for wrapped values, i.e., response or optionals
pub const WRAPPER_VALUE_SIZE: u32 = 1;
pub const MAX_TYPE_DEPTH: u8 = 32;

// #[derive(Clone)]
#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Int(i128),
    UInt(u128),
    Bool(bool),
    Sequence(SequenceData),
    Principal(PrincipalData),
    Tuple(TupleData),
    Optional(OptionalData),
    Response(ResponseData),
}

/*
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Int(l0), Int(r0)) => l0 == r0,
            (UInt(l0), UInt(r0)) => l0 == r0,
            (Bool(l0), Bool(r0)) => l0 == r0,
            (Sequence(SequenceData::Buffer(l0)), Sequence(SequenceData::Buffer(r0))) => l0.data == r0.data,
            (Sequence(SequenceData::List(l0)), Sequence(SequenceData::List(r0))) => l0.data == r0.data,
            (Sequence(SequenceData::String(CharType::UTF8(l0))), Sequence(SequenceData::String(CharType::UTF8(r0)))) => l0.data == r0.data,
            (Sequence(SequenceData::String(CharType::ASCII(l0))), Sequence(SequenceData::String(CharType::ASCII(r0)))) => l0.data == r0.data,
            (Sequence(l0), Sequence(r0)) => l0 == r0,
            (Principal(l0), Principal(r0)) => l0 == r0,
            (Tuple(l0), Tuple(r0)) => l0 == r0,
            (Optional(l0), Optional(r0)) => l0 == r0,
            (Response(l0), Response(r0)) => l0 == r0,
            _ => false
        }
    }
}

impl Eq for Value {}
*/

pub const NONE: Value = Value::Optional(OptionalData { data: None });

impl Value {
    pub fn buff_from(buff_data: Vec<u8>) -> Value {
        Value::Sequence(SequenceData::Buffer(BuffData { data: buff_data }))
    }

    pub fn okay(data: Value) -> Value {
        Value::Response(ResponseData {
            committed: true,
            data: Box::new(data),
        })
    }

    pub fn error(data: Value) -> Value {
        Value::Response(ResponseData {
            committed: false,
            data: Box::new(data),
        })
    }

    pub fn some(data: Value) -> Value {
        Value::Optional(OptionalData {
            data: Some(Box::new(data)),
        })
    }

    pub fn none() -> Value {
        NONE.clone()
    }

    pub fn list_from(list_data: Vec<Value>) -> Value {
        Value::Sequence(SequenceData::List(ListData { data: list_data }))
    }

    pub fn tuple_from_data(mut data: Vec<(ClarityName, Value)>) -> Value {
        let mut data_map = BTreeMap::new();
        for (name, value) in data.drain(..) {
            data_map.insert(name, value);
        }
        Value::Tuple(TupleData { data_map })
    }

    pub fn string_ascii_from_bytes(bytes: Vec<u8>) -> Value {
        Value::Sequence(SequenceData::String(CharType::ASCII(ASCIIData {
            data: bytes,
        })))
    }

    pub fn string_utf8_from_bytes(bytes: Vec<u8>) -> Value {
        let validated_utf8_str = String::from_utf8_lossy(&bytes);
        let mut data = vec![];
        for char in validated_utf8_str.chars() {
            let mut encoded_char: Vec<u8> = vec![0; char.len_utf8()];
            char.encode_utf8(&mut encoded_char[..]);
            data.push(encoded_char);
        }

        Value::Sequence(SequenceData::String(CharType::UTF8(UTF8Data { data })))
    }

    pub fn type_prefix(&self) -> TypePrefix {
        use Value::*;
        match self {
            Int(_) => TypePrefix::Int,
            UInt(_) => TypePrefix::UInt,
            Bool(true) => TypePrefix::BoolTrue,
            Bool(false) => TypePrefix::BoolFalse,
            Principal(PrincipalData::Standard(_)) => TypePrefix::PrincipalStandard,
            Principal(PrincipalData::Contract(_)) => TypePrefix::PrincipalContract,
            Response(ResponseData {
                committed: true,
                data: _,
            }) => TypePrefix::ResponseOk,
            Response(ResponseData {
                committed: false,
                data: _,
            }) => TypePrefix::ResponseErr,
            Optional(OptionalData { data: None }) => TypePrefix::OptionalNone,
            Optional(OptionalData { data: Some(_) }) => TypePrefix::OptionalSome,
            Tuple(_) => TypePrefix::Tuple,
            Sequence(SequenceData::Buffer(_)) => TypePrefix::Buffer,
            Sequence(SequenceData::List(_)) => TypePrefix::List,
            Sequence(SequenceData::String(CharType::ASCII(_))) => TypePrefix::StringASCII,
            Sequence(SequenceData::String(CharType::UTF8(_))) => TypePrefix::StringUTF8,
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
            Optional(OptionalData { data: Some(value) }) => {
                write!(w, "(some ")?;
                Value::repr_string_to_buffer(value, w)?;
                write!(w, ")")
            }
            Optional(OptionalData { data: None }) => write!(w, "none"),
            Response(ResponseData {
                committed: true,
                data,
            }) => {
                write!(w, "(ok ")?;
                Value::repr_string_to_buffer(data, w)?;
                write!(w, ")")
            }
            Response(ResponseData {
                committed: false,
                data,
            }) => {
                write!(w, "(err ")?;
                Value::repr_string_to_buffer(data, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.data_map.iter() {
                    write!(w, " ({} ", name)?;
                    Value::repr_string_to_buffer(value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            Principal(PrincipalData::Standard(data)) => {
                write!(w, "'{}", c32_address(data.0, &data.1).unwrap())
            }
            Principal(PrincipalData::Contract(data)) => {
                write!(
                    w,
                    "'{}.{}",
                    c32_address(data.issuer.0, &data.issuer.1).unwrap(),
                    data.name
                )
            }
            Sequence(SequenceData::Buffer(value)) => {
                write!(w, "{}", encode_hex(&value.data))
            }
            Sequence(SequenceData::List(value)) => {
                write!(w, "(list")?;
                for val in &value.data {
                    write!(w, " ")?;
                    Value::repr_string_to_buffer(val, w)?;
                }
                write!(w, ")")
            }
            Sequence(SequenceData::String(CharType::ASCII(data))) => {
                write!(w, "\"")?;
                for c in data.data.iter() {
                    write!(w, "{}", std::ascii::escape_default(*c))?;
                }
                write!(w, "\"")
            }
            Sequence(SequenceData::String(CharType::UTF8(data))) => {
                write!(w, "u\"")?;
                for c in data.data.iter() {
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
            Optional(OptionalData { data: Some(value) }) => {
                write!(w, "(optional ")?;
                Value::type_signature_to_buffer(value, w)?;
                write!(w, ")")
            }
            Optional(OptionalData { data: None }) => write!(w, "(optional UnknownType)"),
            Response(ResponseData {
                committed: true,
                data,
            }) => {
                write!(w, "(response ")?;
                Value::type_signature_to_buffer(data, w)?;
                write!(w, " UnknownType)")
            }
            Response(ResponseData {
                committed: false,
                data,
            }) => {
                write!(w, "(response UnknownType ")?;
                Value::type_signature_to_buffer(data, w)?;
                write!(w, ")")
            }
            Tuple(data) => {
                write!(w, "(tuple")?;
                for (name, value) in data.data_map.iter() {
                    write!(w, " ({} ", name)?;
                    Value::type_signature_to_buffer(value, w)?;
                    write!(w, ")")?;
                }
                write!(w, ")")
            }
            Principal(_) => write!(w, "principal"),
            Sequence(SequenceData::Buffer(value)) => write!(w, "(buff {})", value.data.len()),
            Sequence(SequenceData::List(value)) => {
                write!(w, "(list {} ", value.data.len())?;
                if value.data.len() > 0 {
                    // TODO: this should use the least common supertype
                    Value::type_signature_to_buffer(&value.data[0], w)?;
                } else {
                    write!(w, "UnknownType")?;
                }
                write!(w, ")")
            }
            Sequence(SequenceData::String(CharType::ASCII(data))) => {
                write!(w, "(string-ascii {})", data.data.len())
            }
            Sequence(SequenceData::String(CharType::UTF8(data))) => {
                write!(w, "(string-utf8 {})", data.data.len() * 4)
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct OptionalData {
    pub data: Option<Box<Value>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ResponseData {
    pub committed: bool,
    pub data: Box<Value>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct TupleData {
    pub data_map: BTreeMap<ClarityName, Value>,
}

#[derive(Clone)]
pub struct TupleTypeSignature {
    pub type_map: BTreeMap<ClarityName, TypeSignature>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum SequenceData {
    Buffer(BuffData),
    List(ListData),
    String(CharType),
}

#[derive(Clone, Eq, PartialEq)]
pub enum CharType {
    UTF8(UTF8Data),
    ASCII(ASCIIData),
}

#[derive(Clone, Eq, PartialEq)]
pub struct UTF8Data {
    pub data: Vec<Vec<u8>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ASCIIData {
    pub data: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ListData {
    pub data: Vec<Value>,
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

#[derive(Clone)]
pub struct ListTypeData {
    max_len: u32,
    entry_type: Box<TypeSignature>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct BuffData {
    pub data: Vec<u8>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum PrincipalData {
    Standard(StandardPrincipalData),
    Contract(QualifiedContractIdentifier),
}

#[derive(Clone, Eq, PartialEq)]
pub struct StandardPrincipalData(pub u8, pub [u8; 20]);

#[derive(Clone, Eq, PartialEq)]
pub struct QualifiedContractIdentifier {
    pub issuer: StandardPrincipalData,
    pub name: ContractName,
}

#[derive(Clone)]
pub struct TraitIdentifier {
    pub name: ClarityName,
    pub contract_identifier: QualifiedContractIdentifier,
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
