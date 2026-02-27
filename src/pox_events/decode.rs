use std::collections::BTreeMap;

use crate::address::c32::c32_address;
use crate::clarity_value::types::{ClarityName, ClarityValue, Value};
use crate::hex::encode_hex;

use super::btc_address::pox_address_to_btc_address;
use super::types::*;

/// Decode a Clarity value into a PoX synthetic event.
/// Returns `Ok(None)` if the value is a `ResponseErr` (non-event).
/// Returns `Err` if the structure is unexpected.
pub fn decode_pox_synthetic_event(
    clarity_value: &ClarityValue,
    network: StacksNetwork,
) -> Result<Option<PoxSyntheticEvent>, String> {
    // 1. Root must be ResponseOk; ResponseErr means no event.
    let inner = match &clarity_value.value {
        Value::ResponseOk(inner) => inner,
        Value::ResponseErr(_) => return Ok(None),
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event Clarity type, expected ResponseOk, got {:?}",
                other.type_prefix()
            ))
        }
    };

    // 2. Inner must be a Tuple
    let op_data = match &inner.value {
        Value::Tuple(map) => map,
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event Clarity type, expected Tuple, got {:?}",
                other.type_prefix()
            ))
        }
    };

    // 3. Extract base fields
    let stacker = clarity_principal_to_string(get_tuple_field(op_data, "stacker")?)?;
    let locked = extract_uint(get_tuple_field(op_data, "locked")?)?;
    let balance = extract_uint(get_tuple_field(op_data, "balance")?)?;
    let burnchain_unlock_height =
        extract_uint(get_tuple_field(op_data, "burnchain-unlock-height")?)?;

    // 4. Extract event name
    let name_str = match &get_tuple_field(op_data, "name")?.value {
        Value::StringASCII(bytes) => {
            String::from_utf8(bytes.clone()).map_err(|e| format!("Invalid event name: {}", e))?
        }
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event name type, expected StringASCII, got {:?}",
                other.type_prefix()
            ))
        }
    };

    let event_name = PoxEventName::from_str(&name_str)
        .ok_or_else(|| format!("Unexpected PoX synthetic event data name: {}", name_str))?;

    // 5. Extract inner data tuple
    let event_data_tuple = match &get_tuple_field(op_data, "data")?.value {
        Value::Tuple(map) => map,
        other => {
            return Err(format!(
                "Unexpected PoX synthetic event data payload type, expected Tuple, got {:?}",
                other.type_prefix()
            ))
        }
    };

    // 6. Extract pox-addr if present
    let (pox_addr, pox_addr_raw) = if event_data_tuple.contains_key("pox-addr") {
        extract_pox_addr(get_tuple_field(event_data_tuple, "pox-addr")?, network)?
    } else {
        (None, None)
    };

    let mut base = PoxEventBase {
        stacker,
        locked,
        balance,
        burnchain_unlock_height,
        pox_addr,
        pox_addr_raw,
    };

    // 7. Match on event name, extract type-specific fields and apply balance patches
    let data = match event_name {
        PoxEventName::HandleUnlock => {
            let first_cycle_locked =
                extract_uint(get_tuple_field(event_data_tuple, "first-cycle-locked")?)?;
            let first_unlocked_cycle =
                extract_uint(get_tuple_field(event_data_tuple, "first-unlocked-cycle")?)?;

            // Balance patch: balance += locked
            base.balance = base.balance.saturating_add(base.locked);

            PoxEventData::HandleUnlock {
                first_cycle_locked,
                first_unlocked_cycle,
            }
        }
        PoxEventName::StackStx => {
            let lock_amount =
                extract_uint(get_tuple_field(event_data_tuple, "lock-amount")?)?;
            let lock_period =
                extract_uint(get_tuple_field(event_data_tuple, "lock-period")?)?;
            let start_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "start-burn-height")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let signer_key =
                extract_optional_buffer_hex(event_data_tuple.get("signer-key"))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patches
            base.burnchain_unlock_height = unlock_burn_height;
            base.balance = base.balance.saturating_sub(lock_amount);
            base.locked = lock_amount;

            PoxEventData::StackStx {
                lock_amount,
                lock_period,
                start_burn_height,
                unlock_burn_height,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackIncrease => {
            let increase_by =
                extract_uint(get_tuple_field(event_data_tuple, "increase-by")?)?;
            let total_locked =
                extract_uint(get_tuple_field(event_data_tuple, "total-locked")?)?;
            let signer_key =
                extract_optional_buffer_hex(event_data_tuple.get("signer-key"))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patches
            base.balance = base.balance.saturating_sub(increase_by);
            base.locked = base.locked.saturating_add(increase_by);

            PoxEventData::StackIncrease {
                increase_by,
                total_locked,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackExtend => {
            let extend_count =
                extract_uint(get_tuple_field(event_data_tuple, "extend-count")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let signer_key =
                extract_optional_buffer_hex(event_data_tuple.get("signer-key"))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patch
            base.burnchain_unlock_height = unlock_burn_height;

            PoxEventData::StackExtend {
                extend_count,
                unlock_burn_height,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStx => {
            let amount_ustx =
                extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let delegate_to = clarity_principal_to_string(
                get_tuple_field(event_data_tuple, "delegate-to")?,
            )?;
            let unlock_burn_height_opt =
                extract_optional_uint(Some(get_tuple_field(event_data_tuple, "unlock-burn-height")?))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patch: if unlock_burn_height is set, use it
            if let Some(ubh) = unlock_burn_height_opt {
                base.burnchain_unlock_height = ubh;
            }

            PoxEventData::DelegateStx {
                amount_ustx,
                delegate_to,
                unlock_burn_height: unlock_burn_height_opt,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackStx => {
            let lock_amount =
                extract_uint(get_tuple_field(event_data_tuple, "lock-amount")?)?;
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let start_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "start-burn-height")?)?;
            let lock_period =
                extract_uint(get_tuple_field(event_data_tuple, "lock-period")?)?;
            let delegator = clarity_principal_to_string(
                get_tuple_field(event_data_tuple, "delegator")?,
            )?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patches
            base.burnchain_unlock_height = unlock_burn_height;
            base.balance = base.balance.saturating_sub(lock_amount);
            base.locked = lock_amount;

            PoxEventData::DelegateStackStx {
                lock_amount,
                unlock_burn_height,
                start_burn_height,
                lock_period,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackIncrease => {
            let increase_by =
                extract_uint(get_tuple_field(event_data_tuple, "increase-by")?)?;
            let total_locked =
                extract_uint(get_tuple_field(event_data_tuple, "total-locked")?)?;
            let delegator = clarity_principal_to_string(
                get_tuple_field(event_data_tuple, "delegator")?,
            )?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patches
            base.balance = base.balance.saturating_sub(increase_by);
            base.locked = base.locked.saturating_add(increase_by);

            PoxEventData::DelegateStackIncrease {
                increase_by,
                total_locked,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::DelegateStackExtend => {
            let unlock_burn_height =
                extract_uint(get_tuple_field(event_data_tuple, "unlock-burn-height")?)?;
            let extend_count =
                extract_uint(get_tuple_field(event_data_tuple, "extend-count")?)?;
            let delegator = clarity_principal_to_string(
                get_tuple_field(event_data_tuple, "delegator")?,
            )?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // Balance patch
            base.burnchain_unlock_height = unlock_burn_height;

            PoxEventData::DelegateStackExtend {
                unlock_burn_height,
                extend_count,
                delegator,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationCommit => {
            let reward_cycle =
                extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx =
                extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let signer_key =
                extract_optional_buffer_hex(event_data_tuple.get("signer-key"))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // No balance patches for aggregation commit
            PoxEventData::StackAggregationCommit {
                reward_cycle,
                amount_ustx,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationCommitIndexed => {
            let reward_cycle =
                extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx =
                extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let signer_key =
                extract_optional_buffer_hex(event_data_tuple.get("signer-key"))?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // No balance patches
            PoxEventData::StackAggregationCommitIndexed {
                reward_cycle,
                amount_ustx,
                signer_key,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::StackAggregationIncrease => {
            let reward_cycle =
                extract_uint(get_tuple_field(event_data_tuple, "reward-cycle")?)?;
            let amount_ustx =
                extract_uint(get_tuple_field(event_data_tuple, "amount-ustx")?)?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // No balance patches
            PoxEventData::StackAggregationIncrease {
                reward_cycle,
                amount_ustx,
                end_cycle_id,
                start_cycle_id,
            }
        }
        PoxEventName::RevokeDelegateStx => {
            let delegate_to = clarity_principal_to_string(
                get_tuple_field(event_data_tuple, "delegate-to")?,
            )?;
            let end_cycle_id =
                extract_optional_uint(event_data_tuple.get("end-cycle-id"))?;
            let start_cycle_id =
                extract_optional_uint(event_data_tuple.get("start-cycle-id"))?;

            // No balance patches
            PoxEventData::RevokeDelegateStx {
                delegate_to,
                end_cycle_id,
                start_cycle_id,
            }
        }
    };

    Ok(Some(PoxSyntheticEvent {
        base,
        name: event_name,
        data,
    }))
}

// ─── Helper functions ───────────────────────────────────────────────────────

fn get_tuple_field<'a>(
    tuple: &'a BTreeMap<ClarityName, ClarityValue>,
    key: &str,
) -> Result<&'a ClarityValue, String> {
    tuple
        .get(key)
        .ok_or_else(|| format!("Missing expected tuple field: {}", key))
}

fn extract_uint(val: &ClarityValue) -> Result<u128, String> {
    match &val.value {
        Value::UInt(v) => Ok(*v),
        other => Err(format!(
            "Expected UInt, got {:?}",
            other.type_prefix()
        )),
    }
}

/// Extract an optional uint from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `OptionalSome(UInt(v))` → `Ok(Some(v))`
/// - `UInt(v)` → `Ok(Some(v))` (for fields that are sometimes bare uints)
fn extract_optional_uint(val: Option<&ClarityValue>) -> Result<Option<u128>, String> {
    match val {
        None => Ok(None),
        Some(cv) => match &cv.value {
            Value::OptionalNone => Ok(None),
            Value::OptionalSome(inner) => match &inner.value {
                Value::UInt(v) => Ok(Some(*v)),
                other => Err(format!(
                    "Expected UInt inside OptionalSome, got {:?}",
                    other.type_prefix()
                )),
            },
            Value::UInt(v) => Ok(Some(*v)),
            other => Err(format!(
                "Expected OptionalSome/OptionalNone/UInt, got {:?}",
                other.type_prefix()
            )),
        },
    }
}

/// Extract a buffer as a hex string from:
/// - `None` (field absent) → `Ok(None)`
/// - `OptionalNone` → `Ok(None)`
/// - `Buffer(bytes)` → `Ok(Some("0x..."))`
/// - `OptionalSome(Buffer(bytes))` → `Ok(Some("0x..."))`
fn extract_optional_buffer_hex(
    val: Option<&ClarityValue>,
) -> Result<Option<String>, String> {
    match val {
        None => Ok(None),
        Some(cv) => match &cv.value {
            Value::OptionalNone => Ok(None),
            Value::Buffer(bytes) => Ok(Some(encode_hex(bytes).to_string())),
            Value::OptionalSome(inner) => match &inner.value {
                Value::Buffer(bytes) => Ok(Some(encode_hex(bytes).to_string())),
                other => Err(format!(
                    "Expected Buffer inside OptionalSome, got {:?}",
                    other.type_prefix()
                )),
            },
            other => Err(format!(
                "Expected Buffer/OptionalSome/OptionalNone, got {:?}",
                other.type_prefix()
            )),
        },
    }
}

/// Convert a Clarity principal value to a string address.
fn clarity_principal_to_string(val: &ClarityValue) -> Result<String, String> {
    match &val.value {
        Value::PrincipalStandard(data) => c32_address(data.0, &data.1),
        Value::PrincipalContract(data) => {
            let addr = c32_address(data.issuer.0, &data.issuer.1)?;
            Ok(format!("{}.{}", addr, data.name))
        }
        other => Err(format!(
            "Unexpected Clarity value type for principal: {:?}",
            other.type_prefix()
        )),
    }
}

/// Extract pox-addr tuple (version + hashbytes) and convert to BTC address.
/// Returns (btc_addr, raw_hex). Gracefully returns (None, None) on encoding errors.
fn extract_pox_addr(
    val: &ClarityValue,
    network: StacksNetwork,
) -> Result<(Option<String>, Option<String>), String> {
    // Handle OptionalNone
    if let Value::OptionalNone = &val.value {
        return Ok((None, None));
    }

    // Handle OptionalSome wrapping
    let addr_tuple = match &val.value {
        Value::OptionalSome(inner) => match &inner.value {
            Value::Tuple(map) => map,
            other => {
                return Err(format!(
                    "Expected Tuple inside OptionalSome for pox-addr, got {:?}",
                    other.type_prefix()
                ))
            }
        },
        Value::Tuple(map) => map,
        other => {
            return Err(format!(
                "Expected Tuple/OptionalSome/OptionalNone for pox-addr, got {:?}",
                other.type_prefix()
            ))
        }
    };

    let version_bytes = match &get_tuple_field(addr_tuple, "version")?.value {
        Value::Buffer(bytes) => bytes.clone(),
        other => {
            return Err(format!(
                "Expected Buffer for pox-addr version, got {:?}",
                other.type_prefix()
            ))
        }
    };

    let hashbytes = match &get_tuple_field(addr_tuple, "hashbytes")?.value {
        Value::Buffer(bytes) => bytes.clone(),
        other => {
            return Err(format!(
                "Expected Buffer for pox-addr hashbytes, got {:?}",
                other.type_prefix()
            ))
        }
    };

    // Build raw hex: version_bytes ++ hashbytes
    let mut raw = Vec::with_capacity(version_bytes.len() + hashbytes.len());
    raw.extend_from_slice(&version_bytes);
    raw.extend_from_slice(&hashbytes);
    let raw_hex = encode_hex(&raw).to_string();

    // Try to encode BTC address; on error, return None for btc_addr (matches TS try/catch)
    let version = if version_bytes.is_empty() {
        return Ok((None, Some(raw_hex)));
    } else {
        version_bytes[0]
    };

    let btc_addr = match pox_address_to_btc_address(version, &hashbytes, network) {
        Ok(addr) => Some(addr),
        Err(_) => None,
    };

    Ok((btc_addr, Some(raw_hex)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_err_returns_none() {
        // (err u1) = 0x08 0x01 0x00...01
        let cv = ClarityValue::new(Value::ResponseErr(Box::new(ClarityValue::new(
            Value::UInt(1),
        ))));
        let result = decode_pox_synthetic_event(&cv, StacksNetwork::Mainnet).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_non_response_errors() {
        let cv = ClarityValue::new(Value::UInt(42));
        let result = decode_pox_synthetic_event(&cv, StacksNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_uint_works() {
        let cv = ClarityValue::new(Value::UInt(12345));
        assert_eq!(extract_uint(&cv).unwrap(), 12345);
    }

    #[test]
    fn test_extract_optional_uint_none() {
        assert_eq!(extract_optional_uint(None).unwrap(), None);
        let cv = ClarityValue::new(Value::OptionalNone);
        assert_eq!(extract_optional_uint(Some(&cv)).unwrap(), None);
    }

    #[test]
    fn test_extract_optional_uint_some() {
        let cv = ClarityValue::new(Value::OptionalSome(Box::new(ClarityValue::new(
            Value::UInt(999),
        ))));
        assert_eq!(extract_optional_uint(Some(&cv)).unwrap(), Some(999));
    }

    #[test]
    fn test_extract_optional_buffer_hex() {
        let cv = ClarityValue::new(Value::Buffer(vec![0xab, 0xcd]));
        assert_eq!(
            extract_optional_buffer_hex(Some(&cv)).unwrap(),
            Some("0xabcd".to_string())
        );

        let cv_none = ClarityValue::new(Value::OptionalNone);
        assert_eq!(extract_optional_buffer_hex(Some(&cv_none)).unwrap(), None);

        assert_eq!(extract_optional_buffer_hex(None).unwrap(), None);
    }
}
