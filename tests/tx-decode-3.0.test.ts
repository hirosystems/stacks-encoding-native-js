import {
  AnchorModeID,
  ClarityVersion,
  decodeTransaction,
  PostConditionAuthFlag,
  PostConditionModeID,
  PrincipalTypeID,
  TenureChangeCause,
  TransactionVersion,
  TxAuthFieldTypeID,
  TxPayloadNakamotoCoinbase,
  TxPayloadTypeID,
  TxPublicKeyEncoding,
  TxSpendingConditionMultiSigHashMode,
  TxSpendingConditionSingleSigHashMode
} from '../index.js';

test('stacks3.0 - decode tx - tenure change', () => {
  const tenureChangeTx = '808000000004001dc27eba0247f8cc9575e7d45e50a0bc7e72427d000000000000001d000000000000000000011dc72b6dfd9b36e414a2709e3b01eb5bbdd158f9bc77cd2ca6c3c8b0c803613e2189f6dacf709b34e8182e99d3a1af15812b75e59357d9c255c772695998665f010200000000076f2ff2c4517ab683bf2d588727f09603cc3e9328b9c500e21a939ead57c0560af8a3a132bd7d56566f2ff2c4517ab683bf2d588727f09603cc3e932828dcefb98f6b221eef731cabec7538314441c1e0ff06b44c22085d41aae447c1000000010014ff3cb19986645fd7e71282ad9fea07d540a60e';
  const decoded = decodeTransaction(tenureChangeTx);
  expect(decoded).toEqual({
    "tx_id": "0xd443c1edb6bbcbdb702884a688b3ed09cc2d81e391f09c4d91ac881806979620",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0x1dc27eba0247f8cc9575e7d45e50a0bc7e72427d",
          "address": "STEW4ZNT093ZHK4NEQKX8QJGM2Y7WWJ2FQQS5C19"
        },
        "nonce": "29",
        "tx_fee": "0",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x011dc72b6dfd9b36e414a2709e3b01eb5bbdd158f9bc77cd2ca6c3c8b0c803613e2189f6dacf709b34e8182e99d3a1af15812b75e59357d9c255c772695998665f"
      }
    },
    "anchor_mode": AnchorModeID.OnChainOnly,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.TenureChange,
      "tenure_consensus_hash": "0x6f2ff2c4517ab683bf2d588727f09603cc3e9328",
      "prev_tenure_consensus_hash": "0xb9c500e21a939ead57c0560af8a3a132bd7d5656",
      "burn_view_consensus_hash": "0x6f2ff2c4517ab683bf2d588727f09603cc3e9328",
      "previous_tenure_end": "0x28dcefb98f6b221eef731cabec7538314441c1e0ff06b44c22085d41aae447c1",
      "previous_tenure_blocks": 1,
      "cause": TenureChangeCause.BlockFound,
      "pubkey_hash": "0x14ff3cb19986645fd7e71282ad9fea07d540a60e",
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

test("stacks 3.0 - decode tx - non-sequential multi-sig", () => {
  const tx =
    "8080000000040535e2fdeee173024af6848ca6e335691b55498fc4000000000000000000000000000000640000000300028bd9dd96b66534e23cbcce4e69447b92bf1d738edb83182005cfb3b402666e42020158146dc95e76926e3add7289821e983e0dd2f2b0bf464c8e94bb082a213a91067ced1381a64bd03afa662992099b04d4c3f538cc6afa3d043ae081e25ebbde6f0300e30e7e744c6eef7c0a4d1a2dad6f0daa3c7655eb6e9fd6c34d1efa87b648d3e55cdd004ca4e8637cddad3316f3fbd6146665fad2e7ca26725ad09f58c4e43aa0000203020000000000051a70f696e2bda63701e044609eb7a7ce5876571905000000000000271000000000000000000000000000000000000000000000000000000000000000000000";
  const decoded = decodeTransaction(tx);
  expect(decoded).toEqual({
    "tx_id": "0xf7f30ad912e9433743fb614b17842e8a366a04cc882e7fd94ff59fa9c2638674",
    "version": TransactionVersion.Testnet,
    "chain_id": 0x80000000,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "tx_fee": "100",
        "nonce": "0",
        "fields": [
          {
            "type_id": TxAuthFieldTypeID.PublicKeyCompressed,
            "public_key":
              "0x028bd9dd96b66534e23cbcce4e69447b92bf1d738edb83182005cfb3b402666e42",
          },
          {
            "type_id": TxAuthFieldTypeID.SignatureCompressed,
            "signature":
              "0x0158146dc95e76926e3add7289821e983e0dd2f2b0bf464c8e94bb082a213a91067ced1381a64bd03afa662992099b04d4c3f538cc6afa3d043ae081e25ebbde6f",
          },
          {
            "type_id": TxAuthFieldTypeID.SignatureUncompressed,
            "signature":
              "0x00e30e7e744c6eef7c0a4d1a2dad6f0daa3c7655eb6e9fd6c34d1efa87b648d3e55cdd004ca4e8637cddad3316f3fbd6146665fad2e7ca26725ad09f58c4e43aa0",
          },
        ],
        "hash_mode": TxSpendingConditionMultiSigHashMode.P2SHNonSequential,
        "signatures_required": 2,
        "signer": {
          "address": "SNTY5ZFEW5SG4JQPGJ6ADRSND4DNAJCFRHVZBYR8",
          "address_hash_bytes": "0x35e2fdeee173024af6848ca6e335691b55498fc4",
          "address_version": 21,
        },
      },
    },
    "anchor_mode": AnchorModeID.Any,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.TokenTransfer,
      "amount": "10000",
      "recipient": {
        "type_id": PrincipalTypeID.Standard,
        "address": "ST1RFD5Q2QPK3E0F08HG9XDX7SSC7CNRS0QR0SGEV",
        "address_hash_bytes": "0x70f696e2bda63701e044609eb7a7ce5876571905",
        "address_version": 26,
      },
      "memo_hex":
        "0x00000000000000000000000000000000000000000000000000000000000000000000",
    },
  });
});

