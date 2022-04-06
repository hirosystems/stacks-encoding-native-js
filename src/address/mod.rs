use neon::prelude::*;
use neon::types::buffer::TypedArray;

use crate::neon_util::arg_as_bytes;

use self::bitcoin_address::{
    BitcoinAddress, ADDRESS_VERSION_MAINNET_MULTISIG, ADDRESS_VERSION_MAINNET_SINGLESIG,
    ADDRESS_VERSION_TESTNET_MULTISIG, ADDRESS_VERSION_TESTNET_SINGLESIG,
};
use self::c32::c32_address;
use self::c32::c32_address_decode;
use self::stacks_address::StacksAddress;
use self::stacks_address::{
    C32_ADDRESS_VERSION_MAINNET_MULTISIG, C32_ADDRESS_VERSION_MAINNET_SINGLESIG,
    C32_ADDRESS_VERSION_TESTNET_MULTISIG, C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
};

pub mod b58;
pub mod bitcoin_address;
pub mod c32;
pub mod stacks_address;

fn btc_to_stx_addr_version_byte(version: u8) -> Option<u8> {
    match version {
        ADDRESS_VERSION_MAINNET_SINGLESIG => Some(C32_ADDRESS_VERSION_MAINNET_SINGLESIG),
        ADDRESS_VERSION_MAINNET_MULTISIG => Some(C32_ADDRESS_VERSION_MAINNET_MULTISIG),
        ADDRESS_VERSION_TESTNET_SINGLESIG => Some(C32_ADDRESS_VERSION_TESTNET_SINGLESIG),
        ADDRESS_VERSION_TESTNET_MULTISIG => Some(C32_ADDRESS_VERSION_TESTNET_MULTISIG),
        _ => None,
    }
}

fn stx_to_btc_version_byte(version: u8) -> Option<u8> {
    match version {
        C32_ADDRESS_VERSION_MAINNET_SINGLESIG => Some(ADDRESS_VERSION_MAINNET_SINGLESIG),
        C32_ADDRESS_VERSION_MAINNET_MULTISIG => Some(ADDRESS_VERSION_MAINNET_MULTISIG),
        C32_ADDRESS_VERSION_TESTNET_SINGLESIG => Some(ADDRESS_VERSION_TESTNET_SINGLESIG),
        C32_ADDRESS_VERSION_TESTNET_MULTISIG => Some(ADDRESS_VERSION_TESTNET_MULTISIG),
        _ => None,
    }
}

fn btc_addr_to_stx_addr_version(addr: &BitcoinAddress) -> Result<u8, String> {
    let btc_version =
        bitcoin_address::address_type_to_version_byte(&addr.addrtype, &addr.network_id);
    btc_to_stx_addr_version_byte(btc_version).ok_or_else(|| {
        format!(
            "Failed to decode Bitcoin version byte to Stacks version byte: {}",
            btc_version
        )
    })
}

fn btc_addr_to_stx_addr(addr: &BitcoinAddress) -> Result<StacksAddress, String> {
    let version = btc_addr_to_stx_addr_version(addr)?;
    Ok(StacksAddress {
        version: version,
        hash160_bytes: addr.hash160_bytes.clone(),
    })
}

fn stx_addr_to_btc_addr(addr: &StacksAddress) -> String {
    let btc_version = stx_to_btc_version_byte(addr.version)
        // fallback to version
        .unwrap_or(addr.version);
    let mut all_bytes = vec![btc_version];
    all_bytes.extend(addr.hash160_bytes.iter());
    b58::check_encode_slice(&all_bytes)
}

pub fn is_valid_stacks_address(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let address_string = cx.argument::<JsString>(0)?.value(&mut cx);
    let address = c32_address_decode(&address_string);
    match address {
        Ok(_) => Ok(cx.boolean(true)),
        Err(_) => Ok(cx.boolean(false)),
    }
}

pub fn decode_stacks_address(mut cx: FunctionContext) -> JsResult<JsArray> {
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

pub fn stacks_address_from_parts(mut cx: FunctionContext) -> JsResult<JsString> {
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
    let stacks_address = StacksAddress::from_string(&input)?;
    let bitcoin_address = stx_addr_to_btc_addr(&stacks_address);
    Ok(bitcoin_address)
}

pub fn stacks_to_bitcoin_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let stacks_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let btc_address =
        stacks_to_bitcoin_address_internal(stacks_address_arg).or_else(|e| cx.throw_error(e))?;
    let btc_address = cx.string(btc_address);
    Ok(btc_address)
}

pub fn bitcoin_to_stacks_address(mut cx: FunctionContext) -> JsResult<JsString> {
    let bitcoin_address_arg = cx.argument::<JsString>(0)?.value(&mut cx);
    let bitcoin_address = bitcoin_address::from_b58(&bitcoin_address_arg)
        .or_else(|e| cx.throw_error(format!("Error parsing Bitcoin address: {}", e)))?;

    let stacks_addr = btc_addr_to_stx_addr(&bitcoin_address).or_else(|e| {
        cx.throw_error(format!(
            "Error getting Stacks address version from Bitcoin address: {}",
            e
        ))
    })?;

    let stacks_addr = c32_address(stacks_addr.version, &stacks_addr.hash160_bytes)
        .or_else(|e| cx.throw_error(format!("Error converting to C32 address: {}", e)))?;

    Ok(cx.string(stacks_addr))
}

#[cfg(feature = "profiling")]
pub fn perf_test_c32_encode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
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
pub fn perf_test_c32_decode(mut cx: FunctionContext) -> JsResult<JsBuffer> {
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
