import { memoToString } from '../';

test('memo 1', () => {
  const decoded = memoToString('0385180b0409122e622f706f6f6c696e2e636f6d2ffabe6d6da594842fb1be531df010e9489339c2ce500be4a758d42f3f2d48eab40bd771ba01000000000000008ecc586dc48eecad394f780bc5e680b11200c3f6080000000000');
  expect(decoded).toBe(".b/poolin.com/ mm / S H 9 P X /?-H q XmĎ 9Ox 怱");
});

test('memo 2', () => {
  const decoded = memoToString('037c180b2cfabe6d6d5e0eb001a2eaea9c5e39b7f54edd5c23eb6e684dab1995191f664658064ba7dc10000000f09f909f092f4632506f6f6c2f6500000000000000000000000000000000000000000000000000000000000000000000000500f3fa0200');
  expect(decoded).toBe("| , mm^ ^9 N \\# nhM fFX K 🐟 /F2Pool/e");
});

test('memo 3', () => {
  const decoded = memoToString('0381180b182f5669614254432f4d696e65642062792072796a70707a2f2cfabe6d6d7468749b963232960edad14885a60a0a0231d922e616078b220f92a8d0f1336e100000000000000010aa68a20fa55e1a4ef2a46e77ab2e020000000000');
  expect(decoded).toBe("/ViaBTC/Mined by ryjppz/, mmtht 22 H 1 \" \" 3n h ^ N nw .");
});
