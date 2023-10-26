import * as bindings from './loader';
export * from './loader';
export const StacksNativeEncodingBindings = bindings;
export default StacksNativeEncodingBindings;

export type TxPostCondition = PostConditionStx | PostConditionFungible | PostConditionNonfungible;

export interface DecodedPostConditionsResult {
    post_condition_mode: PostConditionModeID;
    post_conditions: TxPostCondition[]
}

export interface DecodedTxResult {
    /** Hex encoded string of the serialized transaction */
    tx_id: string;
    version: TransactionVersion;
    chain_id: number;
    auth: TxAuthStandard | TxAuthSponsored;
    anchor_mode: AnchorModeID;
    post_condition_mode: PostConditionModeID;
    post_conditions: TxPostCondition[];
    /** Hex string */
    post_conditions_buffer: string;
    payload: TxPayloadTokenTransfer | TxPayloadSmartContract | TxPayloadContractCall | TxPayloadPoisonMicroblock | TxPayloadCoinbase | TxPayloadCoinbaseToAltRecipient | TxPayloadVersionedSmartContract | TxPayloadTenureChange;
}

export enum PostConditionAssetInfoID {
    STX = 0,
    FungibleAsset = 1,
    NonfungibleAsset = 2,
}

export interface PostConditionStx {
    asset_info_id: PostConditionAssetInfoID.STX;
    principal: PostConditionPrincipal;
    condition_code: PostConditionFungibleConditionCodeID;
    condition_name: PostConditionFungibleConditionCodeName;
    amount: string;
}

export interface PostConditionFungible {
    asset_info_id: PostConditionAssetInfoID.FungibleAsset;
    principal: PostConditionPrincipal;
    asset: PostConditionAssetInfo;
    condition_code: PostConditionFungibleConditionCodeID;
    condition_name: PostConditionFungibleConditionCodeName;
    amount: string;
}

export interface PostConditionNonfungible {
    asset_info_id: PostConditionAssetInfoID.NonfungibleAsset;
    principal: PostConditionPrincipal;
    asset: PostConditionAssetInfo;
    asset_value: ClarityValueAbstract;
    condition_code: PostConditionNonfungibleConditionCodeID;
    condition_name: PostConditionNonFungibleConditionName;
}

export interface PostConditionAssetInfo {
    contract_address: string;
    contract_name: string;
    asset_name: string;
}

export enum PostConditionNonfungibleConditionCodeID {
    Sent = 0x10,
    NotSent = 0x11,
}

export enum PostConditionNonFungibleConditionName {
    Sent = "sent",
    NotSent = "not_sent",
}

export enum PostConditionFungibleConditionCodeID {
    SentEq = 0x01,
    SentGt = 0x02,
    SentGe = 0x03,
    SentLt = 0x04,
    SentLe = 0x05,
}

export enum PostConditionFungibleConditionCodeName {
    SentEq = "sent_equal_to",
    SentGt = "sent_greater_than",
    SentGe = "sent_greater_than_or_equal_to",
    SentLt = "sent_less_than",
    SentLe = "sent_less_than_or_equal_to",
}

export enum PostConditionPrincipalTypeID {
    /** A STX post-condition, which pertains to the origin account's STX. */
    Origin = 0x01,
    /** A Fungible token post-condition, which pertains to one of the origin account's fungible tokens. */
    Standard = 0x02,
    /** A Non-fungible token post-condition, which pertains to one of the origin account's non-fungible tokens. */
    Contract = 0x03,
}

export type PostConditionPrincipal = PostConditionPrincipalOrigin | PostConditionPrincipalStandard | PostConditionPrincipalContract;

export interface PostConditionPrincipalOrigin {
    type_id: PostConditionPrincipalTypeID.Origin
}

export interface PostConditionPrincipalStandard {
    type_id: PostConditionPrincipalTypeID.Standard;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    address: string;
}

export interface PostConditionPrincipalContract {
    type_id: PostConditionPrincipalTypeID.Contract;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    address: string;
    contract_name: string;
}

