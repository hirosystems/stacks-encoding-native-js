use address::{
    bitcoin_address,
    c32::{c32_address, c32_address_decode},
    stacks_address,
};
use blockstack_lib::chainstate::stacks::TransactionPostCondition;
use clarity_value::neon_encoder::decode_clarity_val;
use git_version::git_version;
use hex::encode_hex;
use lazy_static::lazy_static;
use neon::{prelude::*, types::buffer::TypedArray};
use neon_util::*;
use regex::Regex;
use stacks_common::codec::StacksMessageCodec;
use std::{convert::TryInto, io::Cursor, sync::Mutex};
use unicode_segmentation::UnicodeSegmentation;

use crate::stacks_tx::decode_transaction;

mod address;
mod clarity_value;
mod hex;
mod neon_util;
mod stacks_tx;
mod unicode_printable;

const GIT_VERSION: &str = git_version!(
    args = ["--all", "--long", "--always"],
    fallback = "unavailable"
);

fn get_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.string(GIT_VERSION);
    Ok(version)
}

fn decode_clarity_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let mut cursor: Cursor<&[u8]> = Cursor::new(&val_bytes);
    let clarity_value = clarity_value::types::Value::deserialize_read(&mut cursor, true)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;

    let root_obj = cx.empty_object();
    decode_clarity_val(&mut cx, &root_obj, &clarity_value, true, val_bytes)?;

    return Ok(root_obj);
}

fn decode_clarity_value_type_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let type_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        clarity_value::types::Value::deserialize_read(&mut cursor, false)
            .map_err(|err| err.as_string())
            .map(|val| val.value.type_signature())
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(type_string))
}

fn decode_clarity_value_to_repr(mut cx: FunctionContext) -> JsResult<JsString> {
    let repr_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        clarity_value::types::Value::deserialize_read(&mut cursor, false)
            .map_err(|err| err.as_string())
            .map(|val| val.value.repr_string())
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(repr_string))
}

fn decode_tx_post_conditions(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;
    let resp_obj = cx.empty_object();

    // first byte is post condition mode
    let post_condition_mode = cx.number(input_bytes[0]);
    resp_obj.set(&mut cx, "post_condition_mode", post_condition_mode)?;

    /*
    match post_condition_mode {
        1 => {
            let mode = cx.string("allow");
            resp_obj.set(&mut cx, "post_condition_mode", mode)?;
        }
        2 => {
            let mode = cx.string("deny");
            resp_obj.set(&mut cx, "post_condition_mode", mode)?;
        }
        _ => cx.throw_error(format!(
            "PostConditionMode byte must be either 1 or 2 but was {}",
            post_condition_mode
        ))?,
    };
    */
    let array_result = if input_bytes.len() > 4 {
        // next 4 bytes are array length
        let result_length = u32::from_be_bytes(input_bytes[1..5].try_into().or_else(|e| {
            cx.throw_error(format!(
                "Error reading post condition bytes {}, {}",
                encode_hex(&input_bytes),
                e
            ))
        })?);
        let array_result = JsArray::new(&mut cx, result_length);
        // next bytes are serialized post condition items
        let cursor = &mut &input_bytes[5..];
        let mut i: u32 = 0;
        while !cursor.is_empty() {
            let post_condition =
                TransactionPostCondition::consensus_deserialize(cursor).or_else(|e| {
                    cx.throw_error(format!("Error deserializing post condition: {}", e))
                })?;
            let value_obj = cx.empty_object();
            post_condition.neon_js_serialize(&mut cx, &value_obj, &())?;
            array_result.set(&mut cx, i, value_obj)?;
            i = i + 1;
        }
        array_result
    } else {
        cx.empty_array()
    };
    resp_obj.set(&mut cx, "post_conditions", array_result)?;
    Ok(resp_obj)
}

fn decode_clarity_value_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    let input_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let result_length = if input_bytes.len() >= 4 {
        u32::from_be_bytes(input_bytes[..4].try_into().unwrap())
    } else {
        0
    };

    let array_result = JsArray::new(&mut cx, result_length);

    let deep: bool = match cx.argument_opt(1) {
        Some(arg) => arg
            .downcast_or_throw::<JsBoolean, _>(&mut cx)?
            .value(&mut cx),
        None => false,
    };

    if input_bytes.len() > 4 {
        let val_slice = &input_bytes[4..];
        let mut byte_cursor = std::io::Cursor::new(val_slice);
        let val_len = val_slice.len() as u64;
        let mut i: u32 = 0;
        while byte_cursor.position() < val_len - 1 {
            let cursor_pos = byte_cursor.position();
            let clarity_value =
                clarity_value::types::Value::deserialize_read(&mut byte_cursor, deep).or_else(
                    |e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)),
                )?;
            let decoded_bytes =
                &byte_cursor.get_ref()[cursor_pos as usize..byte_cursor.position() as usize];
            let value_obj = cx.empty_object();
            decode_clarity_val(&mut cx, &value_obj, &clarity_value, deep, decoded_bytes)?;
            array_result.set(&mut cx, i, value_obj)?;
            i = i + 1;
        }
    }
    Ok(array_result)
}

