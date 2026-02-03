//! Neon JS encoder module for converting stacks-codec types to JavaScript objects.
//!
//! This module provides conversion functions from Rust types (from stacks-codec, clarity,
//! and stacks-common crates) to Neon JavaScript objects that match the TypeScript
//! interfaces defined in index.ts.

use neon::prelude::*;
use stacks_codec::codec::{
    AssetInfo, AssetInfoID, FungibleConditionCode, MultisigHashMode,
    NakamotoBlock, NakamotoBlockHeader, NonfungibleConditionCode,
    OrderIndependentMultisigHashMode,
    PostConditionPrincipal, SinglesigHashMode, StacksMicroblockHeader,
    StacksTransaction, TransactionAuth,
    TransactionAuthField, TransactionAuthFieldID,
    TransactionPayload, TransactionPostCondition, TransactionPublicKeyEncoding,
    TransactionSpendingCondition, TransactionVersion, Txid,
};
use clarity::codec::StacksMessageCodec;
use clarity::vm::types::{PrincipalData, StandardPrincipalData, Value};
use clarity::vm::ClarityVersion;
use clarity::types::chainstate::StacksAddress;
use stacks_common::address::c32::{c32_address, c32_address_decode};
use stacks_common::util::hash::Hash160;

use crate::hex::encode_hex;

/// Trait for serializing Rust types to Neon JS objects
pub trait NeonSerialize {
    fn neon_serialize<'a>(&self, cx: &mut FunctionContext<'a>) -> JsResult<'a, JsValue>;
}

/// Encode a StacksAddress to a JS object with version, hash bytes, and c32 address string
pub fn encode_stacks_address<'a>(
    cx: &mut FunctionContext<'a>,
    address: &StacksAddress,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let version = cx.number(address.version() as f64);
    obj.set(cx, "address_version", version)?;
    
    let hash_bytes = cx.string(encode_hex(address.bytes().as_ref()));
    obj.set(cx, "address_hash_bytes", hash_bytes)?;
    
    let c32_addr = c32_address(address.version(), address.bytes().as_ref())
        .unwrap_or_else(|_| "invalid".to_string());
    let addr_str = cx.string(c32_addr);
    obj.set(cx, "address", addr_str)?;
    
    Ok(obj)
}

/// Encode a Hash160 signer to a decoded stacks address object
pub fn encode_signer_address<'a>(
    cx: &mut FunctionContext<'a>,
    version: u8,
    signer: &Hash160,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let ver = cx.number(version as f64);
    obj.set(cx, "address_version", ver)?;
    
    let hash_bytes = cx.string(encode_hex(signer.as_ref()));
    obj.set(cx, "address_hash_bytes", hash_bytes)?;
    
    let c32_addr = c32_address(version, signer.as_ref())
        .unwrap_or_else(|_| "invalid".to_string());
    let addr_str = cx.string(c32_addr);
    obj.set(cx, "address", addr_str)?;
    
    Ok(obj)
}

/// Encode a StandardPrincipalData to a JS object
pub fn encode_standard_principal<'a>(
    cx: &mut FunctionContext<'a>,
    principal: &StandardPrincipalData,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let type_id = cx.number(5_f64); // PrincipalTypeID.Standard
    obj.set(cx, "type_id", type_id)?;
    
    // Get address string and decode to get version and hash
    let addr_string = principal.to_string();
    let (version, hash160) = c32_address_decode(&addr_string)
        .unwrap_or((0, vec![0u8; 20]));
    
    let version_js = cx.number(version as f64);
    obj.set(cx, "address_version", version_js)?;
    
    let hash_bytes = cx.string(encode_hex(&hash160));
    obj.set(cx, "address_hash_bytes", hash_bytes)?;
    
    let addr_str = cx.string(&addr_string);
    obj.set(cx, "address", addr_str)?;
    
    Ok(obj)
}

/// Encode a PrincipalData to a JS object
pub fn encode_principal<'a>(
    cx: &mut FunctionContext<'a>,
    principal: &PrincipalData,
) -> JsResult<'a, JsObject> {
    match principal {
        PrincipalData::Standard(std_principal) => {
            encode_standard_principal(cx, std_principal)
        }
        PrincipalData::Contract(contract_id) => {
            let obj = cx.empty_object();
            
            let type_id = cx.number(6_f64); // PrincipalTypeID.Contract
            obj.set(cx, "type_id", type_id)?;
            
            // Get issuer address string and decode
            let issuer_addr = contract_id.issuer.to_string();
            let (version, hash160) = c32_address_decode(&issuer_addr)
                .unwrap_or((0, vec![0u8; 20]));
            
            let version_js = cx.number(version as f64);
            obj.set(cx, "address_version", version_js)?;
            
            let hash_bytes = cx.string(encode_hex(&hash160));
            obj.set(cx, "address_hash_bytes", hash_bytes)?;
            
            let addr_str = cx.string(&issuer_addr);
            obj.set(cx, "address", addr_str)?;
            
            let contract_name = cx.string(contract_id.name.as_str());
            obj.set(cx, "contract_name", contract_name)?;
            
            Ok(obj)
        }
    }
}

