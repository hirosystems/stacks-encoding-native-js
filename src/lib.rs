//! Stacks Encoding Native JS
//!
//! This crate provides Node.js bindings (via Neon) for encoding and decoding
//! Stacks blockchain wire format data. It uses the stacks-codec crate from
//! the Clarinet project for all encoding/decoding operations.

use std::io::Cursor;

use git_version::git_version;
use neon::prelude::*;

use clarity::codec::StacksMessageCodec;
use clarity::vm::types::{PrincipalData, Value};
use stacks_codec::codec::{
    NakamotoBlock, NakamotoBlockHeader, StacksTransaction, TransactionPostCondition, Txid,
};
use stacks_common::address::c32::{c32_address, c32_address_decode};
use stacks_common::util::hash::Sha512Trunc256Sum;

pub mod hex;
pub mod memo;
pub mod neon_encoder;
pub mod neon_util;

use crate::hex::encode_hex;
use crate::memo::memo_to_string;
use crate::neon_encoder::{
    encode_clarity_value, encode_nakamoto_block, encode_nakamoto_block_header,
    encode_post_condition, encode_transaction,
};
use crate::neon_util::{arg_as_bytes, arg_as_bytes_copied};

const GIT_VERSION: &str = git_version!(
    args = ["--all", "--long", "--always"],
    fallback = "unavailable"
);

fn get_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.string(GIT_VERSION);
    Ok(version)
}

/// Decode a Clarity value and return its repr string
pub fn decode_clarity_value_to_repr(mut cx: FunctionContext) -> JsResult<JsString> {
    let repr_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        Value::consensus_deserialize(&mut cursor)
            .map_err(|e| format!("Failed to deserialize: {:?}", e))
            .map(|val| format!("{}", val))
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(repr_string))
}

/// Decode a Clarity value and return its type name
pub fn decode_clarity_value_type_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let type_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        Value::consensus_deserialize(&mut cursor)
            .map_err(|e| format!("Failed to deserialize: {:?}", e))
            .map(|val| get_clarity_type_name(&val))
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(type_string))
}

fn get_clarity_type_name(value: &Value) -> String {
    match value {
        Value::Int(_) => "int".to_string(),
        Value::UInt(_) => "uint".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Sequence(clarity::vm::types::SequenceData::Buffer(b)) => {
            format!("(buff {})", b.data.len())
        }
        Value::Sequence(clarity::vm::types::SequenceData::String(
            clarity::vm::types::CharType::ASCII(s),
        )) => format!("(string-ascii {})", s.data.len()),
        Value::Sequence(clarity::vm::types::SequenceData::String(
            clarity::vm::types::CharType::UTF8(s),
        )) => format!("(string-utf8 {})", s.data.len()),
        Value::Sequence(clarity::vm::types::SequenceData::List(l)) => {
            if l.data.is_empty() {
                "(list)".to_string()
            } else {
                format!("(list {} {})", l.data.len(), get_clarity_type_name(&l.data[0]))
            }
        }
        Value::Principal(PrincipalData::Standard(_)) => "principal".to_string(),
        Value::Principal(PrincipalData::Contract(_)) => "principal".to_string(),
        Value::Tuple(t) => {
            let fields: Vec<String> = t
                .data_map
                .iter()
                .map(|(k, v)| format!("({} {})", k, get_clarity_type_name(v)))
                .collect();
            format!("(tuple {})", fields.join(" "))
        }
        Value::Optional(opt) => match &opt.data {
            Some(inner) => format!("(optional {})", get_clarity_type_name(inner)),
            None => "(optional none)".to_string(),
        },
        Value::Response(resp) => {
            let inner_type = get_clarity_type_name(&resp.data);
            if resp.committed {
                format!("(response {} _)", inner_type)
            } else {
                format!("(response _ {})", inner_type)
            }
        }
        _ => "unknown".to_string(),
    }
}

/// Decode a Clarity value to a JS object
pub fn decode_clarity_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let mut cursor = Cursor::new(&*val_bytes);
    let clarity_value = Value::consensus_deserialize(&mut cursor)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {:?}", e)))?;

    let root_obj = encode_clarity_value(&mut cx, &clarity_value, &val_bytes, true)?;
    Ok(root_obj)
}

