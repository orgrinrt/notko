#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow;
use core::convert::Infallible;
pub struct Just<T>(pub T);
pub enum Maybe<T> { Is(T), Isnt }
pub const trait ConstTry {
    type Output;
    type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
const impl<T> ConstTry for Maybe<T> {
    type Output = T;
    type Residual = Maybe<Infallible>;
    fn from_output(output: Self::Output) -> Self { Maybe::Is(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Maybe::Is(value) => ControlFlow::Continue(value),
            Maybe::Isnt => ControlFlow::Break(Maybe::Isnt),
        }
    }
}
