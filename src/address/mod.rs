use std::error;
use std::fmt;

use std::convert::TryFrom;

use self::bitcoin_address::BitcoinAddress;
use self::bitcoin_address::ADDRESS_VERSION_MAINNET_MULTISIG;
use self::bitcoin_address::ADDRESS_VERSION_MAINNET_SINGLESIG;
use self::bitcoin_address::ADDRESS_VERSION_TESTNET_MULTISIG;
use self::bitcoin_address::ADDRESS_VERSION_TESTNET_SINGLESIG;
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

    pub fn from_version(version: u8) -> AddressHashMode {
        match version {
            C32_ADDRESS_VERSION_TESTNET_SINGLESIG | C32_ADDRESS_VERSION_MAINNET_SINGLESIG => {
                AddressHashMode::SerializeP2PKH
            }
            _ => AddressHashMode::SerializeP2SH,
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

pub fn btc_addr_to_stx_addr_version(addr: &BitcoinAddress) -> Result<u8, String> {
    let btc_version =
        bitcoin_address::address_type_to_version_byte(&addr.addrtype, &addr.network_id);
    btc_to_stx_addr_version_byte(btc_version).ok_or_else(|| {
        format!(
            "Failed to decode Bitcoin version byte to Stacks version byte: {}",
            btc_version
        )
    })
}

pub fn btc_addr_to_stx_addr(addr: &BitcoinAddress) -> Result<StacksAddress, String> {
    let version = btc_addr_to_stx_addr_version(addr)?;
    Ok(StacksAddress {
        version: version,
        hash160_bytes: addr.hash160_bytes.clone(),
    })
}

pub fn stx_addr_to_btc_addr(addr: &StacksAddress) -> String {
    let btc_version = stx_to_btc_version_byte(addr.version)
        // fallback to version
        .unwrap_or(addr.version);
    let mut all_bytes = vec![btc_version];
    all_bytes.extend(addr.hash160_bytes.iter());
    b58::check_encode_slice(&all_bytes)
}
