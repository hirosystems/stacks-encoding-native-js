import * as assert from 'node:assert';
import { ALICE_TUPLE_CV_HEX } from './encode-values';
import { 
  decodeClarityValue,
  ClarityTypeID,
  ClarityValueTuple,
  ClarityValuePrincipalStandard,
  ClarityValueStringAscii,
  ClarityValueInt,
  ClarityValueBoolTrue,
  ClarityValueResponseOk,
  ClarityValueBuffer,
  ClarityValueList,
  ClarityValueStringUtf8,
  ClarityValueOptionalSome,
  ClarityValueUInt,
} from 'stacks-encoding-native-js';

// Specify types to provides a typed view of the decoded value.
const decoded = decodeClarityValue<ClarityValueTuple<{
  active: ClarityValueBoolTrue;
  address: ClarityValuePrincipalStandard;
  alias: ClarityValueStringAscii;
  balance: ClarityValueUInt;
  ping: ClarityValueResponseOk<ClarityValueInt>;
  public_key: ClarityValueOptionalSome<ClarityValueBuffer>;
  tags: ClarityValueList<ClarityValueStringUtf8>;
}>>(ALICE_TUPLE_CV_HEX);

// The annotated types are _not_ automatically checked at runtime, so type checks are needed for error handling
assert.strictEqual(decoded.type_id, ClarityTypeID.Tuple);
assert.strictEqual(decoded.data.active.type_id, ClarityTypeID.BoolTrue);
assert.strictEqual(decoded.data.address.type_id, ClarityTypeID.PrincipalStandard);
assert.strictEqual(decoded.data.alias.type_id, ClarityTypeID.StringAscii);
assert.strictEqual(decoded.data.balance.type_id, ClarityTypeID.UInt);
assert.strictEqual(decoded.data.ping.type_id, ClarityTypeID.ResponseOk);
assert.strictEqual(decoded.data.ping.value.type_id, ClarityTypeID.Int);
assert.strictEqual(decoded.data.public_key.type_id, ClarityTypeID.OptionalSome);
assert.strictEqual(decoded.data.public_key.value.type_id, ClarityTypeID.Buffer);
assert.strictEqual(decoded.data.tags.type_id, ClarityTypeID.List);
assert.strictEqual(decoded.data.tags.list[0].type_id, ClarityTypeID.StringUtf8);

// Now we can safely access typed properties
assert.strictEqual(decoded.data.active.value, true);
assert.strictEqual(decoded.data.address.address, 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW');
assert.strictEqual(decoded.data.alias.data, 'Alice');
assert.strictEqual(decoded.data.balance.value, '2000');
assert.strictEqual(decoded.data.ping.value.value, '250');
assert.strictEqual(decoded.data.public_key.value.buffer, '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58');
assert.deepStrictEqual(decoded.data.tags.list.map(v => v.data), ['contributor', 'og ⭐', 'clarity']);

console.log('ok: decode cv hex string to typed object');
