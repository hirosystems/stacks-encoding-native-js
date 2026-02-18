import * as fs from 'fs';
import * as path from 'path';
import {
  decodeNakamotoBlock,
  decodeStacksBlock,
  DecodedNakamotoBlockResult,
  DecodedStacksBlockResult,
} from '../index';

describe('Nakamoto block decoding', () => {
  it('should decode a Nakamoto block', () => {
    const blockBuffer = fs.readFileSync(path.join(__dirname, 'fixtures/nakamoto-block.bin'));
    const result = decodeNakamotoBlock(blockBuffer);

    expect(result).toHaveProperty('block_id');
    expect(result).toHaveProperty('header');
    expect(result).toHaveProperty('txs');

    expect(result.header.version).toBe(0);
    expect(result.header.chain_length).toBe('1');
    expect(result.header.burn_spent).toBe('100');
    expect(result.header.consensus_hash).toBe('0x1111111111111111111111111111111111111111');
    expect(result.header.parent_block_id).toBe(
      '0x2222222222222222222222222222222222222222222222222222222222222222'
    );
    expect(result.header.tx_merkle_root).toBe(
      '0x3333333333333333333333333333333333333333333333333333333333333333'
    );
    expect(result.header.state_index_root).toBe(
      '0x4444444444444444444444444444444444444444444444444444444444444444'
    );

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
      '00', // version
      '0000000000000001', // total_work.burn
      '0000000000000001', // total_work.work
      '00'.repeat(80), // VRF proof (80 bytes)
      '1111111111111111111111111111111111111111111111111111111111111111', // parent_block (32 bytes)
      '2222222222222222222222222222222222222222222222222222222222222222', // parent_microblock (32 bytes)
      '0000', // parent_microblock_sequence (0)
      '3333333333333333333333333333333333333333333333333333333333333333', // tx_merkle_root (32 bytes)
      '4444444444444444444444444444444444444444444444444444444444444444', // state_index_root (32 bytes)
      '5555555555555555555555555555555555555555', // microblock_pubkey_hash (20 bytes)
    ];

    const txsParts = [
      '00000000', // tx count (0)
    ];

    const blockHex = headerParts.join('') + txsParts.join('');

    const result = decodeStacksBlock(blockHex);

    expect(result).toHaveProperty('block_hash');
    expect(result).toHaveProperty('header');
    expect(result).toHaveProperty('txs');

    expect(result.header.version).toBe(0);
    expect(result.header.total_work.burn).toBe('1');
    expect(result.header.total_work.work).toBe('1');
    expect(result.header.parent_block).toBe(
      '0x1111111111111111111111111111111111111111111111111111111111111111'
    );
    expect(result.header.parent_microblock).toBe(
      '0x2222222222222222222222222222222222222222222222222222222222222222'
    );
    expect(result.header.parent_microblock_sequence).toBe(0);
    expect(result.header.tx_merkle_root).toBe(
      '0x3333333333333333333333333333333333333333333333333333333333333333'
    );
    expect(result.header.state_index_root).toBe(
      '0x4444444444444444444444444444444444444444444444444444444444444444'
    );
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
