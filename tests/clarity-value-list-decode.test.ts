import { decodeClarityValueList } from '../index.js';

test('decode value array 1', () => {
  const decoded = decodeClarityValueList('0x0000');
  expect(decoded).toEqual([]);
});

test('decode value array 2', () => {
  const decoded = decodeClarityValueList('0x');
  expect(decoded).toEqual([]);
});

test('decode value array 3', () => {
  const decoded = decodeClarityValueList('0x000000060616e685b016b3b6cd9ebf35f38e5ae29392e2acd51d176167653030302d676f7665726e616e63652d746f6b656e0616e685b016b3b6cd9ebf35f38e5ae29392e2acd51d0a746f6b656e2d777374780100000000000000000000000002faf0800100000000000000000000000002faf08001000000000000000000000012f2fd95060a010000000000000000000000116c7a7446');
  expect(decoded).toEqual([
    { "hex": "0x0616e685b016b3b6cd9ebf35f38e5ae29392e2acd51d176167653030302d676f7665726e616e63652d746f6b656e", "repr": "'SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9.age000-governance-token", "type_id": 6 },
    { "hex": "0x0616e685b016b3b6cd9ebf35f38e5ae29392e2acd51d0a746f6b656e2d77737478", "repr": "'SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9.token-wstx", "type_id": 6 },
    { "hex": "0x0100000000000000000000000002faf080", "repr": "u50000000", "type_id": 1 },
    { "hex": "0x0100000000000000000000000002faf080", "repr": "u50000000", "type_id": 1 },
    { "hex": "0x01000000000000000000000012f2fd9506", "repr": "u81386116358", "type_id": 1 },
    { "hex": "0x0a010000000000000000000000116c7a7446", "repr": "(some u74834408518)", "type_id": 10 }
  ]);
});

