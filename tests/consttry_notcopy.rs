//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-context proofs for `ConstTry` / `ConstFromResidual` on payloads that
//! are **not** `Copy`.
//!
//! `consttry_smoke.rs` proves the const path only at `u32`. Every type it uses
//! is `Copy`, so the suite cannot see whether the const path accepts anything
//! else, and for a long time it did not. These proofs are that missing column:
//! a non-Copy payload, a nested one, and `Outcome` with a non-Copy error, each
//! evaluated at compile time.
//!
//! A const proof cannot be configuration-neutral, because the plain path is not
//! const-callable at all. Runtime reach is what must match across the feature,
//! and `consttry_parity.rs` asserts that half.

#![cfg_attr(feature = "const", feature(const_trait_impl))]
// Writing an `impl const From` in this target needs the gate in THIS crate;
// a consumer doing the same conversion pays the same cost.
#![cfg_attr(feature = "const", feature(const_convert))]

use core::convert::Infallible;
use core::ops::ControlFlow;
use notko::{ConstFromResidual, ConstTry, Just, Maybe, Outcome};

/// Not `Copy`, and no destructor. The type the `const` feature used to refuse.
#[derive(PartialEq, Eq, Debug)]
pub struct NotCopy(pub u32);

/// Not `Copy`, wrapping another non-Copy field. No `Drop` impl anywhere.
#[derive(PartialEq, Eq, Debug)]
pub struct NotCopyNested(pub NotCopy);

// ---------------------------------------------------------------- const proofs

const _JUST_NOTCOPY_BRANCH: () = {
    match <Just<NotCopy> as ConstTry>::branch(Just::new(NotCopy(42))) {
        ControlFlow::Continue(v) => assert!(v.0 == 42),
        ControlFlow::Break(_) => panic!("Just::branch broke on a non-Copy payload"),
    }
};

const _JUST_NOTCOPY_ROUND_TRIP: () = {
    let j = <Just<NotCopy> as ConstTry>::from_output(NotCopy(7));
    match <Just<NotCopy> as ConstTry>::branch(j) {
        ControlFlow::Continue(v) => assert!(v.0 == 7),
        ControlFlow::Break(_) => panic!("round trip broke"),
    }
};

const _MAYBE_NOTCOPY_IS: () = {
    match <Maybe<NotCopy> as ConstTry>::branch(Maybe::Is(NotCopy(13))) {
        ControlFlow::Continue(v) => assert!(v.0 == 13),
        ControlFlow::Break(_) => panic!("Maybe::Is should Continue"),
    }
};

const _MAYBE_NOTCOPY_ISNT: () = {
    match <Maybe<NotCopy> as ConstTry>::branch(Maybe::Isnt) {
        ControlFlow::Continue(_) => panic!("Maybe::Isnt should Break"),
        ControlFlow::Break(residual) => match residual {
            Maybe::Isnt => {}
            Maybe::Is(_) => panic!("residual should be Isnt"),
        },
    }
};

const _NESTED_NOTCOPY: () = {
    match <Just<NotCopyNested> as ConstTry>::branch(Just::new(NotCopyNested(NotCopy(5)))) {
        ControlFlow::Continue(v) => assert!(v.0.0 == 5),
        ControlFlow::Break(_) => panic!("nested non-Copy broke"),
    }
};

// Outcome with a non-Copy payload AND a non-Copy error. Both type parameters
// carried a `Copy` bound in the const path, so both are exercised.
const _OUTCOME_NOTCOPY_OK: () = {
    match <Outcome<NotCopy, NotCopy> as ConstTry>::branch(Outcome::Ok(NotCopy(101))) {
        ControlFlow::Continue(v) => assert!(v.0 == 101),
        ControlFlow::Break(_) => panic!("Outcome::Ok should Continue"),
    }
};

const _OUTCOME_NOTCOPY_ERR: () = {
    match <Outcome<NotCopy, NotCopy> as ConstTry>::branch(Outcome::Err(NotCopy(7))) {
        ControlFlow::Continue(_) => panic!("Outcome::Err should Break"),
        ControlFlow::Break(residual) => match residual {
            Outcome::Err(e) => assert!(e.0 == 7),
            Outcome::Ok(_) => panic!("residual should be Err"),
        },
    }
};

// `Outcome`'s ConstFromResidual has no const proof anywhere in the suite today,
// which is why the dropped `From` conversion went unnoticed. This is that proof,
// in the reflexive `E == F` shape the shipped const impl supports.
const _OUTCOME_FROM_RESIDUAL_REFLEXIVE: () = {
    let residual: Outcome<Infallible, NotCopy> = Outcome::Err(NotCopy(9));
    let o: Outcome<u32, NotCopy> = <Outcome<u32, NotCopy> as ConstFromResidual<
        Outcome<Infallible, NotCopy>,
    >>::from_residual(residual);
    match <Outcome<u32, NotCopy> as ConstTry>::branch(o) {
        ControlFlow::Continue(_) => panic!("from_residual(Err) should Break"),
        ControlFlow::Break(residual) => match residual {
            Outcome::Err(e) => assert!(e.0 == 9),
            Outcome::Ok(_) => panic!("residual should be Err"),
        },
    }
};

const _JUST_FROM_RESIDUAL_NOTCOPY: () = {
    // `Infallible` is uninhabited, so this only has to type-check to mean
    // something: it asserts the impl is reachable for a non-Copy `T`.
    let _ = |r: Infallible| -> Just<NotCopy> {
        <Just<NotCopy> as ConstFromResidual<Infallible>>::from_residual(r)
    };
};

// ------------------------------------------------------- cross-error conversion

/// The `E -> F` conversion the const path dropped. `outcome_consttry_const.rs`
/// documented its absence as a known divergence from the plain path; nothing in
/// the suite asserted it in either configuration, which is why the divergence
/// could sit there. These are that assertion.
#[derive(PartialEq, Eq, Debug)]
pub struct ErrLow(pub u32);
#[derive(PartialEq, Eq, Debug)]
pub struct ErrHigh(pub u32);

const impl From<ErrLow> for ErrHigh {
    fn from(e: ErrLow) -> Self {
        ErrHigh(e.0)
    }
}

const _OUTCOME_FROM_RESIDUAL_CONVERTS: () = {
    let residual: Outcome<Infallible, ErrLow> = Outcome::Err(ErrLow(4));
    let o: Outcome<u32, ErrHigh> = <Outcome<u32, ErrHigh> as ConstFromResidual<
        Outcome<Infallible, ErrLow>,
    >>::from_residual(residual);
    match <Outcome<u32, ErrHigh> as ConstTry>::branch(o) {
        ControlFlow::Continue(_) => panic!("Err residual should Break"),
        ControlFlow::Break(residual) => match residual {
            Outcome::Err(e) => assert!(e.0 == 4),
            Outcome::Ok(_) => panic!("residual should be Err"),
        },
    }
};

// A const target whose assertions are all `const _` blocks reports zero tests,
// which reads like a skipped target rather than a passing one. This mirrors one
// proof at runtime so the count is non-zero and the target is visibly alive.
#[test]
fn notcopy_const_proofs_compiled() {
    match <Just<NotCopy> as ConstTry>::branch(Just::new(NotCopy(42))) {
        ControlFlow::Continue(v) => assert_eq!(v, NotCopy(42)),
        ControlFlow::Break(_) => panic!("Just::branch broke"),
    }
}
