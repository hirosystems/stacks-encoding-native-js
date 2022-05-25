use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::address::stacks_address::StacksAddress;
use crate::clarity_value::deserialize::TypePrefix;
use crate::clarity_value::types::{ClarityName, ClarityValue};
use crate::post_condition::deserialize::TransactionPostCondition;
use crate::serialize_util::DeserializeError;

pub struct StacksTransaction {
    pub version: TransactionVersion,
    pub chain_id: u32,
    pub auth: TransactionAuth,
    pub anchor_mode: TransactionAnchorMode,
    pub post_conditions_serialized: Vec<u8>,
    pub post_condition_mode: TransactionPostConditionMode,
    pub post_conditions: Vec<TransactionPostCondition>,
    pub payload: TransactionPayload,
}

impl StacksTransaction {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version_u8 = fd.read_u8()?;
        let chain_id: u32 = fd.read_u32::<BigEndian>()?;
        let auth = TransactionAuth::deserialize(fd)?;
        let anchor_mode_u8 = fd.read_u8()?;

        let cursor_pos = fd.position() as usize;
        let post_condition_mode_u8 = fd.read_u8()?;
        let post_conditions: Vec<TransactionPostCondition> = {
            let len = fd.read_u32::<BigEndian>()?;
            let mut results: Vec<TransactionPostCondition> = Vec::with_capacity(len as usize);
            for _ in 0..len {
                results.push(TransactionPostCondition::deserialize(fd)?);
            }
            results
        };
        let post_conditions_serialized = fd.get_ref()[cursor_pos..fd.position() as usize].to_vec();

        let payload = TransactionPayload::deserialize(fd)?;

        let version = if (version_u8 & 0x80) == 0 {
            TransactionVersion::Mainnet
        } else {
            TransactionVersion::Testnet
        };

        let anchor_mode = match anchor_mode_u8 {
            x if x == TransactionAnchorMode::OffChainOnly as u8 => {
                TransactionAnchorMode::OffChainOnly
            }
            x if x == TransactionAnchorMode::OnChainOnly as u8 => {
                TransactionAnchorMode::OnChainOnly
            }
            x if x == TransactionAnchorMode::Any as u8 => TransactionAnchorMode::Any,
            _ => {
                return Err(format!(
                    "Failed to parse transaction: invalid anchor mode {}",
                    anchor_mode_u8
                ))?;
            }
        };

        let post_condition_mode = match post_condition_mode_u8 {
            x if x == TransactionPostConditionMode::Allow as u8 => {
                TransactionPostConditionMode::Allow
            }
            x if x == TransactionPostConditionMode::Deny as u8 => {
                TransactionPostConditionMode::Deny
            }
            _ => {
                return Err(format!(
                    "Failed to parse transaction: invalid post-condition mode {}",
                    post_condition_mode_u8
                ))?;
            }
        };

        Ok(StacksTransaction {
            version,
            chain_id,
            auth,
            anchor_mode,
            post_conditions_serialized,
            post_condition_mode,
            post_conditions,
            payload,
        })
    }
}

impl TransactionAuth {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let type_id = fd.read_u8()?;
        let auth = match type_id {
            x if x == TransactionAuthFlags::AuthStandard as u8 => {
                let origin_auth = TransactionSpendingCondition::deserialize(fd)?;
                TransactionAuth::Standard(origin_auth)
            }
            x if x == TransactionAuthFlags::AuthSponsored as u8 => {
                let origin_auth = TransactionSpendingCondition::deserialize(fd)?;
                let sponsor_auth = TransactionSpendingCondition::deserialize(fd)?;
                TransactionAuth::Sponsored(origin_auth, sponsor_auth)
            }
            _ => {
                return Err(format!(
                    "Failed to parse transaction authorization: unrecognized auth flags {}",
                    type_id
                ))?;
            }
        };
        Ok(auth)
    }
}

impl TransactionSpendingCondition {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let hash_mode_u8 = fd.read_u8()?;
        fd.set_position(fd.position() - 1);
        let cond = {
            if SinglesigHashMode::from_u8(hash_mode_u8).is_some() {
                let cond = SinglesigSpendingCondition::deserialize(fd)?;
                TransactionSpendingCondition::Singlesig(cond)
            } else if MultisigHashMode::from_u8(hash_mode_u8).is_some() {
                let cond = MultisigSpendingCondition::deserialize(fd)?;
                TransactionSpendingCondition::Multisig(cond)
            } else {
                return Err(format!(
                    "Failed to parse spending condition: invalid hash mode {}",
                    hash_mode_u8
                ))?;
            }
        };

