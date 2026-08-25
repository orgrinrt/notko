//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Cold / Outcome-based rewrite. Always emits `Outcome<T, E>` regardless of
//! build profile.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Expr, ExprReturn, ItemFn, Path, Result, Type, parse_quote};

use super::helpers::{is_err_call, is_ok_call, result_inner_types};
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    // See `maybe::rewrite`: an unrecognised return type is emitted untouched,
    // body included.
    let Some((t, e)) = result_inner_types(&func.sig.output) else {
        return Ok(quote! { #func });
    };
    set_outcome_return(&tier.krate, &mut func, t, e);

    let mut rewriter = OutcomeRewriter {
        krate: tier.krate.clone(),
    };
    rewriter.visit_block_mut(&mut func.block);

    let inline = if tier.inline {
        quote! { #[inline] }
    } else {
        TokenStream::new()
    };
    let attrs = &func.attrs;
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    Ok(quote! {
        #inline
        #(#attrs)*
        #vis #sig #block
    })
}

fn set_outcome_return(krate: &Path, func: &mut ItemFn, t: Type, e: Type) {
    func.sig.output = parse_quote! { -> #krate::Outcome<#t, #e> };
}

/// Visitor that rewrites:
/// - `Ok(x)` → `<krate>::Outcome::Ok(x)`
/// - `Err(e)` → `<krate>::Outcome::Err(e)`
pub struct OutcomeRewriter {
    /// The crate `Ok` and `Err` are rewritten through.
    pub krate: Path,
}

impl VisitMut for OutcomeRewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if matches!(expr, Expr::Closure(_)) {
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Call(call) = expr {
            if is_ok_call(call) {
                let val = call.args.first().unwrap().clone();
                let k = &self.krate;
                *expr = parse_quote! { #k::Outcome::Ok(#val) };
                return;
            }
            if is_err_call(call) {
                let val = call.args.first().unwrap().clone();
                let k = &self.krate;
                *expr = parse_quote! { #k::Outcome::Err(#val) };
            }
        }
    }

    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, ret);
        if let Some(inner) = &mut ret.expr {
            let replacement = match inner.as_ref() {
                Expr::Call(call) if is_ok_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    let k = &self.krate;
                    Some(parse_quote! { #k::Outcome::Ok(#val) })
                }
                Expr::Call(call) if is_err_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    let k = &self.krate;
                    Some(parse_quote! { #k::Outcome::Err(#val) })
                }
                _ => None,
            };
            if let Some(r) = replacement {
                // Writing through the box rather than replacing it, so the
                // rewrite reuses the allocation the tree already holds.
                **inner = r;
            }
        }
    }

    fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {}
}
