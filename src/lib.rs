use git_version::git_version;
use neon::prelude::*;

use crate::address::{
    bitcoin_to_stacks_address, decode_stacks_address, is_valid_stacks_address,
    stacks_address_from_parts, stacks_to_bitcoin_address,
};
use crate::clarity_value::{
    decode_clarity_value, decode_clarity_value_array, decode_clarity_value_to_repr,
    decode_clarity_value_type_name,
};
use crate::memo::memo_to_string;
use crate::post_condition::decode_tx_post_conditions;
use crate::stacks_tx::decode_transaction;

mod address;
mod clarity_value;
mod hex;
mod memo;
mod neon_util;
mod post_condition;
mod serialize_util;
mod stacks_tx;

const GIT_VERSION: &str = git_version!(
    args = ["--all", "--long", "--always"],
    fallback = "unavailable"
);

fn get_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.string(GIT_VERSION);
    Ok(version)
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
        cx.export_function("perfTestC32Encode", crate::address::perf_test_c32_encode)?;
        cx.export_function("perfTestC32Decode", crate::address::perf_test_c32_decode)?;
    }

    Ok(())
}
