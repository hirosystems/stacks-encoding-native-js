import {
  decodeNakamotoBlock,
  decodeNakamotoBlockHeader,
  DecodedNakamotoBlockHeader,
  DecodedNakamotoBlockResult,
} from '../index.js';

// Test vector derived from stacks-core test `codec_nakamoto_header`
// Header structure:
//   version: 1
//   chain_length: 2
//   burn_spent: 3
//   consensus_hash: [0x04; 20]
//   parent_block_id: [0x05; 32]
//   tx_merkle_root: [0x06; 32]
//   state_index_root: [0x07; 32]
//   timestamp: 8
//   miner_signature: empty (65 zeros)
//   signer_signature: 1 signature (65 bytes of 0x01)
//   pox_treatment: BitVec::zeros(8)

// Build the header hex programmatically for clarity
const buildHeaderHex = () => {
  const parts = [
    '01', // version (1 byte)
    '0000000000000002', // chain_length (u64 BE, 8 bytes)
    '0000000000000003', // burn_spent (u64 BE, 8 bytes)
    '0404040404040404040404040404040404040404', // consensus_hash (20 bytes)
    '0505050505050505050505050505050505050505050505050505050505050505', // parent_block_id (32 bytes)
    '0606060606060606060606060606060606060606060606060606060606060606', // tx_merkle_root (32 bytes)
    '0707070707070707070707070707070707070707070707070707070707070707', // state_index_root (32 bytes)
    '0000000000000008', // timestamp (u64 BE, 8 bytes)
    '00'.repeat(65), // miner_signature (65 bytes = 130 hex chars)
    '00000001', // signer_signature count (u32 BE, 4 bytes)
    '01'.repeat(65), // signer_signature[0] (65 bytes = 130 hex chars)
    '00080000000100', // pox_treatment bitvec (7 bytes)
  ];
  return '0x' + parts.join('');
};

const NAKAMOTO_HEADER_HEX = buildHeaderHex();

test('decode nakamoto block header', () => {
  const decoded = decodeNakamotoBlockHeader(NAKAMOTO_HEADER_HEX);
  
  expect(decoded.version).toEqual(1);
  expect(decoded.chain_length).toEqual('2');
  expect(decoded.burn_spent).toEqual('3');
  expect(decoded.consensus_hash).toEqual('0x0404040404040404040404040404040404040404');
  expect(decoded.parent_block_id).toEqual('0x0505050505050505050505050505050505050505050505050505050505050505');
  expect(decoded.tx_merkle_root).toEqual('0x0606060606060606060606060606060606060606060606060606060606060606');
  expect(decoded.state_index_root).toEqual('0x0707070707070707070707070707070707070707070707070707070707070707');
  expect(decoded.timestamp).toEqual('8');
  expect(decoded.miner_signature).toEqual('0x' + '00'.repeat(65));
  expect(decoded.signer_signature).toHaveLength(1);
  expect(decoded.signer_signature[0]).toEqual('0x' + '01'.repeat(65));
  // pox_treatment should be hex encoded
  expect(decoded.pox_treatment).toBeDefined();
});

test('decode nakamoto block header from Buffer', () => {
  const headerBuffer = Buffer.from(NAKAMOTO_HEADER_HEX.slice(2), 'hex');
  const decoded = decodeNakamotoBlockHeader(headerBuffer);
  
  expect(decoded.version).toEqual(1);
  expect(decoded.chain_length).toEqual('2');
  expect(decoded.burn_spent).toEqual('3');
});

test('decode nakamoto block header - validates header structure', () => {
  const decoded: DecodedNakamotoBlockHeader = decodeNakamotoBlockHeader(NAKAMOTO_HEADER_HEX);
  
  // Type assertions to validate the interface
  const version: number = decoded.version;
  const chainLength: string = decoded.chain_length;
  const burnSpent: string = decoded.burn_spent;
  const consensusHash: string = decoded.consensus_hash;
  const parentBlockId: string = decoded.parent_block_id;
  const txMerkleRoot: string = decoded.tx_merkle_root;
  const stateIndexRoot: string = decoded.state_index_root;
  const timestamp: string = decoded.timestamp;
  const minerSignature: string = decoded.miner_signature;
  const signerSignatures: string[] = decoded.signer_signature;
  const poxTreatment: string = decoded.pox_treatment;
  
  expect(version).toBe(1);
  expect(chainLength).toBe('2');
  expect(burnSpent).toBe('3');
  expect(consensusHash.startsWith('0x')).toBe(true);
  expect(parentBlockId.startsWith('0x')).toBe(true);
  expect(txMerkleRoot.startsWith('0x')).toBe(true);
  expect(stateIndexRoot.startsWith('0x')).toBe(true);
  expect(timestamp).toBe('8');
  expect(minerSignature.startsWith('0x')).toBe(true);
  expect(Array.isArray(signerSignatures)).toBe(true);
  expect(poxTreatment.startsWith('0x')).toBe(true);
});

test('decode nakamoto block header - invalid hex should throw', () => {
  expect(() => {
    decodeNakamotoBlockHeader('0xdeadbeef');
  }).toThrow();
});

test('decode nakamoto block header - empty input should throw', () => {
  expect(() => {
    decodeNakamotoBlockHeader('0x');
  }).toThrow();
});

// Note: Full block decoding requires a valid block with transactions.
// The block format is: header + transaction count (u64 BE) + serialized transactions
// For a minimal block test, we'd need at least a tenure change tx and coinbase tx.
// The following test verifies that the function exists and throws on invalid input.

test('decode nakamoto block - invalid hex should throw', () => {
  expect(() => {
    decodeNakamotoBlock('0xdeadbeef');
  }).toThrow();
});

test('decode nakamoto block - validates block structure', () => {
  // A minimal Nakamoto block would need:
  // 1. A valid header
  // 2. Transaction count (u64 BE)
  // 3. At least tenure change tx + coinbase tx for tenure start blocks
  //
  // For now, we test that an empty transaction list block can be parsed
  // (though it won't validate semantically in the real chain)
  
  // Block = header + tx_count (0 transactions for test)
  const blockWithNoTxs = NAKAMOTO_HEADER_HEX + '0000000000000000';
  
  const decoded: DecodedNakamotoBlockResult = decodeNakamotoBlock(blockWithNoTxs);
  
  // Validate structure
  expect(decoded.block_id).toBeDefined();
  expect(decoded.block_id.startsWith('0x')).toBe(true);
  expect(decoded.header).toBeDefined();
  expect(decoded.header.version).toEqual(1);
  expect(decoded.txs).toBeDefined();
  expect(Array.isArray(decoded.txs)).toBe(true);
  expect(decoded.txs).toHaveLength(0);
});

test('decode nakamoto block - block_id is derived from header', () => {
  const blockWithNoTxs = NAKAMOTO_HEADER_HEX + '0000000000000000';
  const decoded = decodeNakamotoBlock(blockWithNoTxs);
  
  // block_id should be a 32-byte hash (64 hex chars + 0x prefix)
  expect(decoded.block_id).toMatch(/^0x[a-f0-9]{64}$/);
  
  // The block_id should be consistent
  const decoded2 = decodeNakamotoBlock(blockWithNoTxs);
  expect(decoded.block_id).toEqual(decoded2.block_id);
});
