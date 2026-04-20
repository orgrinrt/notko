//! Cold / Outcome-based rewrite. Always emits `Outcome<T, E>` regardless of
//! build profile.

use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{parse_quote, Expr, ExprMacro, ExprReturn, ItemFn, Result, Stmt, StmtMacro, Type};

use super::helpers::{
    extract_result_inner_types, is_err_call, is_ok_call, macro_last_ident_is,
    stmt_macro_last_ident_is,
};
use crate::tiers::CustomTier;

pub fn rewrite(tier: CustomTier, mut func: ItemFn) -> Result<TokenStream> {
    let (ok_ty, err_ty) = extract_result_inner_types(&func.sig.output);
    if let (Some(t), Some(e)) = (ok_ty, err_ty) {
        set_outcome_return(&mut func, t, e);
    }

    let mut rewriter = OutcomeRewriter { rewrite_diagnose: true };
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

fn set_outcome_return(func: &mut ItemFn, t: Type, e: Type) {
    func.sig.output = parse_quote! { -> ::notko::Outcome<#t, #e> };
}

/// Visitor that rewrites:
/// - `Ok(x)` → `::notko::Outcome::Ok(x)`
/// - `Err(e)` → `::notko::Outcome::Err(e)`
/// - Optionally (when `rewrite_diagnose` is true) keeps `diagnose!(...)`
///   calls verbatim so cold diagnostic sinks stay active in release.
pub struct OutcomeRewriter {
    pub rewrite_diagnose: bool,
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
                *expr = parse_quote! { ::notko::Outcome::Ok(#val) };
                return;
            }
            if is_err_call(call) {
                let val = call.args.first().unwrap().clone();
                *expr = parse_quote! { ::notko::Outcome::Err(#val) };
                return;
            }
        }

        if self.rewrite_diagnose {
            if let Expr::Macro(mac) = expr {
                if macro_last_ident_is(mac, "diagnose") {
                    *expr = diagnose_always_expr(mac);
                }
            }
        }
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        syn::visit_mut::visit_stmt_mut(self, stmt);
        if self.rewrite_diagnose {
            if let Stmt::Macro(mac) = stmt {
                if stmt_macro_last_ident_is(mac, "diagnose") {
                    *stmt = diagnose_always_stmt(mac);
                }
            }
        }
    }

    fn visit_expr_return_mut(&mut self, ret: &mut ExprReturn) {
        syn::visit_mut::visit_expr_return_mut(self, ret);
        if let Some(inner) = &mut ret.expr {
            let replacement = match inner.as_ref() {
                Expr::Call(call) if is_ok_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(parse_quote! { ::notko::Outcome::Ok(#val) })
                },
                Expr::Call(call) if is_err_call(call) => {
                    let val = call.args.first().unwrap().clone();
                    Some(parse_quote! { ::notko::Outcome::Err(#val) })
                },
                _ => None,
            };
            if let Some(r) = replacement {
                *inner = Box::new(r);
            }
        }
    }

    fn visit_item_fn_mut(&mut self, _: &mut ItemFn) {}
}

/// Pass `diagnose!(...)` through to a hypothetical always-on sink dispatcher.
/// Notko itself does not ship the sink; this is a convention that downstream
/// diagnostics crates (e.g., a notko-diagnostics sibling) implement.
fn diagnose_always_expr(mac: &ExprMacro) -> Expr {
    let tokens = &mac.mac.tokens;
    parse_quote! {
        ::notko::__diagnose_cold__(#tokens)
    }
}

fn diagnose_always_stmt(mac: &StmtMacro) -> Stmt {
    let tokens = &mac.mac.tokens;
    parse_quote! {
        ::notko::__diagnose_cold__(#tokens);
    }
}
