use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::serialize_util::DeserializeError;
use crate::stacks_tx::deserialize::{
    BlockHeaderHash, MessageSignature, Sha512Trunc256Sum, StacksTransaction,
};

/// Consensus hash - 20 bytes
pub struct ConsensusHash(pub [u8; 20]);

/// Stacks block ID - 32 bytes (hash of consensus hash + block header hash)
pub struct StacksBlockId(pub [u8; 32]);

/// Trie hash for MARF - 32 bytes
pub struct TrieHash(pub [u8; 32]);

/// A bitvector with a maximum size
pub struct BitVec {
    pub data: Vec<u8>,
    pub len: u16,
}

impl BitVec {
    pub fn deserialize(fd: &mut Cursor<&[u8]>, max_size: u16) -> Result<Self, DeserializeError> {
        let len = fd.read_u16::<BigEndian>()?;
        if len == 0 {
            return Err("BitVec lengths must be positive".into());
        }
        if len > max_size {
            return Err(format!(
                "BitVec length exceeded maximum. Max size = {}, len = {}",
                max_size, len
            )
            .into());
        }

        let expected_data_len = Self::data_len(len);
        let data_len = fd.read_u32::<BigEndian>()?;
        if data_len as u16 != expected_data_len {
            return Err(format!(
                "BitVec data length mismatch: expected {}, got {}",
                expected_data_len, data_len
            )
            .into());
        }
        let mut data = vec![0u8; data_len as usize];
        fd.read_exact(&mut data)?;

        Ok(BitVec { data, len })
    }

    /// Return the number of bytes needed to store `len` bits.
    fn data_len(len: u16) -> u16 {
        len / 8 + if len % 8 == 0 { 0 } else { 1 }
    }

    /// Get the value at the given index
    pub fn get(&self, index: u16) -> Option<bool> {
        if index >= self.len {
            return None;
        }
        let byte_index = (index / 8) as usize;
        let bit_index = index % 8;
        Some((self.data[byte_index] & (1 << (7 - bit_index))) != 0)
    }
}

/// Header for a Nakamoto block (Stacks 3.x+)
pub struct NakamotoBlockHeader {
    /// Version byte
    pub version: u8,
    /// The total number of StacksBlock and NakamotoBlocks preceding this block
    pub chain_length: u64,
    /// Total amount of BTC spent producing the sortition that selected this block's miner
    pub burn_spent: u64,
    /// The consensus hash of the burnchain block that selected this tenure
    pub consensus_hash: ConsensusHash,
    /// The index block hash of the immediate parent of this block
    pub parent_block_id: StacksBlockId,
    /// The root of a SHA512/256 merkle tree over all this block's transactions
    pub tx_merkle_root: Sha512Trunc256Sum,
    /// The MARF trie root hash after this block has been processed
    pub state_index_root: TrieHash,
    /// Unix timestamp of when this block was mined
    pub timestamp: u64,
    /// Recoverable ECDSA signature from the tenure's miner
    pub miner_signature: MessageSignature,
    /// Set of recoverable ECDSA signatures over the block header from the signer set
    pub signer_signature: Vec<MessageSignature>,
    /// Bitvec indicating which reward addresses should be punished
    pub pox_treatment: BitVec,
}

impl NakamotoBlockHeader {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version = fd.read_u8()?;
        let chain_length = fd.read_u64::<BigEndian>()?;
        let burn_spent = fd.read_u64::<BigEndian>()?;

        let mut consensus_hash_bytes = [0u8; 20];
        fd.read_exact(&mut consensus_hash_bytes)?;
        let consensus_hash = ConsensusHash(consensus_hash_bytes);

        let mut parent_block_id_bytes = [0u8; 32];
        fd.read_exact(&mut parent_block_id_bytes)?;
        let parent_block_id = StacksBlockId(parent_block_id_bytes);

        let mut tx_merkle_root_bytes = [0u8; 32];
        fd.read_exact(&mut tx_merkle_root_bytes)?;
        let tx_merkle_root = Sha512Trunc256Sum(tx_merkle_root_bytes);

        let mut state_index_root_bytes = [0u8; 32];
        fd.read_exact(&mut state_index_root_bytes)?;
        let state_index_root = TrieHash(state_index_root_bytes);

        let timestamp = fd.read_u64::<BigEndian>()?;

        let mut miner_signature_bytes = [0u8; 65];
        fd.read_exact(&mut miner_signature_bytes)?;
        let miner_signature = MessageSignature(miner_signature_bytes);

        // Read signer signatures (length-prefixed array)
        let signer_sig_count = fd.read_u32::<BigEndian>()?;
        let mut signer_signature = Vec::with_capacity(signer_sig_count as usize);
        for _ in 0..signer_sig_count {
            let mut sig_bytes = [0u8; 65];
            fd.read_exact(&mut sig_bytes)?;
            signer_signature.push(MessageSignature(sig_bytes));
        }

