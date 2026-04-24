//! `#[lower_by(Hot | Warm | Cold)]` AST-rewriting attribute for notko primitives.
//!
//! This crate is a thin proc-macro entry point. All rewrite logic lives in
//! the sibling [`notko-macros-core`] library so third-party proc-macro
//! crates can reuse it.
//!
//! See the crate README for the tier table, usage patterns, and the shape
//! of custom-tier `notko-optimizers/<Name>.rs` files.
//!
//! [`notko-macros-core`]: https://github.com/orgrinrt/notko/tree/dev/notko-macros-core

use proc_macro::TokenStream;

/// Attribute macro: lower a function's body to the named fallibility tier.
///
/// Built-ins: `Hot`, `Warm`, `Cold`. The argument is a bare ident matching
/// the ZST marker's name in [`notko_macros_core::tiers`]. Unknown tier names
/// are resolved by looking up `<CARGO_MANIFEST_DIR>/notko-optimizers/<Name>.rs`
/// at expansion time.
#[proc_macro_attribute]
pub fn lower_by(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    match notko_macros_core::rewrite::entry(attr, item) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
