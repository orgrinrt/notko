//! Parse the `#[profile(...)]` attribute arguments.

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result};

/// Parsed attribute argument: a single tier identifier.
pub struct TierArg {
    pub name: String,
    pub span: proc_macro2::Span,
}

impl Parse for TierArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "expected a single tier name, e.g., `#[profile(Hot)]`",
            ));
        }
        Ok(TierArg {
            name: ident.to_string(),
            span: ident.span(),
        })
    }
}

pub fn parse_tier_arg(tokens: TokenStream) -> Result<TierArg> {
    syn::parse2::<TierArg>(tokens)
}
