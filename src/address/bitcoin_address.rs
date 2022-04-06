use super::b58;

pub const ADDRESS_VERSION_MAINNET_SINGLESIG: u8 = 0;
pub const ADDRESS_VERSION_MAINNET_MULTISIG: u8 = 5;
pub const ADDRESS_VERSION_TESTNET_SINGLESIG: u8 = 111;
pub const ADDRESS_VERSION_TESTNET_MULTISIG: u8 = 196;

pub enum BitcoinAddressType {
    PublicKeyHash,
    ScriptHash,
}

pub enum BitcoinNetworkType {
    Mainnet,
    Testnet,
    #[allow(dead_code)]
    Regtest,
}

pub struct BitcoinAddress {
    pub addrtype: BitcoinAddressType,
    pub network_id: BitcoinNetworkType,
    pub hash160_bytes: [u8; 20],
}

fn version_byte_to_address_type(version: u8) -> Option<(BitcoinAddressType, BitcoinNetworkType)> {
    match version {
        ADDRESS_VERSION_MAINNET_SINGLESIG => Some((
            BitcoinAddressType::PublicKeyHash,
            BitcoinNetworkType::Mainnet,
        )),
        ADDRESS_VERSION_MAINNET_MULTISIG => {
            Some((BitcoinAddressType::ScriptHash, BitcoinNetworkType::Mainnet))
        }
        ADDRESS_VERSION_TESTNET_SINGLESIG => Some((
            BitcoinAddressType::PublicKeyHash,
            BitcoinNetworkType::Testnet,
        )),
        ADDRESS_VERSION_TESTNET_MULTISIG => {
            Some((BitcoinAddressType::ScriptHash, BitcoinNetworkType::Testnet))
        }
        _ => None,
    }
}

/// Instantiate an address from a b58check string
/// Note that the network type will be 'testnet' if there is a testnet or regtest version byte
pub fn from_b58(addrb58: &str) -> Result<BitcoinAddress, String> {
    let bytes = b58::from_check(addrb58).map_err(|e| format!("{}", e))?;

    if bytes.len() != 21 {
        return Err(format!("Invalid address: {} bytes", bytes.len()));
    }

    let version = bytes[0];

    let typeinfo_opt = version_byte_to_address_type(version);
    if typeinfo_opt.is_none() {
        return Err(format!("Invalid address: unrecognized version {}", version));
    }

    let mut payload_bytes = [0; 20];
    let b = &bytes[1..21];
    payload_bytes.copy_from_slice(b);

    let (addrtype, network_id) = typeinfo_opt.unwrap();

    Ok(BitcoinAddress {
        network_id: network_id,
        addrtype: addrtype,
        hash160_bytes: payload_bytes,
    })
}

pub fn address_type_to_version_byte(
    addrtype: &BitcoinAddressType,
    network_id: &BitcoinNetworkType,
) -> u8 {
    match (addrtype, network_id) {
        (BitcoinAddressType::PublicKeyHash, BitcoinNetworkType::Mainnet) => {
            ADDRESS_VERSION_MAINNET_SINGLESIG
        }
        (BitcoinAddressType::ScriptHash, BitcoinNetworkType::Mainnet) => {
            ADDRESS_VERSION_MAINNET_MULTISIG
        }
        (BitcoinAddressType::PublicKeyHash, BitcoinNetworkType::Testnet)
        | (BitcoinAddressType::PublicKeyHash, BitcoinNetworkType::Regtest) => {
            ADDRESS_VERSION_TESTNET_SINGLESIG
        }
        (BitcoinAddressType::ScriptHash, BitcoinNetworkType::Testnet)
        | (BitcoinAddressType::ScriptHash, BitcoinNetworkType::Regtest) => {
            ADDRESS_VERSION_TESTNET_MULTISIG
        }
    }
}
