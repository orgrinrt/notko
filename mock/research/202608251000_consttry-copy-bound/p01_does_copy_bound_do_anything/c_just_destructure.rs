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
const impl<T> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;
    fn from_output(output: Self::Output) -> Self { Just(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        let Just(inner) = self;            // full move, no residual shell
        ControlFlow::Continue(inner)
    }
}