fn is_valid_stacks_address(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let address = c32_address_decode(&address_string);
    match address {
        Ok(_) => Ok(cx.boolean(true)),
        Err(_) => Ok(cx.boolean(false)),
    }
}

fn decode_stacks_address(mut cx: FunctionContext) -> JsResult<JsArray> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let address = c32_address_decode(&address_string)
        .or_else(|e| cx.throw_error(format!("Error parsing Stacks address {}", e)))?;
    let version = cx.number(address.0);

    let mut hash160 = unsafe { JsBuffer::uninitialized(&mut cx, address.1.len()) }?;
    hash160.as_mut_slice(&mut cx).copy_from_slice(&address.1);

    let array_resp = JsArray::new(&mut cx, 2);
    array_resp.set(&mut cx, 0, version)?;
    array_resp.set(&mut cx, 1, hash160)?;
    Ok(array_resp)
}

fn stacks_address_from_parts(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let stacks_address = arg_as_bytes(&mut cx, 1, |bytes| {
        let addr = c32_address(version as u8, bytes)
            .or_else(|e| Err(format!("Error converting to C32 address: {}", e)))?;
        Ok(addr)
    })
    .or_else(|e| cx.throw_error(e)?)?;
    let resp = cx.string(stacks_address);
    Ok(resp)
}

fn stacks_to_bitcoin_address_internal(input: String) -> Result<String, String> {
    let stacks_address = stacks_address::StacksAddress::from_string(&input)?;
    let bitcoin_address = address::stx_addr_to_btc_addr(&stacks_address);
    Ok(bitcoin_address)
}

fn stacks_to_bitcoin_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let stacks_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let btc_address =
        stacks_to_bitcoin_address_internal(stacks_address_arg).or_else(|e| cx.throw_error(e))?;
    let btc_address = cx.string(btc_address);
    Ok(btc_address)
}

fn bitcoin_to_stacks_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let bitcoin_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let bitcoin_address = bitcoin_address::from_b58(&bitcoin_address_arg)
        .or_else(|e| cx.throw_error(format!("Error parsing Bitcoin address: {}", e)))?;

    let stacks_addr_version =
        address::btc_addr_to_stx_addr_version(&bitcoin_address).or_else(|e| {
            cx.throw_error(format!(
                "Error getting Stacks address version from Bitcoin address: {}",
                e
            ))
        })?;

    let stacks_addr = c32_address(stacks_addr_version, &bitcoin_address.hash160_bytes)
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;

    Ok(cx.string(stacks_addr))
}

fn memo_normalize<T: AsRef<[u8]>>(input: T) -> String {
    let memo_str = String::from_utf8_lossy(input.as_ref());
    let mut result_str: String = String::with_capacity(memo_str.len());
    for g in memo_str.graphemes(true) {
        let chars: Vec<char> = g.chars().collect();
        // If char length is greater than one, assume printable grapheme cluster
        if chars.len() == 1 {
            if unicode_printable::is_printable(chars[0]) {
                result_str.push(chars[0]);
            } else {
                result_str.push(' ');
            }
        } else {
            result_str.push_str(g);
        }
    }
    lazy_static! {
        // Match one or more spans of `�` (unicode replacement character) and/or `\s` (whitespace)
        static ref UNICODE_REPLACEMENT_RE: Regex = Regex::new(r"(\u{FFFD}|\s)+").unwrap();
    }
    let memo_no_unknown = UNICODE_REPLACEMENT_RE.replace_all(&result_str, " ");
    let memo_no_invalid = memo_no_unknown.trim();
    memo_no_invalid.to_string()
}

fn memo_to_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let normalized = arg_as_bytes(&mut cx, 0, |input_bytes| Ok(memo_normalize(input_bytes)))
        .or_else(|e| cx.throw_error(e))?;
    let str_result = cx.string(normalized);
    Ok(str_result)
}

#[cfg(feature = "profiling")]
fn perf_test_c32_encode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    use rand::Rng;
    let mut inputs: Vec<(u8, [u8; 20])> = vec![];
    for _ in 0..2000 {
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        inputs.push((random_version, random_bytes));
    }

    let profiler = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;

    for (version, bytes) in inputs {
        for _ in 0..50_000 {
            c32_address(version, &bytes).unwrap();
        }
    }

    let report = profiler.report().build().unwrap();
    let mut buf = Vec::new();
    report
        .flamegraph(&mut buf)
        .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;

    let result = JsBuffer::external(&mut cx, buf);
    Ok(result)
}

