export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): any;

export function decodeClarityValueToRepr(arg: string | Buffer): any;

export function decodeClarityValue(arg: string | Buffer, includeAbi?: boolean): ParsedClarityValue;

export function decodeClarityValueList(arg: string | Buffer): ParsedClarityValue[];

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

export interface BaseParsedClarityValue {
    /** Type signature */
    type: string;
    /** Clarity repr value */
    repr: string;
    /** Hex encoded string of the serialized Clarity value */
    hex: string;
    /** Type represented as contract ABI JSON */
    abi_type?: any;
    /** Clarity type identifier */
    type_id: ClarityTypeID;
}

export interface ParsedClarityValueInt extends BaseParsedClarityValue {
    type_id: ClarityTypeID.Int;
    /** String-quoted signed integer */
    value: string;
}

export interface ParsedClarityValueUInt extends BaseParsedClarityValue {
    type_id: ClarityTypeID.UInt;
    /** String-quoted unsigned integer */
    value: string;
}

export interface ParsedClarityValueBoolTrue extends BaseParsedClarityValue {
    type_id: ClarityTypeID.BoolTrue;
}

export interface ParsedClarityValueBoolFalse extends BaseParsedClarityValue {
    type_id: ClarityTypeID.BoolFalse;
}

export interface ParsedClarityValueBuffer extends BaseParsedClarityValue {
    type_id: ClarityTypeID.Buffer;
    buffer: Buffer;
}

export interface ParsedClarityValueList extends BaseParsedClarityValue {
    type_id: ClarityTypeID.List;
    list: ParsedClarityValue[];
}

export interface ParsedClarityValueStringAscii extends BaseParsedClarityValue {
    type_id: ClarityTypeID.StringAscii;
    data: string;
}

export interface ParsedClarityValueStringUtf8 extends BaseParsedClarityValue {
    type_id: ClarityTypeID.StringUtf8;
    data: string;
}

export interface ParsedClarityValuePrincipalStandard extends BaseParsedClarityValue {
    type_id: ClarityTypeID.PrincipalStandard;
    address: string;
}

export interface ParsedClarityValuePrincipalContract extends BaseParsedClarityValue {
    type_id: ClarityTypeID.PrincipalContract;
    address: string;
    contract_name: string;
}

export interface ParsedClarityValueTuple extends BaseParsedClarityValue {
    type_id: ClarityTypeID.Tuple;
    data: Record<string, ParsedClarityValue>;
}

export interface ParsedClarityValueOptionalSome extends BaseParsedClarityValue {
    type_id: ClarityTypeID.OptionalSome;
    value: ParsedClarityValue;
}

export interface ParsedClarityValueOptionalNone extends BaseParsedClarityValue {
    type_id: ClarityTypeID.OptionalNone;
}

export interface ParsedClarityValueResponseOk extends BaseParsedClarityValue {
    type_id: ClarityTypeID.ResponseOk;
    value: ParsedClarityValue;
}

export interface ParsedClarityValueResponseError extends BaseParsedClarityValue {
    type_id: ClarityTypeID.ResponseError;
    value: ParsedClarityValue;
}

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