test("stacks 3.0 - decode tx - non-sequential multi-sig 2", () => {
  const tx =
    "0x0000000001040506cb614d936db3a4c31b1a73e516e88d1f7579040000000000000015000000000006c34000000005000212e37e65b9741f09eeed659ceafe577020f15032d14f6de394c93ac2aeae2cd802013e4081ba21cf55a79d44364979bd69b1ff752af92a0402ca12cb9af1a46f0a11442b4fab5a47b8b25fda073e4e1f2782eba1de1dce18564ece471eb8693280b30200942b48e62e7c90aeb3c37f94022c60b8ecc1cf54d666812f6cbd267b12dc99df07c2955aabb9e136a6616eb20fe2bc5719cf3168644e379182ccbf51418de9ea02010df1a1161a65deafc79c170f5ef05707c399802c6257a1f1fb600414ed6ee08772b12a031e67682aeb179320da141b4679be718aa0545efca4714b3aa63db1660201f865ae066057deeeafaa4b7ad796b94f5d965e3de92699aedf49d9756b00c3617703979b67d9c0daa7f30184f722f7fa75edb0c6f4041a28d8e619323915a32000030302000000000216495ca29b5e23da51265dfc0388ea91539e5b67c510706f6e7469732d6272696467652d7635116d696e742d6274632d66726f6d2d627463000000060616495ca29b5e23da51265dfc0388ea91539e5b67c512706f6e7469732d6272696467652d7042544301000000000000000000000000004c4b400516d105ee658ba40a2e24662498dddc41a3d0452ab00200000021904e124b3d17611f769d641e4f366697ea850fe94c630e916b90feb6521f613d0102000000200000000000000000000145608686892318974350a6a12c218318f104f066f00c010000000000000000000000000004c51e";
  const decoded = decodeTransaction(tx);
  expect(decoded).toEqual({
    "tx_id": "0xf38135deaacc8bbb1bedc9d5e976148ef45f32daa24867a071d7b9541e8e0988",
    "version": 0,
    "chain_id": 1,
    "auth": {
      "type_id": 4,
      "origin_condition": {
        "hash_mode": 5,
        "signer": {
          "address_version": 20,
          "address_hash_bytes": "0x06cb614d936db3a4c31b1a73e516e88d1f757904",
          "address": "SM3CPRADJDPV79633CD77S8PX26HYXBS0KS80ZD5"
        }, 
        "nonce": "21", 
        "tx_fee": "443200", 
        "fields": [
          { "type_id": 0, "public_key": "0x0212e37e65b9741f09eeed659ceafe577020f15032d14f6de394c93ac2aeae2cd8" },
          { "type_id": 2, "signature": "0x013e4081ba21cf55a79d44364979bd69b1ff752af92a0402ca12cb9af1a46f0a11442b4fab5a47b8b25fda073e4e1f2782eba1de1dce18564ece471eb8693280b3" },
          { "type_id": 2, "signature": "0x00942b48e62e7c90aeb3c37f94022c60b8ecc1cf54d666812f6cbd267b12dc99df07c2955aabb9e136a6616eb20fe2bc5719cf3168644e379182ccbf51418de9ea" },
          { "type_id": 2, "signature": "0x010df1a1161a65deafc79c170f5ef05707c399802c6257a1f1fb600414ed6ee08772b12a031e67682aeb179320da141b4679be718aa0545efca4714b3aa63db166" },
          { "type_id": 2, "signature": "0x01f865ae066057deeeafaa4b7ad796b94f5d965e3de92699aedf49d9756b00c3617703979b67d9c0daa7f30184f722f7fa75edb0c6f4041a28d8e619323915a320" }
        ],
        "signatures_required": 3
      }
    },
    "anchor_mode": 3,
    "post_condition_mode": 2,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": 2,
      "address_version": 22,
      "address_hash_bytes": "0x495ca29b5e23da51265dfc0388ea91539e5b67c5",
      "address": "SP14NS8MVBRHXMM96BQY0727AJ59SWPV7RMHC0NCG",
      "contract_name": "pontis-bridge-v5",
      "function_name": "mint-btc-from-btc",
      "function_args": [
        { "repr": "'SP14NS8MVBRHXMM96BQY0727AJ59SWPV7RMHC0NCG.pontis-bridge-pBTC", "hex": "0x0616495ca29b5e23da51265dfc0388ea91539e5b67c512706f6e7469732d6272696467652d70425443", "type_id": 6 },
        { "repr": "u5000000", "hex": "0x01000000000000000000000000004c4b40", "type_id": 1 },
        { "repr": "'SP38GBVK5HEJ0MBH4CRJ9HQEW86HX0H9AP1HZ3SVZ", "hex": "0x0516d105ee658ba40a2e24662498dddc41a3d0452ab0", "type_id": 5 },
        { "repr": "0x904e124b3d17611f769d641e4f366697ea850fe94c630e916b90feb6521f613d01", "hex": "0x0200000021904e124b3d17611f769d641e4f366697ea850fe94c630e916b90feb6521f613d01", "type_id": 2 },
        { "repr": "0x0000000000000000000145608686892318974350a6a12c218318f104f066f00c", "hex": "0x02000000200000000000000000000145608686892318974350a6a12c218318f104f066f00c", "type_id": 2 },
        { "repr": "u312606", "hex": "0x010000000000000000000000000004c51e", "type_id": 1 }
      ],
      "function_args_buffer": "0x000000060616495ca29b5e23da51265dfc0388ea91539e5b67c512706f6e7469732d6272696467652d7042544301000000000000000000000000004c4b400516d105ee658ba40a2e24662498dddc41a3d0452ab00200000021904e124b3d17611f769d641e4f366697ea850fe94c630e916b90feb6521f613d0102000000200000000000000000000145608686892318974350a6a12c218318f104f066f00c010000000000000000000000000004c51e"
    }
  });
});

