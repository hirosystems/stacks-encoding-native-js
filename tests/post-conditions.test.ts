import { decodePostConditions } from '../';

test('decode post conditions', () => {
  const postConditions = decodePostConditions('020000000200021642779fa5c48120aa60c18eb05a164bad77bf2cdd0100000000002ab9800103167e9152cdbbb9fef066df4e1b88b19bcb313acc901b6c69717569646974792d746f6b656e2d76356b6d6e77733563676c1608633eac058f2e6ab41613a0a537c7ea1a79cdd20f6d69616d69636f696e2d746f6b656e096d69616d69636f696e030000000000000423');
  expect(postConditions).toEqual(
    {
      "post_condition_mode": 2,
      "post_conditions": [
        {
          "amount": "2800000",
          "asset_info_id": 0,
          "condition_code": 1,
          "condition_name": "sent_equal_to",
          "principal": {
            "address": "SP117F7X5RJ0J1AK0R67B0PGP9EPQFFSCVQNASZBC",
            "address_hash_bytes": "0x42779fa5c48120aa60c18eb05a164bad77bf2cdd",
            "address_version": 22,
            "type_id": 2
          }
        },
        {
          "amount": "1059",
          "asset": {
            "asset_name": "miamicoin",
            "contract_address": "SP466FNC0P7JWTNM2R9T199QRZN1MYEDTAR0KP27",
            "contract_name": "miamicoin-token"
          },
          "asset_info_id": 1,
          "condition_code": 3,
          "condition_name": "sent_greater_than_or_equal_to",
          "principal": {
            "address": "SP1Z92MPDQEWZXW36VX71Q25HKF5K2EPCJ304F275",
            "address_hash_bytes": "0x7e9152cdbbb9fef066df4e1b88b19bcb313acc90",
            "address_version": 22,
            "contract_name": "liquidity-token-v5kmnws5cgl",
            "type_id": 3
          }
        }
      ]
    }
  );
});

