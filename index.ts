import * as bindings from './index.node';
export * from './index.node';
export const StacksNativeEncodingBindings = bindings;
export default StacksNativeEncodingBindings;

export interface DecodedClarityValueListResult {
    /** Byte span for the given serialized Clarity value list (u32be length-prefixed) */
    buffer: Buffer;
    /** Deserialized Clarity values */
    array: ParsedClarityValue[];
}

export interface DecodedTxResult {
    /** Hex encoded string of the serialized transaction */
    tx_id: string;
    version: TransactionVersion;
    chain_id: number;
    auth: TxAuthStandard | DecodedTxAuthSponsored;
    anchor_mode: AnchorModeID;
    post_condition_mode: PostConditionModeID;
    post_conditions: any[]; // TODO
    post_conditions_buffer: Buffer;
    payload: TxPayloadTokenTransfer | TxPayloadSmartContract | TxPayloadContractCall | TxPayloadPoisonMicroblock | TxPayloadCoinbase;
}

export interface TxPayloadTokenTransfer {
    type_id: TxPayloadTypeID.TokenTransfer;
    recipient: PrincipalStandardData | PrincipalContractData;
    amount: string;
    /** Hex encoded string of the 34-bytes */
    memo: string;
}

export enum PrincipalTypeID {
    PrincipalStandard = 5,
    PrincipalContract = 6,
}

export interface PrincipalStandardData {
    type_id: PrincipalTypeID.PrincipalStandard;
    address_version: number;
    address_hash_bytes: Buffer;
    address: string;
}

export interface PrincipalContractData {
    type_id: PrincipalTypeID.PrincipalContract;
    contract_name: string;
    address_version: number;
    address_hash_bytes: Buffer;
    address: string;
}

export interface TxPayloadSmartContract { 
    type_id: TxPayloadTypeID.SmartContract;
    contract_name: string;
    code_body: string;
}

export interface TxPayloadContractCall {
    type_id: TxPayloadTypeID.ContractCall;
    contract_name: string;
    function_name: string;
    function_args: ParsedClarityValue[];
    function_args_buffer: Buffer;
}

export interface TxPayloadPoisonMicroblock {
    type_id: TxPayloadTypeID.PoisonMicroblock;
    microblock_header_1: TxMicroblockHeader;
    microblock_header_2: TxMicroblockHeader;
}

export interface TxPayloadCoinbase {
    type_id: TxPayloadTypeID.Coinbase;
    payload_buffer: Buffer;
}

export enum TxPayloadTypeID {
    TokenTransfer = 0,
    SmartContract = 1,
    ContractCall = 2,
    PoisonMicroblock = 3,
    Coinbase = 4,
}

export enum TxPostConditionAuthFlag {
    AuthStandard = 0x04,
    AuthSponsored = 0x05,
}

export interface TxAuthStandard {
    type_id: TxPostConditionAuthFlag;
    origin_condition: DecodedTxSpendingConditionSingleSig | DecodedTxSpendingConditionMultiSig;
}

