use byteorder::{BigEndian, ReadBytesExt};
use std::{
    convert::{TryFrom, TryInto},
    io::{Cursor, Read},
};

use crate::address::stacks_address::StacksAddress;
use crate::clarity_value;
use crate::clarity_value::types::{
    ClarityName, ClarityValue, ContractName, CONTRACT_MAX_NAME_LENGTH, CONTRACT_MIN_NAME_LENGTH,
    MAX_STRING_LEN,
};

pub enum TransactionPostCondition {
    STX(PostConditionPrincipal, FungibleConditionCode, u64),
    Fungible(
        PostConditionPrincipal,
        AssetInfo,
        FungibleConditionCode,
        u64,
    ),
    Nonfungible(
        PostConditionPrincipal,
        AssetInfo,
        ClarityValue,
        NonfungibleConditionCode,
    ),
}

pub enum PostConditionPrincipal {
    Origin,
    Standard(StacksAddress),
    Contract(StacksAddress, ContractName),
}

#[repr(u8)]
pub enum PostConditionPrincipalID {
    Origin = 0x01,
    Standard = 0x02,
    Contract = 0x03,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum FungibleConditionCode {
    SentEq = 0x01,
    SentGt = 0x02,
    SentGe = 0x03,
    SentLt = 0x04,
    SentLe = 0x05,
}

impl TryFrom<u8> for FungibleConditionCode {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(FungibleConditionCode::SentEq),
            0x02 => Ok(FungibleConditionCode::SentGt),
            0x03 => Ok(FungibleConditionCode::SentGe),
            0x04 => Ok(FungibleConditionCode::SentLt),
            0x05 => Ok(FungibleConditionCode::SentLe),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum NonfungibleConditionCode {
    Sent = 0x10,
    NotSent = 0x11,
}

impl TryFrom<u8> for NonfungibleConditionCode {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x10 => Ok(NonfungibleConditionCode::Sent),
            0x11 => Ok(NonfungibleConditionCode::NotSent),
            _ => Err(()),
        }
    }
}

pub struct AssetInfo {
    pub contract_address: StacksAddress,
    pub contract_name: ContractName,
    pub asset_name: ClarityName,
}

#[repr(u8)]
pub enum AssetInfoID {
    STX = 0,
    FungibleAsset = 1,
    NonfungibleAsset = 2,
}

#[derive(Debug)]
pub struct DeserializeError {
    pub error: String,
}

impl From<String> for DeserializeError {
    fn from(error: String) -> Self {
        DeserializeError { error }
    }
}

impl From<std::io::Error> for DeserializeError {
    fn from(error: std::io::Error) -> Self {
        format!("Deserialize error: {}", error).into()
    }
}

impl TransactionPostCondition {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let asset_info_id: u8 = fd.read_u8()?;
        let postcond = match asset_info_id {
            x if x == AssetInfoID::STX as u8 => {
                let principal = PostConditionPrincipal::deserialize(fd)?;
                let condition_u8: u8 = fd.read_u8()?;
                let amount: u64 = fd.read_u64::<BigEndian>()?;

                let condition_code: FungibleConditionCode =
                    condition_u8.try_into().map_err(|_| {
                        format!("Error parsing FungibleConditionCode: {}", condition_u8)
                    })?;

                TransactionPostCondition::STX(principal, condition_code, amount)
            }
            x if x == AssetInfoID::FungibleAsset as u8 => {
                let principal = PostConditionPrincipal::deserialize(fd)?;
                let asset = AssetInfo::deserialize(fd)?;
                let condition_u8: u8 = fd.read_u8()?;
                let amount: u64 = fd.read_u64::<BigEndian>()?;

                let condition_code: FungibleConditionCode =
                    condition_u8.try_into().map_err(|_| {
                        format!("Error parsing FungibleConditionCode: {}", condition_u8)
                    })?;

                TransactionPostCondition::Fungible(principal, asset, condition_code, amount)
            }
            x if x == AssetInfoID::NonfungibleAsset as u8 => {
                let principal = PostConditionPrincipal::deserialize(fd)?;
                let asset = AssetInfo::deserialize(fd)?;
                let asset_value = {
                    let cursor_pos = fd.position();
                    let mut val = clarity_value::types::Value::deserialize_read(fd, false)
                        .map_err(|e| format!("Error deserializing Clarity value: {}", e))?;
                    let decoded_bytes = &fd.get_ref()[cursor_pos as usize..fd.position() as usize];
                    val.serialized_bytes = Some(decoded_bytes.to_vec());
                    val
                };
                let condition_u8: u8 = fd.read_u8()?;

                let condition_code: NonfungibleConditionCode =
                    condition_u8.try_into().map_err(|_| {
                        format!("Error parsing NonfungibleConditionCode: {}", condition_u8)
                    })?;

                TransactionPostCondition::Nonfungible(principal, asset, asset_value, condition_code)
            }
            _ => Err(format!(
                "Failed to parse transaction: unknown asset info ID {}",
                asset_info_id
            ))?,
        };

