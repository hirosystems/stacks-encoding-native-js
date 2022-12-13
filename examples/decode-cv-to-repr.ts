import * as assert from 'node:assert';
import { ALICE_TUPLE_CV_HEX } from './encode-values';
import { decodeClarityValueToRepr } from 'stacks-encoding-native-js';

const reprStr = decodeClarityValueToRepr(ALICE_TUPLE_CV_HEX);

assert.strictEqual(
  reprStr, 
  `(tuple (active true) (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) (alias "Alice") (balance u2000) (ping (ok 250)) (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) (tags (list u"contributor" u"og \\u{e2ad90}" u"clarity")))`
);

console.log('ok: decode cv hex string to repr string');
