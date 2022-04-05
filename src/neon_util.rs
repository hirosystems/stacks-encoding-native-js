use neon::{prelude::*, types::buffer::TypedArray};

use crate::hex::decode_hex;

pub trait NeonJsSerialize<ExtraCtx = (), TResult = ()> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &ExtraCtx,
    ) -> NeonResult<TResult>;
}

/*
pub fn eval<'a, 'b, C: Context<'a>>(
    cx: &mut C,
    script: Handle<'b, JsString>,
) -> JsResult<'a, JsValue> {
    let env = cx.env().to_raw();
    build(cx.env(), |out| unsafe {
        neon_runtime::string::run_script(out, env, script.to_raw())
    })
}
*/

#[allow(dead_code)]
pub fn console_log<'a, C: Context<'a>, S: AsRef<str>>(cx: &mut C, msg: S) -> NeonResult<()> {
    let console_global: Handle<JsObject> = cx.global().get(cx, "console")?;
    let log_fn: Handle<JsFunction> = console_global.get(cx, "log")?;
    log_fn
        .call_with(cx)
        .arg(cx.string(msg))
        .apply::<JsValue, _>(cx)?;
    Ok(())
}

#[allow(dead_code)]
pub fn console_log_val(cx: &mut FunctionContext, msg: Handle<JsValue>) -> NeonResult<()> {
    let console_global: Handle<JsObject> = cx.global().get(cx, "console")?;
    let log_fn: Handle<JsFunction> = console_global.get(cx, "log")?;
    log_fn.call_with(cx).arg(msg).apply::<JsValue, _>(cx)?;
    Ok(())
}

#[allow(dead_code)]
pub fn json_parse<'a, C: Context<'a>, S: AsRef<str>>(
    cx: &mut C,
    input: S,
) -> NeonResult<Handle<'a, JsValue>> {
    let json_global: Handle<JsObject> = cx.global().get(cx, "JSON")?;
    let json_parse: Handle<JsFunction> = json_global.get(cx, "parse")?;
    let result: Handle<JsValue> = json_parse.call_with(cx).arg(cx.string(input)).apply(cx)?;
    Ok(result)
}

pub fn arg_as_bytes_copied(cx: &mut FunctionContext, arg_index: i32) -> NeonResult<Box<[u8]>> {
    let input_arg: Handle<JsValue> = cx.argument(arg_index)?;
    if let Ok(handle) = input_arg.downcast::<JsString, _>(cx) {
        let val_bytes = decode_hex(handle.value(cx))
            .or_else(|e| cx.throw_error(format!("Hex parsing error: {}", e)))?;
        Ok(val_bytes)
    } else if let Ok(handle) = input_arg.downcast::<JsBuffer, _>(cx) {
        let slice = handle.as_slice(cx);
        let val_bytes: Box<[u8]> = slice.into();
        Ok(val_bytes)
    } else {
        cx.throw_error("Argument must be a hex string or a Buffer")
    }
}

pub fn arg_as_bytes<F, T>(cx: &mut FunctionContext, arg_index: i32, cb: F) -> Result<T, String>
where
    F: Fn(&[u8]) -> Result<T, String>,
{
    let input_arg: Handle<JsValue> = cx
        .argument(arg_index)
        .or_else(|e| Err(format!("Error getting function arg {}: {}", arg_index, e)))?;
    if let Ok(handle) = input_arg.downcast::<JsString, _>(cx) {
        let val_bytes =
            decode_hex(handle.value(cx)).or_else(|e| Err(format!("Hex parsing error: {}", e)))?;
        cb(&val_bytes)
    } else if let Ok(handle) = input_arg.downcast::<JsBuffer, _>(cx) {
        let slice = handle.as_slice(cx);
        cb(slice)
    } else {
        Err("Argument must be a hex string or a Buffer".to_string())
    }
}
