use blockstack_lib::{
    address::AddressHashMode,
    chainstate::stacks::{
        AssetInfo, AssetInfoID, PostConditionPrincipal, PostConditionPrincipalID,
        StacksMicroblockHeader, TransactionContractCall, TransactionPayload, TransactionPayloadID,
        TransactionPostCondition, TransactionSmartContract,
    },
    chainstate::stacks::{
        MultisigSpendingCondition, SinglesigSpendingCondition, StacksTransaction, TransactionAuth,
        TransactionAuthField, TransactionAuthFieldID, TransactionAuthFlags,
        TransactionPublicKeyEncoding, TransactionSpendingCondition, TransactionVersion,
    },
    types::{chainstate::StacksAddress, StacksPublicKeyBuffer},
    util::hash::Hash160,
    vm::{
        analysis::contract_interface_builder::ContractInterfaceAtomType,
        types::{CharType, PrincipalData, SequenceData},
    },
};
use clarity::vm::types::{
    signatures::TypeSignature as ClarityTypeSignature, Value as ClarityValue,
};
use git_version::git_version;
use hex::{FromHex, FromHexError};
use neon::{prelude::*, types::buffer::TypedArray};
use sha2::{Digest, Sha512_256};
use stacks_common::codec::StacksMessageCodec;
use std::convert::{TryFrom, TryInto};

const GIT_VERSION: &str = git_version!(
    args = ["--all", "--long", "--always"],
    fallback = "unavailable"
);

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

fn get_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let version = cx.string(GIT_VERSION);
    Ok(version)
}

fn decode_hex<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
    if data.as_ref()[0] == '0' as u8 && data.as_ref()[1] == 'x' as u8 {
        return decode_hex(&data.as_ref()[2..]);
    }
    FromHex::from_hex(data)
}

// Copied from non-public definition at
// https://github.com/stacks-network/stacks-blockchain/blob/65570bbd8f6b2549e9b4d5bbe4c934538ec0cedc/clarity/src/vm/types/serialization.rs#L111
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Copy)]
enum ClarityTypePrefix {
    Int = 0,
    UInt = 1,
    Buffer = 2,
    BoolTrue = 3,
    BoolFalse = 4,
    PrincipalStandard = 5,
    PrincipalContract = 6,
    ResponseOk = 7,
    ResponseErr = 8,
    OptionalNone = 9,
    OptionalSome = 10,
    List = 11,
    Tuple = 12,
    StringASCII = 13,
    StringUTF8 = 14,
}

fn console_log<S: AsRef<str>>(cx: &mut FunctionContext, msg: S) -> NeonResult<()> {
    let console_global: Handle<JsObject> = cx.global().get(cx, "console")?;
    let log_fn: Handle<JsFunction> = console_global.get(cx, "log")?;
    log_fn
        .call_with(cx)
        .arg(cx.string(msg))
        .apply::<JsValue, _>(cx)?;
    Ok(())
}

fn console_log_val(cx: &mut FunctionContext, msg: Handle<JsValue>) -> NeonResult<()> {
    let console_global: Handle<JsObject> = cx.global().get(cx, "console")?;
    let log_fn: Handle<JsFunction> = console_global.get(cx, "log")?;
    log_fn.call_with(cx).arg(msg).apply::<JsValue, _>(cx)?;
    Ok(())
}

fn json_parse<'a, C: Context<'a>, S: AsRef<str>>(
    cx: &mut C,
    input: S,
) -> NeonResult<Handle<'a, JsValue>> {
    let json_global: Handle<JsObject> = cx.global().get(cx, "JSON")?;
    let json_parse: Handle<JsFunction> = json_global.get(cx, "parse")?;
    let result: Handle<JsValue> = json_parse.call_with(cx).arg(cx.string(input)).apply(cx)?;
    Ok(result)
}

fn first_arg_as_bytes<'a>(cx: &mut FunctionContext<'a>) -> NeonResult<Vec<u8>> {
    let input_arg: Handle<JsValue> = cx.argument(0)?;
    if let Ok(handle) = input_arg.downcast::<JsString, _>(cx) {
        let val_bytes = decode_hex(handle.value(cx))
            .or_else(|e| cx.throw_error(format!("Hex parsing error: {}", e)))?;
        Ok(val_bytes)
    } else if let Ok(mut handle) = input_arg.downcast::<JsBuffer, _>(cx) {
        let slice = handle.as_mut_slice(cx);
        let val_bytes: Vec<u8> = slice.into();
        Ok(val_bytes)
    } else {
        cx.throw_error("Argument must be a hex string or a Buffer")
    }
}

