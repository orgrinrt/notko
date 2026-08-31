//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `#[profile(Hot | Warm | Cold)]`, which takes an ordinary `Result` function and
//! rewrites both its signature and its body into the [notko] tier named on the
//! tag, so the choice sits in one place instead of in every type spelled out
//! inside the function.
//!
//! There is nothing else here. The rewrite itself lives in
//! [`notko-macros-core`], as an ordinary library rather than a proc-macro one,
//! so an attribute of your own can build on it instead of doing the rewrite
//! again, and the tiers live in [notko], which is what the rewritten body names.
//!
//! Do note the rewrite fires on a return type spelled `Result<T, E>` or
//! `Outcome<T, E>` with both arguments written out, and anything else is emitted
//! exactly as written with no error and no warning, the common `type Result<T>`
//! alias included. The crate README carries that, the tier table, and the shape
//! of a custom `notko-optimisers/<Name>.rs`.
//!
//! [notko]: https://crates.io/crates/notko
//! [`notko-macros-core`]: https://crates.io/crates/notko-macros-core

use proc_macro::TokenStream;

/// Annotate a function with a fallibility profile. The proc-macro rewrites
/// the body per the profile's strategy at expansion time.
///
/// Built-ins: `Hot`, `Warm`, `Cold`. The argument is a bare ident matching
/// the ZST marker's name in [`notko_macros_core::tiers`]. Unknown profile
/// names are resolved by looking up
/// `<CARGO_MANIFEST_DIR>/notko-optimisers/<Name>.rs` at expansion time.
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    match notko_macros_core::rewrite::entry(attr, item) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
