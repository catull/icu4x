// @generated
include!("collation_reordering_v1.rs.data");
include!("collation_diacritics_v1.rs.data");
include!("collation_jamo_v1.rs.data");
include!("collation_metadata_v1.rs.data");
include!("collation_tailoring_v1.rs.data");
include!("collation_special_primaries_v1.rs.data");
include!("collation_root_v1.rs.data");
/// Marks a type as a data provider. You can then use macros like
/// `impl_core_helloworld_v1` to add implementations.
///
/// ```ignore
/// struct MyProvider;
/// const _: () = {
///     include!("path/to/generated/macros.rs");
///     make_provider!(MyProvider);
///     impl_core_helloworld_v1!(MyProvider);
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __make_provider {
    ($ name : ty) => {
        #[clippy::msrv = "1.82"]
        impl $name {
            #[allow(dead_code)]
            pub(crate) const MUST_USE_MAKE_PROVIDER_MACRO: () = ();
        }
        icu_provider::marker::impl_data_provider_never_marker!($name);
    };
}
#[doc(inline)]
pub use __make_provider as make_provider;
/// This macro requires the following crates:
/// * `icu`
/// * `icu_provider`
/// * `icu_provider/baked`
/// * `zerovec`
#[allow(unused_macros)]
macro_rules! impl_data_provider {
    ($ provider : ty) => {
        make_provider!($provider);
        impl_collation_reordering_v1!($provider);
        impl_collation_diacritics_v1!($provider);
        impl_collation_jamo_v1!($provider);
        impl_collation_metadata_v1!($provider);
        impl_collation_tailoring_v1!($provider);
        impl_collation_special_primaries_v1!($provider);
        impl_collation_root_v1!($provider);
    };
}
