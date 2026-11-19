// PROBE p05-a: op's "ConstCopyish" as a plain marker trait with a blanket impl.
// Question: does a user-defined marker carry enough for the const drop check?
#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible;
pub struct Just<T>(pub T);
pub trait ConstCopyish {}
impl<T: Copy> ConstCopyish for T {}
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T: ConstCopyish> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn from_output(o: Self::Output) -> Self { Just(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
