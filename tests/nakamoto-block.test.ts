import { decodeNakamotoBlock, decodeStacksBlock, DecodedNakamotoBlockResult, DecodedStacksBlockResult } from '../index';

describe('Nakamoto block decoding', () => {
  // This is a minimal valid Nakamoto block structure for testing
  // In practice, you would use real block data from the API
  
  it('should decode a simple Nakamoto block header structure', () => {
    // Build a minimal test block with:
    // - header with version, chain_length, burn_spent, consensus_hash, etc.
    // - 0 transactions
    // - minimal pox_treatment bitvec
    
    const headerParts = [
      '00',                                                                 // version
      '0000000000000001',                                                   // chain_length (1)
      '0000000000000064',                                                   // burn_spent (100)
      '1111111111111111111111111111111111111111',                           // consensus_hash (20 bytes)
      '2222222222222222222222222222222222222222222222222222222222222222',   // parent_block_id (32 bytes)
      '3333333333333333333333333333333333333333333333333333333333333333',   // tx_merkle_root (32 bytes)
      '4444444444444444444444444444444444444444444444444444444444444444',   // state_index_root (32 bytes)
      '0000000065b0c2d0',                                                   // timestamp (unix timestamp)
      '00'.repeat(65),                                                      // miner_signature (65 bytes)
      '00000000',                                                           // signer_signature count (0)
      '0001',                                                               // pox_treatment len (1 bit)
      '00',                                                                 // pox_treatment data (1 byte)
    ];
    
    const txsParts = [
      '00000000',                                                           // tx count (0)
    ];
    
    const blockHex = headerParts.join('') + txsParts.join('');
    
    const result = decodeNakamotoBlock(blockHex);
    
    expect(result).toHaveProperty('block_id');
    expect(result).toHaveProperty('header');
    expect(result).toHaveProperty('txs');
    
    expect(result.header.version).toBe(0);
    expect(result.header.chain_length).toBe('1');
    expect(result.header.burn_spent).toBe('100');
    expect(result.header.consensus_hash).toBe('0x1111111111111111111111111111111111111111');
    expect(result.header.parent_block_id).toBe('0x2222222222222222222222222222222222222222222222222222222222222222');
    expect(result.header.tx_merkle_root).toBe('0x3333333333333333333333333333333333333333333333333333333333333333');
    expect(result.header.state_index_root).toBe('0x4444444444444444444444444444444444444444444444444444444444444444');
    
    expect(result.header.pox_treatment).toBeDefined();
    expect(result.header.pox_treatment.len).toBe(1);
    expect(result.header.pox_treatment.bits).toHaveLength(1);
    
    expect(result.txs).toHaveLength(0);
    
    // Computed hashes should be hex strings (with 0x prefix)
    expect(result.header.block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.header.index_block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.block_id).toMatch(/^0x[0-9a-f]{64}$/);
    
    // block_id should equal index_block_hash
    expect(result.block_id).toBe(result.header.index_block_hash);
  });

  it('should decode a Nakamoto block with a transaction', () => {
    // Build a test block with one token transfer transaction
    const headerParts = [
      '00',                                                                 // version
      '0000000000000002',                                                   // chain_length (2)
      '00000000000000c8',                                                   // burn_spent (200)
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',                           // consensus_hash (20 bytes)
      'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',   // parent_block_id (32 bytes)
      'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',   // tx_merkle_root (32 bytes)
      'dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',   // state_index_root (32 bytes)
      '0000000065b0c2d0',                                                   // timestamp
      '00'.repeat(65),                                                      // miner_signature (65 bytes)
      '00000000',                                                           // signer_signature count (0)
      '0008',                                                               // pox_treatment len (8 bits)
      'ff',                                                                 // pox_treatment data (all 1s)
    ];
    
    // A simple token transfer transaction
    // Note: This is the format for a mainnet token transfer
    const txHex = [
      '00',                                                                 // version (mainnet)
      '00000001',                                                           // chain_id (mainnet)
      '04',                                                                 // auth type (standard)
      '00',                                                                 // hash mode (p2pkh)
      '1111111111111111111111111111111111111111',                           // signer (20 bytes)
      '0000000000000001',                                                   // nonce (1)
      '0000000000000064',                                                   // fee (100)
      '00',                                                                 // key_encoding (compressed)
      '00'.repeat(65),                                                      // signature (65 bytes)
      '03',                                                                 // anchor mode (any)
      '01',                                                                 // post_condition_mode (allow)
      '00000000',                                                           // post_conditions count (0)
      '00',                                                                 // payload type (token transfer)
      '05',                                                                 // principal type (standard)
      '16',                                                                 // address version (testnet)
      '2222222222222222222222222222222222222222',                           // address hash (20 bytes)
      '0000000000000001',                                                   // amount (1)
      '00'.repeat(34),                                                      // memo (34 bytes)
    ].join('');

    const blockHex = headerParts.join('') + '00000001' + txHex;
    
    const result = decodeNakamotoBlock(blockHex);
    
    expect(result.txs).toHaveLength(1);
    expect(result.txs[0]).toHaveProperty('version');
    expect(result.txs[0]).toHaveProperty('chain_id');
    expect(result.txs[0]).toHaveProperty('auth');
    expect(result.txs[0]).toHaveProperty('payload');
    
    // Check pox_treatment
    expect(result.header.pox_treatment.len).toBe(8);
    expect(result.header.pox_treatment.bits).toEqual([true, true, true, true, true, true, true, true]);
  });

  it('should handle invalid block data gracefully', () => {
    expect(() => {
      decodeNakamotoBlock('deadbeef');
    }).toThrow();
  });
});

