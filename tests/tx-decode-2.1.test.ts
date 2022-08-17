import { ClarityVersion, decodeTransaction, PrincipalTypeID, TransactionVersion, TxPayloadTypeID } from '../index.js';
import * as path from 'path';
import * as fs from 'fs';

test('stacks2.1 - decode tx - coinbase pay to alt - standard principal', () => {
  const coinbasePayToAddressTx = '0x80800000000400fd3cd910d78fe7c4cd697d5228e51a912ff2ba740000000000000004000000000000000001008d36064b250dba5d3221ac235a9320adb072cfc23cd63511e6d814f97f0302e66c2ece80d7512df1b3e90ca6dce18179cb67b447973c739825ce6c6756bc247d010200000000050000000000000000000000000000000000000000000000000000000000000000051aba27f99e007c7f605a8305e318c1abde3cd220ac';
  const decoded = decodeTransaction(coinbasePayToAddressTx);
  expect(decoded).toEqual(
    {
      "anchor_mode": 1,
      "auth": {
        "origin_condition": {
          "hash_mode": 0,
          "key_encoding": 1,
          "nonce": "4",
          "signature": "0x008d36064b250dba5d3221ac235a9320adb072cfc23cd63511e6d814f97f0302e66c2ece80d7512df1b3e90ca6dce18179cb67b447973c739825ce6c6756bc247d",
          "signer": {
            "address": "ST3YKSP8GTY7YFH6DD5YN4A753A8JZWNTEJFG78GN",
            "address_hash_bytes": "0xfd3cd910d78fe7c4cd697d5228e51a912ff2ba74",
            "address_version": 26
          },
          "tx_fee": "0"
        },
        "type_id": 4
      },
      "chain_id": 0x80000000,
      "payload": {
        "payload_buffer": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "recipient": {
          "address": "ST2X2FYCY01Y7YR2TGC2Y6661NFF3SMH0NGXPWTV5",
          "address_hash_bytes": "0xba27f99e007c7f605a8305e318c1abde3cd220ac",
          "address_version": 26,
          "type_id": PrincipalTypeID.Standard
        },
        "type_id": TxPayloadTypeID.CoinbaseToAltRecipient,
      },
      "post_condition_mode": 2,
      "post_conditions": [],
      "post_conditions_buffer": "0x0200000000",
      "tx_id": "0x449f5ea5c541bbbbbf7a1bff2434c449dca2ae3cdc52ba8d24b0bd0d3632d9bc",
      "version": TransactionVersion.Testnet,
    }
  );
});

test('stacks2.1 - decode tx - coinbase pay to alt - contract principal', () => {
  const coinbasePayToContractTx = '0x8080000000040055a0a92720d20398211cd4c7663d65d018efcc1f00000000000000030000000000000000010118da31f542913e8c56961b87ee4794924e655a28a2034e37ef4823eeddf074747285bd6efdfbd84eecdf62cffa7c1864e683c688f4c105f4db7429066735b4e2010200000000050000000000000000000000000000000000000000000000000000000000000000061aba27f99e007c7f605a8305e318c1abde3cd220ac0b68656c6c6f5f776f726c64';
  const decoded = decodeTransaction(coinbasePayToContractTx);
  expect(decoded).toEqual(
    {
      "anchor_mode": 1,
      "auth": {
        "origin_condition": {
          "hash_mode": 0,
          "key_encoding": 1,
          "nonce": "3",
          "signature": "0x0118da31f542913e8c56961b87ee4794924e655a28a2034e37ef4823eeddf074747285bd6efdfbd84eecdf62cffa7c1864e683c688f4c105f4db7429066735b4e2",
          "signer": {
            "address": "ST1AT1A97439076113KACESHXCQ81HVYC3XWGT2F5",
            "address_hash_bytes": "0x55a0a92720d20398211cd4c7663d65d018efcc1f",
            "address_version": 26
          },
          "tx_fee": "0"
        },
        "type_id": 4
      },
      "chain_id": 0x80000000,
      "payload": {
        "payload_buffer": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "recipient": {
          "address": "ST2X2FYCY01Y7YR2TGC2Y6661NFF3SMH0NGXPWTV5",
          "address_hash_bytes": "0xba27f99e007c7f605a8305e318c1abde3cd220ac",
          "address_version": 26,
          "contract_name": "hello_world",
          "type_id": PrincipalTypeID.Contract
        },
        "type_id": TxPayloadTypeID.CoinbaseToAltRecipient,
      },
      "post_condition_mode": 2,
      "post_conditions": [],
      "post_conditions_buffer": "0x0200000000",
      "tx_id": "0xbd1a9e1d60ca29fc630633170f396f5b6b85c9620bd16d63384ebc5a01a1829b",
      "version": TransactionVersion.Testnet,
    }
  );
});

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