export interface TxPayloadTokenTransfer {
    type_id: TxPayloadTypeID.TokenTransfer;
    recipient: PrincipalStandardData | PrincipalContractData;
    amount: string;
    /** Hex encoded string of the 34-bytes */
    memo_hex: string;
}

export enum PrincipalTypeID {
    Standard = 5,
    Contract = 6,
}

export interface PrincipalStandardData {
    type_id: PrincipalTypeID.Standard;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    address: string;
}

export interface PrincipalContractData {
    type_id: PrincipalTypeID.Contract;
    contract_name: string;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    address: string;
}

export interface TxPayloadSmartContract { 
    type_id: TxPayloadTypeID.SmartContract;
    contract_name: string;
    code_body: string;
}

export interface TxPayloadContractCall {
    type_id: TxPayloadTypeID.ContractCall;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    address: string;
    contract_name: string;
    function_name: string;
    function_args: ClarityValueAbstract[];
    /** Hex string */
    function_args_buffer: string;
}

export interface TxPayloadPoisonMicroblock {
    type_id: TxPayloadTypeID.PoisonMicroblock;
    microblock_header_1: TxMicroblockHeader;
    microblock_header_2: TxMicroblockHeader;
}

export interface TxPayloadCoinbase {
    type_id: TxPayloadTypeID.Coinbase;
    /** Hex string */
    payload_buffer: string;
}

export interface TxPayloadCoinbaseToAltRecipient {
    type_id: TxPayloadTypeID.CoinbaseToAltRecipient;
    /** Hex string */
    payload_buffer: string;
    recipient: PrincipalStandardData | PrincipalContractData;
}

export interface TxPayloadVersionedSmartContract {
    type_id: TxPayloadTypeID.VersionedSmartContract;
    clarity_version: ClarityVersion;
    contract_name: string;
    code_body: string;
}

export interface TxPayloadTenureChange {
  type_id: TxPayloadTypeID.TenureChange;
  /** (Hex string) Stacks Block hash */
  previous_tenure_end: string;
  /** The number of blocks produced in the previous tenure */
  previous_tenure_blocks: number;
  /** Cause of change in mining tenure. Depending on cause, tenure can be ended or extended. */
  cause: TenureChangeCause;
  /** (Hex string) The ECDSA public key hash of the current tenure */
  pubkey_hash: string;
  /** (Hex string) A Schnorr signature from at least 70% of the Stackers */
  signature: string;
  /** (Hex string) A bitmap of which Stackers signed */
  signers: string;
}

export enum TenureChangeCause {
  /** A valid winning block-commit */
  BlockFound = 0,
  /** No winning block-commits */
  NoBlockFound = 1,
  /** A "null miner" won the block-commit */
  NullMiner = 2,
}

export enum TxPayloadTypeID {
    TokenTransfer = 0,
    SmartContract = 1,
    ContractCall = 2,
    PoisonMicroblock = 3,
    Coinbase = 4,
    CoinbaseToAltRecipient = 5,
    VersionedSmartContract = 6,
    TenureChange = 7,
}

export enum PostConditionAuthFlag {
    Standard = 0x04,
    Sponsored = 0x05,
}

export interface TxAuthStandard {
    type_id: PostConditionAuthFlag.Standard;
    origin_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
}

export interface TxAuthSponsored {
    type_id: PostConditionAuthFlag.Sponsored;
    origin_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
    sponsor_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
}

export enum TxSpendingConditionSingleSigHashMode {
    /** hash160(public-key), same as bitcoin's p2pkh */
    P2PKH = 0x00,
    /** hash160(segwit-program-00(p2pkh)), same as bitcoin's p2sh-p2wpkh */
    P2WPKH = 0x02,
}

export enum TxSpendingConditionMultiSigHashMode {
    /** hash160(multisig-redeem-script), same as bitcoin's multisig p2sh */
    P2SH = 0x01,
    /** hash160(segwit-program-00(public-keys)), same as bitcoin's p2sh-p2wsh */
    P2WSH = 0x03,
}