fn decode_clarity_val(
    cx: &mut FunctionContext,
    cur_obj: &JsObject,
    val: &ClarityValue,
    include_abi_type: bool,
) -> NeonResult<()> {
    let type_signature = ClarityTypeSignature::type_of(&val);

    let signature_repr = cx.string(type_signature.to_string());
    cur_obj.set(cx, "signature_repr", signature_repr)?;

    if include_abi_type {
        let abi_type = ContractInterfaceAtomType::from_type_signature(&type_signature);
        // TODO: this is silly and slow, should deserialize the ContractInterfaceAtomType object directly into Neon JsObject
        let abi_json =
            serde_json::to_string(&abi_type).or_else(|e| cx.throw_error(format!("{}", e)))?;
        let abi_type_obj = json_parse(cx, abi_json)?;
        cur_obj.set(cx, "abi_type", abi_type_obj)?;
    }

    // TODO: is there a perfect overlap between stacks.js clarity json and contract ABI schema?

    match val {
        ClarityValue::Int(val) => {
            let type_id = cx.number(ClarityTypePrefix::Int as u8);
            cur_obj.set(cx, "type_id", type_id)?;
            let val_string = cx.string(val.to_string());
            cur_obj.set(cx, "value", val_string)?;
        }
        ClarityValue::UInt(val) => {
            let type_id = cx.number(ClarityTypePrefix::UInt as u8);
            cur_obj.set(cx, "type_id", type_id)?;
            let val_string = cx.string(val.to_string());
            cur_obj.set(cx, "value", val_string)?;
        }
        ClarityValue::Bool(val) => {
            match val {
                true => {
                    let type_id = cx.number(ClarityTypePrefix::BoolTrue as u8);
                    cur_obj.set(cx, "type_id", type_id)?
                }
                false => {
                    let type_id = cx.number(ClarityTypePrefix::BoolFalse as u8);
                    cur_obj.set(cx, "type_id", type_id)?
                }
            };
        }
        ClarityValue::Sequence(val) => match val {
            SequenceData::Buffer(buff) => {
                let type_id = cx.number(ClarityTypePrefix::Buffer as u8);
                cur_obj.set(cx, "type_id", type_id)?;
                let obj_buffer = JsBuffer::external(cx, buff.data.to_vec());
                cur_obj.set(cx, "buffer", obj_buffer)?;
            }
            SequenceData::List(list) => {
                let type_id = cx.number(ClarityTypePrefix::List as u8);
                cur_obj.set(cx, "type_id", type_id)?;
                let list_obj = JsArray::new(cx, list.len());
                for (i, x) in list.data.iter().enumerate() {
                    let item_obj = cx.empty_object();
                    decode_clarity_val(cx, &item_obj, x, include_abi_type)?;
                    list_obj.set(cx, i as u32, item_obj)?;
                }
                cur_obj.set(cx, "list", list_obj)?;
            }
            SequenceData::String(str) => match str {
                CharType::ASCII(str_data) => {
                    let type_id = cx.number(ClarityTypePrefix::StringASCII as u8);
                    cur_obj.set(cx, "type_id", type_id)?;
                    let data = cx.string(str_data.to_string());
                    cur_obj.set(cx, "data", data)?;
                }
                CharType::UTF8(str_data) => {
                    let type_id = cx.number(ClarityTypePrefix::StringUTF8 as u8);
                    cur_obj.set(cx, "type_id", type_id)?;
                    let data = cx.string(str_data.to_string());
                    cur_obj.set(cx, "data", data)?;
                }
            },
        },
        ClarityValue::Principal(val) => match val {
            PrincipalData::Standard(standard_principal) => {
                let type_id = cx.number(ClarityTypePrefix::PrincipalStandard as u8);
                cur_obj.set(cx, "type_id", type_id)?;
                let address_string = cx.string(standard_principal.to_address().as_str());
                cur_obj.set(cx, "address", address_string)?;
            }
            PrincipalData::Contract(contract_identifier) => {
                let type_id = cx.number(ClarityTypePrefix::PrincipalContract as u8);
                cur_obj.set(cx, "type_id", type_id)?;
                let address_string = cx.string(contract_identifier.issuer.to_address().as_str());
                cur_obj.set(cx, "address", address_string)?;
                let contract_name = cx.string(contract_identifier.name.as_str());
                cur_obj.set(cx, "contract_name", contract_name)?;
            }
        },
        ClarityValue::Tuple(val) => {
            let type_id = cx.number(ClarityTypePrefix::Tuple as u8);
            cur_obj.set(cx, "type_id", type_id)?;
            let tuple_obj = cx.empty_object();
            for (key, value) in val.data_map.iter() {
                let val_obj = cx.empty_object();
                decode_clarity_val(cx, &val_obj, value, include_abi_type)?;
                tuple_obj.set(cx, key.as_str(), val_obj)?;
            }
            cur_obj.set(cx, "data", tuple_obj)?;
        }
        ClarityValue::Optional(val) => match &val.data {
            Some(data) => {
                let type_id = cx.number(ClarityTypePrefix::OptionalSome as u8);
                cur_obj.set(cx, "type_id", type_id)?;
                let option_obj = cx.empty_object();
                decode_clarity_val(cx, &option_obj, &data, include_abi_type)?;
                cur_obj.set(cx, "value", option_obj)?;
            }
            None => {
                let type_id = cx.number(ClarityTypePrefix::OptionalNone as u8);
                cur_obj.set(cx, "type_id", type_id)?;
            }
        },
        ClarityValue::Response(val) => {
            if val.committed {
                let type_id = cx.number(ClarityTypePrefix::ResponseOk as u8);
                cur_obj.set(cx, "type_id", type_id)?;
            } else {
                let type_id = cx.number(ClarityTypePrefix::ResponseErr as u8);
                cur_obj.set(cx, "type_id", type_id)?;
            }
            let response_obj = cx.empty_object();
            decode_clarity_val(cx, &response_obj, &val.data, include_abi_type)?;
            cur_obj.set(cx, "value", response_obj)?;
        }
    };
    Ok(())
}

