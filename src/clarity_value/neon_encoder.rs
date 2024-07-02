use crate::address::c32::c32_address;
use crate::clarity_value;
use crate::hex::encode_hex;
use neon::prelude::*;

pub fn decode_clarity_val<T: AsRef<[u8]>>(
    cx: &mut FunctionContext,
    cur_obj: &Handle<JsObject>,
    val: &clarity_value::types::ClarityValue,
    deep: bool,
    bytes: T,
) -> NeonResult<()> {
    let repr_string = cx.string(val.value.repr_string());
    cur_obj.set(cx, "repr", repr_string)?;

    let hex = cx.string(encode_hex(bytes.as_ref()));
    cur_obj.set(cx, "hex", hex)?;

    let type_id = cx.number(val.value.type_prefix().to_u8());
    cur_obj.set(cx, "type_id", type_id)?;

    if deep {
        use clarity_value::types::Value::*;
        match &val.value {
            Int(val) => {
                let val_string = cx.string(val.to_string());
                cur_obj.set(cx, "value", val_string)?;
            }
            UInt(val) => {
                let val_string = cx.string(val.to_string());
                cur_obj.set(cx, "value", val_string)?;
            }
            Bool(val) => {
                let val_boolean = cx.boolean(*val);
                cur_obj.set(cx, "value", val_boolean)?;
            }
            Buffer(buff) => {
                let obj_buffer = cx.string(encode_hex(buff));
                cur_obj.set(cx, "buffer", obj_buffer)?;
            }
            List(data) => {
                let list_obj = JsArray::new(cx, data.len());
                for (i, x) in data.iter().enumerate() {
                    let item_obj = cx.empty_object();
                    decode_clarity_val(
                        cx,
                        &item_obj,
                        x,
                        deep,
                        x.serialized_bytes.as_ref().unwrap(),
                    )?;
                    list_obj.set(cx, i as u32, item_obj)?;
                }
                cur_obj.set(cx, "list", list_obj)?;
            }
            StringASCII(str_data) => {
                let data = cx.string(String::from_utf8_lossy(str_data));
                cur_obj.set(cx, "data", data)?;
            }
            StringUTF8(str_data) => {
                let utf8_bytes: Vec<u8> = str_data.iter().cloned().flatten().collect();
                let utf8_str = String::from_utf8_lossy(&utf8_bytes);
                let data = cx.string(utf8_str);
                cur_obj.set(cx, "data", data)?;
            }
            PrincipalStandard(standard_principal) => {
                let address_version = cx.number(standard_principal.0);
                cur_obj.set(cx, "address_version", address_version)?;

                let address_hash_bytes = cx.string(encode_hex(&standard_principal.1));
                cur_obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

                let address_string = c32_address(standard_principal.0, &standard_principal.1)
                    .or_else(|e| {
                        cx.throw_error(format!("Error converting to C32 address: {}", e))
                    })?;

                let address = cx.string(address_string);
                cur_obj.set(cx, "address", address)?;
            }
            PrincipalContract(contract_identifier) => {
                let address_version = cx.number(contract_identifier.issuer.0);
                cur_obj.set(cx, "address_version", address_version)?;

                let address_hash_bytes = cx.string(encode_hex(&contract_identifier.issuer.1));
                cur_obj.set(cx, "address_hash_bytes", address_hash_bytes)?;

                let address_string =
                    c32_address(contract_identifier.issuer.0, &contract_identifier.issuer.1)
                        .or_else(|e| {
                            cx.throw_error(format!("Error converting to C32 address: {}", e))
                        })?;

                let address = cx.string(address_string);
                cur_obj.set(cx, "address", address)?;

                let contract_name = cx.string(contract_identifier.name.as_str());
                cur_obj.set(cx, "contract_name", contract_name)?;
            }
            Tuple(val) => {
                let tuple_obj = cx.empty_object();
                for (key, value) in val.iter() {
                    let val_obj = cx.empty_object();
                    decode_clarity_val(
                        cx,
                        &val_obj,
                        value,
                        deep,
                        value.serialized_bytes.as_ref().unwrap(),
                    )?;
                    tuple_obj.set(cx, key.as_str(), val_obj)?;
                }
                cur_obj.set(cx, "data", tuple_obj)?;
            }
            OptionalSome(data) => {
                let option_obj = cx.empty_object();
                decode_clarity_val(
                    cx,
                    &option_obj,
                    data,
                    deep,
                    data.serialized_bytes.as_ref().unwrap(),
                )?;
                cur_obj.set(cx, "value", option_obj)?;
            }
            OptionalNone => {
                let value = cx.null();
                cur_obj.set(cx, "value", value)?;
            }
            ResponseOk(val) | ResponseErr(val) => {
                let response_obj = cx.empty_object();
                decode_clarity_val(
                    cx,
                    &response_obj,
                    &val,
                    deep,
                    val.serialized_bytes.as_ref().unwrap(),
                )?;
                cur_obj.set(cx, "value", response_obj)?;
            }
        };
    }
    Ok(())
}
