use super::c32::c32_address_decode;

pub struct StacksAddress {
    pub version: u8,
    pub hash160_bytes: [u8; 20],
}

impl StacksAddress {
    pub fn new(version: u8, hash160_bytes: [u8; 20]) -> Self {
        Self {
            version,
            hash160_bytes,
        }
    }

    pub fn from_string(s: &str) -> Result<StacksAddress, String> {
        let (version, bytes) = match c32_address_decode(s) {
            Ok((v, b)) => (v, b),
            Err(e) => {
                return Err(format!("Error decoding c32 address: {}", e));
            }
        };

        Ok(StacksAddress {
            version: version,
            hash160_bytes: bytes,
        })
    }
}
