import * as assert from 'node:assert';
import { decodeStacksAddress } from 'stacks-encoding-native-js';

const stacksAddress = 'ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ';

const [version, hash160] = decodeStacksAddress(stacksAddress);

assert.strictEqual(version, 26);
assert.strictEqual(hash160, '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6');

console.log('ok: stacks address to parts');
