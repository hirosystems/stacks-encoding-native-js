# stacks-encoding-native-js

[![npm version](https://badge.fury.io/js/stacks-encoding-native-js.svg)](https://www.npmjs.com/package/stacks-encoding-native-js)
[![ci](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/ci.yml/badge.svg)](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/ci.yml)

`stacks-encoding-native-js` is a Node.js [native addon](https://nodejs.org/api/addons.html) library written in Rust, which provides functions for decoding binary/wire formats used in the Stacks blockchain. Features include:

- **Clarity values** - Decode serialized Clarity values to repr strings or structured objects
- **Transactions** - Decode Stacks transactions including all payload types
- **Nakamoto blocks** - Decode Nakamoto block headers and full blocks
- **Post-conditions** - Decode transaction post-conditions
- **Addresses** - Convert between Stacks and Bitcoin address formats

Under the hood, this library uses the [`stacks-codec`](https://github.com/stx-labs/clarinet/tree/main/components/stacks-codec) crate from Clarinet for wire format deserialization, ensuring compatibility with the canonical Stacks implementation.

_This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon)._

### Runtime Support
* Node.js v16+

### Platform Support

* Linux (glibc) x86_64
* Linux (glibc) aarch64
* Linux (musl) x86_64
* Linux (musl) aarch64
* MacOS x86_64
* MacOS aarch64 (Apple Silicon M1)
* Windows x86_64

## Installation and Usage

```shell
npm install stacks-encoding-native-js
```

### Decoding serialized Clarity values
_Example Clarity value:_
<details>
<summary>Expand</summary>

```clar
(tuple 
  (active true) 
  (address SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) 
  (alias "Alice") 
  (balance u2000) 
  (ping (ok 250)) 
  (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) 
  (tags (u"contributor" u"og \u{e2ad90}" u"clarity"))
)
```
</details>

#### Decode serialized Clarity value to repr string
```ts
import * as assert from 'node:assert';
import { decodeClarityValueToRepr } from 'stacks-encoding-native-js';

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

const reprStr = decodeClarityValueToRepr(hex);

assert.strictEqual(
  reprStr, 
  `(tuple (active true) (address SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (u"contributor" u"og \\u{e2ad90}" u"clarity")))`
);
```

#### Decode serialized Clarity value to object
```ts
import * as assert from 'node:assert';
import { decodeClarityValue } from 'stacks-encoding-native-js';

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

// Decode into JSON object
const decoded = decodeClarityValue(hex);
```

Decoded Clarity value output result:

<details>
<summary>Expand</summary>

```ts
// Result object
assert.deepStrictEqual(decoded, {
  repr: `(tuple (active true) (address SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (u"contributor" u"og \\u{e2ad90}" u"clarity")))`,
  hex: '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479',
  type_id: 12,
  data: {
    active: { repr: 'true', hex: '0x03', type_id: 3, value: true },
    address: {
      repr: "SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW",
      hex: '0x0516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b',
      type_id: 5,
      address_version: 22,
      address_hash_bytes: '0x142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b',
      address: 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW'
    },
    alias: {
      repr: '"Alice"',
      hex: '0x0d00000005416c696365',
      type_id: 13,
      data: 'Alice'
    },
    balance: {
      repr: 'u2000',
      hex: '0x01000000000000000000000000000007d0',
      type_id: 1,
      value: '2000'
    },
    ping: {
      repr: '(ok 250)',
      hex: '0x0700000000000000000000000000000000fa',
      type_id: 7,
      value: {
        repr: '250',
        hex: '0x00000000000000000000000000000000fa',
        type_id: 0,
        value: '250'
      }
    },
    public_key: {
      repr: '(some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)',
      hex: '0x0a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
      type_id: 10,
      value: {
        repr: '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
        hex: '0x020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58',
        type_id: 2,
        buffer: '0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58'
      }
    },
    tags: {
      repr: '(u"contributor" u"og \\u{e2ad90}" u"clarity")',
      hex: '0x0b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479',
      type_id: 11,
      list: [
        {
          repr: 'u"contributor"',
          hex: '0x0e0000000b636f6e7472696275746f72',
          type_id: 14,
          data: 'contributor'
        },
        {
          repr: 'u"og \\u{e2ad90}"',
          hex: '0x0e000000066f6720e2ad90',
          type_id: 14,
          data: 'og ⭐'
        },
        {
          repr: 'u"clarity"',
          hex: '0x0e00000007636c6172697479',
          type_id: 14,
          data: 'clarity'
        }
      ]
    }
  }
});
```
</details>

#### Decode serialized Clarity value to typed object

```ts
import * as assert from 'node:assert';
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

// Serialized hex string of the example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

// Provide a typed view of the decoded value
const decoded = decodeClarityValue<ClarityValueTuple<{
  active: ClarityValueBoolTrue;
  address: ClarityValuePrincipalStandard;
  alias: ClarityValueStringAscii;
  balance: ClarityValueUInt;
  ping: ClarityValueResponseOk<ClarityValueInt>;
  public_key: ClarityValueOptionalSome<ClarityValueBuffer>;
  tags: ClarityValueList<ClarityValueStringUtf8>;
}>>(hex);

assert.deepStrictEqual(decoded.data.tags.list.map(v => v.data), ['contributor', 'og ⭐', 'clarity']);
```

The annotated types are _not_ automatically checked at runtime, so type checks are needed for error handling:

<details>
<summary>Expand</summary>

```ts
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
```
</details>

### Decoding principals

#### Principal from serialized Clarity value

```ts
import * as assert from 'node:assert';
import { decodeClarityValueToPrincipal } from 'stacks-encoding-native-js';

// Serialized hex string of an example Clarity value (0x-prefix optional, Buffer / Uint8Array also accepted)
const standardPrincipal = decodeClarityValueToPrincipal('0x0516a13dce8114be0f707f94470a2e5e86eb402f2923');
assert.strictEqual(standardPrincipal, 'SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM');

const contractPrincipal = decodeClarityValueToPrincipal('0x0616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c616273');
assert.strictEqual(contractPrincipal, 'SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs');
```

#### Stacks address from parts

```ts
import * as assert from 'node:assert';
import { stacksAddressFromParts } from 'stacks-encoding-native-js';

const stacksAddressData = {
  version: 26,
  // Serialized hex string (0x-prefix optional, Buffer / Uint8Array also accepted)
  hash160: '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6'
};

const stacksAddress = stacksAddressFromParts(stacksAddressData.version, stacksAddressData.hash160);
assert.strictEqual(stacksAddress, 'ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
```

#### Stacks address to parts

```ts
import * as assert from 'node:assert';
import { decodeStacksAddress } from 'stacks-encoding-native-js';

const [version, hash160] = decodeStacksAddress('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
assert.strictEqual(version, 26);
assert.strictEqual(hash160, '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6');
```

### Decoding transactions

```ts
import { 
  decodeTransaction,
  TransactionVersion,
  TxPayloadTypeID,
  PostConditionAuthFlag,
  TxPayloadContractCall,
} from 'stacks-encoding-native-js';

// Serialized hex string (0x-prefix optional, Buffer / Uint8Array also accepted)
const rawTx = '0000000001040089f5fd1f719e4449c980de38e3504be6770a2698000000000000014500000000000001f400008b510c9e20dc22040953d9d7eabf2038008fa4d89a5a6cb78bb9d513e75cd0df3924af9ce3b5f185705bc2f6ba3071710ec6a8803ed6da4addc40a05a01ee0f503020000000102021689f5fd1f719e4449c980de38e3504be6770a269816a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627314626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba1002162bcf9762d5b90bc36dc1b4759b1727690f92ddd30e6d61726b6574706c6163652d76340a6c6973742d6173736574000000040616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c61627301000000000000000000000000000008ba010000000000000000000000000c84588001000000000000000000000000000000c8';

const decoded = decodeTransaction(rawTx);

// Transaction ID
console.log(decoded.tx_id); // "0x49bcdba540d5c486f6f7a71de639a128a65f0378d7571e20c75a61661dd5b469"

// Transaction metadata
console.log(decoded.version);   // TransactionVersion.Mainnet (0)
console.log(decoded.chain_id);  // 1

// Auth information
console.log(decoded.auth.type_id);  // PostConditionAuthFlag.Standard (4)
console.log(decoded.auth.origin_condition.signer.address);  // "SP24ZBZ8ZE6F48JE9G3F3HRTG9FK7E2H6K2QZ3Q1K"
console.log(decoded.auth.origin_condition.nonce);  // "325"
console.log(decoded.auth.origin_condition.tx_fee);  // "500"

// Payload (contract call in this example)
const payload = decoded.payload as TxPayloadContractCall;
console.log(payload.type_id);        // TxPayloadTypeID.ContractCall (2)
console.log(payload.address);        // "SPNWZ5V2TPWGQGVDR6T7B6RQ4XMGZ4PXTEE0VQ0S"
console.log(payload.contract_name);  // "marketplace-v4"
console.log(payload.function_name);  // "list-asset"
console.log(payload.function_args);  // Array of decoded Clarity values

// Post conditions
console.log(decoded.post_conditions);  // Array of post condition objects
```

### Decoding Nakamoto blocks

Nakamoto blocks can be decoded using dedicated functions for either the full block or just the header:

#### Decode Nakamoto block header

```ts
import { decodeNakamotoBlockHeader } from 'stacks-encoding-native-js';

// Serialized hex string of a Nakamoto block header (0x-prefix optional, Buffer also accepted)
const headerHex = '0x...';
const header = decodeNakamotoBlockHeader(headerHex);

console.log(header.version);           // Block header version
console.log(header.chain_length);      // Total blocks in chain history (string)
console.log(header.burn_spent);        // BTC spent in sortition (string)
console.log(header.consensus_hash);    // Consensus hash (hex string)
console.log(header.parent_block_id);   // Parent block ID (hex string)
console.log(header.tx_merkle_root);    // Transaction merkle root (hex string)
console.log(header.state_index_root);  // State index root (hex string)
console.log(header.timestamp);         // Unix timestamp (string)
console.log(header.miner_signature);   // Miner signature (hex string)
console.log(header.signer_signature);  // Array of signer signatures (hex strings)
console.log(header.pox_treatment);     // PoX treatment bitvec (hex string)
```

#### Decode full Nakamoto block

```ts
import { decodeNakamotoBlock } from 'stacks-encoding-native-js';

// Serialized hex string of a Nakamoto block (0x-prefix optional, Buffer also accepted)
const blockHex = '0x...';
const block = decodeNakamotoBlock(blockHex);

console.log(block.block_id);           // Block ID / hash (hex string)
console.log(block.header);             // Decoded header (same structure as above)
console.log(block.txs);                // Array of decoded transactions
```

## Project Layout

The directory structure of this project is:

<pre>
/
├── Cargo.toml       # The Cargo <a href="https://doc.rust-lang.org/cargo/reference/manifest.html">manifest file</a>
├── package.json     # The npm <a href="https://docs.npmjs.com/cli/v7/configuring-npm/package-json">manifest file</a>
├── native/          # The <a href="https://nodejs.org/api/addons.html">Node addon</a> modules built by this project
├── src/
│   ├── lib.rs       # Main Neon bindings and exported functions
│   ├── neon_encoder.rs  # Converts Rust types to JavaScript objects
│   ├── hex.rs       # Hex encoding/decoding utilities
│   ├── neon_util.rs # Neon helper utilities
│   └── memo/        # Memo string normalization
├── index.ts         # TypeScript definitions for the JS interface
├── loader.js        # Platform-specific addon loader
├── loader.d.ts      # Type definitions for exported functions
├── build.js         # Script to build the native addon
├── tests/*.ts       # Unit tests for the Neon interop layer
├── examples/        # Example usage scripts
└── perf-tests/      # Performance benchmark scripts
</pre>

This library uses the [`stacks-codec`](https://github.com/stx-labs/clarinet/tree/main/components/stacks-codec) crate for deserializing Stacks wire formats as defined in [SIP-005](https://github.com/stacksgov/sips/blob/main/sips/sip-005/sip-005-blocks-and-transactions.md).

## NPM Library Bundling

The Node addon modules for all supported platforms are compiled by [CI](.github/workflows/ci.yml) and bundled inside the npm package. The native binary files are small enough that the bundled npm package is an acceptable ~20 MB in size.