export function getVersion(): string;

export function decodeTransaction(arg: string | Buffer): any;

export function decodeClarityValueToRepr(arg: string | Buffer): any;

export function decodeClarityValueToJson(arg: string | Buffer, includeAbi?: boolean): any;

export function decodeClarityValueList(arg: string | Buffer): any;

export function decodePostConditions(arg: string | Buffer): any;

export function getStacksAddress(version: number, hash160: Buffer): string;