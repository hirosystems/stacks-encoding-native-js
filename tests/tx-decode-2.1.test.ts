import { ClarityVersion, decodeTransaction, TransactionVersion, TxPayloadTypeID } from '../index.js';
import * as path from 'path';
import * as fs from 'fs';

test('stacks2.1 - decode tx - versioned smart contract 1', () => {
  const filePath = path.join(__dirname, 'data', 'versioned-smart-contract-tx.hex');
  const coinbaseVersion2_1 = fs.readFileSync(filePath, { encoding: 'ascii' });
  const decoded = decodeTransaction(coinbaseVersion2_1);

  expect(decoded).toEqual(
    {
      "anchor_mode": 3,
      "auth": {
        "origin_condition": {
          "hash_mode": 0,
          "key_encoding": 1,
          "nonce": "0",
          "signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
          "signer": {
            "address": "ST000000000000000000002AMW42H",
            "address_hash_bytes": "0x0000000000000000000000000000000000000000",
            "address_version": 26
          },
          "tx_fee": "0"
        },
        "type_id": 4
      },
      "chain_id": 0,
      "payload": {
        "clarity_version": ClarityVersion.Clarity2,
        "code_body": expect.stringContaining(";; PoX testnet constants\n;; Min/max number of"),
        "contract_name": "pox-2",
        "type_id": TxPayloadTypeID.VersionedSmartContract,
      },
      "post_condition_mode": 2,
      "post_conditions": [],
      "post_conditions_buffer": "0x0200000000",
      "tx_id": "0x33c573f5ed06f1feecaa4a9df0225e109416dbba9792abb0cd94869bbad4a88a",
      "version": TransactionVersion.Testnet,
    }
  );
});