/// Encode a Clarity Value to a JS object
pub fn encode_clarity_value<'a>(
    cx: &mut FunctionContext<'a>,
    value: &Value,
    hex_bytes: &[u8],
    deep: bool,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    // repr string
    let repr = cx.string(format!("{}", value));
    obj.set(cx, "repr", repr)?;
    
    // hex encoding
    let hex = cx.string(encode_hex(hex_bytes));
    obj.set(cx, "hex", hex)?;
    
    // type_id
    let type_id = get_clarity_type_id(value);
    let type_id_js = cx.number(type_id as f64);
    obj.set(cx, "type_id", type_id_js)?;
    
    if deep {
        encode_clarity_value_deep(cx, &obj, value)?;
    }
    
    Ok(obj)
}

fn get_clarity_type_id(value: &Value) -> u8 {
    match value {
        Value::Int(_) => 0,
        Value::UInt(_) => 1,
        Value::Sequence(clarity::vm::types::SequenceData::Buffer(_)) => 2,
        Value::Bool(true) => 3,
        Value::Bool(false) => 4,
        Value::Principal(PrincipalData::Standard(_)) => 5,
        Value::Principal(PrincipalData::Contract(_)) => 6,
        Value::Response(resp) if resp.committed => 7,
        Value::Response(_) => 8,
        Value::Optional(opt) if opt.data.is_none() => 9,
        Value::Optional(_) => 10,
        Value::Sequence(clarity::vm::types::SequenceData::List(_)) => 11,
        Value::Tuple(_) => 12,
        Value::Sequence(clarity::vm::types::SequenceData::String(clarity::vm::types::CharType::ASCII(_))) => 13,
        Value::Sequence(clarity::vm::types::SequenceData::String(clarity::vm::types::CharType::UTF8(_))) => 14,
        _ => 255,
    }
}

fn encode_clarity_value_deep<'a>(
    cx: &mut FunctionContext<'a>,
    obj: &Handle<'a, JsObject>,
    value: &Value,
) -> NeonResult<()> {
    match value {
        Value::Int(v) => {
            let val_str = cx.string(v.to_string());
            obj.set(cx, "value", val_str)?;
        }
        Value::UInt(v) => {
            let val_str = cx.string(v.to_string());
            obj.set(cx, "value", val_str)?;
        }
        Value::Bool(v) => {
            let val_bool = cx.boolean(*v);
            obj.set(cx, "value", val_bool)?;
        }
        Value::Sequence(clarity::vm::types::SequenceData::Buffer(buff)) => {
            let buffer_hex = cx.string(encode_hex(&buff.data));
            obj.set(cx, "buffer", buffer_hex)?;
        }
        Value::Sequence(clarity::vm::types::SequenceData::String(clarity::vm::types::CharType::ASCII(s))) => {
            let data = cx.string(String::from_utf8_lossy(&s.data).to_string());
            obj.set(cx, "data", data)?;
        }
        Value::Sequence(clarity::vm::types::SequenceData::String(clarity::vm::types::CharType::UTF8(s))) => {
            let utf8_bytes: Vec<u8> = s.data.iter().flat_map(|v| v.clone()).collect();
            let data = cx.string(String::from_utf8_lossy(&utf8_bytes).to_string());
            obj.set(cx, "data", data)?;
        }
        Value::Sequence(clarity::vm::types::SequenceData::List(list_data)) => {
            let list = JsArray::new(cx, list_data.data.len());
            for (i, item) in list_data.data.iter().enumerate() {
                let mut item_bytes = Vec::new();
                item.consensus_serialize(&mut item_bytes).ok();
                let item_obj = encode_clarity_value(cx, item, &item_bytes, true)?;
                list.set(cx, i as u32, item_obj)?;
            }
            obj.set(cx, "list", list)?;
        }
        Value::Principal(principal) => {
            match principal {
                PrincipalData::Standard(std_principal) => {
                    let addr_string = std_principal.to_string();
                    let (version, hash160) = c32_address_decode(&addr_string)
                        .unwrap_or((0, vec![0u8; 20]));
                    
                    let version_js = cx.number(version as f64);
                    obj.set(cx, "address_version", version_js)?;
                    
                    let hash_bytes = cx.string(encode_hex(&hash160));
                    obj.set(cx, "address_hash_bytes", hash_bytes)?;
                    
                    let addr_str = cx.string(&addr_string);
                    obj.set(cx, "address", addr_str)?;
                }
                PrincipalData::Contract(contract_id) => {
                    let issuer_addr = contract_id.issuer.to_string();
                    let (version, hash160) = c32_address_decode(&issuer_addr)
                        .unwrap_or((0, vec![0u8; 20]));
                    
                    let version_js = cx.number(version as f64);
                    obj.set(cx, "address_version", version_js)?;
                    
                    let hash_bytes = cx.string(encode_hex(&hash160));
                    obj.set(cx, "address_hash_bytes", hash_bytes)?;
                    
                    let addr_str = cx.string(&issuer_addr);
                    obj.set(cx, "address", addr_str)?;
                    
                    let contract_name = cx.string(contract_id.name.as_str());
                    obj.set(cx, "contract_name", contract_name)?;
                }
            }
        }
        Value::Optional(opt_data) => {
            match &opt_data.data {
                Some(inner) => {
                    let mut inner_bytes = Vec::new();
                    inner.consensus_serialize(&mut inner_bytes).ok();
                    let inner_obj = encode_clarity_value(cx, inner, &inner_bytes, true)?;
                    obj.set(cx, "value", inner_obj)?;
                }
                None => {
                    let null = cx.null();
                    obj.set(cx, "value", null)?;
                }
            }
        }
        Value::Response(resp_data) => {
            let mut inner_bytes = Vec::new();
            resp_data.data.consensus_serialize(&mut inner_bytes).ok();
            let inner_obj = encode_clarity_value(cx, &resp_data.data, &inner_bytes, true)?;
            obj.set(cx, "value", inner_obj)?;
        }
        Value::Tuple(tuple_data) => {
            let data_obj = cx.empty_object();
            for (key, val) in tuple_data.data_map.iter() {
                let mut val_bytes = Vec::new();
                val.consensus_serialize(&mut val_bytes).ok();
                let val_obj = encode_clarity_value(cx, val, &val_bytes, true)?;
                data_obj.set(cx, key.as_str(), val_obj)?;
            }
            obj.set(cx, "data", data_obj)?;
        }
        _ => {}
    }
    Ok(())
}