/// Decode an array of Clarity values
pub fn decode_clarity_value_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let result_length = if input_bytes.len() >= 4 {
        u32::from_be_bytes(input_bytes[..4].try_into().unwrap())
    } else {
        0
    };

    let array_result = JsArray::new(&mut cx, result_length as usize);

    let deep: bool = match cx.argument_opt(1) {
        Some(arg) => arg
            .downcast_or_throw::<JsBoolean, _>(&mut cx)?
            .value(&mut cx),
        None => false,
    };

    if input_bytes.len() > 4 {
        let val_slice = &input_bytes[4..];
        let mut byte_cursor = Cursor::new(val_slice);
        let val_len = val_slice.len() as u64;
        let mut i: u32 = 0;
        while byte_cursor.position() < val_len {
            let cursor_pos = byte_cursor.position();
            let clarity_value = Value::consensus_deserialize(&mut byte_cursor)
                .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {:?}", e)))?;
            let decoded_bytes =
                &byte_cursor.get_ref()[cursor_pos as usize..byte_cursor.position() as usize];
            let value_obj = encode_clarity_value(&mut cx, &clarity_value, decoded_bytes, deep)?;
            array_result.set(&mut cx, i, value_obj)?;
            i += 1;
        }
    }
    Ok(array_result)
}

/// Decode post conditions from a transaction
pub fn decode_tx_post_conditions(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    let resp_obj = cx.empty_object();

    // first byte is post condition mode
    let post_condition_mode = cx.number(input_bytes[0] as f64);
    resp_obj.set(&mut cx, "post_condition_mode", post_condition_mode)?;

    let array_result = if input_bytes.len() > 4 {
        // next 4 bytes are array length
        let result_length = u32::from_be_bytes(input_bytes[1..5].try_into().or_else(|e| {
            cx.throw_error(format!(
                "Error reading post condition bytes {}, {}",
                encode_hex(&input_bytes),
                e
            ))
        })?);
        let array_result = JsArray::new(&mut cx, result_length as usize);
        // next bytes are serialized post condition items
        let post_condition_bytes = &input_bytes[5..];
        let post_condition_bytes_len = post_condition_bytes.len() as u64;
        let mut cursor = Cursor::new(post_condition_bytes);
        let mut i: u32 = 0;
        while cursor.position() < post_condition_bytes_len {
            let post_condition = TransactionPostCondition::consensus_deserialize(&mut cursor)
                .or_else(|e| cx.throw_error(format!("Error deserializing post condition: {:?}", e)))?;
            let value_obj = encode_post_condition(&mut cx, &post_condition)?;
            array_result.set(&mut cx, i, value_obj)?;
            i += 1;
        }
        array_result
    } else {
        cx.empty_array()
    };
    resp_obj.set(&mut cx, "post_conditions", array_result)?;
    Ok(resp_obj)
}

/// Decode a Stacks transaction
pub fn decode_transaction(mut cx: FunctionContext) -> JsResult<JsObject> {
    let (tx, tx_id, pc_buffer) = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        let tx = StacksTransaction::consensus_deserialize(&mut cursor)
            .map_err(|e| format!("Failed to decode transaction: {:?}", e))?;
        
        // Calculate tx_id
        let tx_id_hash = Sha512Trunc256Sum::from_data(val_bytes);
        let tx_id = Txid(tx_id_hash.0);
        
        // Build post conditions buffer
        let mut pc_buffer = Vec::new();
        pc_buffer.push(tx.post_condition_mode as u8);
        let pc_len = tx.post_conditions.len() as u32;
        pc_buffer.extend(&pc_len.to_be_bytes());
        for pc in &tx.post_conditions {
            pc.consensus_serialize(&mut pc_buffer).ok();
        }
        
        Ok((tx, tx_id, pc_buffer))
    })
    .or_else(|e| cx.throw_error(e))?;

    let tx_json_obj = encode_transaction(&mut cx, &tx, &tx_id, &pc_buffer)?;
    Ok(tx_json_obj)
}

/// Check if a Stacks address is valid
pub fn is_valid_stacks_address(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let result = c32_address_decode(&address_string).is_ok();
    Ok(cx.boolean(result))
}

/// Decode a Stacks address into version and hash160 parts
pub fn decode_stacks_address(mut cx: FunctionContext) -> JsResult<JsArray> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let (version, hash160) = c32_address_decode(&address_string)
        .or_else(|e| cx.throw_error(format!("Error parsing Stacks address: {}", e)))?;

    let version_js = cx.number(version as f64);
    let hash160_js = cx.string(encode_hex(&hash160));

    let array_resp = JsArray::new(&mut cx, 2);
    array_resp.set(&mut cx, 0, version_js)?;
    array_resp.set(&mut cx, 1, hash160_js)?;
    Ok(array_resp)
}

