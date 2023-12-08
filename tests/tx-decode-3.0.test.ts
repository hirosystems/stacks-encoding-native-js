import {
  AnchorModeID,
  decodeTransaction,
  PostConditionAuthFlag,
  PostConditionModeID,
  PrincipalTypeID,
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

test('stacks3.0 - decode tx - nakamoto coinbase - no alt recipient (mockamoto vector)', () => {
  const nakamotoCoinbaseTx = '80800000000400b40723ab4d7781cf1b45083aa043ce4563006c6100000000000000010000000000000000000158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f01020000000008010101010101010101010101010101010101010101010101010101010101010109119054d8cfba5f6aebaac75b0f6671a6917211729fa7bafa35ab0ad68fe243cf4169eb339d8a26ee8e036c8380e3afd63da8aca1f9673d19a59ef00bf13e1ba2e540257d0b471fc591a877a90e04e00b';
  const decoded = decodeTransaction(nakamotoCoinbaseTx);
  expect(decoded).toEqual({
    "tx_id": "0x1ecc33bfdd58a94ff97afb6d64a2ebefb0021f22490767e844ebd80285486e16",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0xb40723ab4d7781cf1b45083aa043ce4563006c61",
          "address": "ST2T0E8XB9NVR3KRV8M43N823SS2P603CC4Y4DG1V"
        },
        "nonce": "1",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x0158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f"
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
      "vrf_proof": "0x119054d8cfba5f6aebaac75b0f6671a6917211729fa7bafa35ab0ad68fe243cf4169eb339d8a26ee8e036c8380e3afd63da8aca1f9673d19a59ef00bf13e1ba2e540257d0b471fc591a877a90e04e00b"
    }
  });

  const payload = decoded.payload as TxPayloadNakamotoCoinbase;
  const txType: TxPayloadTypeID.NakamotoCoinbase = payload.type_id;
  expect(txType).toEqual(TxPayloadTypeID.NakamotoCoinbase);
  expect(payload.recipient).toBeNull();
});

test('stacks3.0 - decode tx - nakamoto coinbase - no alt recipient (stacks-core vector 1)', () => {
  const nakamotoCoinbaseTx = '80800000000400b40723ab4d7781cf1b45083aa043ce4563006c6100000000000000010000000000000000000158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f010200000000081212121212121212121212121212121212121212121212121212121212121212099275df67a68c8745c0ff97b48201ee6db447f7c93b23ae24cdc2400f52fdb08a1a6ac7ec71bf9c9c76e96ee4675ebff60625af28718501047bfd87b810c2d2139b73c23bd69de66360953a642c2a330a';
  const decoded = decodeTransaction(nakamotoCoinbaseTx);
  expect(decoded).toEqual({
    "tx_id": "0x3f23c7c7d865e1ff924950bf03b12eecb949c68f024fcad45b6d8e2420fb77cc",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0xb40723ab4d7781cf1b45083aa043ce4563006c61",
          "address": "ST2T0E8XB9NVR3KRV8M43N823SS2P603CC4Y4DG1V"
        },
        "nonce": "1",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x0158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f"
      }
    },
    "anchor_mode": AnchorModeID.OnChainOnly,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.NakamotoCoinbase,
      "payload_buffer": "0x1212121212121212121212121212121212121212121212121212121212121212",
      "recipient": null,
      "vrf_proof": "0x9275df67a68c8745c0ff97b48201ee6db447f7c93b23ae24cdc2400f52fdb08a1a6ac7ec71bf9c9c76e96ee4675ebff60625af28718501047bfd87b810c2d2139b73c23bd69de66360953a642c2a330a"
    }
  });

  const payload = decoded.payload as TxPayloadNakamotoCoinbase;
  const txType: TxPayloadTypeID.NakamotoCoinbase = payload.type_id;
  expect(txType).toEqual(TxPayloadTypeID.NakamotoCoinbase);
  expect(payload.recipient).toBeNull();
});

test('stacks3.0 - decode tx - nakamoto coinbase - no alt recipient (stacks-core vector 2)', () => {
  const nakamotoCoinbaseTx = '80800000000400b40723ab4d7781cf1b45083aa043ce4563006c6100000000000000010000000000000000000158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f0102000000000812121212121212121212121212121212121212121212121212121212121212120a0601ffffffffffffffffffffffffffffffffffffffff0c666f6f2d636f6e74726163749275df67a68c8745c0ff97b48201ee6db447f7c93b23ae24cdc2400f52fdb08a1a6ac7ec71bf9c9c76e96ee4675ebff60625af28718501047bfd87b810c2d2139b73c23bd69de66360953a642c2a330a';
  const decoded = decodeTransaction(nakamotoCoinbaseTx);
  expect(decoded).toEqual({
    "tx_id": "0x3448d47b2e2ef6db517963e1d8e7534ba84afccac9b2c79c1dcf32b21f56871a",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0xb40723ab4d7781cf1b45083aa043ce4563006c61",
          "address": "ST2T0E8XB9NVR3KRV8M43N823SS2P603CC4Y4DG1V"
        },
        "nonce": "1",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x0158be820619a4838f74e63099bb113fcf7ee13ef3b2bb56728cd19470f9379f05288d4accc987d8dd85de5101776c2ad000784d118e35deb4f02852540bf6dd5f"
      }
    },
    "anchor_mode": AnchorModeID.OnChainOnly,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.NakamotoCoinbase,
      "payload_buffer": "0x1212121212121212121212121212121212121212121212121212121212121212",
      "recipient": {
        "address": "S13ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZXCFYZCG",
        "address_hash_bytes": "0xffffffffffffffffffffffffffffffffffffffff",
        "address_version": 1,
        "contract_name": "foo-contract",
        "type_id": PrincipalTypeID.Contract,
      },
      "vrf_proof": "0x9275df67a68c8745c0ff97b48201ee6db447f7c93b23ae24cdc2400f52fdb08a1a6ac7ec71bf9c9c76e96ee4675ebff60625af28718501047bfd87b810c2d2139b73c23bd69de66360953a642c2a330a"
    }
  });

  const payload = decoded.payload as TxPayloadNakamotoCoinbase;
  const txType: TxPayloadTypeID.NakamotoCoinbase = payload.type_id;
  expect(txType).toEqual(TxPayloadTypeID.NakamotoCoinbase);
  expect(payload.recipient).not.toBeNull();
});