        Ok(cond)
    }
}

impl SinglesigSpendingCondition {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let hash_mode_u8 = fd.read_u8()?;
        let hash_mode = SinglesigHashMode::from_u8(hash_mode_u8).ok_or(format!(
            "Failed to parse singlesig spending condition: unknown hash mode {}",
            hash_mode_u8
        ))?;

        let mut signer = [0u8; 20];
        fd.read_exact(&mut signer)?;

        let nonce = fd.read_u64::<BigEndian>()?;
        let tx_fee = fd.read_u64::<BigEndian>()?;

        let key_encoding_u8 = fd.read_u8()?;
        let key_encoding =
            TransactionPublicKeyEncoding::from_u8(key_encoding_u8).ok_or(format!(
                "Failed to parse singlesig spending condition: unknown key encoding {}",
                key_encoding_u8
            ))?;

        let mut signature_bytes = [0u8; 65];
        fd.read_exact(&mut signature_bytes)?;
        let signature = MessageSignature(signature_bytes);

        // sanity check -- must be compressed if we're using p2wpkh
        if hash_mode == SinglesigHashMode::P2WPKH
            && key_encoding != TransactionPublicKeyEncoding::Compressed
        {
            return Err(format!("Failed to parse singlesig spending condition: incomaptible hash mode and key encoding"))?;
        }

        Ok(SinglesigSpendingCondition {
            signer: signer,
            nonce: nonce,
            tx_fee: tx_fee,
            hash_mode: hash_mode,
            key_encoding: key_encoding,
            signature: signature,
        })
    }
}

impl MultisigSpendingCondition {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let hash_mode_u8 = fd.read_u8()?;
        let hash_mode = MultisigHashMode::from_u8(hash_mode_u8).ok_or(format!(
            "Failed to parse multisig spending condition: unknown hash mode {}",
            hash_mode_u8
        ))?;

        let mut signer = [0u8; 20];
        fd.read_exact(&mut signer)?;
        let nonce = fd.read_u64::<BigEndian>()?;
        let tx_fee = fd.read_u64::<BigEndian>()?;
        let fields: Vec<TransactionAuthField> = {
            let len = fd.read_u32::<BigEndian>()?;
            let mut results: Vec<TransactionAuthField> = Vec::with_capacity(len as usize);
            for _ in 0..len {
                results.push(TransactionAuthField::deserialize(fd)?);
            }
            results
        };

        let signatures_required = fd.read_u16::<BigEndian>()?;

        // read and decode _exactly_ num_signatures signature buffers
        let mut num_sigs_given: u16 = 0;
        let mut have_uncompressed = false;
        for f in fields.iter() {
            match *f {
                TransactionAuthField::Signature(ref key_encoding, _) => {
                    num_sigs_given = num_sigs_given.checked_add(1).ok_or(format!(
                        "Failed to parse multisig spending condition: too many signatures"
                    ))?;
                    if *key_encoding == TransactionPublicKeyEncoding::Uncompressed {
                        have_uncompressed = true;
                    }
                }
                TransactionAuthField::PublicKey(ref pubk) => {
                    if !pubk.compressed {
                        have_uncompressed = true;
                    }
                }
            };
        }

        // must be given the right number of signatures
        if num_sigs_given != signatures_required {
            return Err(format!(
                "Failed to parse multisig spending condition: got {} sigs, expected {}",
                num_sigs_given, signatures_required
            ))?;
        }

        // must all be compressed if we're using P2WSH
        if have_uncompressed && hash_mode == MultisigHashMode::P2WSH {
            return Err(format!(
                "Failed to parse multisig spending condition: expected compressed keys only",
            ))?;
        }

        Ok(MultisigSpendingCondition {
            signer,
            nonce,
            tx_fee,
            hash_mode,
            fields,
            signatures_required,
        })
    }
}

