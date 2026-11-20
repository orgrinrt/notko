#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible;
pub struct Just<T>(pub T);
pub const trait ConstTry { type Output; type Residual;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T: Copy> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
impl<T> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