/// Encode a TransactionPostCondition to a JS object
pub fn encode_post_condition<'a>(
    cx: &mut FunctionContext<'a>,
    pc: &TransactionPostCondition,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    match pc {
        TransactionPostCondition::STX(principal, condition_code, amount) => {
            let asset_info_id = cx.number(AssetInfoID::STX as u8 as f64);
            obj.set(cx, "asset_info_id", asset_info_id)?;
            
            let principal_obj = encode_post_condition_principal(cx, principal)?;
            obj.set(cx, "principal", principal_obj)?;
            
            let code = cx.number(*condition_code as u8 as f64);
            obj.set(cx, "condition_code", code)?;
            
            let code_name = cx.string(fungible_condition_code_name(*condition_code));
            obj.set(cx, "condition_name", code_name)?;
            
            let amt = cx.string(amount.to_string());
            obj.set(cx, "amount", amt)?;
        }
        TransactionPostCondition::Fungible(principal, asset, condition_code, amount) => {
            let asset_info_id = cx.number(AssetInfoID::FungibleAsset as u8 as f64);
            obj.set(cx, "asset_info_id", asset_info_id)?;
            
            let principal_obj = encode_post_condition_principal(cx, principal)?;
            obj.set(cx, "principal", principal_obj)?;
            
            let asset_obj = encode_asset_info(cx, asset)?;
            obj.set(cx, "asset", asset_obj)?;
            
            let code = cx.number(*condition_code as u8 as f64);
            obj.set(cx, "condition_code", code)?;
            
            let code_name = cx.string(fungible_condition_code_name(*condition_code));
            obj.set(cx, "condition_name", code_name)?;
            
            let amt = cx.string(amount.to_string());
            obj.set(cx, "amount", amt)?;
        }
        TransactionPostCondition::Nonfungible(principal, asset, asset_value, condition_code) => {
            let asset_info_id = cx.number(AssetInfoID::NonfungibleAsset as u8 as f64);
            obj.set(cx, "asset_info_id", asset_info_id)?;
            
            let principal_obj = encode_post_condition_principal(cx, principal)?;
            obj.set(cx, "principal", principal_obj)?;
            
            let asset_obj = encode_asset_info(cx, asset)?;
            obj.set(cx, "asset", asset_obj)?;
            
            let mut asset_value_bytes = Vec::new();
            asset_value.consensus_serialize(&mut asset_value_bytes).ok();
            let asset_value_obj = encode_clarity_value(cx, asset_value, &asset_value_bytes, true)?;
            obj.set(cx, "asset_value", asset_value_obj)?;
            
            let code = cx.number(*condition_code as u8 as f64);
            obj.set(cx, "condition_code", code)?;
            
            let code_name = cx.string(nonfungible_condition_code_name(*condition_code));
            obj.set(cx, "condition_name", code_name)?;
        }
    }
    
    Ok(obj)
}

