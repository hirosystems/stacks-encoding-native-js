import * as assert from 'node:assert';
import { decodeClarityValueToPrincipal } from 'stacks-encoding-native-js';

const standardPrincipal = decodeClarityValueToPrincipal('0x0516a13dce8114be0f707f94470a2e5e86eb402f2923');
assert.strictEqual(standardPrincipal, 'SP2GKVKM12JZ0YW3ZJH3GMBJYGVNM0BS94ERA45AM');

const contractPrincipal = decodeClarityValueToPrincipal('0x0616a6a7a70f41adbe8eae708ed7ec2cbf41a272182014626974636f696e2d6d6f6e6b6579732d6c616273');
assert.strictEqual(contractPrincipal, 'SP2KAF9RF86PVX3NEE27DFV1CQX0T4WGR41X3S45C.bitcoin-monkeys-labs');

console.log('ok: decode cv to principal');