export enum ClarityVersion {
    Clarity1 = 1,
    Clarity2 = 2,
}

export interface DecodedTxSpendingConditionSingleSig {
    hash_mode: TxSpendingConditionSingleSigHashMode;
    signer: DecodedStacksAddress;
    nonce: string;
    tx_fee: string;
    /** A 1-byte public key encoding field to indicate whether or not the public key should be compressed before hashing. */
    key_encoding: TxPublicKeyEncoding;
    signature: string;
}

export interface DecodedTxSpendingConditionMultiSig {
    hash_mode: TxSpendingConditionMultiSigHashMode;
    signer: DecodedStacksAddress;
    nonce: string;
    tx_fee: string;
    fields: (TxAuthFieldPublicKey | TxAuthFieldSignature)[];
    signatures_required: number,
}

export enum TxAuthFieldTypeID {
    /** The next 33 bytes are a compressed secp256k1 public key. If the field ID is 0x00, the key will be loaded as a compressed secp256k1 public key. */
    PublicKeyCompressed = 0x00,
    /** The next 33 bytes are a compressed secp256k1 public key. If it is 0x01, then the key will be loaded as an uncompressed secp256k1 public key. */
    PublicKeyUncompressed = 0x01,
    /** The next 65 bytes are a recoverable secp256k1 ECDSA signature. If the field ID is 0x02, then the recovered public key will be loaded as a compressed public key. */
    SignatureCompressed = 0x02,
    /** The next 65 bytes are a recoverable secp256k1 ECDSA signature. If it is 0x03, then the recovered public key will be loaded as an uncompressed public key. */
    SignatureUncompressed = 0x03,
}

export interface TxAuthFieldPublicKey {
    type_id: TxAuthFieldTypeID.PublicKeyCompressed | TxAuthFieldTypeID.PublicKeyUncompressed;
    /** Hex encoded public key bytes. */
    public_key: string;
}

export interface TxAuthFieldSignature {
    type_id: TxAuthFieldTypeID.SignatureCompressed | TxAuthFieldTypeID.SignatureUncompressed;
    /** Hex encoded signatures bytes. */
    signature: string;
}

export interface TxMicroblockHeader {
    /** Hex string */
    buffer: string;
    version: number;
    sequence: number;
    /** Hex string */
    prev_block: string;
    /** Hex string */
    tx_merkle_root: string;
    /** Hex string */
    signature: string;
}

export enum TxPublicKeyEncoding {
    Compressed = 0x00,
    Uncompressed = 0x01,
}

export interface DecodedStacksAddress {
    address_version: number;
    /** Hex-encoded string of the hash160 signer address. */
    address_hash_bytes: string;
    address: string;
}

export enum TransactionVersion {
    Mainnet = 0x00,
    Testnet = 0x80,
}

export enum AnchorModeID {
    /** The transaction MUST be included in an anchored block. */
    OnChainOnly = 1,
    /** The transaction MUST be included in a microblock. */
    OffChainOnly = 2,
    /** The leader can choose where to include the transaction. */
    Any = 3,
}

export enum PostConditionModeID {
    /** This transaction may affect other assets not listed in the post-conditions. */
    Allow = 0x01,
    /** This transaction may NOT affect other assets besides those listed in the post-conditions. */
    Deny = 0x02,
}

export interface ClarityValueCommon {
  /** Clarity repr value */
  repr: string;
  /** Hex encoded string of the serialized Clarity value */
  hex: string;
}

export interface ClarityValueAbstract extends ClarityValueCommon {
  type_id: number;
}

export enum ClarityTypeID {
    Int = 0,
    UInt = 1,
    Buffer = 2,
    BoolTrue = 3,
    BoolFalse = 4,
    PrincipalStandard = 5,
    PrincipalContract = 6,
    ResponseOk = 7,
    ResponseError = 8,
    OptionalNone = 9,
    OptionalSome = 10,
    List = 11,
    Tuple = 12,
    StringAscii = 13,
    StringUtf8 = 14,
}