        // Read pox_treatment bitvec (max 4000 bits)
        let pox_treatment = BitVec::deserialize(fd, 4000)?;

        Ok(NakamotoBlockHeader {
            version,
            chain_length,
            burn_spent,
            consensus_hash,
            parent_block_id,
            tx_merkle_root,
            state_index_root,
            timestamp,
            miner_signature,
            signer_signature,
            pox_treatment,
        })
    }

    /// Compute the block hash (sha512/256 of header fields excluding signer_signature).
    /// This is the same as the "signer signature hash" in the reference implementation.
    pub fn block_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let mut hasher = Sha512_256::new();

        hasher.update([self.version]);
        hasher.update(self.chain_length.to_be_bytes());
        hasher.update(self.burn_spent.to_be_bytes());
        hasher.update(&self.consensus_hash.0);
        hasher.update(&self.parent_block_id.0);
        hasher.update(&self.tx_merkle_root.0);
        hasher.update(&self.state_index_root.0);
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(&self.miner_signature.0);
        hasher.update(self.pox_treatment.len.to_be_bytes());
        hasher.update((self.pox_treatment.data.len() as u32).to_be_bytes());
        hasher.update(&self.pox_treatment.data);

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Compute the block ID (sha512/256 of block_hash + consensus_hash)
    pub fn block_id(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let block_hash = self.block_hash();
        let mut hasher = Sha512_256::new();
        hasher.update(&block_hash);
        hasher.update(&self.consensus_hash.0);

        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        id
    }
}

/// A Nakamoto block (Stacks 3.x+)
pub struct NakamotoBlock {
    pub header: NakamotoBlockHeader,
    pub txs: Vec<StacksTransaction>,
}

impl NakamotoBlock {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let header = NakamotoBlockHeader::deserialize(fd)?;

        // Read transactions (length-prefixed array)
        let tx_count = fd.read_u32::<BigEndian>()?;
        let mut txs = Vec::with_capacity(tx_count as usize);
        for _ in 0..tx_count {
            txs.push(StacksTransaction::deserialize(fd)?);
        }

        Ok(NakamotoBlock { header, txs })
    }
}

/// Header for Stacks 2.x blocks
pub struct StacksBlockHeader {
    pub version: u8,
    /// Total work done on the chain tip this block builds on
    pub total_work: StacksWorkScore,
    /// VRF proof
    pub proof: VRFProof,
    /// Parent block hash
    pub parent_block: BlockHeaderHash,
    /// Parent microblock hash
    pub parent_microblock: BlockHeaderHash,
    /// Parent microblock sequence number
    pub parent_microblock_sequence: u16,
    /// Merkle root of transactions
    pub tx_merkle_root: Sha512Trunc256Sum,
    /// State index root (MARF trie)
    pub state_index_root: TrieHash,
    /// Hash160 of the microblock public key
    pub microblock_pubkey_hash: [u8; 20],
}

/// Work score for Stacks 2.x consensus
pub struct StacksWorkScore {
    pub burn: u64,
    pub work: u64,
}

/// VRF proof - 80 bytes
pub struct VRFProof(pub [u8; 80]);

impl StacksBlockHeader {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let version = fd.read_u8()?;

        // StacksWorkScore
        let burn = fd.read_u64::<BigEndian>()?;
        let work = fd.read_u64::<BigEndian>()?;
        let total_work = StacksWorkScore { burn, work };

        // VRF proof (80 bytes)
        let mut proof_bytes = [0u8; 80];
        fd.read_exact(&mut proof_bytes)?;
        let proof = VRFProof(proof_bytes);

        let mut parent_block_bytes = [0u8; 32];
        fd.read_exact(&mut parent_block_bytes)?;
        let parent_block = BlockHeaderHash(parent_block_bytes);

        let mut parent_microblock_bytes = [0u8; 32];
        fd.read_exact(&mut parent_microblock_bytes)?;
        let parent_microblock = BlockHeaderHash(parent_microblock_bytes);

        let parent_microblock_sequence = fd.read_u16::<BigEndian>()?;

        let mut tx_merkle_root_bytes = [0u8; 32];
        fd.read_exact(&mut tx_merkle_root_bytes)?;
        let tx_merkle_root = Sha512Trunc256Sum(tx_merkle_root_bytes);

        let mut state_index_root_bytes = [0u8; 32];
        fd.read_exact(&mut state_index_root_bytes)?;
        let state_index_root = TrieHash(state_index_root_bytes);

        let mut microblock_pubkey_hash = [0u8; 20];
        fd.read_exact(&mut microblock_pubkey_hash)?;

