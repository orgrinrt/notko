//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Warm / Maybe-based rewrite. Emits `Maybe<T>` regardless of build profile,
//! which discards the error and keeps a one-bit discriminant.
//!
//! Discarding is the whole point rather than an oversight: the warm tier is
//! the one that has decided the error is not worth carrying. A caller who
//! wants it asks for the cold tier instead.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Expr, ExprReturn, ItemFn, Path, Result, Type, parse_quote};

use super::helpers::{extract_result_inner_types, is_err_call, is_ok_call};
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    let (ok_ty, err_ty) = extract_result_inner_types(&func.sig.output);
    if let (Some(t), Some(_)) = (ok_ty, err_ty) {
        set_maybe_return(&tier.krate, &mut func, t);
    }

    let mut rewriter = MaybeRewriter {
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

fn set_maybe_return(krate: &Path, func: &mut ItemFn, t: Type) {
    func.sig.output = parse_quote! { -> #krate::Maybe<#t> };
}

/// Visitor that rewrites:
/// - `Ok(x)` → `<krate>::Maybe::Is(x)`
/// - `Err(_)` → `<krate>::Maybe::Isnt`
pub struct MaybeRewriter {
    /// The crate `Ok` and `Err` are rewritten through.
    pub krate: Path,
}

impl VisitMut for MaybeRewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if matches!(expr, Expr::Closure(_)) {
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Call(call) = expr {
            let k = &self.krate;
            if is_ok_call(call) {
                let val = call.args.first().unwrap().clone();
                *expr = parse_quote! { #k::Maybe::Is(#val) };
                return;
            }
            if is_err_call(call) {
                // The error expression is dropped rather than evaluated. It may
                // have side effects, and keeping it would mean emitting a
                // statement where an expression was, which changes the shape of
                // the surrounding tree.
                *expr = parse_quote! { #k::Maybe::Isnt };
            }
        }
    }

    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, ret);
        if let Some(inner) = &mut ret.expr {
            let k = &self.krate;
            let replacement = match inner.as_ref() {
                Expr::Call(call) if is_ok_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(parse_quote! { #k::Maybe::Is(#val) })
                }
                Expr::Call(call) if is_err_call(call) => Some(parse_quote! { #k::Maybe::Isnt }),
                _ => None,
            };
            if let Some(r) = replacement {
                **inner = r;
            }
        }
    }

    fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {}
}
