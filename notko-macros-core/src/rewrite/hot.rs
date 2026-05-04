//! Hot-strategy rewrite.
//!
//! Emits two cfg-gated versions of the function:
//! - debug / `standalone` / `internal` feature off → `Outcome<T, E>` body.
//! - release + `internal` feature on → `Just<T>` body with Err → panic.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{parse_quote, Expr, ExprMatch, ExprReturn, ItemFn, Pat, Result, Type};

use super::helpers::{
    extract_result_inner_types, is_err_call, is_ok_call,
};
use super::outcome::OutcomeRewriter;
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    let (ok_ty, err_ty) = extract_result_inner_types(&func.sig.output);

    let debug_fn = build_debug(&tier, &func, ok_ty.clone(), err_ty.clone());
    let release_fn = build_release(&tier, &mut func, ok_ty, err_ty);

    Ok(quote! {
        #[cfg(any(not(feature = "internal"), debug_assertions))]
        #debug_fn

        #[cfg(all(feature = "internal", not(debug_assertions)))]
        #release_fn
    })
}

fn build_debug(
    tier: &CustomTier,
    func: &ItemFn,
    ok_ty: Option<Type>,
    err_ty: Option<Type>,
) -> TokenStream {
    let mut out = func.clone();
    if let Some(t) = ok_ty {
        if let Some(e) = err_ty {
            out.sig.output = parse_quote! { -> ::notko::Outcome<#t, #e> };
        }
    }
    let mut rewriter = OutcomeRewriter { rewrite_diagnose: false };
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

fn build_release(
    tier: &CustomTier,
    func: &mut ItemFn,
    ok_ty: Option<Type>,
    _err_ty: Option<Type>,
) -> TokenStream {
    let mut out = func.clone();
    if let Some(t) = ok_ty {
        out.sig.output = parse_quote! { -> ::notko::Just<#t> };
    }

    let mut rewriter = HotRewriter::new(tier.panic_fmt.clone());
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
/// - `Ok(x)` → `::notko::Just::new(x)`
/// - `Err(e)` → `panic!(<panic_fmt>, err = e)` (default fmt uses `{err:?}`)
/// - `match scrut { Ok(x) => body, Err(_) => _ }` → `{ let x = scrut.unwrap(); body }`
pub struct HotRewriter {
    panic_fmt: String,
}

impl HotRewriter {
    pub fn new(panic_fmt: Option<String>) -> Self {
        Self {
            panic_fmt: panic_fmt.unwrap_or_else(|| {
                "hot path invariant violated: {err:?}".to_string()
            }),
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
                    *expr = parse_quote! { ::notko::Just::new(#val) };
                } else if is_err_call(call) {
                    let val = call.args.first().unwrap().clone();
                    let panic_expr = build_panic_expr(&self.panic_fmt, val);
                    *expr = panic_expr;
                }
            },
            Expr::Match(m) => {
                if let Some(rewritten) = rewrite_match(m) {
                    *expr = rewritten;
                }
            },
            _ => {},
        }
    }

    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, ret);
        if let Some(inner) = &mut ret.expr {
            let replacement = match inner.as_ref() {
                Expr::Call(call) if is_ok_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(parse_quote! { ::notko::Just::new(#val) })
                },
                Expr::Call(call) if is_err_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(build_panic_expr(&self.panic_fmt, val))
                },
                _ => None,
            };
            if let Some(r) = replacement {
                *inner = Box::new(r);
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

/// Rewrite `match scrut { Ok(x) => body_ok, Err(_) => body_err }` to
/// `{ let x = (scrut).unwrap(); body_ok }`. Err arm is discarded; in hot
/// release, any Err reaching here is an invariant violation that `.unwrap()`
/// panics on.
fn rewrite_match(m: &ExprMatch) -> Option<Expr> {
    let mut ok_arm = None;
    for arm in &m.arms {
        if let Pat::TupleStruct(ts) = &arm.pat {
            if let Some(seg) = ts.path.segments.last() {
                if seg.ident == "Ok" && ts.elems.len() == 1 {
                    ok_arm = Some((ts.elems.first().unwrap().clone(), arm.body.clone()));
                }
            }
        }
    }
    let (binding, body) = ok_arm?;
    let scrutinee = &m.expr;
    Some(parse_quote! {
        {
            let #binding = (#scrutinee).unwrap();
            #body
        }
    })
}
