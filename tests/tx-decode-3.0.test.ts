import {
  AnchorModeID,
  decodeTransaction,
  PostConditionAuthFlag,
  PostConditionModeID,
  TenureChangeCause,
  TransactionVersion,
  TxPayloadNakamotoCoinbase,
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
      "previous_tenure_blocks": 1,
      "cause": TenureChangeCause.BlockFound,
      "pubkey_hash": "0x0000000000000000000000000000000000000000",
      "signature": "0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817980000000000000000000000000000000000000000000000000000000000000000",
      "signers": "0x"
    }
  });
});

test('stacks3.0 - decode tx - nakamoto coinbase - no alt recipient', () => {
  const tenureChangeTx = '80800000000400ad0cc5ca0b4571dd435a9da7e16cbc662716dceb00000000000000010000000000000000000015833671ecd7432e6412423273eebf8a78d973beb08f690e58ba548f67ee26584967a5bc24d44f27ecca18e82a9956181e9d9cef7c67f718b33c5f5d0f82643801020000000008010101010101010101010101010101010101010101010101010101010101010109000000506f77e9a15503066b515060aa438ae3f5bc5207339b8e2933bdeae0891362d8e7ca2e5b047153904272d5f030ddcc83333676df6583394b0852a7e411b7c8d4c973f17fb7687601891ad7ca6707aa8408';
  const decoded = decodeTransaction(tenureChangeTx);
  expect(decoded).toEqual({
    "tx_id": "0xa18614990f3a67b8ab13ec95846aebd409b2ef85017c900840436ac547a537aa",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0xad0cc5ca0b4571dd435a9da7e16cbc662716dceb",
          "address": "ST2PGSHEA1D2Q3QA3BAETFRBCQHK2E5PWXECD5E7T"
        },
        "nonce": "1",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x0015833671ecd7432e6412423273eebf8a78d973beb08f690e58ba548f67ee26584967a5bc24d44f27ecca18e82a9956181e9d9cef7c67f718b33c5f5d0f826438"
      }
    },
    "anchor_mode": AnchorModeID.OnChainOnly,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.NakamotoCoinbase,
      "payload_buffer": "0x0101010101010101010101010101010101010101010101010101010101010101",
      "recipient": null,
      "vrf_proof": "0x6f77e9a15503066b515060aa438ae3f5bc5207339b8e2933bdeae0891362d8e7ca2e5b047153904272d5f030ddcc83333676df6583394b0852a7e411b7c8d4c973f17fb7687601891ad7ca6707aa8408"
    }
  });

  const payload = decoded.payload as TxPayloadNakamotoCoinbase;
  const txType: TxPayloadTypeID.NakamotoCoinbase = payload.type_id;
  expect(txType).toEqual(TxPayloadTypeID.NakamotoCoinbase);
  expect(payload.recipient).toBeNull();
});