fn fungible_condition_code_name(code: FungibleConditionCode) -> &'static str {
    match code {
        FungibleConditionCode::SentEq => "sent_equal_to",
        FungibleConditionCode::SentGt => "sent_greater_than",
        FungibleConditionCode::SentGe => "sent_greater_than_or_equal_to",
        FungibleConditionCode::SentLt => "sent_less_than",
        FungibleConditionCode::SentLe => "sent_less_than_or_equal_to",
    }
}

fn nonfungible_condition_code_name(code: NonfungibleConditionCode) -> &'static str {
    match code {
        NonfungibleConditionCode::Sent => "sent",
        NonfungibleConditionCode::NotSent => "not_sent",
    }
}

fn encode_post_condition_principal<'a>(
    cx: &mut FunctionContext<'a>,
    principal: &PostConditionPrincipal,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    match principal {
        PostConditionPrincipal::Origin => {
            let type_id = cx.number(1_f64); // PostConditionPrincipalTypeID.Origin
            obj.set(cx, "type_id", type_id)?;
        }
        PostConditionPrincipal::Standard(addr) => {
            let type_id = cx.number(2_f64); // PostConditionPrincipalTypeID.Standard
            obj.set(cx, "type_id", type_id)?;
            
            let version = cx.number(addr.version() as f64);
            obj.set(cx, "address_version", version)?;
            
            let hash_bytes = cx.string(encode_hex(addr.bytes().as_ref()));
            obj.set(cx, "address_hash_bytes", hash_bytes)?;
            
            let c32_addr = c32_address(addr.version(), addr.bytes().as_ref())
                .unwrap_or_else(|_| "invalid".to_string());
            let addr_str = cx.string(c32_addr);
            obj.set(cx, "address", addr_str)?;
        }
        PostConditionPrincipal::Contract(addr, contract_name) => {
            let type_id = cx.number(3_f64); // PostConditionPrincipalTypeID.Contract
            obj.set(cx, "type_id", type_id)?;
            
            let version = cx.number(addr.version() as f64);
            obj.set(cx, "address_version", version)?;
            
            let hash_bytes = cx.string(encode_hex(addr.bytes().as_ref()));
            obj.set(cx, "address_hash_bytes", hash_bytes)?;
            
            let c32_addr = c32_address(addr.version(), addr.bytes().as_ref())
                .unwrap_or_else(|_| "invalid".to_string());
            let addr_str = cx.string(c32_addr);
            obj.set(cx, "address", addr_str)?;
            
            let name = cx.string(contract_name.as_str());
            obj.set(cx, "contract_name", name)?;
        }
    }
    
    Ok(obj)
}

fn encode_asset_info<'a>(
    cx: &mut FunctionContext<'a>,
    asset: &AssetInfo,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let contract_address = c32_address(asset.contract_address.version(), asset.contract_address.bytes().as_ref())
        .unwrap_or_else(|_| "invalid".to_string());
    let addr_str = cx.string(contract_address);
    obj.set(cx, "contract_address", addr_str)?;
    
    let contract_name = cx.string(asset.contract_name.as_str());
    obj.set(cx, "contract_name", contract_name)?;
    
    let asset_name = cx.string(asset.asset_name.as_str());
    obj.set(cx, "asset_name", asset_name)?;
    
    Ok(obj)
}

/// Encode a StacksTransaction to a JS object
pub fn encode_transaction<'a>(
    cx: &mut FunctionContext<'a>,
    tx: &StacksTransaction,
    tx_id: &Txid,
    post_conditions_buffer: &[u8],
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    // tx_id
    let tx_id_str = cx.string(encode_hex(tx_id.as_bytes()));
    obj.set(cx, "tx_id", tx_id_str)?;
    
    // version
    let version = match tx.version {
        TransactionVersion::Mainnet => 0x00,
        TransactionVersion::Testnet => 0x80,
    };
    let version_js = cx.number(version as f64);
    obj.set(cx, "version", version_js)?;
    
    // chain_id
    let chain_id = cx.number(tx.chain_id as f64);
    obj.set(cx, "chain_id", chain_id)?;
    
    // auth
    let auth_obj = encode_transaction_auth(cx, &tx.auth)?;
    obj.set(cx, "auth", auth_obj)?;
    
    // anchor_mode
    let anchor_mode = tx.anchor_mode as u8;
    let anchor_mode_js = cx.number(anchor_mode as f64);
    obj.set(cx, "anchor_mode", anchor_mode_js)?;
    
    // post_condition_mode
    let post_condition_mode = tx.post_condition_mode as u8;
    let post_condition_mode_js = cx.number(post_condition_mode as f64);
    obj.set(cx, "post_condition_mode", post_condition_mode_js)?;
    
    // post_conditions
    let post_conditions = JsArray::new(cx, tx.post_conditions.len());
    for (i, pc) in tx.post_conditions.iter().enumerate() {
        let pc_obj = encode_post_condition(cx, pc)?;
        post_conditions.set(cx, i as u32, pc_obj)?;
    }
    obj.set(cx, "post_conditions", post_conditions)?;
    
    // post_conditions_buffer
    let pc_buffer = cx.string(encode_hex(post_conditions_buffer));
    obj.set(cx, "post_conditions_buffer", pc_buffer)?;
    
    // payload
    let payload_obj = encode_transaction_payload(cx, &tx.payload)?;
    obj.set(cx, "payload", payload_obj)?;
    
    Ok(obj)
}

