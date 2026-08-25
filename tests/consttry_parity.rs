//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Runtime reach of `ConstTry` must not depend on the `const` feature.
//!
//! The `const` feature is meant to decide whether the traits are const traits,
//! not which types they accept. This target carries no `#[cfg]` and no feature
//! gate, so it compiles under both configurations or neither, and a bound that
//! one path adds and the other does not makes it fail to build in exactly one.
//! That build failure is the assertion; the runtime bodies below are secondary.
//!
//! Cargo unifies features across a dependency graph, so a consumer cannot rely
//! on `default-features = false` surviving contact with a sibling crate that
//! takes the defaults. Parity is what makes that unification harmless, which is
//! why it is asserted rather than documented.
//!
//! Run both:
//!   cargo test --test consttry_parity
//!   cargo test --test consttry_parity --no-default-features

use core::convert::Infallible;
use core::ops::ControlFlow;
use notko::{ConstTry, Just, Maybe, Outcome};

/// Not `Copy`, no destructor.
#[derive(PartialEq, Eq, Debug)]
pub struct NotCopy(pub u32);

#[test]
fn just_notcopy_branch_runtime() {
    match <Just<NotCopy> as ConstTry>::branch(Just::new(NotCopy(42))) {
        ControlFlow::Continue(v) => assert_eq!(v, NotCopy(42)),
        ControlFlow::Break(_) => panic!("Just::branch broke"),
    }
}

#[test]
fn outcome_notcopy_err_runtime() {
    match <Outcome<NotCopy, NotCopy> as ConstTry>::branch(Outcome::Err(NotCopy(3))) {
        ControlFlow::Continue(_) => panic!("Err should break"),
        ControlFlow::Break(Outcome::Err(e)) => assert_eq!(e, NotCopy(3)),
        ControlFlow::Break(Outcome::Ok(_)) => panic!("residual should be Err"),
    }
}

/// A type with a real destructor. It must work at RUNTIME through the same
/// impl in both configurations. Const use of it is refused, and that refusal
/// is asserted separately as a compile-fail case.
pub struct HasDrop(pub u32);
impl Drop for HasDrop {
    fn drop(&mut self) {}
}

#[test]
fn hasdrop_branch_runtime() {
    match <Just<HasDrop> as ConstTry>::branch(Just::new(HasDrop(11))) {
        ControlFlow::Continue(v) => assert_eq!(v.0, 11),
        ControlFlow::Break(_) => panic!("Just::branch broke on a Drop payload"),
    }
}

#[test]
fn generic_runtime_no_bound() {
    // A fully generic runtime caller writing no bound at all. This is what a
    // consumer's own helper looks like, and it must keep working.
    fn helper<T>(j: Just<T>) -> ControlFlow<Infallible, T> {
        <Just<T> as ConstTry>::branch(j)
    }
    match helper(Just::new(HasDrop(4))) {
        ControlFlow::Continue(v) => assert_eq!(v.0, 4),
        ControlFlow::Break(_) => unreachable!(),
    }
}

#[test]
fn maybe_notcopy_runtime() {
    match <Maybe<NotCopy> as ConstTry>::branch(Maybe::Is(NotCopy(8))) {
        ControlFlow::Continue(v) => assert_eq!(v, NotCopy(8)),
        ControlFlow::Break(_) => panic!("Maybe::Is should continue"),
    }
}

#[test]
fn outcome_from_output_notcopy_runtime() {
    let o = <Outcome<NotCopy, NotCopy> as ConstTry>::from_output(NotCopy(21));
    match <Outcome<NotCopy, NotCopy> as ConstTry>::branch(o) {
        ControlFlow::Continue(v) => assert_eq!(v, NotCopy(21)),
        ControlFlow::Break(_) => panic!("from_output should round-trip"),
    }
}
