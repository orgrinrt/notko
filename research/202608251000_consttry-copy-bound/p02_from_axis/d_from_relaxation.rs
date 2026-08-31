// PROBE p02-d: does the [const] From impl serve runtime callers with a
// merely-runtime From, and does it still cover the reflexive E == F case
// that the shipped const impl handles today? A regression on either is fatal.
#![feature(const_trait_impl, const_destruct, const_convert)]
#![no_std]
#![crate_type = "lib"]
use core::convert::Infallible;
use core::marker::Destruct;
pub enum Outcome<T, E> { Ok(T), Err(E) }
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output>; }
pub const trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    fn from_residual(residual: R) -> Self; }

const impl<T, E, F: [const] From<E> + [const] Destruct> ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, F>
where E: [const] Destruct {
    fn from_residual(r: Outcome<Infallible, E>) -> Self {
        match r { Outcome::Err(e) => Outcome::Err(F::from(e)), Outcome::Ok(n) => match n {} }
    }
}

pub struct ErrA(pub u32);
pub struct ErrB(pub u32);
// a RUNTIME-only From. Deliberately NOT const.
impl From<ErrA> for ErrB { fn from(a: ErrA) -> ErrB { ErrB(a.0) } }

// (1) reflexive E == F, in CONST context. The case shipped today. Must survive.
pub const fn reflexive_const(r: Outcome<Infallible, ErrA>) -> Outcome<u8, ErrA> {
    <Outcome<u8, ErrA> as ConstFromResidual<Outcome<Infallible, ErrA>>>::from_residual(r)
}
// (2) cross-conversion E -> F at RUNTIME with a non-const From. The case the
//     plain impl has and the const impl currently drops.
pub fn cross_runtime(r: Outcome<Infallible, ErrA>) -> Outcome<u8, ErrB> {
    <Outcome<u8, ErrB> as ConstFromResidual<Outcome<Infallible, ErrA>>>::from_residual(r)
}
