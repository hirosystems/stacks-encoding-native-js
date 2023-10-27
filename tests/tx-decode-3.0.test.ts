import {
  AnchorModeID,
  decodeTransaction,
  PostConditionAuthFlag,
  PostConditionModeID,
  TenureChangeCause,
  TransactionVersion,
  TxPayloadTypeID,
  TxPublicKeyEncoding
} from '../index.js';

test('stacks3.0 - decode tx - tenure change', () => {
  const tenureChangeTx = '00000000010400982f3ec112a5f5928a5c96a914bd733793b896a5000000000000053000000000000002290000c85889dad0d5b08a997a93a28a7c93eb22c324e5f8992dc93e37865ef4f3e0d65383beefeffc4871a2facbc4b590ddf887c80de6638ed4e2ec0e633d1e130f2303010000000007c15258750a06e6ddae0320f978e5d86973933f1803d5bbd35213b54e75d2310f006402e97fca6444b0dc98f6f9a1013c5554975c7ce1c7954135949e6af4b9c56ed9cbf1a61dc83d054fa9cc699c9918af44a9b9ab2e5ccaf9611b86e963f139c49a6c546a8e94d67bb21cda0aa3b05364960e91d4281e7000000015124b91930cea290260f27dd56093f0dbefc4e6c5fa';
  const decoded = decodeTransaction(tenureChangeTx);
  expect(decoded).toEqual({
    "tx_id": "0xb0686254421a5e6c6554b128469e32c9f7684cc7191a92eab47ef27c43d2c242",
    "version": TransactionVersion.Mainnet,
    "chain_id": 1,
    "auth": {
      "type_id": PostConditionAuthFlag.Standard,
      "origin_condition": {
        "hash_mode": 0,
        "signer": {
          "address_version": 22,
          "address_hash_bytes": "0x982f3ec112a5f5928a5c96a914bd733793b896a5",
          "address": "SP2C2YFP12AJZB4MABJBAJ55XECVS7E4PMMZ89YZR"
        },
        "nonce": "1328",
        "tx_fee": "553",
        "key_encoding": TxPublicKeyEncoding.Compressed,
        "signature": "0x00c85889dad0d5b08a997a93a28a7c93eb22c324e5f8992dc93e37865ef4f3e0d65383beefeffc4871a2facbc4b590ddf887c80de6638ed4e2ec0e633d1e130f23"
      }
    },
    "anchor_mode": AnchorModeID.Any,
    "post_condition_mode": PostConditionModeID.Allow,
    "post_conditions": [],
    "post_conditions_buffer": "0x0100000000",
    "payload": {
      "type_id": TxPayloadTypeID.TenureChange,
      "previous_tenure_end": "0xc15258750a06e6ddae0320f978e5d86973933f1803d5bbd35213b54e75d2310f",
      "previous_tenure_blocks": 100,
      "cause": TenureChangeCause.NullMiner,
      "pubkey_hash": "0xe97fca6444b0dc98f6f9a1013c5554975c7ce1c7",
      "signature": "0x954135949e6af4b9c56ed9cbf1a61dc83d054fa9cc699c9918af44a9b9ab2e5ccaf9611b86e963f139c49a6c546a8e94d67bb21cda0aa3b05364960e91d4281e70",
      "signers": "0x124b91930cea290260f27dd56093f0dbefc4e6c5fa"
    }
  });
});
