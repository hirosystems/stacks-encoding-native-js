use std::{collections::BTreeMap, convert::TryFrom, fmt};

use super::types::{ClarityName, TraitIdentifier, MAX_VALUE_SIZE};

#[derive(Clone)]
pub enum TypeSignature {
    NoType,
    IntType,
    UIntType,
    BoolType,
    SequenceType(SequenceSubtype),
    PrincipalType,
    TupleType(TupleTypeSignature),
    OptionalType(Box<TypeSignature>),
    ResponseType(Box<(TypeSignature, TypeSignature)>),
    TraitReferenceType(TraitIdentifier),
}

/*
impl fmt::Display for TypeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeSignature::*;
        match self {
            NoType => write!(f, "UnknownType"),
            IntType => write!(f, "int"),
            UIntType => write!(f, "uint"),
            BoolType => write!(f, "bool"),
            OptionalType(t) => write!(f, "(optional {})", t),
            ResponseType(v) => write!(f, "(response {} {})", v.0, v.1),
            TupleType(t) => write!(f, "{}", t),
            PrincipalType => write!(f, "principal"),
            SequenceType(SequenceSubtype::BufferType(len)) => write!(f, "(buff {})", len.0),
            SequenceType(SequenceSubtype::ListType(list_type_data)) => write!(
                f,
                "(list {} {})",
                list_type_data.max_len, list_type_data.entry_type
            ),
            SequenceType(SequenceSubtype::StringType(StringSubtype::ASCII(len))) => {
                write!(f, "(string-ascii {})", len.0)
            }
            SequenceType(SequenceSubtype::StringType(StringSubtype::UTF8(len))) => {
                write!(f, "(string-utf8 {})", len.0)
            }
            TraitReferenceType(trait_alias) => write!(f, "<{}>", trait_alias.to_string()),
        }
    }
}
*/

#[derive(Clone)]
pub struct TupleTypeSignature {
    type_map: BTreeMap<ClarityName, TypeSignature>,
}

#[derive(Clone)]
pub enum SequenceSubtype {
    BufferType(BufferLength),
    ListType(ListTypeData),
    StringType(StringSubtype),
}

#[derive(Clone)]
pub struct BufferLength(pub u32);

impl TryFrom<u32> for BufferLength {
    type Error = String;
    fn try_from(data: u32) -> Result<BufferLength, String> {
        if data > MAX_VALUE_SIZE {
            Err("Value too large".into())
        } else {
            Ok(BufferLength(data))
        }
    }
}

impl From<BufferLength> for u32 {
    fn from(v: BufferLength) -> u32 {
        v.0
    }
}

impl TryFrom<usize> for BufferLength {
    type Error = String;
    fn try_from(data: usize) -> Result<BufferLength, String> {
        if data > (MAX_VALUE_SIZE as usize) {
            Err("Value too large".into())
        } else {
            Ok(BufferLength(data as u32))
        }
    }
}

#[derive(Clone)]
pub struct ListTypeData {
    max_len: u32,
    entry_type: Box<TypeSignature>,
}

#[derive(Clone)]
pub enum StringSubtype {
    ASCII(BufferLength),
    UTF8(StringUTF8Length),
}

#[derive(Clone)]
pub struct StringUTF8Length(u32);
