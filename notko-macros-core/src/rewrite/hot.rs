//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Hot-strategy rewrite.
//!
//! Emits two cfg-gated versions of the function:
//! - debug / `standalone` / `internal` feature off → `Outcome<T, E>` body.
//! - release + `internal` feature on → `Just<T>` body with Err → panic.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Expr, ExprMatch, ExprReturn, ItemFn, Pat, Path, Result, Type, parse_quote};

use super::helpers::{is_err_call, is_ok_call, path_is_bare, result_inner_types};
use super::outcome::OutcomeRewriter;
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    // One function emitted twice under opposite `cfg`s, so anything the two
    // arms would not do identically is a program that means one thing when it
    // is tested and another when it ships. An unrecognised return type is one
    // of those: the debug arm had nothing to lift it to and left it, and the
    // release arm wrapped it, so a caller compiled in debug and did not in
    // release. Emit it once, untouched, and there is nothing to diverge.
    let Some((ok_ty, err_ty)) = result_inner_types(&func.sig.output) else {
        return Ok(quote! { #func });
    };

    let debug_fn = build_debug(&tier, &func, ok_ty.clone(), err_ty.clone());
    let release_fn = build_release(&tier, &mut func, ok_ty);
    let gate = tier.gate_feature.as_str();

    Ok(quote! {
        #[cfg(any(not(feature = #gate), debug_assertions))]
        #debug_fn

        #[cfg(all(feature = #gate, not(debug_assertions)))]
        #release_fn
    })
}

fn build_debug(tier: &CustomTier, func: &ItemFn, ok_ty: Type, err_ty: Type) -> TokenStream {
    let mut out = func.clone();
    let k = &tier.krate;
    out.sig.output = parse_quote! { -> #k::Outcome<#ok_ty, #err_ty> };
    let mut rewriter = OutcomeRewriter {
        krate: tier.krate.clone(),
    };
    rewriter.visit_block_mut(&mut out.block);

    let inline = inline_attr(tier);
    let attrs = &out.attrs;
    let vis = &out.vis;
    let sig = &out.sig;
    let block = &out.block;
    quote! {
        #inline
        #(#attrs)*
        #vis #sig #block
    }
}

fn build_release(tier: &CustomTier, func: &mut ItemFn, ok_ty: Type) -> TokenStream {
    let mut out = func.clone();
    let k = &tier.krate;
    out.sig.output = parse_quote! { -> #k::Just<#ok_ty> };

    let mut rewriter = HotRewriter::new(tier.panic_fmt.clone(), tier.krate.clone());
    rewriter.visit_block_mut(&mut out.block);

    let inline = inline_attr(tier);
    let attrs = &out.attrs;
    let vis = &out.vis;
    let sig = &out.sig;
    let block = &out.block;
    quote! {
        #inline
        #(#attrs)*
        #vis #sig #block
    }
}

fn inline_attr(tier: &CustomTier) -> TokenStream {
    if tier.inline {
        quote! { #[inline] }
    } else {
        TokenStream::new()
    }
}

/// Visitor that rewrites:
/// - `Ok(x)` → `<krate>::Just::new(x)`
/// - `Err(e)` → `panic!(<panic_fmt>, err = e)` (default fmt uses `{err:?}`)
/// - `match scrut { Ok(x) => body, Err(_) => _ }` → `{ let x = scrut.unwrap(); body }`
pub struct HotRewriter {
    panic_fmt: String,
    krate: Path,
}

impl HotRewriter {
    pub fn new(panic_fmt: Option<String>, krate: Path) -> Self {
        Self {
            panic_fmt: panic_fmt
                .unwrap_or_else(|| "hot path invariant violated: {err:?}".to_string()),
            krate,
        }
    }
}

impl VisitMut for HotRewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if matches!(expr, Expr::Closure(_)) {
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expr);

        match expr {
            Expr::Call(call) => {
                if is_ok_call(call) {
                    let val = call.args.first().unwrap().clone();
                    let k = &self.krate;
                    *expr = parse_quote! { #k::Just::new(#val) };
                } else if is_err_call(call) {
                    let val = call.args.first().unwrap().clone();
                    let panic_expr = build_panic_expr(&self.panic_fmt, val);
                    *expr = panic_expr;
                }
            }
            Expr::Match(m) => {
                if let Some(rewritten) = rewrite_match(m) {
                    *expr = rewritten;
                }
            }
            _ => {}
        }
    }

    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, ret);
        if let Some(inner) = &mut ret.expr {
            let replacement = match inner.as_ref() {
                Expr::Call(call) if is_ok_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    let k = &self.krate;
                    Some(parse_quote! { #k::Just::new(#val) })
                }
                Expr::Call(call) if is_err_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(build_panic_expr(&self.panic_fmt, val))
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

    fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {
        // Do not descend into nested fn items.
    }
}

fn build_panic_expr(fmt: &str, err_val: Expr) -> Expr {
    // The fmt string contains `{err:?}` (or caller-customised placeholders).
    // We pass `err = <val>` so any `{err...}` placeholder captures.
    parse_quote! {
        ::core::panic!(#fmt, err = #err_val)
    }
}

/// Rewrite `match scrut { Ok(x) => ok, Err(_) => err }` to
/// `{ let x = (scrut).unwrap(); ok }`.
///
/// `None` for anything that is not exactly that shape, and the strictness is
/// the point rather than caution. This runs only in the release arm, so every
/// shape it handles differently from the debug arm is a program whose meaning
/// changes when it ships, with no diagnostic on either side.
///
/// So all of the following are declined rather than approximated: a guard on
/// either arm, since a guard decides which arm runs and dropping one keeps the
/// arm and loses the condition; any arm count other than two, since keeping one
/// and discarding the rest is not a partial rewrite but a different program; a
/// refutable pattern inside `Ok`, since `let 0 = ...` does not compile; a
/// qualified path, since `Status::Ok` is somebody else's variant that happens
/// to share a spelling.
///
/// Either arm order is accepted. `Err` first is unusual and means the same
/// thing.
fn rewrite_match(m: &ExprMatch) -> Option<Expr> {
    if m.arms.len() != 2 {
        return None;
    }
    if m.arms.iter().any(|a| a.guard.is_some()) {
        return None;
    }

    let mut ok_arm = None;
    let mut saw_err = false;
    for arm in &m.arms {
        let Pat::TupleStruct(ts) = &arm.pat else {
            return None;
        };
        if ts.qself.is_some() || ts.elems.len() != 1 {
            return None;
        }
        if path_is_bare(&ts.path, "Ok") {
            // The binding lands in a `let`, so it has to be irrefutable.
            match ts.elems.first() {
                Some(Pat::Ident(_) | Pat::Wild(_)) => {}
                _ => return None,
            }
            if ok_arm
                .replace((ts.elems.first().unwrap().clone(), arm.body.clone()))
                .is_some()
            {
                return None;
            }
        } else if path_is_bare(&ts.path, "Err") {
            if saw_err {
                return None;
            }
            saw_err = true;
        } else {
            return None;
        }
    }

    let (binding, body) = ok_arm?;
    if !saw_err {
        return None;
    }
    let scrutinee = &m.expr;
    Some(parse_quote! {
        {
            let #binding = (#scrutinee).unwrap();
            #body
        }
    })
}
