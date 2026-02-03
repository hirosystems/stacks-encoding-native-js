import * as assert from 'node:assert';
import { ALICE_TUPLE_CV_HEX } from './encode-values';
import { decodeClarityValue } from 'stacks-encoding-native-js';

const decoded = decodeClarityValue(ALICE_TUPLE_CV_HEX);

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

console.log('ok: decode cv hex string to object');