/// Decode a Clarity value to a principal string
pub fn decode_clarity_value_to_principal(mut cx: FunctionContext) -> JsResult<JsString> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let mut cursor = Cursor::new(&*val_bytes);
    let value = Value::consensus_deserialize(&mut cursor)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {:?}", e)))?;

    let principal_str = match value {
        Value::Principal(PrincipalData::Standard(std)) => {
            std.to_string()
        }
        Value::Principal(PrincipalData::Contract(contract)) => {
            format!("{}.{}", contract.issuer.to_string(), contract.name)
        }
        Value::Sequence(clarity::vm::types::SequenceData::Buffer(buff)) if buff.data.len() == 21 => {
            // Handle buffer containing principal bytes (version + hash160)
            let version = buff.data[0];
            let hash160 = &buff.data[1..21];
            c32_address(version, hash160).unwrap_or_else(|_| "invalid".to_string())
        }
        _ => {
            return cx.throw_error("Value is not a principal type");
        }
    };

    Ok(cx.string(principal_str))
}

/// Create a Stacks address from version and hash160 parts
pub fn stacks_address_from_parts(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.argument::<JsNumber>(0)?.value(&mut cx) as u8;
    let stacks_address = arg_as_bytes(&mut cx, 1, |bytes| {
        c32_address(version, bytes).map_err(|e| format!("Error converting to C32 address: {}", e))
    })
    .or_else(|e| cx.throw_error(e))?;
    Ok(cx.string(stacks_address))
}

/// Convert a Stacks address to a Bitcoin address
pub fn stacks_to_bitcoin_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let stacks_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let (version, hash160) = c32_address_decode(&stacks_address_arg)
        .or_else(|e| cx.throw_error(format!("Error parsing Stacks address: {}", e)))?;
    
    // Convert Stacks version to Bitcoin version
    let btc_version = match version {
        22 => 0x00, // P2PKH mainnet
        20 => 0x05, // P2SH mainnet  
        26 => 0x6f, // P2PKH testnet
        21 => 0xc4, // P2SH testnet
        _ => version, // fallback
    };
    
    // Encode as base58check
    let mut all_bytes = vec![btc_version];
    all_bytes.extend(&hash160);
    let btc_address = bs58_check_encode(&all_bytes);
    
    Ok(cx.string(btc_address))
}

/// Convert a Bitcoin address to a Stacks address
pub fn bitcoin_to_stacks_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let bitcoin_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let decoded = bs58_check_decode(&bitcoin_address_arg)
        .or_else(|e| cx.throw_error(format!("Error parsing Bitcoin address: {}", e)))?;
    
    if decoded.len() != 21 {
        return cx.throw_error("Invalid Bitcoin address length");
    }
    
    let btc_version = decoded[0];
    let hash160 = &decoded[1..21];
    
    // Convert Bitcoin version to Stacks version
    let stx_version = match btc_version {
        0x00 => 22, // P2PKH mainnet -> SP
        0x05 => 20, // P2SH mainnet -> SM
        0x6f => 26, // P2PKH testnet -> ST
        0xc4 => 21, // P2SH testnet -> SN
        _ => return cx.throw_error(format!("Unknown Bitcoin version byte: {}", btc_version)),
    };
    
    let stacks_addr = c32_address(stx_version, hash160)
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;
    
    Ok(cx.string(stacks_addr))
}

/// Decode a Nakamoto block
pub fn decode_nakamoto_block(mut cx: FunctionContext) -> JsResult<JsObject> {
    let block_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    
    let mut cursor = Cursor::new(&*block_bytes);
    let block = NakamotoBlock::consensus_deserialize(&mut cursor)
        .or_else(|e| cx.throw_error(format!("Error deserializing Nakamoto block: {:?}", e)))?;
    
    let block_obj = encode_nakamoto_block(&mut cx, &block, &block_bytes)?;
    Ok(block_obj)
}

