// generated by diplomat-tool
import type { GeneralCategory } from "./GeneralCategory"
import type { pointer, codepoint } from "./diplomat-runtime.d.ts";


/** 
 * A mask that is capable of representing groups of `General_Category` values.
 *
 * See the [Rust documentation for `GeneralCategoryGroup`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html) for more information.
 */
type GeneralCategoryGroup_obj = {
    mask: number;
};



export class GeneralCategoryGroup {
    
    get mask() : number; 
    set mask(value: number); 
    
    /** Create `GeneralCategoryGroup` from an object that contains all of `GeneralCategoryGroup`s fields.
    * Optional fields do not need to be included in the provided object.
    */
    static fromFields(structObj : GeneralCategoryGroup_obj) : GeneralCategoryGroup;


    /** 
     * See the [Rust documentation for `contains`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.contains) for more information.
     */
    contains(val: GeneralCategory): boolean;

    /** 
     * See the [Rust documentation for `complement`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.complement) for more information.
     */
    complement(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `all`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.all) for more information.
     */
    static all(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `empty`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.empty) for more information.
     */
    static empty(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `union`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.union) for more information.
     */
    union(other: GeneralCategoryGroup_obj): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `intersection`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#method.intersection) for more information.
     */
    intersection(other: GeneralCategoryGroup_obj): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `CasedLetter`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.CasedLetter) for more information.
     */
    static casedLetter(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Letter`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Letter) for more information.
     */
    static letter(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Mark`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Mark) for more information.
     */
    static mark(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Number`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Number) for more information.
     */
    static number(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Other`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Other) for more information.
     */
    static separator(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Letter`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Letter) for more information.
     */
    static other(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Punctuation`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Punctuation) for more information.
     */
    static punctuation(): GeneralCategoryGroup;

    /** 
     * See the [Rust documentation for `Symbol`](https://docs.rs/icu/latest/icu/properties/props/struct.GeneralCategoryGroup.html#associatedconstant.Symbol) for more information.
     */
    static symbol(): GeneralCategoryGroup;

    constructor(structObj : GeneralCategoryGroup_obj);
}