export interface ClarityValueInt extends ClarityValueCommon {
    type_id: ClarityTypeID.Int;
    /** String-quoted signed integer */
    value: string;
}

export interface ClarityValueUInt extends ClarityValueCommon {
    type_id: ClarityTypeID.UInt;
    /** String-quoted unsigned integer */
    value: string;
}

export interface ClarityValueBoolTrue extends ClarityValueCommon {
    type_id: ClarityTypeID.BoolTrue;
    value: true;
}

export interface ClarityValueBoolFalse extends ClarityValueCommon {
    type_id: ClarityTypeID.BoolFalse;
    value: false;
}

export interface ClarityValueBuffer extends ClarityValueCommon {
    type_id: ClarityTypeID.Buffer;
    /** Hex string */
    buffer: string;
}

export interface ClarityValueList<T extends ClarityValue = ClarityValue> extends ClarityValueCommon {
    type_id: ClarityTypeID.List;
    list: T[];
}

export interface ClarityValueStringAscii extends ClarityValueCommon {
    type_id: ClarityTypeID.StringAscii;
    data: string;
}

export interface ClarityValueStringUtf8 extends ClarityValueCommon {
    type_id: ClarityTypeID.StringUtf8;
    data: string;
}

export interface ClarityValuePrincipalStandard extends ClarityValueCommon {
    type_id: ClarityTypeID.PrincipalStandard;
    address: string;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
}

export interface ClarityValuePrincipalContract extends ClarityValueCommon {
    type_id: ClarityTypeID.PrincipalContract;
    address: string;
    address_version: number;
    /** Hex string */
    address_hash_bytes: string;
    contract_name: string;
}

export type ClarityTupleData<T extends ClarityValue = ClarityValue> = { [key: string]: T };

export interface ClarityValueTuple<T extends ClarityTupleData = ClarityTupleData> extends ClarityValueCommon {
    type_id: ClarityTypeID.Tuple;
    data: T;
}

export interface ClarityValueOptionalSome<T extends ClarityValue = ClarityValue> extends ClarityValueCommon {
    type_id: ClarityTypeID.OptionalSome;
    value: T;
}

export interface ClarityValueOptionalNone extends ClarityValueCommon {
    type_id: ClarityTypeID.OptionalNone;
}

export interface ClarityValueResponseOk<T extends ClarityValue = ClarityValue> extends ClarityValueCommon {
    type_id: ClarityTypeID.ResponseOk;
    value: T;
}

export interface ClarityValueResponseError<T extends ClarityValue = ClarityValue> extends ClarityValueCommon {
    type_id: ClarityTypeID.ResponseError;
    value: T;
}

export type ClarityValue = 
    | ClarityValueInt
    | ClarityValueUInt
    | ClarityValueBoolTrue
    | ClarityValueBoolFalse
    | ClarityValueBuffer
    | ClarityValueList
    | ClarityValueStringAscii
    | ClarityValueStringUtf8
    | ClarityValuePrincipalStandard
    | ClarityValuePrincipalContract
    | ClarityValueTuple
    | ClarityValueOptionalSome
    | ClarityValueOptionalNone
    | ClarityValueResponseOk
    | ClarityValueResponseError;

export type ClarityValueOptional<T extends ClarityValue = ClarityValue> = ClarityValueOptionalSome<T> | ClarityValueOptionalNone;
export type ClarityValueBool = ClarityValueBoolTrue | ClarityValueBoolFalse;
export type ClarityValueResponse<TOk extends ClarityValue = ClarityValue, TError extends ClarityValue = ClarityValue> = ClarityValueResponseOk<TOk> | ClarityValueResponseError<TError>;

/**
 * Type for commonly used `(optional bool)`
 */
export type ClarityValueOptionalBool = ClarityValueOptional<ClarityValueBool>;

/**
 * Type for commonly used `(optional uint)`
 */
export type ClarityValueOptionalUInt = ClarityValueOptional<ClarityValueUInt>;
