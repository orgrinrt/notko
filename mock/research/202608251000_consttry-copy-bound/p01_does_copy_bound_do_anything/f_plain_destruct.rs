#![feature(const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow;
use core::convert::Infallible;
use core::marker::Destruct;
pub struct Just<T>(pub T);
pub const trait ConstTry {
    type Output;
    type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
const impl<T: Destruct> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;
    fn from_output(output: Self::Output) -> Self { Just(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}
