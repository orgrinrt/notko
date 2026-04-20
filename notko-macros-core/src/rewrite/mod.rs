//! AST rewriting per fallibility strategy.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, Result};

use crate::discover::resolve_tier;
use crate::parse::parse_tier_arg;
use crate::tiers::{CustomTier, Strategy};

pub mod helpers;
mod hot;
mod outcome;

pub use hot::HotRewriter;
pub use outcome::OutcomeRewriter;

/// Entry point for `#[optimize_for(tier)]` expansion. Parses the attribute
/// argument, resolves the tier (built-in or custom file), and emits the
/// rewritten function.
pub fn entry(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let tier_arg = parse_tier_arg(attr)?;
    let tier = resolve_tier(&tier_arg.name, tier_arg.span)?;
    let input: ItemFn = parse2(item)?;
    rewrite_fn(tier, input)
}

/// Apply a resolved [`CustomTier`] to an `ItemFn`. Public for third-party
/// proc-macro crates authoring their own attribute macros.
pub fn rewrite_fn(tier: CustomTier, input: ItemFn) -> Result<TokenStream> {
    match tier.strategy {
        Strategy::Passthrough => Ok(quote! { #input }),
        Strategy::Hot => hot::rewrite(tier, input),
        Strategy::Cold => outcome::rewrite(tier, input),
    }
}
