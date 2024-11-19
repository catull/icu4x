// generated by diplomat-tool
import type { pointer, codepoint } from "./diplomat-runtime.d.ts";

// Base enumerator definition
/** See the [Rust documentation for `HangulSyllableType`](https://docs.rs/icu/latest/icu/properties/props/struct.HangulSyllableType.html) for more information.
*/
export class HangulSyllableType {
    constructor(value : HangulSyllableType | string);

    get value() : string;

    get ffiValue() : number;

    static NotApplicable : HangulSyllableType;
    static LeadingJamo : HangulSyllableType;
    static VowelJamo : HangulSyllableType;
    static TrailingJamo : HangulSyllableType;
    static LeadingVowelSyllable : HangulSyllableType;
    static LeadingVowelTrailingSyllable : HangulSyllableType;

    toInteger(): number;

    static fromInteger(other: number): HangulSyllableType | null;
}