fn encode_transaction_auth<'a>(
    cx: &mut FunctionContext<'a>,
    auth: &TransactionAuth,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    match auth {
        TransactionAuth::Standard(origin_condition) => {
            let type_id = cx.number(0x04_f64); // AuthStandard
            obj.set(cx, "type_id", type_id)?;
            
            let origin_obj = encode_spending_condition(cx, origin_condition)?;
            obj.set(cx, "origin_condition", origin_obj)?;
        }
        TransactionAuth::Sponsored(origin_condition, sponsor_condition) => {
            let type_id = cx.number(0x05_f64); // AuthSponsored
            obj.set(cx, "type_id", type_id)?;
            
            let origin_obj = encode_spending_condition(cx, origin_condition)?;
            obj.set(cx, "origin_condition", origin_obj)?;
            
            let sponsor_obj = encode_spending_condition(cx, sponsor_condition)?;
            obj.set(cx, "sponsor_condition", sponsor_obj)?;
        }
    }
    
    Ok(obj)
}

fn encode_spending_condition<'a>(
    cx: &mut FunctionContext<'a>,
    condition: &TransactionSpendingCondition,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    match condition {
        TransactionSpendingCondition::Singlesig(data) => {
            let hash_mode_u8 = match &data.hash_mode {
                SinglesigHashMode::P2PKH => 0x00,
                SinglesigHashMode::P2WPKH => 0x02,
            };
            let hash_mode = cx.number(hash_mode_u8 as f64);
            obj.set(cx, "hash_mode", hash_mode)?;
            
            // Determine version based on hash mode
            let version = match &data.hash_mode {
                SinglesigHashMode::P2PKH => 22, // mainnet singlesig default
                SinglesigHashMode::P2WPKH => 20, // mainnet multisig default
            };
            let signer_obj = encode_signer_address(cx, version, &data.signer)?;
            obj.set(cx, "signer", signer_obj)?;
            
            let nonce = cx.string(data.nonce.to_string());
            obj.set(cx, "nonce", nonce)?;
            
            let tx_fee = cx.string(data.tx_fee.to_string());
            obj.set(cx, "tx_fee", tx_fee)?;
            
            let key_encoding = cx.number(data.key_encoding as u8 as f64);
            obj.set(cx, "key_encoding", key_encoding)?;
            
            let signature = cx.string(encode_hex(data.signature.as_bytes()));
            obj.set(cx, "signature", signature)?;
        }
        TransactionSpendingCondition::Multisig(data) => {
            let hash_mode_u8 = match &data.hash_mode {
                MultisigHashMode::P2SH => 0x01,
                MultisigHashMode::P2WSH => 0x03,
            };
            let hash_mode = cx.number(hash_mode_u8 as f64);
            obj.set(cx, "hash_mode", hash_mode)?;
            
            let version = match &data.hash_mode {
                MultisigHashMode::P2SH => 20,
                MultisigHashMode::P2WSH => 20,
            };
            let signer_obj = encode_signer_address(cx, version, &data.signer)?;
            obj.set(cx, "signer", signer_obj)?;
            
            let nonce = cx.string(data.nonce.to_string());
            obj.set(cx, "nonce", nonce)?;
            
            let tx_fee = cx.string(data.tx_fee.to_string());
            obj.set(cx, "tx_fee", tx_fee)?;
            
            let fields = encode_auth_fields(cx, &data.fields)?;
            obj.set(cx, "fields", fields)?;
            
            let signatures_required = cx.number(data.signatures_required as f64);
            obj.set(cx, "signatures_required", signatures_required)?;
        }
        TransactionSpendingCondition::OrderIndependentMultisig(data) => {
            let hash_mode_u8 = match &data.hash_mode {
                OrderIndependentMultisigHashMode::P2SH => 0x05,
                OrderIndependentMultisigHashMode::P2WSH => 0x07,
            };
            let hash_mode = cx.number(hash_mode_u8 as f64);
            obj.set(cx, "hash_mode", hash_mode)?;
            
            let version = match &data.hash_mode {
                OrderIndependentMultisigHashMode::P2SH => 20,
                OrderIndependentMultisigHashMode::P2WSH => 20,
            };
            let signer_obj = encode_signer_address(cx, version, &data.signer)?;
            obj.set(cx, "signer", signer_obj)?;
            
            let nonce = cx.string(data.nonce.to_string());
            obj.set(cx, "nonce", nonce)?;
            
            let tx_fee = cx.string(data.tx_fee.to_string());
            obj.set(cx, "tx_fee", tx_fee)?;
            
            let fields = encode_auth_fields(cx, &data.fields)?;
            obj.set(cx, "fields", fields)?;
            
            let signatures_required = cx.number(data.signatures_required as f64);
            obj.set(cx, "signatures_required", signatures_required)?;
        }
    }
    
    Ok(obj)
}

