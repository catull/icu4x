// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::SourceDataProvider;
use icu::collections::codepointtrie::CodePointTrie;
use icu::properties::props::Script;
use icu::properties::provider::{PropertyScriptWithExtensionsV1, ScriptWithExtensionsProperty};
use icu::properties::script::ScriptWithExt;
use icu_provider::prelude::*;
use std::collections::HashSet;
use std::convert::TryFrom;
use zerovec::{VarZeroVec, ZeroSlice, ZeroVec};

// implement data provider
impl DataProvider<PropertyScriptWithExtensionsV1> for SourceDataProvider {
    fn load(
        &self,
        req: DataRequest,
    ) -> Result<DataResponse<PropertyScriptWithExtensionsV1>, DataError> {
        self.check_req::<PropertyScriptWithExtensionsV1>(req)?;
        let scx_data = self
            .icuexport()?
            .read_and_parse_toml::<super::uprops_serde::script_extensions::Main>(&format!(
                "uprops/{}/scx.toml",
                self.trie_type(),
            ))?
            .script_extensions
            .first()
            .ok_or_else(|| DataError::custom("Could not parse Script_Extensions data from TOML"))?;

        let cpt_data = &scx_data.code_point_trie;
        let scx_array_data = &scx_data.script_code_array;

        let trie = CodePointTrie::<ScriptWithExt>::try_from(cpt_data).map_err(|e| {
            DataError::custom("Could not parse CodePointTrie TOML").with_display_context(&e)
        })?;

        // Convert the input from Vec<Vec<u16>> to Vec<ZeroVec<Script>> so that
        // we can go through the VarZeroVec construction process for a desired result
        // type of VZV<ZeroSlice<Script>>
        let ule_scx_array_data: Vec<ZeroVec<Script>> = scx_array_data
            .iter()
            .map(|v| {
                v.iter()
                    .copied()
                    .map(Script::from_icu4c_value)
                    .collect::<ZeroVec<Script>>()
            })
            .collect::<Vec<ZeroVec<Script>>>();
        let scx_vzv: VarZeroVec<ZeroSlice<Script>> =
            VarZeroVec::from(ule_scx_array_data.as_slice());

        let data_struct = ScriptWithExtensionsProperty {
            trie,
            extensions: scx_vzv,
        };

        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(data_struct),
        })
    }
}

