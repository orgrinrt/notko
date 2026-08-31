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

use super::helpers::{is_err_call, is_ok_call, names_result_ctor, result_inner_types};
use super::outcome::OutcomeRewriter;
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    // One function emitted twice under opposite `cfg`s, so anything the two
    // arms would not do identically is a program that means one thing when it
    // is tested and another when it ships.
    //
    // Three of those are known and each is closed by something nameable, since
    // a paragraph claiming they are all closed is worth nothing on its own:
    //
    // - An unrecognised return type. The debug arm had nothing to lift it to
    //   and left it, the release arm wrapped it. Closed below, by emitting the
    //   function once and untouched.
    // - `?` in the body. The release arm narrows to `Just<T>`, which had no
    //   `FromResidual`, so the operator compiled in debug and not in release.
    //   Closed in `notko::just` by the impls that make it panic, which is what
    //   a written-out `Err` already does here.
    // - An error type the panic cannot format. Closed by
    //   `arms_agree_on_the_error`, which makes the debug arm demand of it
    //   exactly what the release arm's panic demands, for every error type
    //   except one written `impl Trait`. That exception is named there.
    //
    // All three are pinned by `notko-macros/tests/consumer_cfg.rs`, which
    // builds a real consumer crate in both arms and compares the two.
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
    if let Some(guard) = arms_agree_on_the_error(tier, &err_ty) {
        out.block.stmts.insert(0, guard);
    }

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

/// A statement that makes the debug arm demand of the error type exactly what
/// the release arm's panic demands of it.
///
/// The release arm rewrites `Err(e)` to `panic!(fmt, err = e)`, and the default
/// format reads the error with `{err:?}`. The debug arm never formats it, so an
/// error type without `Debug` compiled in the arm that is tested and did not
/// compile in the arm that ships. That is the divergence the pair exists to
/// prevent, landing on the consumer at the moment they build for release.
///
/// A closure rather than a nested function, because a nested item cannot name
/// the enclosing function's generic parameters and the error type routinely
/// does. It is never called, so it costs nothing but the type check, which is
/// the whole reason it is here.
///
/// It sees whatever the format string actually asks for, so a `panic_fmt`
/// written against `Display` demands that instead of `Debug`. One written
/// against nothing at all is a build failure in both arms rather than a
/// weaker demand, since `panic!("no placeholder", err = e)` is `named argument
/// never used`.
///
/// **What it does not cover**, stated rather than left to be discovered: an
/// error type written `impl Trait`. `impl Trait` is not allowed in a closure
/// parameter, so emitting this against one is a build failure in the debug arm
/// and nothing in the release arm, which is the divergence this exists to
/// close, pointing the other way. It shipped that way for one review round.
/// Such a type is skipped, and a `-> Result<T, impl Trait>` whose bounds do not
/// carry what the panic needs still diverges. Closing that means reading the
/// bounds, which is a different mechanism and is not built.
fn arms_agree_on_the_error(tier: &CustomTier, err_ty: &Type) -> Option<syn::Stmt> {
    if matches!(err_ty, Type::ImplTrait(_)) {
        return None;
    }
    let fmt = tier
        .panic_fmt
        .clone()
        .unwrap_or_else(|| "hot path invariant violated: {err:?}".to_string());
    Some(parse_quote! {
        #[allow(unused, unreachable_code, clippy::diverging_sub_expression)]
        let _arms_agree = |err: #err_ty| ::core::panic!(#fmt, err = err);
    })
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
    krate:     Path,
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

        // `return Err(e)` becomes the panic, rather than a `return` wrapping
        // one. Checked before descending, because descending rewrites the
        // inner call first and leaves `return ::core::panic!(..)`, which is an
        // unreachable expression: the warning then fires on every consumer's
        // release build, from inside a macro they cannot see into.
        if let Expr::Return(ret) = expr
            && let Some(inner) = &mut ret.expr
            && matches!(inner.as_ref(), Expr::Call(c) if is_err_call(c))
        {
            // The payload still gets visited, since it is an ordinary
            // expression and may hold anything.
            let Expr::Call(call) = inner.as_mut() else {
                unreachable!("the pattern above matched a call")
            };
            let mut val = call.args.first().unwrap().clone();
            self.visit_expr_mut(&mut val);
            *expr = build_panic_expr(&self.panic_fmt, val);
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
                    let k = &self.krate;
                    Some(parse_quote! { #k::Just::new(#val) })
                },
                // `Err` is not here. `return Err(e)` is replaced whole in
                // `visit_expr_mut`, before the descent that reaches this,
                // because a `return` around a diverging expression warns.
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
/// refutable pattern inside `Ok`, since `let 0 = ...` does not compile; and a
/// path whose owner is somebody else, since `Status::Ok` is a variant that
/// happens to share a spelling. `Result::Ok` and `Ok` are the same
/// constructor and both are accepted.
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
        if names_result_ctor(&ts.path, "Ok") {
            // The binding lands in a `let`, so it has to be irrefutable.
            match ts.elems.first() {
                Some(Pat::Ident(_) | Pat::Wild(_)) => {},
                _ => return None,
            }
            if ok_arm
                .replace((ts.elems.first().unwrap().clone(), arm.body.clone()))
                .is_some()
            {
                return None;
            }
        } else if names_result_ctor(&ts.path, "Err") {
            if saw_err {
                return None;
            }
            saw_err = true;
        } else {
            return None;
        }
    }

    // Two arms, each one of `Ok` or `Err`, and a second `Ok` or a second `Err`
    // already returned. So an `Ok` arm having been found means the other is
    // `Err`, and `saw_err` has nothing left to say.
    let (binding, body) = ok_arm?;
    debug_assert!(saw_err, "two arms, one Ok, and the other was not Err");
    let scrutinee = &m.expr;
    Some(parse_quote! {
        {
            let #binding = (#scrutinee).unwrap();
            #body
        }
    })
}
