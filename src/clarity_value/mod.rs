use std::{convert::TryInto, io::Cursor};

use neon::prelude::*;

use crate::neon_util::{arg_as_bytes, arg_as_bytes_copied};

use self::{neon_encoder::decode_clarity_val, types::ClarityValue};

pub mod deserialize;
pub mod neon_encoder;
pub mod types;

pub fn decode_clarity_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    let mut cursor: Cursor<&[u8]> = Cursor::new(&val_bytes);
    let clarity_value = ClarityValue::deserialize(&mut cursor, true)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;

    let root_obj = cx.empty_object();
    decode_clarity_val(&mut cx, &root_obj, &clarity_value, true, val_bytes)?;

    return Ok(root_obj);
}

pub fn decode_clarity_value_type_name(mut cx: FunctionContext) -> JsResult<JsString> {
    let type_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        ClarityValue::deserialize(&mut cursor, false)
            .map_err(|err| err.as_string())
            .map(|val| val.value.type_signature())
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(type_string))
}

pub fn decode_clarity_value_to_repr(mut cx: FunctionContext) -> JsResult<JsString> {
    let repr_string = arg_as_bytes(&mut cx, 0, |val_bytes| {
        let mut cursor = Cursor::new(val_bytes);
        ClarityValue::deserialize(&mut cursor, false)
            .map_err(|err| err.as_string())
            .map(|val| val.value.repr_string())
    })
    .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
    Ok(cx.string(repr_string))
}

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
        let mut byte_cursor = std::io::Cursor::new(val_slice);
        let val_len = val_slice.len() as u64;
        let mut i: u32 = 0;
        while byte_cursor.position() < val_len {
            let cursor_pos = byte_cursor.position();
            let clarity_value = ClarityValue::deserialize(&mut byte_cursor, deep)
                .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;
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
