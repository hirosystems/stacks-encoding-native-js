import {
  AnchorModeID,
  decodeTransaction,
  PostConditionAuthFlag,
  PostConditionModeID,
  TenureChangeCause,
  TransactionVersion,
  TxPayloadTypeID,
  TxPublicKeyEncoding
} from '../index.js';

test('stacks3.0 - decode tx - tenure change', () => {
  const tenureChangeTx = '808000000004000f873150e9790e305b701aa8c7b3bcff9e31a5f9000000000000000000000000000000000001d367da530b92f4984f537f0b903c330eb5158262afa08d67cbbdea6c8e2ecae06008248ac147fc34101d3cc207b1b3e386e0f53732b5548bd5abe1570c2271340302000000000755c9861be5cff984a20ce6d99d4aa65941412889bdc665094136429b84f8c2ee00000001000000000000000000000000000000000000000000000000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817980000000000000000000000000000000000000000000000000000000000000000';
  const decoded = decodeTransaction(tenureChangeTx);
  expect(decoded).toEqual({
    "tx_id": "0xc00148be5e8edb457d1bd1ae7ae5fdc2b74b64455f714d512e717deddeedf069",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0x0f873150e9790e305b701aa8c7b3bcff9e31a5f9",
          "address": "ST7RECAGX5WGWC2VE0DAHHXKQKZSWCD5Z4JRG6SR"
        },
        "nonce": "0",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x01d367da530b92f4984f537f0b903c330eb5158262afa08d67cbbdea6c8e2ecae06008248ac147fc34101d3cc207b1b3e386e0f53732b5548bd5abe1570c227134"
      }
    },
    "anchor_mode": AnchorModeID.Any,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.TenureChange,
      "previous_tenure_end": "0x55c9861be5cff984a20ce6d99d4aa65941412889bdc665094136429b84f8c2ee",
      "previous_tenure_blocks": 0,
      "cause": TenureChangeCause.BlockFound,
      "pubkey_hash": "0x0100000000000000000000000000000000000000",
      "signature": "0x0000000000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817980000000000000000000000000000000000000000000000000000",
      "signers": "0x"
    }
  });
});
