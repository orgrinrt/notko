// PROBE p01-a: the shipped shape. `T: Copy` on the const impl.
// Question: does it compile? (control: establishes the baseline builds at all)
#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow;
use core::convert::Infallible;

pub struct Just<T>(pub T);

pub const trait ConstTry {
    type Output;
    type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

const impl<T: Copy> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;
    fn from_output(output: Self::Output) -> Self { Just(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}
