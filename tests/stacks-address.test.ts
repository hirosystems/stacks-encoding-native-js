import { stacksAddressFromParts } from '../index.js';


test('stacks address from bytes and hash160 hex', () => {
  let address = stacksAddressFromParts(26, '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6');
  expect(address).toBe('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
});

test('stacks address from bytes and hash160 buffer', () => {
  let address = stacksAddressFromParts(26, Buffer.from('cd1f5bc9aa49e7417cee3e5dba1a92567da41af6', 'hex'));
  expect(address).toBe('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
});
