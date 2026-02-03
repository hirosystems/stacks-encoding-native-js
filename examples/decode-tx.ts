import * as assert from 'node:assert';
import { 
  decodeTransaction,
  AnchorModeID,
  TransactionVersion,
  TxPayloadTypeID,
  TxSpendingConditionSingleSigHashMode,
  ClarityTypeID,
  PostConditionModeID,
  PostConditionAuthFlag,
  PostConditionAssetInfoID,
  PostConditionNonfungibleConditionCodeID,
  PostConditionNonFungibleConditionName,
  TxPayloadContractCall,
  PostConditionNonfungible,
  PostConditionPrincipalStandard,
} from 'stacks-encoding-native-js';

const decoded = decodeTransaction('0000000001040089f5fd1f719e4449c980de38e3504be6770a2698000000000000014500000000000001f400008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f503020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba1002162bcf9762d5b90bc36dc1b4759b1727690f92ddd30e6d61726b6574706c6163652d76340a6c6973742d6173736574000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8');

// Verify basic transaction structure
assert.strictEqual(decoded.tx_id, "0x49bcdba540d5c486f6f7a71de639a128a65f0378d7571e20c75a61661dd5b469");
assert.strictEqual(decoded.version, TransactionVersion.Mainnet);
assert.strictEqual(decoded.chain_id, 1);
assert.strictEqual(decoded.anchor_mode, AnchorModeID.Any);
assert.strictEqual(decoded.post_condition_mode, PostConditionModeID.Deny);

// Verify auth
assert.strictEqual(decoded.auth.type_id, PostConditionAuthFlag.Standard);
assert.strictEqual(decoded.auth.origin_condition.hash_mode, TxSpendingConditionSingleSigHashMode.P2PKH);
assert.strictEqual(decoded.auth.origin_condition.signer.address, "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K");
assert.strictEqual(decoded.auth.origin_condition.nonce, "325");
assert.strictEqual(decoded.auth.origin_condition.tx_fee, "500");

// Verify payload (contract call)
const payload = decoded.payload as TxPayloadContractCall;
assert.strictEqual(payload.type_id, TxPayloadTypeID.ContractCall);
assert.strictEqual(payload.address, "SPNWZ5V2TPWGQGVDR6T7B6RQ4XMGZ4PXTEE0VQ0S");
assert.strictEqual(payload.contract_name, "marketplace-v4");
assert.strictEqual(payload.function_name, "list-asset");
assert.strictEqual(payload.function_args.length, 4);

// Verify first function arg (contract principal)
assert.strictEqual(payload.function_args[0].type_id, ClarityTypeID.PrincipalContract);
assert.strictEqual(payload.function_args[0].repr, "SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs");

// Verify post conditions
assert.strictEqual(decoded.post_conditions.length, 1);
const postCondition = decoded.post_conditions[0] as PostConditionNonfungible;
assert.strictEqual(postCondition.asset_info_id, PostConditionAssetInfoID.NonfungibleAsset);
assert.strictEqual(postCondition.condition_code, PostConditionNonfungibleConditionCodeID.Sent);
assert.strictEqual(postCondition.condition_name, PostConditionNonFungibleConditionName.Sent);
assert.strictEqual(postCondition.asset.contract_address, "SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C");
assert.strictEqual(postCondition.asset.contract_name, "bitcoin-monkeys-labs");
assert.strictEqual((postCondition.principal as PostConditionPrincipalStandard).address, "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K");

console.log('ok: decode transaction');