#[cfg(feature = "profiling")]
fn perf_test_c32_decode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    use rand::Rng;
    let mut inputs: Vec<String> = vec![];
    for _ in 0..2000 {
        let random_version: u8 = rand::thread_rng().gen_range(0..31);
        let random_bytes = rand::thread_rng().gen::<[u8; 20]>();
        let addr = c32_address(random_version, &random_bytes).unwrap();
        inputs.push(addr);
    }

    let profiler = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;

    for _ in 0..50_000 {
        for addr in &inputs {
            c32_address_decode(&addr).unwrap();
        }
    }

    let report = profiler.report().build().unwrap();
    let mut buf = Vec::new();
    report
        .flamegraph(&mut buf)
        .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;

    let result = JsBuffer::external(&mut cx, buf);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        let input = b"hello world";
        let hex_str = encode_hex(input);
        let repr = hex_str.to_string();
        assert_eq!(repr, "0x68656c6c6f20776f726c64");
    }

    #[test]
    fn test_memo_decode_whitespace() {
        let input = "hello   world";
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unknown_unicode() {
        let input = "hello�world  test part1   goodbye�world  test part2     ";
        let output1 = memo_normalize(input);
        let expected1 = "hello world test part1 goodbye world test part2";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_misc_btc_coinbase() {
        let input = hex::decode_hex("037e180b04956b4e68627463706f6f6c2f3266646575fabe6d6df77973b452568eb2f43593285804dad9d7ef057eada5ff9f2a1634ec43f514b1020000008e9b20aa0ebfd204924b040000000000").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "~ kNhbtcpool/2fdeu mm ys RV 5 (X ~ * 4 C K";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_misc_btc_coinbase_2() {
        let input = hex::decode_hex("037c180b2cfabe6d6d5e0eb001a2eaea9c5e39b7f54edd5c23eb6e684dab1995191f664658064ba7dc10000000f09f909f092f4632506f6f6c2f6500000000000000000000000000000000000000000000000000000000000000000000000500f3fa0200").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "| , mm^ ^9 N \\# nhM fFX K 🐟 /F2Pool/e";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_grapheme_extended() {
        let input = "👩‍👩‍👧‍👦 hello world";
        let output1 = memo_normalize(input);
        let expected1 = "👩‍👩‍👧‍👦 hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unicode() {
        let input = hex::decode_hex("f09f87b3f09f87b12068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "🇳🇱 hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_start() {
        let input = hex::decode_hex("00000000000068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_end() {
        let input = hex::decode_hex("68656c6c6f20776f726c64000000000000").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_padded_middle() {
        let input =
            hex::decode_hex("68656c6c6f20776f726c6400000000000068656c6c6f20776f726c64").unwrap();
        let output1 = memo_normalize(input);
        let expected1 = "hello world hello world";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_unicode_scalar() {
        let input = "hello worldy̆ test";
        let output1 = memo_normalize(input);
        let expected1 = "hello worldy̆ test";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_memo_decode_zero_width_joiner() {
        let input = "👨\u{200D}👩";
        let output1 = memo_normalize(input);
        let expected1 = "👨‍👩";
        assert_eq!(output1, expected1);
    }

    #[test]
    fn test_stacks_to_bitcoin_address_mainnet() {
        let input = "SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM";
        let output = stacks_to_bitcoin_address_internal(input.to_string()).unwrap();
        let expected = "1FhZqHcrXaWcNCJPEGn2BRZ9angJvYfTBT";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_stacks_to_bitcoin_address_testnet() {
        let input = "ST2M9C0SHDV4FMXF3R0P98H8GQPW5824DVEJ9MVQZ";
        let output = stacks_to_bitcoin_address_internal(input.to_string()).unwrap();
        let expected = "mvtMXL9MYH8HaNz7u9AgapGqoFYpNDfKBx";
        assert_eq!(output, expected);
    }

    /*
    #[test]
    fn test_bitcoin_to_stacks_address_mainnet() {
        let input = "1FhZqHcrXaWcNCJPEGn2BRZ9angJvYfTBT";
        let output = stacks_address_from_bitcoin_address(input.to_string()).unwrap().to_string();
        let expected = "SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_bitcoin_to_stacks_address_testnet() {
        let input = "mvtMXL9MYH8HaNz7u9AgapGqoFYpNDfKBx";
        let output = stacks_address_from_bitcoin_address(input.to_string()).unwrap().to_string();
        let expected = "ST2M9C0SHDV4FMXF3R0P98H8GQPW5824DVEJ9MVQZ";
        assert_eq!(output, expected);
    }
    */
}

#[cfg(feature = "profiling")]
lazy_static! {
    static ref PROFILER: Mutex<Option<pprof::ProfilerGuard<'static>>> = Mutex::new(None);
}

#[cfg(feature = "profiling")]
fn start_profiler(mut cx: FunctionContext) -> JsResult<JsString> {
    let mut profiler = PROFILER
        .lock()
        .or_else(|e| cx.throw_error(format!("Failed to aquire lock: {}", e))?)?;
    if profiler.is_some() {
        cx.throw_error("Profiler already started")?;
    }
    let profiler_guard = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;
    *profiler = Some(profiler_guard);
    let res = cx.string("Profiler started");
    Ok(res)
}

#[cfg(feature = "profiling")]
fn create_profiler(mut cx: FunctionContext) -> JsResult<JsFunction> {
    let profiler_guard = pprof::ProfilerGuard::new(100)
        .or_else(|e| cx.throw_error(format!("Failed to create profiler guard: {}", e))?)?;
    let profiler_cell = std::cell::RefCell::new(profiler_guard);
    JsFunction::new(&mut cx, move |mut cx| {
        let profiler = profiler_cell.borrow_mut();
        let report = match profiler.report().build() {
            Ok(report) => report,
            Err(err) => cx.throw_error(format!("Error generating report: {}", err))?,
        };
        // let report_str = format!("{:?}", report);
        // Ok(cx.string(report_str))

        let mut buf = Vec::new();
        report
            .flamegraph(&mut buf)
            .or_else(|e| cx.throw_error(format!("Error creating flamegraph: {}", e)))?;
        let result = JsBuffer::external(&mut cx, buf);
        Ok(result)
    })
}

/*
#[cfg(feature = "profiling")]
fn stop_profiler_pprof(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let mut profiler = PROFILER
        .lock()
        .or_else(|e| cx.throw_error(format!("Failed to aquire lock: {}", e))?)?;
    let report_result = match &*profiler {
        None => cx.throw_error("No profiler started")?,
        Some(profiler) => profiler.report().build(),
    };
    let report = match report_result {
        Ok(report) => report,
        Err(err) => cx.throw_error(format!("Error generating report: {}", err))?,
    };

    let mut buf = Vec::new();

    let profile = report
        .pprof()
        .or_else(|e| cx.throw_error(format!("Error creating pprof: {}", e)))?;
    pprof::protos::Message::encode(&profile, &mut buf)
        .or_else(|e| cx.throw_error(format!("Error encoding pprof profile: {}", e)))?;

    *profiler = None;

    let result = JsBuffer::external(&mut cx, buf);
    Ok(result)
}
*/

#[cfg(feature = "profiling")]
fn stop_profiler(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let mut profiler = PROFILER
        .lock()
        .or_else(|e| cx.throw_error(format!("Failed to aquire lock: {}", e))?)?;
    let report_result = match &*profiler {
        None => cx.throw_error("No profiler started")?,
        Some(profiler) => profiler.report().build(),
    };
    let report = match report_result {
        Ok(report) => report,
        Err(err) => cx.throw_error(format!("Error generating report: {}", err))?,
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
    cx.export_function(
        "decodeClarityValueToTypeName",
        decode_clarity_value_type_name,
    )?;
    cx.export_function("decodeClarityValue", decode_clarity_value)?;
    cx.export_function("decodeClarityValueList", decode_clarity_value_array)?;
    cx.export_function("decodePostConditions", decode_tx_post_conditions)?;
    cx.export_function("decodeTransaction", decode_transaction)?;
    cx.export_function("stacksToBitcoinAddress", stacks_to_bitcoin_address)?;
    cx.export_function("bitcoinToStacksAddress", bitcoin_to_stacks_address)?;
    cx.export_function("isValidStacksAddress", is_valid_stacks_address)?;
    cx.export_function("decodeStacksAddress", decode_stacks_address)?;
    cx.export_function("stacksAddressFromParts", stacks_address_from_parts)?;
    cx.export_function("memoToString", memo_to_string)?;

    #[cfg(feature = "profiling")]
    {
        cx.export_function("startProfiler", start_profiler)?;
        cx.export_function("stopProfiler", stop_profiler)?;
        cx.export_function("createProfiler", create_profiler)?;
        cx.export_function("perfTestC32Encode", perf_test_c32_encode)?;
        cx.export_function("perfTestC32Decode", perf_test_c32_decode)?;
    }

    Ok(())
}