impl TransactionAuthField {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let field_id = fd.read_u8()?;
        let field = match field_id {
            x if x == TransactionAuthFieldID::PublicKeyCompressed as u8 => {
                let mut pubkey_bytes = [0u8; 33];
                fd.read_exact(&mut pubkey_bytes)?;
                let pubkey_buf = StacksPublicKeyBuffer(pubkey_bytes);
                TransactionAuthField::PublicKey(Secp256k1PublicKey {
                    compressed: true,
                    key: pubkey_buf,
                })
            }
            x if x == TransactionAuthFieldID::PublicKeyUncompressed as u8 => {
                let mut pubkey_bytes = [0u8; 33];
                fd.read_exact(&mut pubkey_bytes)?;
                let pubkey_buf = StacksPublicKeyBuffer(pubkey_bytes);
                TransactionAuthField::PublicKey(Secp256k1PublicKey {
                    compressed: false,
                    key: pubkey_buf,
                })
            }
            x if x == TransactionAuthFieldID::SignatureCompressed as u8 => {
                let mut sig_bytes = [0u8; 65];
                fd.read_exact(&mut sig_bytes)?;
                let sig = MessageSignature(sig_bytes);
                TransactionAuthField::Signature(TransactionPublicKeyEncoding::Compressed, sig)
            }
            x if x == TransactionAuthFieldID::SignatureUncompressed as u8 => {
                let mut sig_bytes = [0u8; 65];
                fd.read_exact(&mut sig_bytes)?;
                let sig = MessageSignature(sig_bytes);
                TransactionAuthField::Signature(TransactionPublicKeyEncoding::Uncompressed, sig)
            }
            _ => {
                return Err(format!(
                    "Failed to parse auth field: unkonwn auth field ID {}",
                    field_id
                ))?;
            }
        };
        Ok(field)
    }
}

impl SinglesigHashMode {
    pub fn from_u8(n: u8) -> Option<SinglesigHashMode> {
        match n {
            x if x == SinglesigHashMode::P2PKH as u8 => Some(SinglesigHashMode::P2PKH),
            x if x == SinglesigHashMode::P2WPKH as u8 => Some(SinglesigHashMode::P2WPKH),
            _ => None,
        }
    }
}

impl MultisigHashMode {
    pub fn from_u8(n: u8) -> Option<MultisigHashMode> {
        match n {
            x if x == MultisigHashMode::P2SH as u8 => Some(MultisigHashMode::P2SH),
            x if x == MultisigHashMode::P2WSH as u8 => Some(MultisigHashMode::P2WSH),
            _ => None,
        }
    }
}

impl TransactionPublicKeyEncoding {
    pub fn from_u8(n: u8) -> Option<TransactionPublicKeyEncoding> {
        match n {
            x if x == TransactionPublicKeyEncoding::Compressed as u8 => {
                Some(TransactionPublicKeyEncoding::Compressed)
            }
            x if x == TransactionPublicKeyEncoding::Uncompressed as u8 => {
                Some(TransactionPublicKeyEncoding::Uncompressed)
            }
            _ => None,
        }
    }
}

impl TransactionPayload {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let type_id = fd.read_u8()?;
        let payload = match type_id {
            x if x == TransactionPayloadID::TokenTransfer as u8 => {
                let principal = PrincipalData::deserialize(fd)?;
                let amount = fd.read_u64::<BigEndian>()?;
                let mut memo_bytes = [0u8; 34];
                fd.read_exact(&mut memo_bytes)?;
                let memo = TokenTransferMemo(memo_bytes);
                TransactionPayload::TokenTransfer(principal, amount, memo)
            }
            x if x == TransactionPayloadID::ContractCall as u8 => {
                let payload = TransactionContractCall::deserialize(fd)?;
                TransactionPayload::ContractCall(payload)
            }
            x if x == TransactionPayloadID::SmartContract as u8 => {
                let payload = TransactionSmartContract::deserialize(fd)?;
                TransactionPayload::SmartContract(payload)
            }
            x if x == TransactionPayloadID::PoisonMicroblock as u8 => {
                let h1 = StacksMicroblockHeader::deserialize(fd)?;
                let h2 = StacksMicroblockHeader::deserialize(fd)?;
                TransactionPayload::PoisonMicroblock(h1, h2)
            }
            x if x == TransactionPayloadID::Coinbase as u8 => {
                let mut payload_bytes = [0u8; 32];
                fd.read_exact(&mut payload_bytes)?;
                let payload = CoinbasePayload(payload_bytes);
                TransactionPayload::Coinbase(payload)
            }
            _ => {
                return Err(format!(
                    "Failed to parse transaction -- unknown payload ID {}",
                    type_id
                ))?;
            }
        };

        Ok(payload)
    }
}

impl TransactionContractCall {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let address = StacksAddress::deserialize(fd)?;
        let contract_name = ClarityName::deserialize(fd)?;
        let function_name = ClarityName::deserialize(fd)?;
        let function_args: Vec<ClarityValue> = {
            let len = fd.read_u32::<BigEndian>()?;
            let mut results: Vec<ClarityValue> = Vec::with_capacity(len as usize);
            for _ in 0..len {
                results.push(ClarityValue::deserialize(fd, true)?);
            }
            results
        };

