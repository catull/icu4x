// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::{ule::VarULE, VarZeroSlice, VarZeroVec};
use databake::*;

impl<T: VarULE + ?Sized> Bake for VarZeroVec<'_, T> {
    fn bake(&self, env: &CrateEnv) -> TokenStream {
        env.insert("zerovec");
        if self.is_empty() {
            quote! { zerovec::VarZeroVec::new() }
        } else {
            let bytes = databake::Bake::bake(&self.as_bytes(), env);
            // Safe because self.as_bytes is a safe input
            quote! { unsafe { zerovec::VarZeroVec::from_bytes_unchecked(#bytes) } }
        }
    }
}

impl<T: VarULE + ?Sized> BakeSize for VarZeroVec<'_, T> {
    fn borrows_size(&self) -> usize {
        self.as_bytes().len()
    }
}

impl<T: VarULE + ?Sized> Bake for &VarZeroSlice<T> {
    fn bake(&self, env: &CrateEnv) -> TokenStream {
        env.insert("zerovec");
        if self.is_empty() {
            quote! { zerovec::VarZeroSlice::new_empty() }
        } else {
            let bytes = databake::Bake::bake(&self.as_bytes(), env);
            // Safe because self.as_bytes is a safe input
            quote! { unsafe { zerovec::VarZeroSlice::from_bytes_unchecked(#bytes) } }
        }
    }
}

impl<T: VarULE + ?Sized> BakeSize for &VarZeroSlice<T> {
    fn borrows_size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.as_bytes().len()
        }
    }
}

#[test]
fn test_baked_vec() {
    test_bake!(VarZeroVec<str>, const, crate::VarZeroVec::new(), zerovec);

    test_bake!(
        VarZeroVec<str>,
        const,
        unsafe { crate::VarZeroVec::from_bytes_unchecked(b"\x02\0\0\0\0\0\x05\0helloworld") },
        zerovec
    );
}

#[test]
fn test_baked_slice() {
    test_bake!(
        &VarZeroSlice<str>,
        const,
        crate::VarZeroSlice::new_empty(),
        zerovec
    );
    test_bake!(
        &VarZeroSlice<str>,
        const,
        unsafe { crate::VarZeroSlice::from_bytes_unchecked(b"\x02\0\0\0\0\0\x05\0helloworld") },
        zerovec
    );
}