describe('Stacks 2.x block decoding', () => {
  it('should decode a simple Stacks 2.x block header structure', () => {
    // Build a minimal test block with:
    // - header with version, total_work, proof, parent_block, etc.
    // - 0 transactions
    
    const headerParts = [
      '00',                                                                 // version
      '0000000000000001',                                                   // total_work.burn
      '0000000000000001',                                                   // total_work.work
      '00'.repeat(80),                                                      // VRF proof (80 bytes)
      '1111111111111111111111111111111111111111111111111111111111111111',   // parent_block (32 bytes)
      '2222222222222222222222222222222222222222222222222222222222222222',   // parent_microblock (32 bytes)
      '0000',                                                               // parent_microblock_sequence (0)
      '3333333333333333333333333333333333333333333333333333333333333333',   // tx_merkle_root (32 bytes)
      '4444444444444444444444444444444444444444444444444444444444444444',   // state_index_root (32 bytes)
      '5555555555555555555555555555555555555555',                           // microblock_pubkey_hash (20 bytes)
    ];
    
    const txsParts = [
      '00000000',                                                           // tx count (0)
    ];
    
    const blockHex = headerParts.join('') + txsParts.join('');
    
    const result = decodeStacksBlock(blockHex);
    
    expect(result).toHaveProperty('block_hash');
    expect(result).toHaveProperty('header');
    expect(result).toHaveProperty('txs');
    
    expect(result.header.version).toBe(0);
    expect(result.header.total_work.burn).toBe('1');
    expect(result.header.total_work.work).toBe('1');
    expect(result.header.parent_block).toBe('0x1111111111111111111111111111111111111111111111111111111111111111');
    expect(result.header.parent_microblock).toBe('0x2222222222222222222222222222222222222222222222222222222222222222');
    expect(result.header.parent_microblock_sequence).toBe(0);
    expect(result.header.tx_merkle_root).toBe('0x3333333333333333333333333333333333333333333333333333333333333333');
    expect(result.header.state_index_root).toBe('0x4444444444444444444444444444444444444444444444444444444444444444');
    expect(result.header.microblock_pubkey_hash).toBe('0x5555555555555555555555555555555555555555');
    
    expect(result.txs).toHaveLength(0);
    
    // Computed hash should be hex string (with 0x prefix)
    expect(result.header.block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.block_hash).toMatch(/^0x[0-9a-f]{64}$/);
    expect(result.block_hash).toBe(result.header.block_hash);
  });

  it('should handle invalid block data gracefully', () => {
    expect(() => {
      decodeStacksBlock('deadbeef');
    }).toThrow();
  });
});
