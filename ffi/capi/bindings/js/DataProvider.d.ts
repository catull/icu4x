// generated by diplomat-tool
import type { DataError } from "./DataError"
import type { LocaleFallbacker } from "./LocaleFallbacker"
import type { pointer, codepoint } from "./diplomat-runtime.d.ts";



/**
 * An ICU4X data provider, capable of loading ICU4X data keys from some source.
 *
 * Currently the only source supported is loading from "blob" formatted data from a bytes buffer or the file system.
 *
 * If you wish to use ICU4X's builtin "compiled data", use the version of the constructors that do not have `_with_provider`
 * in their names.
 *
 * See the [Rust documentation for `icu_provider`](https://docs.rs/icu_provider/2.0.0/icu_provider/index.html) for more information.
 */
export class DataProvider {
    /** @internal */
    get ffiValue(): pointer;
    /** @internal */
    constructor();


    /**
     * See the [Rust documentation for `try_new_from_blob`](https://docs.rs/icu_provider_blob/2.0.0/icu_provider_blob/struct.BlobDataProvider.html#method.try_new_from_blob) for more information.
     */
    static fromByteSlice(blob: Uint8Array): DataProvider;

    /**
     * Creates a provider that tries the current provider and then, if the current provider
     * doesn't support the data key, another provider `other`.
     *
     * This takes ownership of the `other` provider, leaving an empty provider in its place.
     *
     * See the [Rust documentation for `ForkByMarkerProvider`](https://docs.rs/icu_provider_adapters/2.0.0/icu_provider_adapters/fork/type.ForkByMarkerProvider.html) for more information.
     */
    forkByMarker(other: DataProvider): void;

    /**
     * Same as `fork_by_key` but forks by locale instead of key.
     *
     * See the [Rust documentation for `IdentifierNotFoundPredicate`](https://docs.rs/icu_provider_adapters/2.0.0/icu_provider_adapters/fork/predicates/struct.IdentifierNotFoundPredicate.html) for more information.
     */
    forkByLocale(other: DataProvider): void;

    /**
     * See the [Rust documentation for `new`](https://docs.rs/icu_provider_adapters/2.0.0/icu_provider_adapters/fallback/struct.LocaleFallbackProvider.html#method.new) for more information.
     *
     * Additional information: [1](https://docs.rs/icu_provider_adapters/2.0.0/icu_provider_adapters/fallback/struct.LocaleFallbackProvider.html)
     */
    enableLocaleFallbackWith(fallbacker: LocaleFallbacker): void;
}