fn decode_clarity_value_to_json(mut cx: FunctionContext) -> JsResult<JsObject> {
    let val_bytes = first_arg_as_bytes(&mut cx)?;
    let cursor = &mut &val_bytes[..];
    let clarity_value = ClarityValue::consensus_deserialize(cursor)
        .or_else(|e| cx.throw_error(format!("Clarity parsing error: {}", e)))?;

    let include_abi_types_arg = cx.argument_opt(1);
    let include_abi_types = match include_abi_types_arg {
        Some(arg) => arg
            .downcast_or_throw::<JsBoolean, _>(&mut cx)?
            .value(&mut cx),
        None => false,
    };

    let repr_str = cx.string(format!("{}", clarity_value));
    let root_obj = cx.empty_object();
    decode_clarity_val(&mut cx, &root_obj, &clarity_value, include_abi_types)?;

    let resp_obj = cx.empty_object();
    resp_obj.set(&mut cx, "repr", repr_str)?;
    resp_obj.set(&mut cx, "value", root_obj)?;

    return Ok(resp_obj);
}

fn decode_clarity_value_to_repr(mut cx: FunctionContext) -> JsResult<JsString> {
    let val_bytes = first_arg_as_bytes(&mut cx)?;
    let cursor = &mut &val_bytes[..];
    let clarity_value = ClarityValue::consensus_deserialize(cursor)
        .or_else(|e| cx.throw_error(format!("Clarity parsing error: {}", e)))?;
    Ok(cx.string(format!("{}", clarity_value)))
}

fn decode_tx_post_conditions(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input_bytes = first_arg_as_bytes(&mut cx)?;
    // first byte is post condition mode
    let post_condition_mode = input_bytes[0];
    let resp_obj = cx.empty_object();
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
    // next 4 bytes are array length
    let result_length = u32::from_be_bytes(input_bytes[1..5].try_into().unwrap());
    let array_result = JsArray::new(&mut cx, result_length);
    // next bytes are serialized post condition items
    let cursor = &mut &input_bytes[5..];
    let mut i: u32 = 0;
    while !cursor.is_empty() {
        let post_condition = TransactionPostCondition::consensus_deserialize(cursor)
            .or_else(|e| cx.throw_error(format!("Error deserializing post condition: {}", e)))?;
        let value_obj = cx.empty_object();
        post_condition.neon_js_serialize(&mut cx, &value_obj, &())?;
        array_result.set(&mut cx, i, value_obj)?;
        i = i + 1;
    }
    resp_obj.set(&mut cx, "post_conditions", array_result)?;
    Ok(resp_obj)
}

