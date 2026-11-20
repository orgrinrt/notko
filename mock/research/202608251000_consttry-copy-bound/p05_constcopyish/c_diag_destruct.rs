#![feature(const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible; use core::marker::Destruct;
pub struct Just<T>(pub T);
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T: [const] Destruct> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn from_output(o: Self::Output) -> Self { Just(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
pub struct HasDrop(pub u32);
impl Drop for HasDrop { fn drop(&mut self) {} }
pub const fn boom() -> u32 {
    match Just(HasDrop(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
