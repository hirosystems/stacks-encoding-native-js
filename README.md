# stacks-encoding-native-js

[![npm version](https://badge.fury.io/js/stacks-encoding-native-js.svg)](https://www.npmjs.com/package/stacks-encoding-native-js)
[![test](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/test.yml/badge.svg)](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/test.yml)
[![build](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/build.yml/badge.svg)](https://github.com/hirosystems/stacks-encoding-native-js/actions/workflows/build.yml)

`stacks-encoding-native-js` is a Node.js [native addon](https://nodejs.org/api/addons.html) library written in Rust, which provides functions for decoding binary/wire formats used in the Stacks blockchain. Features include Clarity values, transactions, post-conditions, Stacks and Bitcoin addresses, and more. 

Various ASM/SIMD optimizations are used in areas which are prone to causing CPU bottlenecks when used in hot paths, e.g. decoding raw Clarity values on the fly.

_This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon)._

### Platform Support

* Linux (glibc) x86_64
* Linux (glibc) aarch64
* Linux (musl) x86_64
* Linux (musl) aarch64
* MacOS x86_64
* MacOS aarch64 (Apple Silicon M1)
* Windows x86_64

### Runtime Support
* Node.js v16+
* CommonJS (the examples using ESM syntax are compiled to CJS via Typescript)

## Installation and Usage

```shell
npm install stacks-encoding-native-js
```

### Decode raw Clarity value
_Example Clarity value:_
<details>
<summary>Expand</summary>

```clar
(tuple 
  (active true) 
  (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) 
  (alias "Alice") 
  (balance u2000) 
  (ping (ok 250)) 
  (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) 
  (tags (list u"contributor" u"og \u{e2ad90}" u"clarity"))
)
```
</details>

#### Decode serialized Clarity value to object
```ts
import { decodeClarityValue } from 'stacks-encoding-native-js';
import * as assert from 'node:assert';

// A serialized hex string of the example Clarity value
const hex = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';

// Decode into JSON object
const decoded = decodeClarityValue(hex);
```

#### Decoded output result
<details>
<summary>Expand</summary>

```ts
// Result object
assert.deepStrictEqual(decoded, {
  repr: `(tuple (active true) (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (list u"contributor" u"og \\u{e2ad90}" u"clarity")))`,
  hex: '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479',
  type_id: 12,
  data: {
    active: { repr: 'true', hex: '0x03', type_id: 3, value: true },
    address: {
      repr: "'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW",
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
      repr: '(list u"contributor" u"og \\u{e2ad90}" u"clarity")',
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
import * as assert from 'node:assert';

// Provide a typed view of the decoded value
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
```

## Project Layout

The directory structure of this project is:

<pre>
/
├── Cargo.toml    # The Cargo <a href="https://doc.rust-lang.org/cargo/reference/manifest.html">manifest file</a>
├── package.json  # The npm <a href="https://docs.npmjs.com/cli/v7/configuring-npm/package-json">manifest file</a>
├── native/       # The <a href="https://nodejs.org/api/addons.html">Node addon</a> modules built by this project, these are <a href="https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries">dynamically-linked shared objects</a>
├── src/**/*.rs   # Directory containing the Rust source code for the project
|── index.ts      # Typescript definitions for the js interface exposed by the Node addon
|── loader.js     # Script to determine which addon file to load based on the executing target platform
|── loader.d.ts   # Type defintions for the functions exported by the Node addon
|── builder.js    # Script to build the native Node addon for the executing host platform
├── tests/*.ts    # Js/ts unit tests, primarily testing the Neon (rust<->nodejs) interop layer
└── perf-tests/   # Scripts to run performance benchmarks used by commands in package.json
</pre>

The Rust source code inside the `src/**/deserialize.rs` files are responsible for deserializing the Stacks blockchain wire/binary formats defined in [SIP-005](https://github.com/stacksgov/sips/blob/main/sips/sip-005/sip-005-blocks-and-transactions.md). 

## NPM Library Bundling

The Node addon modules for all supported platforms are compiled by [CI](.github/workflows/build.yml) and bundled inside the npm package. The native binary files are small enough that the bundled npm package is an acceptable ~20 MB in size. 