//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Shared AST utilities used by both hot and cold rewriters.

use syn::{Expr, ExprCall, GenericArgument, Path, PathArguments, ReturnType, Type};

/// Return `true` if `call` is `Ok(x)`, written bare, with one argument.
pub fn is_ok_call(call: &ExprCall) -> bool {
    is_bare_ctor(&call.func, "Ok") && call.args.len() == 1
}

/// Return `true` if `call` is `Err(x)`, written bare, with one argument.
pub fn is_err_call(call: &ExprCall) -> bool {
    is_bare_ctor(&call.func, "Err") && call.args.len() == 1
}

/// Whether an expression is the bare constructor `name`, one segment and no
/// qualification.
///
/// The single segment is the whole of the check and it is not a detail. A
/// proc macro cannot resolve names, so the only honest question it can ask is
/// what the author wrote. Comparing the last segment instead answers yes for
/// `Status::Ok`, `http::StatusCode::Ok` and every other type whose variant
/// happens to share a spelling with `Result`'s, and the consumer then gets an
/// error naming a path they never typed inside a function that mentions none
/// of ours.
///
/// Declining `Result::Ok` and `core::result::Result::Ok` is the cost. Both are
/// legal and both are rare, and declining is silent and harmless where getting
/// it wrong is neither.
pub fn is_bare_ctor(func: &Expr, name: &str) -> bool {
    let Expr::Path(p) = func else { return false };
    p.qself.is_none() && p.path.leading_colon.is_none() && path_is_bare(&p.path, name)
}

/// Whether a path is exactly `name`, one segment carrying no arguments.
pub fn path_is_bare(path: &Path, name: &str) -> bool {
    path.segments.len() == 1
        && path.segments[0].ident == name
        && path.segments[0].arguments.is_none()
}

/// `T` and `E` from a return type of `Result<T, E>` or `Outcome<T, E>`.
///
/// `None` for everything else, and everything else genuinely means everything
/// else: a bare type, a unit return, an alias carrying one argument, a path
/// this cannot resolve. All of them are shapes with no fallibility to lift, and
/// the caller's job on `None` is to emit the function untouched.
///
/// The alias case is the one worth naming, because it is ordinary rather than
/// exotic. A crate writing `type Result<T> = core::result::Result<T, Error>`
/// and returning `Result<u32>` reaches here as a `Result` path with one
/// argument, and no amount of looking at it will say what the second one is.
/// An earlier version answered `(Some(u32), None)`, which read as "the whole
/// return type" to one caller and as "the ok type" to another, and the hot
/// strategy took the second reading and rewrote a release signature its own
/// debug arm had left alone.
pub fn result_inner_types(ret: &ReturnType) -> Option<(Type, Type)> {
    let ReturnType::Type(_, ty) = ret else {
        return None;
    };
    let Type::Path(type_path) = ty.as_ref() else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }
    let last = type_path.path.segments.last()?;
    if last.ident != "Result" && last.ident != "Outcome" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|a| match a {
        GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    });
    let t = types.next()?;
    let e = types.next()?;
    // A third would mean this is not the two-parameter shape being read.
    if types.next().is_some() {
        return None;
    }
    Some((t, e))
}