fn decode_clarity_value_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    let input_bytes = first_arg_as_bytes(&mut cx)?;
    let result_length = u32::from_be_bytes(input_bytes[..4].try_into().unwrap());
    let array_result = JsArray::new(&mut cx, result_length);

    let val_slice = &input_bytes[4..];
    let mut byte_cursor = std::io::Cursor::new(val_slice);
    let val_len = val_slice.len() as u64;
    let mut i: u32 = 0;
    while byte_cursor.position() < val_len - 1 {
        let cur_start = byte_cursor.position() as usize;
        let clarity_value = ClarityValue::consensus_deserialize(&mut byte_cursor)
            .or_else(|e| cx.throw_error(format!("{}", e)))?;
        let cur_end = byte_cursor.position() as usize;
        let value_slice = &val_slice[cur_start..cur_end];
        let value_hex = cx.string(format!("0x{}", hex::encode(value_slice)));
        let value_type = cx.string(ClarityTypeSignature::type_of(&clarity_value).to_string());
        let value_repr = cx.string(clarity_value.to_string());
        let value_obj = cx.empty_object();
        let value_buff = JsBuffer::external(&mut cx, value_slice.to_vec());
        value_obj.set(&mut cx, "type", value_type)?;
        value_obj.set(&mut cx, "repr", value_repr)?;
        value_obj.set(&mut cx, "hex", value_hex)?;
        value_obj.set(&mut cx, "buffer", value_buff)?;
        array_result.set(&mut cx, i, value_obj)?;
        i = i + 1;
    }
    Ok(array_result)
}

fn decode_transaction(mut cx: FunctionContext) -> JsResult<JsObject> {
    let input_bytes = first_arg_as_bytes(&mut cx)?;
    let byte_cursor = &mut &input_bytes[..];
    let tx = StacksTransaction::consensus_deserialize(byte_cursor)
        .or_else(|e| cx.throw_error(format!("Failed to decode transaction: {:?}\n", &e)))?;
    let tx_json_obj = cx.empty_object();

    let tx_id_bytes = Sha512_256::digest(input_bytes);
    let tx_id = cx.string(format!("0x{}", hex::encode(tx_id_bytes)));
    tx_json_obj.set(&mut cx, "tx_id", tx_id)?;

    tx.neon_js_serialize(&mut cx, &tx_json_obj, &())?;
    // let tx_json = serde_json::to_string(&tx).or_else(|e| cx.throw_error(format!("Failed to serialize transaction to JSON: {}", e)))?;
    Ok(tx_json_obj)
}

pub trait NeonJsSerialize<ExtraCtx = (), TResult = ()> {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &ExtraCtx,
    ) -> NeonResult<TResult>;
}

impl NeonJsSerialize for StacksTransaction {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version_number = cx.number(self.version as u8);
        obj.set(cx, "version", version_number)?;

        let chain_id = cx.number(self.chain_id);
        obj.set(cx, "chain_id", chain_id)?;

        let auth_obj = cx.empty_object();
        self.auth.neon_js_serialize(
            cx,
            &auth_obj,
            &TxSerializationContext {
                transaction_version: self.version,
            },
        )?;
        obj.set(cx, "auth", auth_obj)?;

        let anchor_mode = cx.number(self.anchor_mode as u8);
        obj.set(cx, "anchor_mode", anchor_mode)?;

        let post_condition_mode = cx.number(self.post_condition_mode as u8);
        obj.set(cx, "post_condition_mode", post_condition_mode)?;

        // TODO: raw post conditions binary slice is already determined during raw tx deserialization, ideally
        // try to use that rather than re-serializing (slow)
        let mut post_conditions_raw = u32::to_be_bytes(self.post_conditions.len() as u32).to_vec();
        let post_conditions = JsArray::new(cx, self.post_conditions.len() as u32);
        for (i, x) in self.post_conditions.iter().enumerate() {
            let post_condition_obj = cx.empty_object();
            let mut val_bytes = x.neon_js_serialize(cx, &post_condition_obj, &())?;
            post_conditions_raw.append(&mut val_bytes);
            post_conditions.set(cx, i as u32, post_condition_obj)?;
        }
        obj.set(cx, "post_conditions", post_conditions)?;