test('stacks 3.0 - decode tx - versioned smart contract Clariy 3', () => {
  const tx = '8080000000040005572d04565d56f67e84ad7e20deedd8e7bba2fd00000000000000000000000000000bb800010c3287ab0587cf952022e079d261baeacc533d8c92b754136271450eb121ba4c0849f3c47f5a6f967dd1088e8242a9f9ed0454bfadaa19c71d25c8812e47160a030200000000060307636f756e7465720000004e28646566696e652d646174612d76617220636f756e742075696e74207530290a28646566696e652d726561642d6f6e6c7920286765742d636f756e742920287661722d67657420636f756e742929';
  const decoded = decodeTransaction(tx);

  expect(decoded).toEqual({
    "tx_id": "0xcd544ee70fe7f15043245217d1ec16cc8d7f68b739adc6ab2925c1c6fd9edc3b",
    "version": TransactionVersion.Testnet,
    "chain_id": 2147483648,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": TxSpendingConditionSingleSigHashMode.P2PKH,
        "signer": {
          "address_version": 26,
          "address_hash_bytes": "0x05572d04565d56f67e84ad7e20deedd8e7bba2fd",
          "address": "ST2NEB84ASENDXKYGJPQW86YXQCEFEX2ZQPG87ND"
        },
        "nonce": "0",
        "tx_fee": "3000",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x010c3287ab0587cf952022e079d261baeacc533d8c92b754136271450eb121ba4c0849f3c47f5a6f967dd1088e8242a9f9ed0454bfadaa19c71d25c8812e47160a"
      }
    },
    "anchor_mode": AnchorModeID.Any,
    "post_condition_mode": PostConditionModeID.Deny,
    "post_conditions": [],
    "post_conditions_buffer": "0x0200000000",
    "payload": {
      "type_id": TxPayloadTypeID.VersionedSmartContract,
      "clarity_version": ClarityVersion.Clarity3,
      "contract_name": "counter",
      "code_body": "(define-data-var count uint u0)\n(define-read-only (get-count) (var-get count))"
    }
  });
});
