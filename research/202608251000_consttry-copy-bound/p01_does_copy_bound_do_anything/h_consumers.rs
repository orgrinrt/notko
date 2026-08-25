// PROBE p01-h: does [const] Destruct actually admit the types Copy rejects?
// Three consumers: non-Copy/non-Drop, Drop, const Drop.
#![feature(const_trait_impl, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::ControlFlow;
use core::convert::Infallible;
use core::marker::Destruct;
pub struct Just<T>(pub T);
pub const trait ConstTry {
    type Output; type Residual;
    fn from_output(output: Self::Output) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}
const impl<T: [const] Destruct> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;
    fn from_output(output: Self::Output) -> Self { Just(output) }
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}

// (1) non-Copy, non-Drop. This is the type the whole complaint is about.
pub struct NotCopy(pub u32);
pub const fn use_notcopy() -> u32 {
    match Just(NotCopy(7)).branch() {
        ControlFlow::Continue(v) => v.0,
        ControlFlow::Break(_) => 0,
    }
}
