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

/// Whether an expression names `Result`'s constructor `name`, as far as a
/// proc macro can tell without resolving anything.
///
/// Two accepted shapes, and the second is what keeps this in step with
/// [`result_inner_types`]:
///
/// - `Ok(x)`, one segment, with or without a turbofish. `Ok::<u32, E>(x)` is
///   the same constructor with its parameters spelled out.
/// - `Result::Ok(x)` and any qualification of it, such as
///   `core::result::Result::Ok(x)`, plus the same for `Outcome`.
///
/// What is declined is a last segment that merely shares the spelling:
/// `Status::Ok`, `http::StatusCode::Ok`, and every other type whose variant is
/// called `Ok`. Comparing the last segment alone answers yes for all of them,
/// and the consumer then gets an error naming a path they never typed inside a
/// function that mentions none of ours.
///
/// The owner segment is the whole of the check and it is not a detail. It is
/// also the reason the two ends have to agree: [`result_inner_types`] reads a
/// return type by its last segment, so a signature written
/// `core::result::Result<u32, E>` is lifted. A body check that declined
/// `Result::Ok` then left the function returning one type and constructing
/// another, which is a compile error in the consumer's crate rather than the
/// quiet no-op declining is supposed to be.
pub fn is_bare_ctor(func: &Expr, name: &str) -> bool {
    let Expr::Path(p) = func else { return false };
    p.qself.is_none() && names_result_ctor(&p.path, name)
}

/// Whether a path names `Result`'s or `Outcome`'s constructor `name`.
pub fn names_result_ctor(path: &Path, name: &str) -> bool {
    let segments = &path.segments;
    let Some(last) = segments.last() else {
        return false;
    };
    if last.ident != name {
        return false;
    }
    match segments.len() {
        // `Ok(x)`, or `Ok::<T, E>(x)`. A leading `::` would make it a path
        // into a crate root, where a bare constructor never lives.
        1 => path.leading_colon.is_none(),
        // `Result::Ok(x)`, however far it is qualified. The owner is what
        // distinguishes it from `Status::Ok`.
        _ => {
            let owner = &segments[segments.len() - 2].ident;
            owner == "Result" || owner == "Outcome"
        }
    }
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
