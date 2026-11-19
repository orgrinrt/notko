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
