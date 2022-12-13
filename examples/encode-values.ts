// Currently `stacks-encoding-native-js` does not support creating/serializing Clarity values, so use the `stacks.js` library instead
import * as stacksJs from '@stacks/transactions';
import * as assert from 'node:assert';

/*
(tuple 
  (active true) 
  (address 'SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW) 
  (alias "Alice") 
  (balance u2000) 
  (ping (ok 250)) 
  (public_key (some 0x02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58)) 
  (tags (list u"contributor" u"og \u{e2ad90}" u"clarity"))
)
*/
const cvTuple = stacksJs.tupleCV({
  active: stacksJs.trueCV(),
  address: stacksJs.standardPrincipalCV('SPA2MZWV9N67TBYVWTE0PSSKMJ2F6YXW7CBE6YPW'),
  alias: stacksJs.stringAsciiCV('Alice'),
  balance: stacksJs.uintCV(2000n),
  ping: stacksJs.responseOkCV(stacksJs.intCV(250)),
  public_key: stacksJs.someCV(
    stacksJs.bufferCV(Buffer.from('02d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e58', 'hex'))
  ),
  tags: stacksJs.listCV([
    stacksJs.stringUtf8CV('contributor'),
    stacksJs.stringUtf8CV('og ⭐'),
    stacksJs.stringUtf8CV('clarity'),
  ]),
});

const cvHex = stacksJs.cvToHex(cvTuple);

const expected = '0x0c00000007066163746976650307616464726573730516142a7f9b4d4c7d2fdbe69c0b6733a484f37bbc3b05616c6961730d00000005416c6963650762616c616e636501000000000000000000000000000007d00470696e670700000000000000000000000000000000fa0a7075626c69635f6b65790a020000002102d4dada83bff981f0cb7ebafcfc6fc7cb5e078b9ee2302a93aae19fb90f872e5804746167730b000000030e0000000b636f6e7472696275746f720e000000066f6720e2ad900e00000007636c6172697479';
assert.strictEqual(cvHex, expected);

console.log('ok: encode clarity value to hex string')

export { cvHex as ALICE_TUPLE_CV_HEX };