        Ok(TransactionContractCall {
            address,
            contract_name,
            function_name,
            function_args,
        })
    }
}

impl TransactionSmartContract {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let name = ClarityName::deserialize(fd)?;
        let code_body = StacksString::deserialize(fd)?;
        Ok(TransactionSmartContract { name, code_body })
    }
}

impl StacksMicroblockHeader {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let cursor_pos = fd.position() as usize;

        let version = fd.read_u8()?;
        let sequence = fd.read_u16::<BigEndian>()?;

        let mut prev_block_bytes = [0u8; 32];
        fd.read_exact(&mut prev_block_bytes)?;
        let prev_block = BlockHeaderHash(prev_block_bytes);

        let mut tx_merkle_root_bytes = [0u8; 32];
        fd.read_exact(&mut tx_merkle_root_bytes)?;
        let tx_merkle_root = Sha512Trunc256Sum(tx_merkle_root_bytes);

        let mut signature_bytes = [0u8; 65];
        fd.read_exact(&mut signature_bytes)?;
        let signature = MessageSignature(signature_bytes);

        let serialized_bytes = fd.get_ref()[cursor_pos..fd.position() as usize].to_vec();

        Ok(StacksMicroblockHeader {
            version,
            sequence,
            prev_block,
            tx_merkle_root,
            signature,
            serialized_bytes,
        })
    }
}

impl StacksString {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len = fd.read_u32::<BigEndian>()?;
        let mut bytes: Vec<u8> = vec![0u8; len as usize];
        fd.read_exact(&mut bytes)?;
        Ok(StacksString(bytes))
    }
}

impl PrincipalData {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let mut header = [0];
        fd.read_exact(&mut header)?;

        let prefix = TypePrefix::from_u8(header[0]).ok_or_else(|| "Bad principal prefix")?;

        match prefix {
            TypePrefix::PrincipalStandard => Ok(PrincipalData::Standard(
                StandardPrincipalData::deserialize(fd)?,
            )),
            TypePrefix::PrincipalContract => {
                let issuer = StandardPrincipalData::deserialize(fd)?;
                let name = ClarityName::deserialize(fd)?;
                Ok(PrincipalData::Contract(QualifiedContractIdentifier {
                    issuer,
                    name,
                }))
            }
            _ => Err("Bad principal prefix".into()),
        }
    }
}

