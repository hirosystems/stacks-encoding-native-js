export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): any;

export function decodeClarityValueToRepr(arg: string | Buffer): any;

export function decodeClarityValue(arg: string | Buffer, includeAbi?: boolean): ParsedClarityValue;

export interface DecodedClarityValueListResult {
    /** Byte span for the given serialized Clarity value list (u32be length-prefixed) */
    buffer: Buffer;
    /** Deserialized Clarity values */
    array: ParsedClarityValue[];
}

export function decodeClarityValueList(arg: string | Buffer, includeAbi?: boolean): DecodedClarityValueListResult;

export function decodePostConditions(arg: string | Buffer): any;

export function getStacksAddress(version: number, hash160: Buffer): string;

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
