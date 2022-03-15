import type { DecodedClarityValueListResult, DecodedPostConditionsResult, DecodedTxResult, ParsedClarityValue } from ".";

export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): DecodedTxResult;

export function decodeClarityValueToRepr(arg: string | Buffer): string;

export function decodeClarityValue<T extends ParsedClarityValue = ParsedClarityValue>(arg: string | Buffer, includeAbi?: boolean): T;

export function decodeClarityValueList(arg: string | Buffer, includeAbi?: boolean): DecodedClarityValueListResult;

export function decodePostConditions(arg: string | Buffer): DecodedPostConditionsResult;

export function getStacksAddress(version: number, hash160: string | Buffer): string;

export function stacksToBitcoinAddress(stackAddress: string): string;

export function bitcoinToStacksAddress(bitcoinAddress: string): string;

export function isValidStacksAddress(address: string): boolean;

export function decodeStacksAddress(address: string): [version: number, hash160: Buffer];

export function stacksAddressFromParts(version: number, hash160: string | Buffer): string;

export function memoToString(memo: string | Buffer): string;
