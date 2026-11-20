// PROBE p06-g: dodge the const drop check with ManuallyDrop instead of a
// Destruct bound. If this works, the fix needs no feature beyond
// const_trait_impl, which matters because const_destruct is unvetted.
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
        // park `self` where no destructor is owed, then read the payload out
        let md = ManuallyDrop::new(self);
        let inner = unsafe { core::ptr::read(&md.0) };
        ControlFlow::Continue(inner)
    }
}
pub struct NotCopy(pub u32);
pub const fn use_notcopy() -> u32 {
    match Just(NotCopy(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
