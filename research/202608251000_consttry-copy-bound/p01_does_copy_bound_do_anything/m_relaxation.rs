// PROBE p01-m: THE DECISIVE ONE.
// A `[const]` bound is meant to be *conditionally* const: it must hold
// "constly" only when the callee is invoked in a const context. If that is
// true, then ONE const impl carrying `T: [const] Destruct` serves BOTH
// audiences: const callers get every T without a destructor, and RUNTIME
// callers get every T at all, including one with a non-const Drop.
// If this compiles, the feature flag has nothing left to switch between
// and the narrowing does not exist.
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
pub struct NotCopy(pub u32);

// (1) RUNTIME caller, type with a real non-const destructor.
pub fn runtime_hasdrop() -> u32 {
    match Just(HasDrop(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
// (2) RUNTIME caller, non-Copy non-Drop.
pub fn runtime_notcopy() -> u32 {
    match Just(NotCopy(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
// (3) CONST caller, non-Copy non-Drop. Must work.
pub const fn const_notcopy() -> u32 {
    match Just(NotCopy(7)).branch() { ControlFlow::Continue(v) => v.0, ControlFlow::Break(_) => 0 }
}
// (4) RUNTIME caller, fully generic with NO bound at all.
pub fn runtime_generic<T>(j: Just<T>) -> ControlFlow<Infallible, T> { j.branch() }
