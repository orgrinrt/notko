//! Shared AST utilities used by both hot and cold rewriters.

use syn::{Expr, ExprCall, ExprMacro, GenericArgument, PathArguments, ReturnType, StmtMacro, Type};

/// Return `true` if `call` is `Ok(x)` with exactly one argument.
pub fn is_ok_call(call: &ExprCall) -> bool {
    last_ident_is(&call.func, "Ok") && call.args.len() == 1
}

/// Return `true` if `call` is `Err(x)` with exactly one argument.
pub fn is_err_call(call: &ExprCall) -> bool {
    last_ident_is(&call.func, "Err") && call.args.len() == 1
}

fn last_ident_is(func: &Expr, name: &str) -> bool {
    if let Expr::Path(p) = func {
        if let Some(seg) = p.path.segments.last() {
            return seg.ident == name;
        }
    }
    false
}

/// Extract `T` and `E` from `Result<T, E>`. Returns `(Some(T), Some(E))` if
/// the return type is a recognised `Result<T, E>`. Returns
/// `(Some(whole), None)` for any other concrete return type. Returns
/// `(None, None)` for unit (`-> ()`).
pub fn extract_result_inner_types(ret: &ReturnType) -> (Option<Type>, Option<Type>) {
    match ret {
        ReturnType::Default => (None, None),
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(last_seg) = type_path.path.segments.last() {
                    if last_seg.ident == "Result" || last_seg.ident == "Outcome" {
                        if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                            let mut iter = args.args.iter().filter_map(|a| match a {
                                GenericArgument::Type(t) => Some(t.clone()),
                                _ => None,
                            });
                            let t = iter.next();
                            let e = iter.next();
                            return (t, e);
                        }
                    }
                }
            }
            (Some((**ty).clone()), None)
        },
    }
}

pub fn macro_last_ident_is(mac: &ExprMacro, name: &str) -> bool {
    mac.mac
        .path
        .segments
        .last()
        .map(|seg| seg.ident == name)
        .unwrap_or(false)
}

pub fn stmt_macro_last_ident_is(mac: &StmtMacro, name: &str) -> bool {
    mac.mac
        .path
        .segments
        .last()
        .map(|seg| seg.ident == name)
        .unwrap_or(false)
}
