import * as assert from 'node:assert';
import { 
  decodeTransaction,
  AnchorModeID,
  TransactionVersion,
  TxPayloadTypeID,
  TxSpendingConditionSingleSigHashMode,
  TxPublicKeyEncoding,
  ClarityTypeID,
  PostConditionModeID,
  PostConditionAuthFlag,
  PostConditionAssetInfoID,
  PostConditionNonfungibleConditionCodeID,
  PostConditionNonFungibleConditionName
} from 'stacks-encoding-native-js';

const decoded = decodeTransaction('0000000001040089f5fd1f719e4449c980de38e3504be6770a2698000000000000014500000000000001f400008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f503020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba1002162bcf9762d5b90bc36dc1b4759b1727690f92ddd30e6d61726b6574706c6163652d76340a6c6973742d6173736574000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8');

assert.deepStrictEqual(decoded, {
  tx_id: "0x49bcdba540d5c486f6f7a71de639a128a65f0378d7571e20c75a61661dd5b469",
  chain_id: 1,
  version: TransactionVersion.Mainnet,
  anchor_mode: AnchorModeID.Any,
  auth: {
    type_id: PostConditionAuthFlag.Standard,
    origin_condition: {
      hash_mode: TxSpendingConditionSingleSigHashMode.P2PKH,
      key_encoding: TxPublicKeyEncoding.Compressed,
      nonce: "325",
      signature: "0x008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f5",
      signer: {
        address: "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K",
        address_hash_bytes: "0x89f5fd1f719e4449c980de38e3504be6770a2698",
        address_version: 22
      },
      tx_fee: "500"
    }
  },
  payload: {
    type_id: TxPayloadTypeID.ContractCall,
    address: "SPNWZ5V2TPWGQGVDR6T7B6RQ4XMGZ4PXTEE0VQ0S",
    address_hash_bytes: "0x2bcf9762d5b90bc36dc1b4759b1727690f92ddd3",
    address_version: 22,
    contract_name: "marketplace-v4",
    function_name: "list-asset",
    function_args: [
      {
        type_id: ClarityTypeID.PrincipalContract,
        hex: "0x0616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c616273",
        repr: "'SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x01000000000000000000000000000008ba",
        repr: "u2234"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x010000000000000000000000000c845880",
        repr: "u210000000"
      },
      {
        type_id: ClarityTypeID.UInt,
        hex: "0x01000000000000000000000000000000c8",
        repr: "u200"
      }
    ],
    function_args_buffer: "0x000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8"
  },
  post_condition_mode: PostConditionModeID.Deny,
  post_conditions: [
    {
      condition_code: PostConditionNonfungibleConditionCodeID.Sent,
      condition_name: PostConditionNonFungibleConditionName.Sent,
      asset: {
        asset_name: "bitcoin-monkeys-labs",
        contract_address: "SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C",
        contract_name: "bitcoin-monkeys-labs"
      },
      asset_info_id: PostConditionAssetInfoID.NonfungibleAsset,
      asset_value: {
        hex: "0x01000000000000000000000000000008ba",
        repr: "u2234",
        type_id: ClarityTypeID.UInt
      },
      principal: {
        address: "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K",
        address_hash_bytes: "0x89f5fd1f719e4449c980de38e3504be6770a2698",
        address_version: 22,
        type_id: ClarityTypeID.Buffer
      }
    }
  ],
  post_conditions_buffer: "0x020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba10"
});
