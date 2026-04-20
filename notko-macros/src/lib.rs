//! `#[optimize_for(tier)]` — AST-rewriting attribute for the notko primitives.
//!
//! This crate is a thin proc-macro entry point; all rewrite logic lives in
//! the sibling [`notko-macros-core`] library so third-party proc-macro
//! crates can reuse it.
//!
//! See the crate README for the tier table, usage patterns, and the shape
//! of custom-tier `notko-optimizers/<name>.rs` files.
//!
//! [`notko-macros-core`]: https://github.com/orgrinrt/notko/tree/dev/notko-macros-core

use proc_macro::TokenStream;

/// Attribute macro: rewrite a function's body per the named fallibility tier.
///
/// Built-ins: `hot`, `warm`, `cold`. Unknown tier names are resolved by
/// looking up `<CARGO_MANIFEST_DIR>/notko-optimizers/<name>.rs` at
/// expansion time.
#[proc_macro_attribute]
pub fn optimize_for(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    match notko_macros_core::rewrite::entry(attr, item) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
