#![feature(const_trait_impl)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::mem::ManuallyDrop;
pub enum Maybe<T> { Is(T), Isnt }
union Xfer<T> { whole: ManuallyDrop<Maybe<T>>, part: ManuallyDrop<T> }
pub const trait ConstTry { type Output; type Residual;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T> ConstTry for Maybe<T> {
    type Output = T; type Residual = Maybe<core::convert::Infallible>;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Maybe::Is(v) => ControlFlow::Continue(v),
            Maybe::Isnt => ControlFlow::Break(Maybe::Isnt),
        }
    }
}
