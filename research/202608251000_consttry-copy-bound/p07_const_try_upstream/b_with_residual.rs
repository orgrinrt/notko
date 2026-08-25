// PROBE p07-a: can notko's types impl core::ops::Try CONSTLY, and does `?`
// then work in a const fn? src/consttry.rs:8 says Try is not a const trait and
// :13 says `?` "stays non-const" as a consequence. Both are claims about the
// pinned toolchain and both are testable.
#![feature(try_trait_v2, try_trait_v2_residual, const_try, const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::{ControlFlow, FromResidual, Residual, Try};
use core::convert::Infallible;
use core::marker::Destruct;

pub enum Maybe<T> { Is(T), Isnt }

const impl<T: [const] Destruct> Try for Maybe<T> {
    type Output = T;
    type Residual = Maybe<Infallible>;
    fn from_output(o: Self::Output) -> Self { Maybe::Is(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self { Maybe::Is(v) => ControlFlow::Continue(v), Maybe::Isnt => ControlFlow::Break(Maybe::Isnt) }
    }
}
const impl<T: [const] Destruct> FromResidual<Maybe<Infallible>> for Maybe<T> {
    fn from_residual(r: Maybe<Infallible>) -> Self {
        match r { Maybe::Isnt => Maybe::Isnt, Maybe::Is(n) => match n {} }
    }
}

const impl<T> Residual<T> for Maybe<Infallible> {
    type TryType = Maybe<T>;
}

pub struct NotCopy(pub u32);

// THE TEST: `?` inside a const fn, on a non-Copy payload.
pub const fn uses_question_mark(m: Maybe<NotCopy>) -> Maybe<u32> {
    let v = m?;
    Maybe::Is(v.0)
}

const _PROOF: () = {
    match uses_question_mark(Maybe::Is(NotCopy(9))) {
        Maybe::Is(v) => assert!(v == 9),
        Maybe::Isnt => panic!("should be Is"),
    }
};
