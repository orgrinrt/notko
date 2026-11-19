// PROBE p05-b: "ConstCopyish" as a const trait whose supertrait is the real
// engine. This is op's shape with the bound that actually does the work
// "filled in", which is what he asked for.
#![feature(const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow; use core::convert::Infallible; use core::marker::Destruct;
pub struct Just<T>(pub T);

/// Typestate marker: "this type has nothing that must run when it is destroyed,
/// so a const evaluator can let it go out of scope."
pub const trait ConstCopyish: [const] Destruct {}
const impl<T: [const] Destruct> ConstCopyish for T {}

pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>; }
const impl<T: [const] ConstCopyish> ConstTry for Just<T> {
    type Output = T; type Residual = Infallible;
    fn from_output(o: Self::Output) -> Self { Just(o) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> { ControlFlow::Continue(self.0) }
}
pub struct NotCopy(pub u32);
pub const fn use_notcopy() -> u32 {
    match Just(NotCopy(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
pub struct HasDrop(pub u32);
impl Drop for HasDrop { fn drop(&mut self) {} }
pub fn runtime_hasdrop() -> u32 {
    match Just(HasDrop(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