fn encode_auth_fields<'a>(
    cx: &mut FunctionContext<'a>,
    fields: &[TransactionAuthField],
) -> JsResult<'a, JsArray> {
    let arr = JsArray::new(cx, fields.len());
    
    for (i, field) in fields.iter().enumerate() {
        let field_obj = cx.empty_object();
        
        match field {
            TransactionAuthField::PublicKey(pubkey) => {
                let type_id = if pubkey.compressed() {
                    TransactionAuthFieldID::PublicKeyCompressed
                } else {
                    TransactionAuthFieldID::PublicKeyUncompressed
                };
                let type_id_js = cx.number(type_id as u8 as f64);
                field_obj.set(cx, "type_id", type_id_js)?;
                
                let pubkey_hex = cx.string(encode_hex(&pubkey.to_bytes_compressed()));
                field_obj.set(cx, "public_key", pubkey_hex)?;
            }
            TransactionAuthField::Signature(encoding, sig) => {
                let type_id = match encoding {
                    TransactionPublicKeyEncoding::Compressed => TransactionAuthFieldID::SignatureCompressed,
                    TransactionPublicKeyEncoding::Uncompressed => TransactionAuthFieldID::SignatureUncompressed,
                };
                let type_id_js = cx.number(type_id as u8 as f64);
                field_obj.set(cx, "type_id", type_id_js)?;
                
                let sig_hex = cx.string(encode_hex(sig.as_bytes()));
                field_obj.set(cx, "signature", sig_hex)?;
            }
        }
        
        arr.set(cx, i as u32, field_obj)?;
    }
    
    Ok(arr)
}

