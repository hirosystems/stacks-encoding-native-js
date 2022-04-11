import { decodeClarityValueToPrincipal, stacksAddressFromParts } from '../index.js';

test('stacks address from bytes and hash160 hex', () => {
  let address = stacksAddressFromParts(26, '0xcd1f5bc9aa49e7417cee3e5dba1a92567da41af6');
  expect(address).toBe('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
});

test('stacks address from bytes and hash160 buffer', () => {
  let address = stacksAddressFromParts(26, Buffer.from('cd1f5bc9aa49e7417cee3e5dba1a92567da41af6', 'hex'));
  expect(address).toBe('ST36HYPY9N94YEGBWXRZ5VEGTJ9B7V90TYTM9HGTJ');
});

test('stacks address from clarity value', () => {
  const inputBytes = '0x0516a13dce8114be0f707f94470a2e5e86eb402f2923';
  const address = decodeClarityValueToPrincipal(inputBytes);
  expect(address).toBe('SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM');
});
