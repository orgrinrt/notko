// PROBE p01-b: THE QUESTION. Same thing with the `T: Copy` bound deleted.
// The source comment (src/consttry.rs:27-28) claims this cannot work because
// "const fn cannot evaluate destructors for arbitrary generic T".
// But neither method drops a T: `branch` moves the payload out of a newtype
// with no other fields, `from_output` moves it in. If nothing is dropped,
// the bound guards nothing and this compiles.
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

const impl<T> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;
    fn from_output(output: Self::Output) -> Self { Just(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}
