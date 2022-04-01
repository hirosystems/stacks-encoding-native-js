import type { DecodedPostConditionsResult, DecodedTxResult, ParsedClarityValue } from ".";

export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): DecodedTxResult;

export function decodeClarityValueToRepr(arg: string | Buffer): string;

export function decodeClarityValueToTypeName(arg: string | Buffer): string;

export function decodeClarityValue<T extends ParsedClarityValue = ParsedClarityValue>(arg: string | Buffer, includeAbi?: boolean): T;

/**
 * 
 * @param arg 
 * @param includeAbi 
 * @param deep - If not specified as true, then the deserialized objects will only contain the 
 * properties `hex, repr, type, type_id`. And nested types like Tuple, List, Response, etc will
 * not contain decoded children.
 * TODO: fix the clarity result type definition to be more accurate.
 */
export function decodeClarityValueList(arg: string | Buffer, includeAbi?: boolean, deep?: false): ParsedClarityValue[];

export function decodeClarityValueList2(arg: string | Buffer): {repr: string; hex: string; type_id: number;}[];

export function decodePostConditions(arg: string | Buffer): DecodedPostConditionsResult;

export function stacksToBitcoinAddress(stackAddress: string): string;

export function bitcoinToStacksAddress(bitcoinAddress: string): string;

export function isValidStacksAddress(address: string): boolean;

export function decodeStacksAddress(address: string): [version: number, hash160: Buffer];

export function stacksAddressFromParts(version: number, hash160: string | Buffer): string;

export function memoToString(memo: string | Buffer): string;

export function startProfiler(): string;

export function stopProfiler(): Buffer;

export function createProfiler(): () => Buffer;

export function perfTestC32Encode(): Buffer;
