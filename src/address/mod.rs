use std::error;
use std::fmt;

use std::convert::TryFrom;

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

pub mod b58;
pub mod bitcoin_address;
pub mod c32;
pub mod stacks_address;

pub const C32_ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 22; // P
pub const C32_ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 20; // M
pub const C32_ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 26; // T
pub const C32_ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 21; // N

#[derive(Debug)]
pub enum Error {
    InvalidCrockford32,
    InvalidVersion(u8),
    EmptyData,
    /// Invalid character encountered
    BadByte(u8),
    /// Checksum was not correct (expected, actual)
    BadChecksum(u32, u32),
    /// The length (in bytes) of the object was not correct
    /// Note that if the length is excessively long the provided length may be
    /// an estimate (and the checksum step may be skipped).
    InvalidLength(usize),
    /// Checked data was less than 4 bytes
    TooShort(usize),
    /// Any other error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidCrockford32 => write!(f, "Invalid crockford 32 string"),
            Error::InvalidVersion(ref v) => write!(f, "Invalid version {}", v),
            Error::EmptyData => f.write_str("Empty data"),
            Error::BadByte(b) => write!(f, "invalid base58 character 0x{:x}", b),
            Error::BadChecksum(exp, actual) => write!(
                f,
                "base58ck checksum 0x{:x} does not match expected 0x{:x}",
                actual, exp
            ),
            Error::InvalidLength(ell) => write!(f, "length {} invalid for this base58 type", ell),
            Error::TooShort(_) => write!(f, "base58ck data not even long enough for a checksum"),
            Error::Other(ref s) => f.write_str(s),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
    fn description(&self) -> &'static str {
        match *self {
            Error::InvalidCrockford32 => "Invalid crockford 32 string",
            Error::InvalidVersion(_) => "Invalid version",
            Error::EmptyData => "Empty data",
            Error::BadByte(_) => "invalid b58 character",
            Error::BadChecksum(_, _) => "invalid b58ck checksum",
            Error::InvalidLength(_) => "invalid length for b58 type",
            Error::TooShort(_) => "b58ck data less than 4 bytes",
            Error::Other(_) => "unknown b58 error",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum AddressHashMode {
    // serialization modes for public keys to addresses.
    // We support four different modes due to legacy compatibility with Stacks v1 addresses:
    SerializeP2PKH = 0x00,  // hash160(public-key), same as bitcoin's p2pkh
    SerializeP2SH = 0x01,   // hash160(multisig-redeem-script), same as bitcoin's multisig p2sh
    SerializeP2WPKH = 0x02, // hash160(segwit-program-00(p2pkh)), same as bitcoin's p2sh-p2wpkh
    SerializeP2WSH = 0x03,  // hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh
}

impl AddressHashMode {
    pub fn to_version_mainnet(&self) -> u8 {
        match *self {
            AddressHashMode::SerializeP2PKH => C32_ADDRESS_VERSION_MAINNET_SINGLESIG,
            _ => C32_ADDRESS_VERSION_MAINNET_MULTISIG,
        }
    }

    pub fn to_version_testnet(&self) -> u8 {
        match *self {
            AddressHashMode::SerializeP2PKH => C32_ADDRESS_VERSION_TESTNET_SINGLESIG,
            _ => C32_ADDRESS_VERSION_TESTNET_MULTISIG,
        }
    }
}

/// Given the u8 of an AddressHashMode, deduce the AddressHashNode
impl TryFrom<u8> for AddressHashMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<AddressHashMode, Self::Error> {
        match value {
            x if x == AddressHashMode::SerializeP2PKH as u8 => Ok(AddressHashMode::SerializeP2PKH),
            x if x == AddressHashMode::SerializeP2SH as u8 => Ok(AddressHashMode::SerializeP2SH),
            x if x == AddressHashMode::SerializeP2WPKH as u8 => {
                Ok(AddressHashMode::SerializeP2WPKH)
            }
            x if x == AddressHashMode::SerializeP2WSH as u8 => Ok(AddressHashMode::SerializeP2WSH),
            _ => Err(Error::InvalidVersion(value)),
        }
    }
}

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
