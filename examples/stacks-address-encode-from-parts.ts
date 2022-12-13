import * as assert from 'node:assert';
import { stacksAddressFromParts } from 'stacks-encoding-native-js';

const stacksAddressData = {
  version: 26,
  hash160: '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6'
};

const principal = stacksAddressFromParts(stacksAddressData.version, stacksAddressData.hash160);

assert.strictEqual(principal, 'ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');

console.log('ok: stacks address from address parts');
