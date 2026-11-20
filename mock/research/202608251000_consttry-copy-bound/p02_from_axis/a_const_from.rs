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
    fn from_residual(residual: R) -> Self;
}
const impl<T, E, F: [const] From<E>> ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, F> {
    fn from_residual(r: Outcome<Infallible, E>) -> Self {
        match r { Outcome::Err(e) => Outcome::Err(F::from(e)), Outcome::Ok(n) => match n {} }
    }
}