/// Decode a Nakamoto block header
pub fn decode_nakamoto_block_header(mut cx: FunctionContext) -> JsResult<JsObject> {
    let header_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    
    let mut cursor = Cursor::new(&*header_bytes);
    let header = NakamotoBlockHeader::consensus_deserialize(&mut cursor)
        .or_else(|e| cx.throw_error(format!("Error deserializing Nakamoto block header: {:?}", e)))?;
    
    let header_obj = encode_nakamoto_block_header(&mut cx, &header)?;
    Ok(header_obj)
}

// Base58 check encoding helper
fn bs58_check_encode(data: &[u8]) -> String {
    use stacks_common::util::hash::DoubleSha256;
    let checksum = DoubleSha256::from_data(data);
    let mut with_checksum = data.to_vec();
    with_checksum.extend(&checksum.as_bytes()[0..4]);
    bs58::encode(&with_checksum).into_string()
}

// Base58 check decoding helper
fn bs58_check_decode(input: &str) -> Result<Vec<u8>, String> {
    use stacks_common::util::hash::DoubleSha256;
    let decoded = bs58::decode(input)
        .into_vec()
        .map_err(|e| format!("Base58 decode error: {}", e))?;
    
    if decoded.len() < 4 {
        return Err("Input too short for checksum".to_string());
    }
    
    let data_len = decoded.len() - 4;
    let data = &decoded[..data_len];
    let checksum = &decoded[data_len..];
    
    let computed_checksum = DoubleSha256::from_data(data);
    if &computed_checksum.as_bytes()[0..4] != checksum {
        return Err("Checksum mismatch".to_string());
    }
    
    Ok(data.to_vec())
}

#[cfg(feature = "profiling")]
lazy_static::lazy_static! {
    static ref PROFILER: std::sync::Mutex<Option<pprof::ProfilerGuard<'static>>> =
        std::sync::Mutex::new(None);
}

#[cfg(feature = "profiling")]
fn start_profiler(mut cx: FunctionContext) -> JsResult<JsString> {
    let mut profiler = PROFILER
        .lock()
        .or_else(|e| cx.throw_error(format!("Failed to acquire lock: {}", e)))?;
    if profiler.is_some() {
        return cx.throw_error("Profiler already started");
    }
    let profiler_guard = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e)))?;
    *profiler = Some(profiler_guard);
    Ok(cx.string("Profiler started"))
}

#[cfg(feature = "profiling")]
fn stop_profiler(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let mut profiler = PROFILER
        .lock()
        .or_else(|e| cx.throw_error(format!("Failed to acquire lock: {}", e)))?;
    let report_result = match &*profiler {
        None => return cx.throw_error("No profiler started"),
        Some(profiler) => profiler.report().build(),
    };
    let report = match report_result {
        Ok(report) => report,
        Err(err) => return cx.throw_error(format!("Error generating report: {}", err)),
    };

    let mut buf = Vec::new();
    report
        .flamegraph(&mut buf)
        .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;

    *profiler = None;

    let result = JsBuffer::external(&mut cx, buf);
    Ok(result)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("getVersion", get_version)?;
    cx.export_function("decodeClarityValueToRepr", decode_clarity_value_to_repr)?;
    cx.export_function("decodeClarityValueToTypeName", decode_clarity_value_type_name)?;
    cx.export_function("decodeClarityValue", decode_clarity_value)?;
    cx.export_function("decodeClarityValueList", decode_clarity_value_array)?;
    cx.export_function("decodePostConditions", decode_tx_post_conditions)?;
    cx.export_function("decodeTransaction", decode_transaction)?;
    cx.export_function("stacksToBitcoinAddress", stacks_to_bitcoin_address)?;
    cx.export_function("bitcoinToStacksAddress", bitcoin_to_stacks_address)?;
    cx.export_function("isValidStacksAddress", is_valid_stacks_address)?;
    cx.export_function("decodeStacksAddress", decode_stacks_address)?;
    cx.export_function("decodeClarityValueToPrincipal", decode_clarity_value_to_principal)?;
    cx.export_function("stacksAddressFromParts", stacks_address_from_parts)?;
    cx.export_function("memoToString", memo_to_string)?;
    cx.export_function("decodeNakamotoBlock", decode_nakamoto_block)?;
    cx.export_function("decodeNakamotoBlockHeader", decode_nakamoto_block_header)?;

    #[cfg(feature = "profiling")]
    {
        cx.export_function("startProfiler", start_profiler)?;
        cx.export_function("stopProfiler", stop_profiler)?;
    }

    Ok(())
}
