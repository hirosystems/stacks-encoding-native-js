import { DecodedClarityValueListResult, ParsedClarityValue } from "./types";

export interface StacksEncodingNativeBindings {
    getVersion(): string;

    decodeTransaction(arg: string | Buffer): any;
    
    decodeClarityValueToRepr(arg: string | Buffer): string;
    
    decodeClarityValue<T extends ParsedClarityValue = ParsedClarityValue>(arg: string | Buffer, includeAbi?: boolean): T;
    
    decodeClarityValueList(arg: string | Buffer, includeAbi?: boolean): DecodedClarityValueListResult;
    
    decodePostConditions(arg: string | Buffer): any;
    
    getStacksAddress(version: number, hash160: Buffer): string;
}

export * from "./types";

export const stacksEncodingNativeBindings: StacksEncodingNativeBindings = require("../index.node");
export default stacksEncodingNativeBindings;