        let post_conditions_buff = JsBuffer::external(cx, post_conditions_raw);
        obj.set(cx, "post_conditions_buffer", post_conditions_buff)?;

        let payload_obj = cx.empty_object();
        self.payload.neon_js_serialize(cx, &payload_obj, &())?;
        obj.set(cx, "payload", payload_obj)?;

        Ok(())
    }
}

struct TxSerializationContext {
    transaction_version: TransactionVersion,
}

impl NeonJsSerialize<TxSerializationContext> for TransactionAuth {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match *self {
            TransactionAuth::Standard(ref origin_condition) => {
                let type_id = cx.number(TransactionAuthFlags::AuthStandard as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_condition_obj = cx.empty_object();
                origin_condition.neon_js_serialize(cx, &origin_condition_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_condition_obj)?;
            }
            TransactionAuth::Sponsored(ref origin_condition, ref sponsor_condition) => {
                let type_id = cx.number(TransactionAuthFlags::AuthSponsored as u8);
                obj.set(cx, "type_id", type_id)?;

                let origin_condition_obj = cx.empty_object();
                origin_condition.neon_js_serialize(cx, &origin_condition_obj, extra_ctx)?;
                obj.set(cx, "origin_condition", origin_condition_obj)?;

                let sponsor_condition_obj = cx.empty_object();
                sponsor_condition.neon_js_serialize(cx, &sponsor_condition_obj, extra_ctx)?;
                obj.set(cx, "sponsor_condition", sponsor_condition_obj)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for TransactionSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        match *self {
            TransactionSpendingCondition::Singlesig(ref data) => {
                data.neon_js_serialize(cx, obj, &extra_ctx)?;
            }
            TransactionSpendingCondition::Multisig(ref data) => {
                data.neon_js_serialize(cx, obj, &extra_ctx)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for SinglesigSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let hash_mode_int = self.hash_mode.clone() as u8;

        let hash_mode = cx.number(hash_mode_int);
        obj.set(cx, "hash_mode", hash_mode)?;

        let signer = cx.string(format!("0x{}", hex::encode(&self.signer)));
        obj.set(cx, "signer", signer)?;

        let stacks_address_hash_mode = AddressHashMode::try_from(hash_mode_int).unwrap();
        let stacks_address_version = match extra_ctx.transaction_version {
            TransactionVersion::Mainnet => stacks_address_hash_mode.to_version_mainnet(),
            TransactionVersion::Testnet => stacks_address_hash_mode.to_version_testnet(),
        };
        let stacks_address =
            cx.string(StacksAddress::new(stacks_address_version, self.signer).to_string());
        obj.set(cx, "signer_stacks_address", stacks_address)?;

        // TODO: bigint
        let nonce = cx.string(self.nonce.to_string());
        obj.set(cx, "nonce", nonce)?;

        // TODO: bigint
        let tx_fee = cx.string(self.tx_fee.to_string());
        obj.set(cx, "tx_fee", tx_fee)?;

        let key_encoding = cx.number(self.key_encoding as u8);
        obj.set(cx, "key_encoding", key_encoding)?;

        let signature = cx.string(format!("0x{}", hex::encode(&self.signature)));
        obj.set(cx, "signature", signature)?;

        Ok(())
    }
}

impl NeonJsSerialize<TxSerializationContext> for MultisigSpendingCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &TxSerializationContext,
    ) -> NeonResult<()> {
        let hash_mode_int = self.hash_mode.clone() as u8;
        let hash_mode = cx.number(hash_mode_int);
        obj.set(cx, "hash_mode", hash_mode)?;

        let signer = cx.string(format!("0x{}", hex::encode(&self.signer)));
        obj.set(cx, "signer", signer)?;

        let stacks_address_hash_mode = AddressHashMode::try_from(hash_mode_int).unwrap();
        let stacks_address_version = match extra_ctx.transaction_version {
            TransactionVersion::Mainnet => stacks_address_hash_mode.to_version_mainnet(),
            TransactionVersion::Testnet => stacks_address_hash_mode.to_version_testnet(),
        };
        let stacks_address =
            cx.string(StacksAddress::new(stacks_address_version, self.signer).to_string());
        obj.set(cx, "signer_stacks_address", stacks_address)?;

        // TODO: bigint
        let nonce = cx.string(self.nonce.to_string());
        obj.set(cx, "nonce", nonce)?;

        // TODO: bigint
        let tx_fee = cx.string(self.tx_fee.to_string());
        obj.set(cx, "tx_fee", tx_fee)?;

        let fields = JsArray::new(cx, self.fields.len().try_into().unwrap());
        for (i, x) in self.fields.iter().enumerate() {
            let field_obj = cx.empty_object();
            x.neon_js_serialize(cx, &field_obj, &())?;
            fields.set(cx, i as u32, field_obj)?;
        }
        obj.set(cx, "fields", fields)?;

        let signatures_required = cx.number(self.signatures_required);
        obj.set(cx, "signatures_required", signatures_required)?;

        Ok(())
    }
}

impl NeonJsSerialize for TransactionAuthField {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            TransactionAuthField::PublicKey(ref pubkey) => {
                let field_id = if pubkey.compressed() {
                    TransactionAuthFieldID::PublicKeyCompressed
                } else {
                    TransactionAuthFieldID::PublicKeyUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let pubkey_buf = StacksPublicKeyBuffer::from_public_key(pubkey);
                let pubkey_hex = cx.string(format!("0x{}", hex::encode(pubkey_buf)));
                obj.set(cx, "public_key", pubkey_hex)?;

                // TODO: add stacks-address encoded format
                // let stacks_address = StacksAddress::from_public_keys().unwrap();
            }
            TransactionAuthField::Signature(ref key_encoding, ref sig) => {
                let field_id = if *key_encoding == TransactionPublicKeyEncoding::Compressed {
                    TransactionAuthFieldID::SignatureCompressed
                } else {
                    TransactionAuthFieldID::SignatureUncompressed
                };
                let type_id = cx.number(field_id as u8);
                obj.set(cx, "type_id", type_id)?;

                let pubkey_hex = cx.string(format!("0x{}", hex::encode(sig)));
                obj.set(cx, "signature", pubkey_hex)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize<(), Vec<u8>> for TransactionPostCondition {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<Vec<u8>> {
        match *self {
            TransactionPostCondition::STX(ref principal, ref fungible_condition, ref amount) => {
                let asset_info_id = cx.number(AssetInfoID::STX as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let condition_code = cx.number(*fungible_condition as u8);
                obj.set(cx, "condition_code", condition_code)?;

                // TODO: bigint
                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Fungible(
                ref principal,
                ref asset_info,
                ref fungible_condition,
                ref amount,
            ) => {
                let asset_info_id = cx.number(AssetInfoID::FungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj, extra_ctx)?;
                obj.set(cx, "asset", asset_info_obj)?;

                let condition_code = cx.number(*fungible_condition as u8);
                obj.set(cx, "condition_code", condition_code)?;

                // TODO: bigint
                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;
            }
            TransactionPostCondition::Nonfungible(
                ref principal,
                ref asset_info,
                ref asset_value,
                ref nonfungible_condition,
            ) => {
                let asset_info_id = cx.number(AssetInfoID::NonfungibleAsset as u8);
                obj.set(cx, "asset_info_id", asset_info_id)?;

                let pricipal_obj = cx.empty_object();
                principal.neon_js_serialize(cx, &pricipal_obj, extra_ctx)?;
                obj.set(cx, "principal", pricipal_obj)?;

                let asset_info_obj = cx.empty_object();
                asset_info.neon_js_serialize(cx, &asset_info_obj, extra_ctx)?;
                obj.set(cx, "asset", asset_info_obj)?;

                let asset_value_obj = cx.empty_object();
                asset_value.neon_js_serialize(cx, &asset_value_obj, extra_ctx)?;
                obj.set(cx, "asset_value", asset_value_obj)?;

                let condition_code = cx.number(*nonfungible_condition as u8);
                obj.set(cx, "condition_code", condition_code)?;
            }
        };
        let value_bytes = TransactionPostCondition::serialize_to_vec(&self);
        Ok(value_bytes)
    }
}

impl NeonJsSerialize for PostConditionPrincipal {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            PostConditionPrincipal::Origin => {
                let type_id = cx.number(PostConditionPrincipalID::Origin as u8);
                obj.set(cx, "type_id", type_id)?;
            }
            PostConditionPrincipal::Standard(ref address) => {
                let type_id = cx.number(PostConditionPrincipalID::Standard as u8);
                obj.set(cx, "type_id", type_id)?;

                let address_str = cx.string(address.to_string());
                obj.set(cx, "address", address_str)?;
            }
            PostConditionPrincipal::Contract(ref address, ref contract_name) => {
                let type_id = cx.number(PostConditionPrincipalID::Contract as u8);
                obj.set(cx, "type_id", type_id)?;

                let address_str = cx.string(address.to_string());
                obj.set(cx, "address", address_str)?;

                let contract_str = cx.string(contract_name.to_string());
                obj.set(cx, "contract_name", contract_str)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for AssetInfo {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let contract_address = cx.string(self.contract_address.to_string());
        obj.set(cx, "contract_address", contract_address)?;

        let contract_name = cx.string(self.contract_name.to_string());
        obj.set(cx, "contract_name", contract_name)?;

        let asset_name = cx.string(self.asset_name.to_string());
        obj.set(cx, "asset_name", asset_name)?;
        Ok(())
    }
}

impl NeonJsSerialize<(), Vec<u8>> for ClarityValue {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<Vec<u8>> {
        // TODO: use the nested clarity value deserializer fn
        let value_bytes = ClarityValue::serialize_to_vec(&self);
        let value_hex = cx.string(format!("0x{}", hex::encode(&value_bytes)));
        let value_type = cx.string(ClarityTypeSignature::type_of(self).to_string());
        let value_repr = cx.string(self.to_string());
        // TODO: raw clarity value binary slice is already determined during deserialization, ideally
        // try to use that rather than re-serializing (slow)
        let value_buff = JsBuffer::external(cx, value_bytes.to_vec());
        obj.set(cx, "type", value_type)?;
        obj.set(cx, "repr", value_repr)?;
        obj.set(cx, "hex", value_hex)?;
        obj.set(cx, "buffer", value_buff)?;
        Ok(value_bytes)
    }
}

impl NeonJsSerialize for TransactionPayload {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        match *self {
            TransactionPayload::TokenTransfer(ref address, ref amount, ref memo) => {
                let type_id = cx.number(TransactionPayloadID::TokenTransfer as u8);
                obj.set(cx, "type_id", type_id)?;

                let recipient_obj = cx.empty_object();
                address.neon_js_serialize(cx, &recipient_obj, extra_ctx)?;
                obj.set(cx, "recipient", recipient_obj)?;

                let amount_str = cx.string(amount.to_string());
                obj.set(cx, "amount", amount_str)?;

                let memo_hex = cx.string(format!("0x{}", hex::encode(memo)));
                obj.set(cx, "memo", memo_hex)?;
            }
            TransactionPayload::ContractCall(ref contract_call) => {
                let type_id = cx.number(TransactionPayloadID::ContractCall as u8);
                obj.set(cx, "type_id", type_id)?;

                contract_call.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::SmartContract(ref smart_contract) => {
                let type_id = cx.number(TransactionPayloadID::SmartContract as u8);
                obj.set(cx, "type_id", type_id)?;

                smart_contract.neon_js_serialize(cx, obj, extra_ctx)?;
            }
            TransactionPayload::PoisonMicroblock(ref h1, ref h2) => {
                let type_id = cx.number(TransactionPayloadID::PoisonMicroblock as u8);
                obj.set(cx, "type_id", type_id)?;

                let microblock_header_1_obj = cx.empty_object();
                h1.neon_js_serialize(cx, &microblock_header_1_obj, extra_ctx)?;
                obj.set(cx, "microblock_header_1", microblock_header_1_obj)?;

                let microblock_header_2_obj = cx.empty_object();
                h2.neon_js_serialize(cx, &microblock_header_2_obj, extra_ctx)?;
                obj.set(cx, "microblock_header_2", microblock_header_2_obj)?;
            }
            TransactionPayload::Coinbase(ref buf) => {
                let type_id = cx.number(TransactionPayloadID::Coinbase as u8);
                obj.set(cx, "type_id", type_id)?;

                let payload_buffer = JsBuffer::external(cx, buf.to_bytes());
                obj.set(cx, "payload_buffer", payload_buffer)?;
            }
        }
        Ok(())
    }
}

impl NeonJsSerialize for PrincipalData {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        match self {
            PrincipalData::Standard(standard_principal) => {
                let type_int = ClarityTypePrefix::PrincipalStandard as u8;
                let type_id = cx.number(type_int);
                obj.set(cx, "type_id", type_id)?;

                let address = cx.string(standard_principal.to_address());
                obj.set(cx, "address", address)?;
            }
            PrincipalData::Contract(contract_identifier) => {
                let type_int = ClarityTypePrefix::PrincipalContract as u8;
                let type_id = cx.number(type_int);
                obj.set(cx, "type_id", type_id)?;

                let address = cx.string(contract_identifier.issuer.to_address());
                obj.set(cx, "address", address)?;

                let contract_name = cx.string(contract_identifier.name.to_string());
                obj.set(cx, "contract_name", contract_name)?;
            }
        };
        Ok(())
    }
}

impl NeonJsSerialize for TransactionContractCall {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        extra_ctx: &(),
    ) -> NeonResult<()> {
        let address = cx.string(self.address.to_string());
        obj.set(cx, "address", address)?;

        let contract_name = cx.string(self.contract_name.to_string());
        obj.set(cx, "contract_name", contract_name)?;

        let function_name = cx.string(self.function_name.to_string());
        obj.set(cx, "function_name", function_name)?;

        // TODO: raw function args binary slice is already determined during raw tx deserialization, ideally
        // try to use that rather than re-serializing (slow)
        let mut function_args_raw = u32::to_be_bytes(self.function_args.len() as u32).to_vec();
        let function_args = JsArray::new(cx, self.function_args.len() as u32);
        for (i, x) in self.function_args.iter().enumerate() {
            let val_obj = cx.empty_object();
            let mut val_bytes = x.neon_js_serialize(cx, &val_obj, extra_ctx)?;
            function_args_raw.append(&mut val_bytes);
            function_args.set(cx, i as u32, val_obj)?;
        }
        obj.set(cx, "function_args", function_args)?;

        let function_args_buff = JsBuffer::external(cx, function_args_raw);
        obj.set(cx, "function_args_buffer", function_args_buff)?;

        Ok(())
    }
}

impl NeonJsSerialize for TransactionSmartContract {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let contract_name = cx.string(self.name.to_string());
        obj.set(cx, "contract_name", contract_name)?;

        let code_body = cx.string(self.code_body.to_string());
        obj.set(cx, "code_body", code_body)?;
        Ok(())
    }
}

impl NeonJsSerialize<(), Vec<u8>> for StacksMicroblockHeader {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<Vec<u8>> {
        let vec = self.serialize_to_vec();

        // TODO: raw microblock header binary slice is already determined during raw tx deserialization, ideally
        // try to use that rather than re-serializing (slow)
        let buffer = JsBuffer::external(cx, vec.clone());
        obj.set(cx, "buffer", buffer)?;

        let version = cx.number(self.version);
        obj.set(cx, "version", version)?;

        let sequence = cx.number(self.sequence);
        obj.set(cx, "sequence", sequence)?;

        let prev_block = JsBuffer::external(cx, self.prev_block.to_bytes());
        obj.set(cx, "prev_block", prev_block)?;

        let tx_merkle_root = JsBuffer::external(cx, self.tx_merkle_root.to_bytes());
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let signature = JsBuffer::external(cx, self.signature.to_bytes());
        obj.set(cx, "signature", signature)?;

        Ok(vec)
    }
}

fn get_stacks_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let address_version = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let address_bytes_arg = cx.argument::<JsBuffer>(1)?;
    let address_hash160 = Hash160(address_bytes_arg.as_slice(&cx).try_into().unwrap());
    let stacks_address = StacksAddress::new(address_version as u8, address_hash160);
    let stacks_address_string = cx.string(stacks_address.to_string());
    return Ok(stacks_address_string);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn list_decode_test() {
        let val_bytes = hex::decode("0b000000640100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d400100000000000000000000000000c65d40").unwrap();
        let value = ClarityValue::consensus_deserialize(&mut &val_bytes[..]).unwrap();
        let result = format!("{}", value);
        let result2 = value.to_string();
        assert_eq!(result2, "asdf");
        assert_eq!(result, "asdf");
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("getVersion", get_version)?;
    cx.export_function("decodeClarityValueToRepr", decode_clarity_value_to_repr)?;
    cx.export_function("decodeClarityValueToJson", decode_clarity_value_to_json)?;
    cx.export_function("decodeClarityValueList", decode_clarity_value_array)?;
    cx.export_function("decodePostConditions", decode_tx_post_conditions)?;
    cx.export_function("decodeTransaction", decode_transaction)?;
    cx.export_function("getStacksAddress", get_stacks_address)?;
    Ok(())
}
