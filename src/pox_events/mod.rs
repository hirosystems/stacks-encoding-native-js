use std::io::Cursor;

use neon::prelude::*;

use crate::clarity_value::types::ClarityValue;
use crate::neon_util::arg_as_bytes_copied;

use self::decode::decode_pox_synthetic_event;
use self::neon_encoder::encode_pox_event;
use self::types::StacksNetwork;

pub mod btc_address;
pub mod decode;
pub mod neon_encoder;
pub mod types;

/// Neon-exported function: decodePoxSyntheticEvent(arg: string | Buffer, network: string)
/// Returns a JS object or null.
pub fn decode_pox_event(mut cx: FunctionContext) -> JsResult<JsValue> {
    // Arg 0: hex string or Buffer (raw Clarity value bytes)
    let val_bytes = arg_as_bytes_copied(&mut cx, 0)?;

    // Arg 1: network string
    let network_str = cx.argument::<JsString>(1)?.value(&mut cx);
    let network = StacksNetwork::from_str(&network_str)
        .or_else(|e| cx.throw_error(e))?;

    // Deserialize Clarity value (no need for serialized_bytes since we don't output hex/repr)
    let mut cursor: Cursor<&[u8]> = Cursor::new(&val_bytes);
    let clarity_value = ClarityValue::deserialize(&mut cursor, false)
        .or_else(|e| cx.throw_error(format!("Error deserializing Clarity value: {}", e)))?;

    // Decode to PoX synthetic event
    let event = decode_pox_synthetic_event(&clarity_value, network)
        .or_else(|e| cx.throw_error(format!("Error decoding PoX synthetic event: {}", e)))?;

    match event {
        Some(evt) => {
            let obj = encode_pox_event(&mut cx, &evt)?;
            Ok(obj.upcast())
        }
        None => Ok(cx.null().upcast()),
    }
}