impl crate::IterableDataProviderCached<PropertyScriptWithExtensionsV1> for SourceDataProvider {
    fn iter_ids_cached(&self) -> Result<HashSet<DataIdentifierCow<'static>>, DataError> {
        Ok(HashSet::from_iter([Default::default()]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_val_from_script_extensions() {
        let provider = SourceDataProvider::new_testing();

        let swe =
            icu::properties::script::ScriptWithExtensions::try_new_unstable(&provider).unwrap();
        let swe = swe.as_borrowed();

        assert_eq!(swe.get_script_val('𐓐'), Script::Osage); // U+104D0 OSAGE CAPITAL LETTER KHA
        assert_eq!(swe.get_script_val('🥳'), Script::Common); // U+1F973 FACE WITH PARTY HORN AND PARTY HAT
        assert_eq!(swe.get_script_val32(0x200D), Script::Inherited); // ZERO WIDTH JOINER
        assert_eq!(swe.get_script_val('௫'), Script::Tamil); // U+0BEB TAMIL DIGIT FIVE
        assert_eq!(swe.get_script_val32(0x11303), Script::Grantha); // GRANTHA SIGN VISARGA
        assert_eq!(swe.get_script_val32(0x30A0), Script::Common); // U+30A0 KATAKANA-HIRAGANA DOUBLE HYPHEN
    }

    #[test]
    fn test_scx_array_from_script_extensions() {
        let provider = SourceDataProvider::new_testing();

        let swe =
            icu::properties::script::ScriptWithExtensions::try_new_unstable(&provider).unwrap();
        let swe = swe.as_borrowed();

        assert_eq!(
            swe.get_script_extensions_val('𐓐') /* U+104D0 OSAGE CAPITAL LETTER KHA */
                .iter()
                .collect::<Vec<_>>(),
            [Script::Osage]
        );
        assert_eq!(
            swe.get_script_extensions_val('🥳') /* U+1F973 FACE WITH PARTY HORN AND PARTY HAT */
                .iter()
                .collect::<Vec<_>>(),
            [Script::Common]
        );
        assert_eq!(
            swe.get_script_extensions_val32(0x200D) // ZERO WIDTH JOINER
                .iter()
                .collect::<Vec<_>>(),
            [Script::Inherited]
        );
        assert_eq!(
            swe.get_script_extensions_val('௫') // U+0BEB TAMIL DIGIT FIVE
                .iter()
                .collect::<Vec<_>>(),
            [Script::Tamil, Script::Grantha]
        );
        assert_eq!(
            swe.get_script_extensions_val32(0x11303) // GRANTHA SIGN VISARGA
                .iter()
                .collect::<Vec<_>>(),
            [Script::Tamil, Script::Grantha]
        );
        assert_eq!(
            swe.get_script_extensions_val32(0x30A0) // KATAKANA-HIRAGANA DOUBLE HYPHEN
                .iter()
                .collect::<Vec<_>>(),
            [Script::Hiragana, Script::Katakana]
        );

        assert_eq!(
            swe.get_script_extensions_val32(0x200D) // ZERO WIDTH JOINER
                .iter()
                .next(),
            Some(Script::Inherited)
        );

        assert!(swe
            .get_script_extensions_val32(0x11303) // GRANTHA SIGN VISARGA
            .contains(&Script::Grantha));

        assert!(!swe
            .get_script_extensions_val32(0x11303) // GRANTHA SIGN VISARGA
            .contains(&Script::Common));

        // // Invalid code point
        assert_eq!(
            swe.get_script_extensions_val32(0x11_0000) // CODE_POINT_MAX + 1 is invalid
                .iter()
                .collect::<Vec<_>>(),
            [Script::Unknown]
        );
    }

    #[test]
    fn test_has_script() {
        let provider = SourceDataProvider::new_testing();

        let swe =
            icu::properties::script::ScriptWithExtensions::try_new_unstable(&provider).unwrap();
        let swe = swe.as_borrowed();

        assert!(swe.has_script('𐓐', Script::Osage));
        assert!(!swe.has_script('𐓐', Script::Common));
        assert!(!swe.has_script('𐓐', Script::Inherited));

        assert!(swe.has_script('🥳', Script::Common));
        assert!(!swe.has_script('🥳', Script::Inherited));

        assert!(!swe.has_script32(0x200D, Script::Common));
        assert!(swe.has_script32(0x200D, Script::Inherited));

        assert!(swe.has_script('௫', Script::Tamil));
        assert!(swe.has_script('௫', Script::Grantha));
        assert!(!swe.has_script('௫', Script::Common));
        assert!(!swe.has_script('௫', Script::Inherited));

        assert!(swe.has_script32(0x11303, Script::Tamil));
        assert!(swe.has_script32(0x11303, Script::Grantha));
        assert!(!swe.has_script32(0x11303, Script::Common));
        assert!(!swe.has_script32(0x11303, Script::Inherited));

        assert!(swe.has_script32(0x30A0, Script::Hiragana));
        assert!(swe.has_script32(0x30A0, Script::Katakana));
        assert!(!swe.has_script32(0x30A0, Script::Common));
        assert!(!swe.has_script32(0x30A0, Script::Inherited));

        // U+0964 DEVANAGARI DANDA
        assert!(!swe.has_script32(0x0964, Script::Common));
        assert!(swe.has_script32(0x0964, Script::Devanagari));
        assert!(swe.has_script32(0x0964, Script::Bengali));

        // TestHasScript() test cases from ICU4J

        // U+063F ARABIC LETTER FARSI YEH WITH THREE DOTS ABOVE
        assert!(!swe.has_script32(0x063F, Script::Common));
        assert!(swe.has_script32(0x063F, Script::Arabic)); // main Script value
        assert!(!swe.has_script32(0x063F, Script::Syriac));
        assert!(!swe.has_script32(0x063F, Script::Thaana));

        // U+0640 ARABIC TATWEEL
        assert!(!swe.has_script32(0x0640, Script::Common)); // main Script value
        assert!(swe.has_script32(0x0640, Script::Arabic));
        assert!(swe.has_script32(0x0640, Script::Syriac));
        assert!(!swe.has_script32(0x0640, Script::Thaana));

        // U+0650 ARABIC KASRA
        assert!(!swe.has_script32(0x0650, Script::Inherited)); // main Script value
        assert!(swe.has_script32(0x0650, Script::Arabic));
        assert!(swe.has_script32(0x0650, Script::Syriac));
        assert!(!swe.has_script32(0x0650, Script::Thaana));

        // U+0660 ARABIC-INDIC DIGIT ZERO
        assert!(!swe.has_script32(0x0660, Script::Common));
        assert!(swe.has_script32(0x0660, Script::Arabic)); // main Script value
        assert!(!swe.has_script32(0x0660, Script::Syriac));
        assert!(swe.has_script32(0x0660, Script::Thaana));

        // U+FDF2 ARABIC LIGATURE ALLAH ISOLATED FORM
        assert!(!swe.has_script32(0xFDF2, Script::Common));
        assert!(swe.has_script32(0xFDF2, Script::Arabic)); // main Script value
        assert!(!swe.has_script32(0xFDF2, Script::Syriac));
        assert!(swe.has_script32(0xFDF2, Script::Thaana));

        // The ICU4J comment for this test says:
        // An unguarded implementation might go into an infinite loop.
        assert!(!swe.has_script32(0x0640, Script::from_icu4c_value(0xAFFE)));
    }

    #[test]
    fn test_get_script_extensions_set() {
        let provider = SourceDataProvider::new_testing();

        let swe =
            icu::properties::script::ScriptWithExtensions::try_new_unstable(&provider).unwrap();
        let swe = swe.as_borrowed();

        let grantha = swe.get_script_extensions_set(Script::Grantha);
        assert!(!grantha.contains32(0x0BE5)); // unknown with unknown script in Tamil block
        assert!(grantha.contains32(0x0BE6)); // TAMIL DIGIT ZERO
        assert!(grantha.contains32(0x0BEB)); // TAMIL DIGIT FIVE
        assert!(grantha.contains32(0x0BEF)); // TAMIL DIGIT NINE
        assert!(grantha.contains32(0x0BF2)); // TAMIL NUMBER ONE THOUSAND
        assert!(grantha.contains32(0x0BF3)); // TAMIL DAY SIGN
        assert!(!grantha.contains32(0x0BF4)); // TAMIL MONTH SIGN
        assert!(grantha.contains32(0x11300)); // GRANTHA SIGN COMBINING ANUSVARA ABOVE
        assert!(grantha.contains32(0x11301)); // GRANTHA SIGN CANDRABINDU
        assert!(grantha.contains32(0x11302)); // GRANTHA SIGN ANUSVARA
        assert!(grantha.contains32(0x11303)); // GRANTHA SIGN VISARGA
        assert!(!grantha.contains32(0x11304)); // unknown with unknown script in Grantha block
        assert!(grantha.contains32(0x11305)); // GRANTHA LETTER A

        let tamil = swe.get_script_extensions_set(Script::Tamil);
        assert!(!tamil.contains32(0x0BE5)); // unknown with unknown script in Tamil block
        assert!(tamil.contains32(0x0BE6)); // TAMIL DIGIT ZERO
        assert!(tamil.contains32(0x0BEB)); // TAMIL DIGIT FIVE
        assert!(tamil.contains32(0x0BEF)); // TAMIL DIGIT NINE
        assert!(tamil.contains32(0x0BF2)); // TAMIL NUMBER ONE THOUSAND
        assert!(tamil.contains32(0x0BF3)); // TAMIL DAY SIGN
        assert!(tamil.contains32(0x0BF4)); // TAMIL MONTH SIGN
        assert!(!tamil.contains32(0x11300)); // GRANTHA SIGN COMBINING ANUSVARA ABOVE
        assert!(tamil.contains32(0x11301)); // GRANTHA SIGN CANDRABINDU
        assert!(!tamil.contains32(0x11302)); // GRANTHA SIGN ANUSVARA
        assert!(tamil.contains32(0x11303)); // GRANTHA SIGN VISARGA
        assert!(!tamil.contains32(0x11304)); // unknown with unknown script in Grantha block
        assert!(!tamil.contains32(0x11305)); // GRANTHA LETTER A

        let hiragana = swe.get_script_extensions_set(Script::Hiragana);
        assert!(hiragana.contains32(0x3046)); // HIRAGANA LETTER U
        assert!(hiragana.contains32(0x309F)); // HIRAGANA DIGRAPH YORI
        assert!(hiragana.contains32(0x30A0)); // KATAKANA-HIRAGANA DOUBLE HYPHEN
        assert!(!hiragana.contains32(0x30A1)); // KATAKANA LETTER SMALL A
        assert!(hiragana.contains32(0x30FB)); // KATAKANA MIDDLE DOT
        assert!(hiragana.contains32(0x30FC)); // KATAKANA-HIRAGANA PROLONGED SOUND MARK
        assert!(!hiragana.contains32(0x30FD)); // KATAKANA ITERATION MARK

        let katakana = swe.get_script_extensions_set(Script::Katakana);
        assert!(!katakana.contains32(0x3046)); // HIRAGANA LETTER U
        assert!(!katakana.contains32(0x309F)); // HIRAGANA DIGRAPH YORI
        assert!(katakana.contains32(0x30A0)); // KATAKANA-HIRAGANA DOUBLE HYPHEN
        assert!(katakana.contains32(0x30A1)); // KATAKANA LETTER SMALL A
        assert!(katakana.contains32(0x30FB)); // KATAKANA MIDDLE DOT
        assert!(katakana.contains32(0x30FC)); // KATAKANA-HIRAGANA PROLONGED SOUND MARK
        assert!(katakana.contains32(0x30FD)); // KATAKANA ITERATION MARK

        let common = swe.get_script_extensions_set(Script::Common);
        assert!(common.contains('🥳'));
        assert!(!common.contains32(0x200D));
        assert!(!common.contains32(0x30A0));

        let inherited = swe.get_script_extensions_set(Script::Inherited);
        assert!(!inherited.contains('🥳'));
        assert!(inherited.contains32(0x200D));
        assert!(!inherited.contains32(0x30A0));

        // inspired by https://github.com/unicode-org/unicodetools/issues/192

        let bangla = swe.get_script_extensions_set(Script::Bengali);
        assert!(bangla.contains32(0x09E7)); // BENGALI DIGIT ONE
        assert!(!bangla.contains32(0x0963)); // DEVANAGARI VOWEL SIGN VOCALIC LL
        assert!(bangla.contains32(0x0964)); // DEVANAGARI DANDA
        assert!(bangla.contains32(0x0965)); // DEVANAGARI DOUBLE DANDA
        assert!(!bangla.contains32(0x0966)); // DEVANAGARI DIGIT ZERO

        let devanagari = swe.get_script_extensions_set(Script::Devanagari);
        assert!(!devanagari.contains32(0x09E7)); // BENGALI DIGIT ONE
        assert!(devanagari.contains32(0x0963)); // DEVANAGARI VOWEL SIGN VOCALIC LL
        assert!(devanagari.contains32(0x0964)); // DEVANAGARI DANDA
        assert!(devanagari.contains32(0x0965)); // DEVANAGARI DOUBLE DANDA
        assert!(devanagari.contains32(0x0966)); // DEVANAGARI DIGIT ZERO

        assert!(!common.contains32(0x0964)); // DEVANAGARI DANDA
        assert!(!common.contains32(0x0965)); // DEVANAGARI DOUBLE DANDA
    }
}
