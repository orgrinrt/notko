// PROBE p01-j: does the [const] Destruct bound go viral into consumer code?
// The cost question. If every downstream generic const fn must repeat the
// bound, that is a real ergonomic tax and belongs in the accounting.
#![feature(const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible; 
pub struct Just<T>(pub T);
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T: Copy> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn from_output(o: Self::Output) -> Self { Just(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
// (a) CONCRETE consumer: no bound written anywhere. Does it just work?
pub struct NotCopy(pub u32);
pub const fn concrete() -> u32 {
    match Just(NotCopy(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
// (b) GENERIC consumer that repeats the bound. Expected to work.
pub const fn generic_with_bound<T: Copy>(j: Just<T>) -> ControlFlow<Infallible, T> {
    j.branch()
}
