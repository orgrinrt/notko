//! A catalogued gap, not a refusal this crate wants.
//!
//! `Just<T>` carries `FromResidual<JustResidual>` and nothing else, so a `?`
//! on an `Outcome` inside a function returning `Just` does not compile. That
//! is the most ordinary line anybody writes in a fallible function, and it is
//! where the hot strategy's two arms part: the debug arm returns `Outcome` and
//! takes the `?`, the release arm returns `Just` and does not.
//!
//! The intended state is that the ladder composes and this file builds. When
//! the missing `FromResidual` impls land, this fails with "expected to fail to
//! compile, but succeeded", and the right answer is to delete it.

use notko::{Just, Outcome};

struct Oops;

fn inner() -> Outcome<u32, Oops> {
    Outcome::Ok(1)
}

fn outer() -> Just<u32> {
    let n = inner()?;
    Just::new(n)
}

fn main() {
    let _ = outer();
}
