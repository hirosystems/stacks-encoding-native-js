import type { DecodedPostConditionsResult, DecodedTxResult, ClarityValue, ClarityValueAbstract, DecodedNakamotoBlockResult, DecodedNakamotoBlockHeader } from ".";

export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): DecodedTxResult;

export function decodeClarityValueToRepr(arg: string | Buffer): string;

export function decodeClarityValueToTypeName(arg: string | Buffer): string;

export function decodeClarityValue<T extends ClarityValue = ClarityValue>(arg: string | Buffer): T;

/**
 * 
 * @param arg 
 * @param deep - If not true, then the deserialized objects will only contain the 
 * properties `hex, repr, type, type_id`. And nested types like Tuple, List, Response, etc will
 * not contain decoded children.
 * TODO: fix the clarity result type definition to be more accurate.
 */
export function decodeClarityValueList(arg: string | Buffer, deep?: false | undefined): ClarityValueAbstract[];

/**
 * 
 * @param arg 
 * @param deep - If not true, then the deserialized objects will only contain the 
 * properties `hex, repr, type, type_id`. And nested types like Tuple, List, Response, etc will
 * not contain decoded children.
 * TODO: fix the clarity result type definition to be more accurate.
 */
export function decodeClarityValueList(arg: string | Buffer, deep: true): ClarityValue[];

export function decodePostConditions(arg: string | Buffer): DecodedPostConditionsResult;

export function stacksToBitcoinAddress(stackAddress: string): string;

export function bitcoinToStacksAddress(bitcoinAddress: string): string;

export function isValidStacksAddress(address: string): boolean;

export function decodeStacksAddress(address: string): [version: number, hash160: string];

export function decodeClarityValueToPrincipal(clarityValue: string | Buffer) : string;

export function stacksAddressFromParts(version: number, hash160: string | Buffer): string;

export function memoToString(memo: string | Buffer): string;

/**
 * Decode a Nakamoto block from serialized bytes
 * @param arg - Hex string or Buffer containing the serialized Nakamoto block
 * @returns Decoded block with header and transactions
 */
export function decodeNakamotoBlock(arg: string | Buffer): DecodedNakamotoBlockResult;

/**
 * Decode a Nakamoto block header from serialized bytes
 * @param arg - Hex string or Buffer containing the serialized Nakamoto block header
 * @returns Decoded block header
 */
export function decodeNakamotoBlockHeader(arg: string | Buffer): DecodedNakamotoBlockHeader;

export function startProfiler(): string;

export function stopProfiler(): Buffer;

export function createProfiler(): () => Buffer;

export function perfTestC32Encode(): Buffer;
