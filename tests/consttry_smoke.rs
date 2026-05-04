//! Const-context smoke tests for `ConstTry` and `ConstFromResidual`.
//!
//! Each test exercises the trait methods inside a `const _: ... = { ... };`
//! block. Failure surfaces as a compile error during const evaluation,
//! catching regressions to non-const callability without a runtime
//! assertion.

#![cfg_attr(feature = "const", feature(const_trait_impl))]

use core::convert::Infallible;
use core::ops::ControlFlow;
use notko::{ConstFromResidual, ConstTry, Just, Maybe, Outcome};

#[cfg(feature = "const")]
const _JUST_BRANCH: () = {
    let j: Just<u32> = Just::new(42);
    match <Just<u32> as ConstTry>::branch(j) {
        ControlFlow::Continue(v) => assert!(v == 42),
        ControlFlow::Break(_) => panic!("Just::branch broke"),
    }
};

#[cfg(feature = "const")]
const _JUST_FROM_OUTPUT: () = {
    let j: Just<u32> = <Just<u32> as ConstTry>::from_output(7);
    match <Just<u32> as ConstTry>::branch(j) {
        ControlFlow::Continue(v) => assert!(v == 7),
        ControlFlow::Break(_) => panic!("Just::branch broke after from_output"),
    }
};

#[cfg(feature = "const")]
const _MAYBE_IS_BRANCH: () = {
    let m: Maybe<u32> = Maybe::Is(13);
    match <Maybe<u32> as ConstTry>::branch(m) {
        ControlFlow::Continue(v) => assert!(v == 13),
        ControlFlow::Break(_) => panic!("Maybe::Is branch should Continue"),
    }
};

#[cfg(feature = "const")]
const _MAYBE_ISNT_BRANCH: () = {
    let m: Maybe<u32> = Maybe::Isnt;
    match <Maybe<u32> as ConstTry>::branch(m) {
        ControlFlow::Continue(_) => panic!("Maybe::Isnt branch should Break"),
        ControlFlow::Break(residual) => match residual {
            Maybe::Isnt => {}
            Maybe::Is(_) => panic!("residual should be Isnt"),
        },
    }
};

#[cfg(feature = "const")]
const _OUTCOME_OK_BRANCH: () = {
    let o: Outcome<u32, u32> = Outcome::Ok(101);
    match <Outcome<u32, u32> as ConstTry>::branch(o) {
        ControlFlow::Continue(v) => assert!(v == 101),
        ControlFlow::Break(_) => panic!("Outcome::Ok should Continue"),
    }
};

#[cfg(feature = "const")]
const _OUTCOME_ERR_BRANCH: () = {
    let o: Outcome<u32, u32> = Outcome::Err(7);
    match <Outcome<u32, u32> as ConstTry>::branch(o) {
        ControlFlow::Continue(_) => panic!("Outcome::Err should Break"),
        ControlFlow::Break(residual) => match residual {
            Outcome::Err(e) => assert!(e == 7),
            Outcome::Ok(_) => panic!("residual should be Err"),
        },
    }
};

// Round-trip via FromResidual on Maybe.
#[cfg(feature = "const")]
const _MAYBE_FROM_RESIDUAL: () = {
    let residual: Maybe<Infallible> = Maybe::Isnt;
    let m: Maybe<u32> = <Maybe<u32> as ConstFromResidual<Maybe<Infallible>>>::from_residual(residual);
    match <Maybe<u32> as ConstTry>::branch(m) {
        ControlFlow::Continue(_) => panic!("from_residual(Isnt) should branch to Break"),
        ControlFlow::Break(_) => {}
    }
};

// Non-const variant smoke. Runs as runtime test on stable / no-default-features.
#[test]
fn maybe_branch_runtime() {
    let m: Maybe<u32> = Maybe::Is(99);
    match <Maybe<u32> as ConstTry>::branch(m) {
        ControlFlow::Continue(v) => assert_eq!(v, 99),
        ControlFlow::Break(_) => panic!("Maybe::Is should continue"),
    }
}

#[test]
fn outcome_branch_runtime() {
    let o: Outcome<u32, &'static str> = Outcome::Err("nope");
    match <Outcome<u32, &'static str> as ConstTry>::branch(o) {
        ControlFlow::Continue(_) => panic!("Outcome::Err should break"),
        ControlFlow::Break(residual) => match residual {
            Outcome::Err(e) => assert_eq!(e, "nope"),
            Outcome::Ok(_) => panic!("residual should be Err"),
        },
    }
}