fn encode_transaction_payload<'a>(
    cx: &mut FunctionContext<'a>,
    payload: &TransactionPayload,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    match payload {
        TransactionPayload::TokenTransfer(recipient, amount, memo) => {
            let type_id = cx.number(0_f64); // TokenTransfer
            obj.set(cx, "type_id", type_id)?;
            
            let recipient_obj = encode_principal(cx, recipient)?;
            obj.set(cx, "recipient", recipient_obj)?;
            
            let amt = cx.string(amount.to_string());
            obj.set(cx, "amount", amt)?;
            
            let memo_hex = cx.string(encode_hex(&memo.0));
            obj.set(cx, "memo_hex", memo_hex)?;
        }
        TransactionPayload::SmartContract(contract, version_opt) => {
            match version_opt {
                Some(version) => {
                    let type_id = cx.number(6_f64); // VersionedSmartContract
                    obj.set(cx, "type_id", type_id)?;
                    
                    let clarity_version = match version {
                        ClarityVersion::Clarity1 => 1,
                        ClarityVersion::Clarity2 => 2,
                        ClarityVersion::Clarity3 => 3,
                        _ => 1,
                    };
                    let ver = cx.number(clarity_version as f64);
                    obj.set(cx, "clarity_version", ver)?;
                }
                None => {
                    let type_id = cx.number(1_f64); // SmartContract
                    obj.set(cx, "type_id", type_id)?;
                }
            }
            
            let contract_name = cx.string(contract.name.as_str());
            obj.set(cx, "contract_name", contract_name)?;
            
            let code_body = cx.string(contract.code_body.to_string());
            obj.set(cx, "code_body", code_body)?;
        }
        TransactionPayload::ContractCall(call) => {
            let type_id = cx.number(2_f64); // ContractCall
            obj.set(cx, "type_id", type_id)?;
            
            let version = cx.number(call.address.version() as f64);
            obj.set(cx, "address_version", version)?;
            
            let hash_bytes = cx.string(encode_hex(call.address.bytes().as_ref()));
            obj.set(cx, "address_hash_bytes", hash_bytes)?;
            
            let c32_addr = c32_address(call.address.version(), call.address.bytes().as_ref())
                .unwrap_or_else(|_| "invalid".to_string());
            let addr_str = cx.string(c32_addr);
            obj.set(cx, "address", addr_str)?;
            
            let contract_name = cx.string(call.contract_name.as_str());
            obj.set(cx, "contract_name", contract_name)?;
            
            let function_name = cx.string(call.function_name.as_str());
            obj.set(cx, "function_name", function_name)?;
            
            // function_args
            let args = JsArray::new(cx, call.function_args.len());
            let mut args_buffer = Vec::new();
            for (i, arg) in call.function_args.iter().enumerate() {
                let mut arg_bytes = Vec::new();
                arg.consensus_serialize(&mut arg_bytes).ok();
                args_buffer.extend(&arg_bytes);
                let arg_obj = encode_clarity_value(cx, arg, &arg_bytes, true)?;
                args.set(cx, i as u32, arg_obj)?;
            }
            obj.set(cx, "function_args", args)?;
            
            let args_buffer_hex = cx.string(encode_hex(&args_buffer));
            obj.set(cx, "function_args_buffer", args_buffer_hex)?;
        }
        TransactionPayload::PoisonMicroblock(header1, header2) => {
            let type_id = cx.number(3_f64); // PoisonMicroblock
            obj.set(cx, "type_id", type_id)?;
            
            let header1_obj = encode_microblock_header(cx, header1)?;
            obj.set(cx, "microblock_header_1", header1_obj)?;
            
            let header2_obj = encode_microblock_header(cx, header2)?;
            obj.set(cx, "microblock_header_2", header2_obj)?;
        }
        TransactionPayload::Coinbase(payload, recipient_opt, vrf_opt) => {
            match (recipient_opt, vrf_opt) {
                (None, None) => {
                    let type_id = cx.number(4_f64); // Coinbase
                    obj.set(cx, "type_id", type_id)?;
                }
                (Some(_), None) => {
                    let type_id = cx.number(5_f64); // CoinbaseToAltRecipient
                    obj.set(cx, "type_id", type_id)?;
                }
                (_, Some(_)) => {
                    let type_id = cx.number(8_f64); // NakamotoCoinbase
                    obj.set(cx, "type_id", type_id)?;
                }
            }
            
            let payload_buffer = cx.string(encode_hex(&payload.0));
            obj.set(cx, "payload_buffer", payload_buffer)?;
            
            if let Some(recipient) = recipient_opt {
                let recipient_obj = encode_principal(cx, recipient)?;
                obj.set(cx, "recipient", recipient_obj)?;
            } else if vrf_opt.is_some() {
                // For NakamotoCoinbase without recipient, set to null
                let null = cx.null();
                obj.set(cx, "recipient", null)?;
            }
            
            if let Some(vrf_proof) = vrf_opt {
                let vrf_hex = cx.string(encode_hex(&vrf_proof.to_bytes()));
                obj.set(cx, "vrf_proof", vrf_hex)?;
            }
        }
        TransactionPayload::TenureChange(tc) => {
            let type_id = cx.number(7_f64); // TenureChange
            obj.set(cx, "type_id", type_id)?;
            
            let tenure_consensus_hash = cx.string(encode_hex(tc.tenure_consensus_hash.as_ref()));
            obj.set(cx, "tenure_consensus_hash", tenure_consensus_hash)?;
            
            let prev_tenure_consensus_hash = cx.string(encode_hex(tc.prev_tenure_consensus_hash.as_ref()));
            obj.set(cx, "prev_tenure_consensus_hash", prev_tenure_consensus_hash)?;
            
            let burn_view_consensus_hash = cx.string(encode_hex(tc.burn_view_consensus_hash.as_ref()));
            obj.set(cx, "burn_view_consensus_hash", burn_view_consensus_hash)?;
            
            let previous_tenure_end = cx.string(encode_hex(tc.previous_tenure_end.as_ref()));
            obj.set(cx, "previous_tenure_end", previous_tenure_end)?;
            
            let previous_tenure_blocks = cx.number(tc.previous_tenure_blocks as f64);
            obj.set(cx, "previous_tenure_blocks", previous_tenure_blocks)?;
            
            let cause = cx.number(tc.cause as u8 as f64);
            obj.set(cx, "cause", cause)?;
            
            let pubkey_hash = cx.string(encode_hex(tc.pubkey_hash.as_ref()));
            obj.set(cx, "pubkey_hash", pubkey_hash)?;
        }
    }
    
    Ok(obj)
}

