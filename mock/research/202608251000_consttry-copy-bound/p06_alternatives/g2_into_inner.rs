#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible; use core::mem::ManuallyDrop;
pub struct Just<T>(pub T);
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn from_output(o: Self::Output) -> Self { Just(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        let md: ManuallyDrop<Just<T>> = ManuallyDrop::new(self);
        let j: Just<T> = ManuallyDrop::into_inner(md);
        ControlFlow::Continue(j.0)
    }
}
