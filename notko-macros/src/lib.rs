//! `#[optimize_for(tier)]` — AST-rewriting attribute for the notko primitives.
//!
//! See the crate README for the tier table and usage patterns.
//!
//! # Extension for third-party proc-macro authors
//!
//! Proc-macro crates cannot re-export non-macro items, so the rewrite
//! primitives (`HotRewriter`, `OutcomeRewriter`, tier types, …) are
//! currently private. A `notko-macros-core` split that exposes them as a
//! normal library crate is tracked separately.
//!
//! Until then, custom tier NAMES with parameterised rewrite strategies are
//! user-extensible via the config-file mechanism in [`discover`]. Truly
//! novel AST rewrites require forking this crate.

use proc_macro::TokenStream;

mod discover;
mod parse;
mod rewrite;
mod tiers;

/// Attribute macro: rewrite a function's body per the named fallibility tier.
///
/// Built-ins: `hot`, `warm`, `cold`. See the crate README for details.
///
/// Unknown tier names are resolved by looking up
/// `<CARGO_MANIFEST_DIR>/notko-optimizers/<name>.rs` at expansion time.
/// See the module-level docs of `discover` and the README for the
/// custom-optimiser file shape.
#[proc_macro_attribute]
pub fn optimize_for(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    match rewrite::entry(attr, item) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