        Ok(postcond)
    }
}

impl PostConditionPrincipal {
    fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let principal_id: u8 = fd.read_u8()?;
        let principal = match principal_id {
            x if x == PostConditionPrincipalID::Origin as u8 => PostConditionPrincipal::Origin,
            x if x == PostConditionPrincipalID::Standard as u8 => {
                let addr = StacksAddress::deserialize(fd)?;
                PostConditionPrincipal::Standard(addr)
            }
            x if x == PostConditionPrincipalID::Contract as u8 => {
                let addr = StacksAddress::deserialize(fd)?;
                let contract_name = ContractName::deserialize(fd)?;
                PostConditionPrincipal::Contract(addr, contract_name)
            }
            _ => Err(format!(
                "Failed to parse transaction: unknown post condition principal ID {}",
                principal_id
            ))?,
        };
        Ok(principal)
    }
}

impl StacksAddress {
    fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version: u8 = fd.read_u8()?;
        let mut hash160 = [0u8; 20];
        fd.read_exact(&mut hash160)?;
        Ok(StacksAddress {
            version: version,
            hash160_bytes: hash160,
        })
    }
}

impl ContractName {
    fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len_byte: u8 = fd.read_u8()?;
        if (len_byte as usize) < CONTRACT_MIN_NAME_LENGTH
            || (len_byte as usize) > CONTRACT_MAX_NAME_LENGTH
        {
            return Err(format!(
                "Failed to deserialize contract name: too short or too long: {}",
                len_byte
            ))?;
        }
        let mut bytes = vec![0u8; len_byte as usize];
        fd.read_exact(&mut bytes)?;

        let s = String::from_utf8(bytes).map_err(|e| {
            format!(
                "Failed to parse Contract name: could not construct from utf8: {}",
                e
            )
        })?;

        let name = ContractName::try_from(s)
            .map_err(|e| format!("Failed to parse Contract name: {:?}", e))?;

        Ok(name)
    }
}

impl ClarityName {
    fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len_byte = fd.read_u8()?;
        if len_byte > MAX_STRING_LEN {
            return Err(format!(
                "Failed to deserialize clarity name: too long: {}",
                len_byte,
            ))?;
        }
        let mut bytes = vec![0u8; len_byte as usize];
        fd.read_exact(&mut bytes)?;

        let s = String::from_utf8(bytes).map_err(|e| {
            format!(
                "Failed to parse Clarity name: could not contruct from utf8: {}",
                e
            )
        })?;

        let name = ClarityName::try_from(s)
            .map_err(|e| format!("Failed to parse Clarity name: {:?}", e))?;
        Ok(name)
    }
}

impl AssetInfo {
    fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let contract_address = StacksAddress::deserialize(fd)?;
        let contract_name = ContractName::deserialize(fd)?;
        let asset_name = ClarityName::deserialize(fd)?;
        Ok(AssetInfo {
            contract_address,
            contract_name,
            asset_name,
        })
    }
}
