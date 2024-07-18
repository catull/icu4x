// generated by diplomat-tool
import type { DataProvider } from "./DataProvider"
import type { WordBreakIteratorUtf16 } from "./WordBreakIteratorUtf16"
import type { pointer, char } from "./diplomat-runtime.d.ts";


/** An ICU4X word-break segmenter, capable of finding word breakpoints in strings.
*
*See the [Rust documentation for `WordSegmenter`](https://docs.rs/icu/latest/icu/segmenter/struct.WordSegmenter.html) for more information.
*/
export class WordSegmenter {
    

    get ffiValue(): pointer;


    static createAuto(provider: DataProvider): WordSegmenter;

    static createLstm(provider: DataProvider): WordSegmenter;

    static createDictionary(provider: DataProvider): WordSegmenter;

    segment(input: string): WordBreakIteratorUtf16;

    

}