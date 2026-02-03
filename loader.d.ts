import type { DecodedPostConditionsResult, DecodedTxResult, DecodedNakamotoBlockResult, DecodedStacksBlockResult, ClarityValue, ClarityValueAbstract } from ".";

export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): DecodedTxResult;

/**
 * Decode a Nakamoto block (Stacks 3.x+).
 * The input should be the raw binary block data as returned by /v3/blocks/{block_id} endpoint.
 * @param arg - Hex string or Buffer containing the raw block data
 */
export function decodeNakamotoBlock(arg: string | Buffer): DecodedNakamotoBlockResult;

/**
 * Decode a Stacks 2.x block.
 * The input should be the raw binary block data as returned by /v2/blocks/{block_id} endpoint.
 * @param arg - Hex string or Buffer containing the raw block data
 */
export function decodeStacksBlock(arg: string | Buffer): DecodedStacksBlockResult;

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

export function startProfiler(): string;

export function stopProfiler(): Buffer;

export function createProfiler(): () => Buffer;

export function perfTestC32Encode(): Buffer;