impl StandardPrincipalData {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let mut version = [0; 1];
        let mut data = [0; 20];
        fd.read_exact(&mut version)?;
        fd.read_exact(&mut data)?;
        Ok(StandardPrincipalData(version[0], data))
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TransactionVersion {
    Mainnet = 0x00,
    Testnet = 0x80,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum TransactionAnchorMode {
    OnChainOnly = 1,  // must be included in a StacksBlock
    OffChainOnly = 2, // must be included in a StacksMicroBlock
    Any = 3,          // either
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum TransactionPostConditionMode {
    Allow = 0x01, // allow any other changes not specified
    Deny = 0x02,  // deny any other changes not specified
}

#[repr(u8)]
pub enum TransactionAuthFlags {
    AuthStandard = 0x04,
    AuthSponsored = 0x05,
}

pub enum TransactionAuth {
    Standard(TransactionSpendingCondition),
    Sponsored(TransactionSpendingCondition, TransactionSpendingCondition), // the second account pays on behalf of the first account
}

pub enum TransactionSpendingCondition {
    Singlesig(SinglesigSpendingCondition),
    Multisig(MultisigSpendingCondition),
}

pub struct MultisigSpendingCondition {
    pub hash_mode: MultisigHashMode,
    pub signer: [u8; 20],
    pub nonce: u64,  // nth authorization from this account
    pub tx_fee: u64, // microSTX/compute rate offered by this account
    pub fields: Vec<TransactionAuthField>,
    pub signatures_required: u16,
}

pub struct SinglesigSpendingCondition {
    pub hash_mode: SinglesigHashMode,
    pub signer: [u8; 20],
    pub nonce: u64,  // nth authorization from this account
    pub tx_fee: u64, // microSTX/compute rate offerred by this account
    pub key_encoding: TransactionPublicKeyEncoding,
    pub signature: MessageSignature,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum MultisigHashMode {
    P2SH = 0x01,
    P2WSH = 0x03,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum SinglesigHashMode {
    P2PKH = 0x00,
    P2WPKH = 0x02,
}

pub struct StacksPublicKeyBuffer(pub [u8; 33]);

pub struct MessageSignature(pub [u8; 65]);

pub struct Secp256k1PublicKey {
    pub key: StacksPublicKeyBuffer,
    pub compressed: bool,
}

pub enum TransactionAuthField {
    PublicKey(Secp256k1PublicKey),
    Signature(TransactionPublicKeyEncoding, MessageSignature),
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum TransactionAuthFieldID {
    // types of auth fields
    PublicKeyCompressed = 0x00,
    PublicKeyUncompressed = 0x01,
    SignatureCompressed = 0x02,
    SignatureUncompressed = 0x03,
}

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum TransactionPublicKeyEncoding {
    // ways we can encode a public key
    Compressed = 0x00,
    Uncompressed = 0x01,
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum TransactionPayloadID {
    TokenTransfer = 0,
    SmartContract = 1,
    ContractCall = 2,
    PoisonMicroblock = 3,
    Coinbase = 4,
}

pub enum TransactionPayload {
    TokenTransfer(PrincipalData, u64, TokenTransferMemo),
    ContractCall(TransactionContractCall),
    SmartContract(TransactionSmartContract),
    PoisonMicroblock(StacksMicroblockHeader, StacksMicroblockHeader),
    Coinbase(CoinbasePayload),
}

pub struct CoinbasePayload(pub [u8; 32]);

pub struct TransactionSmartContract {
    pub name: ClarityName,
    pub code_body: StacksString,
}

pub struct StacksString(pub Vec<u8>);

pub struct BlockHeaderHash(pub [u8; 32]);

pub struct Sha512Trunc256Sum(pub [u8; 32]);

pub struct StacksMicroblockHeader {
    pub version: u8,
    pub sequence: u16,
    pub prev_block: BlockHeaderHash,
    pub tx_merkle_root: Sha512Trunc256Sum,
    pub signature: MessageSignature,
    pub serialized_bytes: Vec<u8>,
}

pub struct TokenTransferMemo(pub [u8; 34]);

pub struct StandardPrincipalData(pub u8, pub [u8; 20]);

pub struct QualifiedContractIdentifier {
    pub issuer: StandardPrincipalData,
    pub name: ClarityName,
}

pub enum PrincipalData {
    Standard(StandardPrincipalData),
    Contract(QualifiedContractIdentifier),
}

pub struct TransactionContractCall {
    pub address: StacksAddress,
    pub contract_name: ClarityName,
    pub function_name: ClarityName,
    pub function_args: Vec<ClarityValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::decode_hex;

    #[test]
    fn test_decode_bug() {
        let input = b"0x00000000010400982f3ec112a5f5928a5c96a914bd733793b896a5000000000000053000000000000002290000c85889dad0d5b08a997a93a28a7c93eb22c324e5f8992dc93e37865ef4f3e0d65383beefeffc4871a2facbc4b590ddf887c80de6638ed4e2ec0e633d1e130f230301000000000216982f3ec112a5f5928a5c96a914bd733793b896a51861726b6164696b6f2d676f7665726e616e63652d76332d310770726f706f7365000000060616982f3ec112a5f5928a5c96a914bd733793b896a51d61726b6164696b6f2d7374616b652d706f6f6c2d64696b6f2d76312d32010000000000000000000000000000ef8801000000000000000000000000000003f00e00000028414950313020557064617465204c54567320616e64204c69717569646174696f6e20526174696f730e0000003168747470733a2f2f6769746875622e636f6d2f61726b6164696b6f2d64616f2f61726b6164696b6f2f70756c6c2f3439330b000000010c0000000507616464726573730516982f3ec112a5f5928a5c96a914bd733793b896a50863616e2d6275726e040863616e2d6d696e7404046e616d650d0000002b61697031302d61726b6164696b6f2d7570646174652d74766c2d6c69717569646174696f6e2d726174696f0e7175616c69666965642d6e616d650616982f3ec112a5f5928a5c96a914bd733793b896a52b61697031302d61726b6164696b6f2d7570646174652d74766c2d6c69717569646174696f6e2d726174696f";
        let bytes = decode_hex(input).unwrap();
        let mut cursor = Cursor::new(bytes.as_ref());
        let tx = StacksTransaction::deserialize(&mut cursor);
        assert!(tx.is_ok());
    }
}