fn encode_microblock_header<'a>(
    cx: &mut FunctionContext<'a>,
    header: &StacksMicroblockHeader,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let mut header_bytes = Vec::new();
    header.consensus_serialize(&mut header_bytes).ok();
    let buffer = cx.string(encode_hex(&header_bytes));
    obj.set(cx, "buffer", buffer)?;
    
    let version = cx.number(header.version as f64);
    obj.set(cx, "version", version)?;
    
    let sequence = cx.number(header.sequence as f64);
    obj.set(cx, "sequence", sequence)?;
    
    let prev_block = cx.string(encode_hex(header.prev_block.as_ref()));
    obj.set(cx, "prev_block", prev_block)?;
    
    let tx_merkle_root = cx.string(encode_hex(header.tx_merkle_root.as_ref()));
    obj.set(cx, "tx_merkle_root", tx_merkle_root)?;
    
    let signature = cx.string(encode_hex(header.signature.as_bytes()));
    obj.set(cx, "signature", signature)?;
    
    Ok(obj)
}

/// Encode a NakamotoBlock to a JS object
pub fn encode_nakamoto_block<'a>(
    cx: &mut FunctionContext<'a>,
    block: &NakamotoBlock,
    _block_bytes: &[u8],
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    // block_id (hash of the serialized block header)
    let mut header_bytes = Vec::new();
    block.header.consensus_serialize(&mut header_bytes).ok();
    let block_id = stacks_common::util::hash::Sha512Trunc256Sum::from_data(&header_bytes);
    let block_id_str = cx.string(encode_hex(block_id.as_ref()));
    obj.set(cx, "block_id", block_id_str)?;
    
    // header
    let header_obj = encode_nakamoto_block_header(cx, &block.header)?;
    obj.set(cx, "header", header_obj)?;
    
    // txs
    let txs = JsArray::new(cx, block.txs.len());
    for (i, tx) in block.txs.iter().enumerate() {
        let tx_id = tx.txid();
        
        // Serialize post conditions for buffer
        let mut pc_buffer = Vec::new();
        pc_buffer.push(tx.post_condition_mode as u8);
        let pc_len = tx.post_conditions.len() as u32;
        pc_buffer.extend(&pc_len.to_be_bytes());
        for pc in &tx.post_conditions {
            pc.consensus_serialize(&mut pc_buffer).ok();
        }
        
        let tx_obj = encode_transaction(cx, tx, &tx_id, &pc_buffer)?;
        txs.set(cx, i as u32, tx_obj)?;
    }
    obj.set(cx, "txs", txs)?;
    
    Ok(obj)
}

/// Encode a NakamotoBlockHeader to a JS object
pub fn encode_nakamoto_block_header<'a>(
    cx: &mut FunctionContext<'a>,
    header: &NakamotoBlockHeader,
) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();
    
    let version = cx.number(header.version as f64);
    obj.set(cx, "version", version)?;
    
    let chain_length = cx.string(header.chain_length.to_string());
    obj.set(cx, "chain_length", chain_length)?;
    
    let burn_spent = cx.string(header.burn_spent.to_string());
    obj.set(cx, "burn_spent", burn_spent)?;
    
    let consensus_hash = cx.string(encode_hex(header.consensus_hash.as_ref()));
    obj.set(cx, "consensus_hash", consensus_hash)?;
    
    let parent_block_id = cx.string(encode_hex(header.parent_block_id.as_ref()));
    obj.set(cx, "parent_block_id", parent_block_id)?;
    
    let tx_merkle_root = cx.string(encode_hex(header.tx_merkle_root.as_ref()));
    obj.set(cx, "tx_merkle_root", tx_merkle_root)?;
    
    let state_index_root = cx.string(encode_hex(header.state_index_root.as_ref()));
    obj.set(cx, "state_index_root", state_index_root)?;
    
    let timestamp = cx.string(header.timestamp.to_string());
    obj.set(cx, "timestamp", timestamp)?;
    
    let miner_signature = cx.string(encode_hex(header.miner_signature.as_bytes()));
    obj.set(cx, "miner_signature", miner_signature)?;
    
    // signer_signature array
    let signer_sigs = JsArray::new(cx, header.signer_signature.len());
    for (i, sig) in header.signer_signature.iter().enumerate() {
        let sig_hex = cx.string(encode_hex(sig.as_bytes()));
        signer_sigs.set(cx, i as u32, sig_hex)?;
    }
    obj.set(cx, "signer_signature", signer_sigs)?;
    
    // pox_treatment as hex
    let mut pox_bytes = Vec::new();
    header.pox_treatment.consensus_serialize(&mut pox_bytes).ok();
    let pox_treatment = cx.string(encode_hex(&pox_bytes));
    obj.set(cx, "pox_treatment", pox_treatment)?;
    
    Ok(obj)
}