export interface DecodedTxAuthSponsored {
    type_id: TxPostConditionAuthFlag;
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

export interface DecodedTxSpendingConditionSingleSig {
    hash_mode: TxSpendingConditionSingleSigHashMode;
    /** Hex-encoded string of the hash160 signer address. */
    signer: string;
    signer_stacks_address: DecodedStacksAddress;
    nonce: string;
    tx_fee: string;
    /** A 1-byte public key encoding field to indicate whether or not the public key should be compressed before hashing. */
    key_encoding: TxPublicKeyEncoding;
    signature: string;
}

export interface DecodedTxSpendingConditionMultiSig {
    hash_mode: TxSpendingConditionMultiSigHashMode;
    /** Hex-encoded string of the hash160 signer address. */
    signer: string;
    signer_stacks_address: DecodedStacksAddress;
    nonce: string;
    tx_fee: string;
    fields: any[]; // TODO
    signatures_required: number,
}

export interface TxMicroblockHeader {
    buffer: Buffer;
    version: number;
    sequence: number;
    prev_block: Buffer;
    tx_merkle_root: Buffer;
    signature: Buffer;
}

export enum TxPublicKeyEncoding {
    Compressed = 0x00,
    Uncompressed = 0x01,
}

export interface DecodedStacksAddress {
    address_version: number;
    address_hash_bytes: Buffer;
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

export interface ClarityValueInt {
    type_id: ClarityTypeID.Int;
    /** String-quoted signed integer */
    value: string;
}

export interface ClarityValueUInt {
    type_id: ClarityTypeID.UInt;
    /** String-quoted unsigned integer */
    value: string;
}

export interface ClarityValueBoolTrue {
    type_id: ClarityTypeID.BoolTrue;
}

export interface ClarityValueBoolFalse {
    type_id: ClarityTypeID.BoolFalse;
}

export interface ClarityValueBuffer {
    type_id: ClarityTypeID.Buffer;
    buffer: Buffer;
}

export interface ClarityValueList<T extends ClarityValue = ClarityValue> {
    type_id: ClarityTypeID.List;
    list: T[];
}

export interface ClarityValueStringAscii {
    type_id: ClarityTypeID.StringAscii;
    data: string;
}

export interface ClarityValueStringUtf8 {
    type_id: ClarityTypeID.StringUtf8;
    data: string;
}

export interface ClarityValuePrincipalStandard {
    type_id: ClarityTypeID.PrincipalStandard;
    address: string;
    address_version: number;
    address_hash_bytes: Buffer;
}

export interface ClarityValuePrincipalContract {
    type_id: ClarityTypeID.PrincipalContract;
    address: string;
    address_version: number;
    address_hash_bytes: Buffer;
    contract_name: string;
}

export type ClarityTupleData<T extends ClarityValue = ClarityValue> = { [key: string]: T };

export interface ClarityValueTuple<T extends ClarityTupleData = ClarityTupleData> {
    type_id: ClarityTypeID.Tuple;
    data: T;
}

export interface ClarityValueOptionalSome<T extends ClarityValue = ClarityValue> {
    type_id: ClarityTypeID.OptionalSome;
    value: T;
}

export interface ClarityValueOptionalNone {
    type_id: ClarityTypeID.OptionalNone;
}

export interface ClarityValueResponseOk<T extends ClarityValue = ClarityValue> {
    type_id: ClarityTypeID.ResponseOk;
    value: T;
}

export interface ClarityValueResponseError<T extends ClarityValue = ClarityValue> {
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

export interface ParsedClarityValueInfo {
    /** Type signature */
    type: string;
    /** Clarity repr value */
    repr: string;
    /** Hex encoded string of the serialized Clarity value */
    hex: string;
    /** Type represented as contract ABI JSON */
    abi_type?: any;
}

export interface ParsedClarityValueInt extends ClarityValueInt, ParsedClarityValueInfo {}
export interface ParsedClarityValueUInt extends ClarityValueUInt, ParsedClarityValueInfo {}
export interface ParsedClarityValueBoolTrue extends ClarityValueBoolTrue, ParsedClarityValueInfo {}
export interface ParsedClarityValueBoolFalse extends ClarityValueBoolFalse, ParsedClarityValueInfo {}
export interface ParsedClarityValueBuffer extends ClarityValueBuffer, ParsedClarityValueInfo {}
export interface ParsedClarityValueList<T extends ParsedClarityValue = ParsedClarityValue> extends ClarityValueList<T>, ParsedClarityValueInfo {}
export interface ParsedClarityValueStringAscii extends ClarityValueStringAscii, ParsedClarityValueInfo {}
export interface ParsedClarityValueStringUtf8 extends ClarityValueStringUtf8, ParsedClarityValueInfo {}
export interface ParsedClarityValuePrincipalStandard extends ClarityValuePrincipalStandard, ParsedClarityValueInfo {}
export interface ParsedClarityValuePrincipalContract extends ClarityValuePrincipalContract, ParsedClarityValueInfo {}
export type ParsedClarityTupleData<T extends ParsedClarityValue = ParsedClarityValue> = { [key: string]: T };
export interface ParsedClarityValueTuple<T extends ParsedClarityTupleData = ParsedClarityTupleData> extends ClarityValueTuple<T>, ParsedClarityValueInfo {}
export interface ParsedClarityValueOptionalSome<T extends ParsedClarityValue = ParsedClarityValue> extends ClarityValueOptionalSome<T>, ParsedClarityValueInfo {}
export interface ParsedClarityValueOptionalNone extends ClarityValueOptionalNone, ParsedClarityValueInfo {}
export interface ParsedClarityValueResponseOk<T extends ParsedClarityValue = ParsedClarityValue> extends ClarityValueResponseOk<T>, ParsedClarityValueInfo {}
export interface ParsedClarityValueResponseError<T extends ParsedClarityValue = ParsedClarityValue> extends ClarityValueResponseError<T>, ParsedClarityValueInfo {}

export type ParsedClarityValue = 
    | ParsedClarityValueInt
    | ParsedClarityValueUInt
    | ParsedClarityValueBoolTrue
    | ParsedClarityValueBoolFalse
    | ParsedClarityValueBuffer
    | ParsedClarityValueList
    | ParsedClarityValueStringAscii
    | ParsedClarityValueStringUtf8
    | ParsedClarityValuePrincipalStandard
    | ParsedClarityValuePrincipalContract
    | ParsedClarityValueTuple
    | ParsedClarityValueOptionalSome
    | ParsedClarityValueOptionalNone
    | ParsedClarityValueResponseOk
    | ParsedClarityValueResponseError;

export type ParsedClarityValueOptional<T extends ParsedClarityValue = ParsedClarityValue> = ParsedClarityValueOptionalSome<T> | ParsedClarityValueOptionalNone;
export type ParsedClarityValueBool = ParsedClarityValueBoolTrue | ParsedClarityValueBoolFalse;
export type ParsedClarityValueResponse<TOk extends ParsedClarityValue = ParsedClarityValue, TError extends ParsedClarityValue = ParsedClarityValue> = ParsedClarityValueResponseOk<TOk> | ParsedClarityValueResponseError<TError>;

/**
 * Type for commonly used `(optional bool)`
 */
export type ParsedClarityValueOptionalBool = ParsedClarityValueOptional<ParsedClarityValueBool>;

/**
 * Type for commonly used `(optional uint)`
 */
export type ParsedClarityValueOptionalUInt = ParsedClarityValueOptional<ParsedClarityValueUInt>;