        Ok(StacksBlockHeader {
            version,
            total_work,
            proof,
            parent_block,
            parent_microblock,
            parent_microblock_sequence,
            tx_merkle_root,
            state_index_root,
            microblock_pubkey_hash,
        })
    }

    /// Compute the block hash
    pub fn block_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha512_256};

        let mut hasher = Sha512_256::new();

        hasher.update([self.version]);
        hasher.update(self.total_work.burn.to_be_bytes());
        hasher.update(self.total_work.work.to_be_bytes());
        hasher.update(&self.proof.0);
        hasher.update(&self.parent_block.0);
        hasher.update(&self.parent_microblock.0);
        hasher.update(self.parent_microblock_sequence.to_be_bytes());
        hasher.update(&self.tx_merkle_root.0);
        hasher.update(&self.state_index_root.0);
        hasher.update(&self.microblock_pubkey_hash);

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// A Stacks 2.x block
pub struct StacksBlock {
    pub header: StacksBlockHeader,
    pub txs: Vec<StacksTransaction>,
}

impl StacksBlock {
    pub fn deserialize(fd: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let header = StacksBlockHeader::deserialize(fd)?;

        // Read transactions (length-prefixed array)
        let tx_count = fd.read_u32::<BigEndian>()?;
        let mut txs = Vec::with_capacity(tx_count as usize);
        for _ in 0..tx_count {
            txs.push(StacksTransaction::deserialize(fd)?);
        }

        Ok(StacksBlock { header, txs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::encode_hex;

    #[test]
    fn test_bitvec_deserialize() {
        // Test a simple bitvec with 8 bits (1 byte of data)
        let data: Vec<u8> = vec![
            0x00, 0x08, // len = 8
            0x00, 0x00, 0x00, 0x01, // data_len = 1
            0b10101010, // data
        ];
        let mut cursor = Cursor::new(data.as_ref());
        let bitvec = BitVec::deserialize(&mut cursor, 4000).unwrap();
        assert_eq!(bitvec.len, 8);
        assert_eq!(bitvec.data, vec![0b10101010]);
        assert_eq!(bitvec.get(0), Some(true));
        assert_eq!(bitvec.get(1), Some(false));
        assert_eq!(bitvec.get(7), Some(false));
        assert_eq!(bitvec.get(8), None);
    }

    #[test]
    fn test_nakamoto_block_deserialize() {
        let data = include_bytes!("../../tests/fixtures/nakamoto-block.bin");
        let mut cursor = Cursor::new(data.as_ref());
        let block = NakamotoBlock::deserialize(&mut cursor);
        assert!(block.is_ok());
        let block = block.unwrap();
        assert_eq!(block.header.version, 0);
        assert_eq!(block.header.chain_length, 557923);
        assert_eq!(block.header.burn_spent, 403018706956);
        assert_eq!(encode_hex(&block.header.consensus_hash.0).as_ref(), "0xe86587f4ed4ca465b87649ace9341d9fdfd113ba");
        assert_eq!(encode_hex(&block.header.parent_block_id.0).as_ref(), "0x8de0fa074023b893f73c8491ab5c93bb3f5af4bd5f0449578b99b508cca61595");
        assert_eq!(encode_hex(&block.header.tx_merkle_root.0).as_ref(), "0x080d35f6c5c02929a00fca1cc6f00a1c3828d905eb61e002ffd4e48f1ecef29d");
        assert_eq!(encode_hex(&block.header.state_index_root.0).as_ref(), "0xbf5ed8f745df2629d0d971fe9667f75a352a5dea4c8a0e451dcaa72b375d28fc");
        assert_eq!(block.header.timestamp, 1738687125);
        assert_eq!(encode_hex(&block.header.miner_signature.0).as_ref(), "0x01b7ef0ca6fb1e109afb5d3a9f08bfee71b8fef82ad9a7e06a5fa9b732394513be7cc962950ce2fc940d4ae7c1cb731d33cd65ec032a3a097ac2669439fe31031d");
        assert_eq!(block.header.signer_signature.len(), 24);
        assert_eq!(block.header.pox_treatment.len, 3891);
        assert_eq!(block.header.pox_treatment.data.len(), 487);
        assert_eq!(encode_hex(&block.header.block_hash()).as_ref(), "0x536b854fa6ada87643e00c4a4880967b4f52404b95dca75780babb048f6a69fc");
        assert_eq!(encode_hex(&block.header.block_id()).as_ref(), "0x05b7fbc03e541271a29baf21ad43e68e48070df018ebe5baa13892f3828be9bd");
        assert_eq!(block.txs.len(), 1);
        assert_eq!(cursor.position() as usize, data.len());
    }
}
