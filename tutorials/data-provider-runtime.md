# Loading Additional Data at Runtime

A key feature of ICU4X is the ability to download data dynamically, allowing clients to load additional locales at runtime.

Dynamic data loading can currently be performed in user code. A future core library API may provide this functionality; please submit feedback in [#2985](https://github.com/unicode-org/icu4x/issues/2985).

The following example loads additional locales bucketed by language. This means that different script and regional variants of the same language are assumed to be in the same dynamically loaded data file. However, clients should choose a dynamic loading strategy that works best for them.

```rust
use icu_provider_adapters::either::EitherProvider;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_adapters::fork::ForkByMarkerProvider;
use icu_provider_adapters::fork::MultiForkByErrorProvider;
use icu_provider_adapters::fork::predicates::IdentifierNotFoundPredicate;
use icu_provider_blob::BlobDataProvider;
use icu_provider_fs::FsDataProvider;
use icu_provider::prelude::*;
use icu_provider::hello_world::*;
use icu::locale::locale;
use icu::locale::subtags::Language;
use std::path::Path;
use writeable::Writeable;

// Create the empty MultiForkByErrorProvider:
let mut provider = MultiForkByErrorProvider::new_with_predicate(
    vec![],
    IdentifierNotFoundPredicate
);

// Pretend we're loading these from the network or somewhere.
struct SingleLocaleProvider(DataLocale);

impl DataProvider<HelloWorldV1> for SingleLocaleProvider {
    fn load(&self, req: DataRequest) -> Result<DataResponse<HelloWorldV1>, DataError> {
        if *req.id.locale != self.0 {
            return Err(DataErrorKind::IdentifierNotFound.with_req(HelloWorldV1::INFO, req));
        }
        HelloWorldProvider.load(req)
    }
}

// Helper function to add data into the growable provider on demand:
let mut get_hello_world_formatter = |prefs: HelloWorldFormatterPreferences| {
    // Try to create the formatter a first time with data that has already been loaded.
    if let Ok(formatter) = HelloWorldFormatter::try_new_unstable(&provider, prefs) {
        return formatter;
    }

    // We failed to create the formatter. Load more data for the language and try creating the formatter a second time.
    let loc = HelloWorldV1::make_locale(prefs.locale_preferences);
    provider.push(SingleLocaleProvider(loc));
    HelloWorldFormatter::try_new_unstable(&provider, prefs)
        .expect("Language data should now be available")
};

// Test that it works:
assert_eq!(
    get_hello_world_formatter(locale!("de").into()).format().write_to_string(),
    "Hallo Welt"
);
assert_eq!(
    get_hello_world_formatter(locale!("ro").into()).format().write_to_string(),
    "Salut, lume"
);
```

## Caching Data Provider

ICU4X has no internal caches because there is no one-size-fits-all solution. It is easy for clients to implement their own cache for ICU4X, and although this is not generally required or recommended, it may be beneficial when latency is of utmost importance and, for example, a less-efficient data provider such as JSON is being used.

The following example illustrates an LRU cache on top of a data provider. A practical application would be a BufferProvider that saves deserialized data payloads as type-erased objects and then checks for a cache hit before calling the inner provider.

```rust
use icu_provider::hello_world::{HelloWorldFormatter, HelloWorldProvider};
use icu_provider::prelude::*;
use icu::locale::locale;
use lru::LruCache;
use std::borrow::{Borrow, Cow};
use std::convert::TryInto;
use std::sync::Mutex;
use yoke::Yokeable;
use zerofrom::ZeroFrom;

/// A data provider that caches response payloads in an LRU cache.
pub struct LruDataCache<P> {
    cache: Mutex<LruCache<CacheKeyWrap, Box<dyn core::any::Any>>>,
    provider: P,
}

/// Key for the cache: DataMarkerInfo and DataLocale. The DataLocale is in a Cow
/// so that it can be borrowed during lookup.
#[derive(Debug, PartialEq, Eq, Hash)]
struct CacheKey<'a>(DataMarkerInfo, Cow<'a, DataLocale>);

/// Wrapper over a fully owned CacheKey, required for key borrowing.
#[derive(Debug, PartialEq, Eq, Hash)]
struct CacheKeyWrap(CacheKey<'static>);

// This impl enables a borrowed DataLocale to be used during cache retrieval.
impl<'a> Borrow<CacheKey<'a>> for CacheKeyWrap {
    fn borrow(&self) -> &CacheKey<'a> {
        &Borrow::<CacheKeyWrap>::borrow(self).0
    }
}

impl<M, P> DataProvider<M> for LruDataCache<P>
where
    M: DataMarker,
    M::DataStruct: ZeroFrom<'static, M::DataStruct>,
    for<'a> <M::DataStruct as Yokeable<'a>>::Output: Clone,
    P: DataProvider<M>,
{
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        {
            // First lock: cache retrieval
            let mut cache = self.cache.lock().unwrap();
            let borrowed_cache_key = CacheKey(M::INFO, Cow::Borrowed(req.id.locale));
            if let Some(res) = cache.get(&borrowed_cache_key) {
                // Note: Cloning a DataPayload is usually cheap, and it is necessary in order to
                // convert the short-lived cache object into one we can return.
                return Ok(res.downcast_ref::<DataResponse<M>>().unwrap().clone());
            }
        }
        // Release the lock to invoke the inner provider
        let response = self.provider.load(req)?;
        let owned_cache_key = CacheKeyWrap(CacheKey(M::INFO, Cow::Owned(req.id.locale.clone())));
        // Second lock: cache storage
        Ok(self.cache.lock()
            .unwrap()
            .get_or_insert(owned_cache_key, || Box::new(response))
            .downcast_ref::<DataResponse<M>>()
            .unwrap()
            .clone()
        )
    }
}

// Usage example:
// While HelloWorldProvider does not need to be cached, it may be useful to cache results from
// more expensive providers, like deserializing BufferProviders or providers doing I/O.
let provider = HelloWorldProvider;
let lru_capacity = 100usize.try_into().unwrap();
let provider = LruDataCache {
    cache: Mutex::new(LruCache::new(lru_capacity)),
    provider,
};

// The cache starts empty:
assert_eq!(provider.cache.lock().unwrap().len(), 0);

assert_eq!(
    "こんにちは世界",
    // Note: It is necessary to use `try_new_unstable` with LruDataCache.
    HelloWorldFormatter::try_new_unstable(
        &provider,
        locale!("ja").into()
    )
    .unwrap()
    .format_to_string()
);

// One item in the cache:
assert_eq!(provider.cache.lock().unwrap().len(), 1);

assert_eq!(
    "ওহে বিশ্ব",
    HelloWorldFormatter::try_new_unstable(
        &provider,
        locale!("bn").into()
    )
    .unwrap()
    .format_to_string()
);

// Two items in the cache:
assert_eq!(provider.cache.lock().unwrap().len(), 2);

assert_eq!(
    "こんにちは世界",
    HelloWorldFormatter::try_new_unstable(
        &provider,
        locale!("ja").into()
    )
    .unwrap()
    .format_to_string()
);

// Still only two items in the cache, since we re-requested "ja" data:
assert_eq!(provider.cache.lock().unwrap().len(), 2);
```

## Overwriting Specific Data Items

ICU4X's explicit data pipeline allows for specific data entries to be overwritten in order to customize the output or comply with policy.

The following example illustrates how to overwrite the decimal separators for a region.

```rust
use core::any::Any;
use icu::decimal::DecimalFormatter;
use icu::decimal::provider::{DecimalSymbolsV1, DecimalSymbolStrsBuilder};
use icu_provider::prelude::*;
use icu_provider_adapters::fixed::FixedProvider;
use icu::locale::locale;
use icu::locale::subtags::region;
use tinystr::tinystr;
use zerovec::VarZeroCow;

pub struct CustomDecimalSymbolsProvider<P>(P);

impl<P, M> DataProvider<M> for CustomDecimalSymbolsProvider<P>
where
    P: DataProvider<M>,
    M: DataMarker,
{
    #[inline]
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        let mut res = self.0.load(req)?;
        if req.id.locale.region == Some(region!("CH")) {
            if let Ok(mut decimal_payload) = res.payload.dynamic_cast_mut::<DecimalSymbolsV1>() {
                decimal_payload.with_mut(|data| {
                    let mut builder = DecimalSymbolStrsBuilder::from(&*data.strings);
                    // Change grouping separator for all Swiss locales to '🐮'
                    builder.grouping_separator = VarZeroCow::new_owned("🐮".into());
                    data.strings = builder.build();
                });
            }
        }
        Ok(res)
    }
}

// Make a wrapped provider that modifies Swiss data requests
let provider = CustomDecimalSymbolsProvider(
    // Base our provider off of the default  builtin
    // "compiled data" shipped by icu::decimal by default.
    icu::decimal::provider::Baked
);

let formatter = DecimalFormatter::try_new_unstable(
    &provider,
    locale!("und").into(),
    Default::default(),
)
.unwrap();

assert_eq!(formatter.format_to_string(&100007i64.into()), "100,007");

let formatter = DecimalFormatter::try_new_unstable(
    &provider,
    locale!("und-CH").into(),
    Default::default(),
)
.unwrap();

assert_eq!(formatter.format_to_string(&100007i64.into()), "100🐮007");
```

### Forking Data Providers

Forking providers can be implemented using `DataPayload::dynamic_cast`. For an example, see that function's documentation.

### Exporting Custom Data Markers

To add custom data markers to your baked data or postcard file, create a forking exportable provider:

```rust
use icu::locale::locale;
use icu::plurals::provider::PluralsCardinalV1;
use icu_provider::prelude::*;
use icu_provider::DataMarker;
use icu_provider_adapters::fork::ForkByMarkerProvider;
use icu_provider_blob::BlobDataProvider;
use icu_provider_export::blob_exporter::BlobExporter;
use icu_provider_export::prelude::*;
use icu_provider_source::SourceDataProvider;
use std::borrow::Cow;
use std::collections::BTreeSet;

icu_provider::data_marker!(CustomV1, Custom<'static>);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, databake::Bake, yoke::Yokeable, zerofrom::ZeroFrom)]
#[databake(path = crate)]
pub struct Custom<'data> {
    pub message: Cow<'data, str>,
};

icu_provider::data_struct!(Custom<'_>);

struct CustomProvider;
impl DataProvider<CustomV1> for CustomProvider {
    fn load(&self, req: DataRequest) -> Result<DataResponse<CustomV1>, DataError> {
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(Custom {
                message: format!("Custom data for locale {}!", req.id.locale).into(),
            }),
        })
    }
}

impl IterableDataProvider<CustomV1> for CustomProvider {
    fn iter_ids(&self) -> Result<BTreeSet<DataIdentifierCow>, DataError> {
        Ok([locale!("es"), locale!("ja")]
            .into_iter()
            .map(DataLocale::from)
            .map(DataIdentifierCow::from_locale)
            .collect())
    }
}

extern crate alloc;
icu_provider::export::make_exportable_provider!(CustomProvider, [CustomV1,]);

let icu4x_source_provider = SourceDataProvider::new();
let custom_source_provider = CustomProvider;

let mut buffer = Vec::<u8>::new();

ExportDriver::new(
    [DataLocaleFamily::FULL],
    DeduplicationStrategy::None.into(),
    LocaleFallbacker::try_new_unstable(&icu4x_source_provider).unwrap(),
)
.with_markers([PluralsCardinalV1::INFO, CustomV1::INFO])
.export(
    &ForkByMarkerProvider::new(icu4x_source_provider, custom_source_provider),
    BlobExporter::new_with_sink(Box::new(&mut buffer)),
)
.unwrap();

let blob_provider = BlobDataProvider::try_new_from_blob(buffer.into()).unwrap();

let locale = DataLocale::from(&locale!("ja"));
let req = DataRequest {
    id: DataIdentifierBorrowed::for_locale(&locale),
    metadata: Default::default(),
};

assert!(blob_provider.load_data(PluralsCardinalV1::INFO, req).is_ok());
assert!(blob_provider.load_data(CustomV1::INFO, req).is_ok());
```

### Accessing the Resolved Locale

ICU4X objects do not store their "resolved locale" because that is not a well-defined concept. Components can load data from many sources, and fallbacks to parent locales or root does not necessarily mean that a locale is not supported.

However, for environments that require this behavior, such as ECMA-402, the data provider can be instrumented to access the resolved locale from `DataResponseMetadata`, as shown in the following example.

```rust
use icu_provider::prelude::*;
use icu_provider::hello_world::*;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu::locale::LocaleFallbacker;
use icu::locale::locale;
use std::sync::RwLock;

pub struct ResolvedLocaleProvider<P> {
    inner: P,
    // This could be a RefCell if thread safety is not required:
    resolved_locale: RwLock<Option<DataLocale>>,
}

impl<M, P> DataProvider<M> for ResolvedLocaleProvider<P>
where
    M: DataMarker,
    P: DataProvider<M>
{
    fn load(&self, req: DataRequest) -> Result<DataResponse<M>, DataError> {
        let mut res = self.inner.load(req)?;
        // Whichever locale gets loaded for `HelloWorldV1::INFO` will be the one
        // we consider the "resolved locale". Although `HelloWorldFormatter` only loads
        // one key, this is a useful distinction for most other formatters.
        if M::INFO == HelloWorldV1::INFO {
            let mut w = self.resolved_locale.write().expect("poison");
            *w = res.metadata.locale.take();
        }
        Ok(res)
    }
}

// Set up a HelloWorldProvider with fallback
let provider = ResolvedLocaleProvider {
    inner: LocaleFallbackProvider::new(
        HelloWorldProvider,
        LocaleFallbacker::new().static_to_owned(),
    ),
    resolved_locale: Default::default(),
};

// Request data for sr-ME...
HelloWorldFormatter::try_new_unstable(
    &provider,
    locale!("sr-ME").into(),
)
.unwrap();

// ...which loads data from sr-Latn.
assert_eq!(
    *provider.resolved_locale.read().expect("poison"),
    Some(locale!("sr-Latn").into()),
);
```
