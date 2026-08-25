// A non-Copy type pushed through ConstTry, exactly the case src/consttry.rs:29
// tells the reader to reach for `default-features = false` to serve.
use notko::{ConstTry, Just};
use core::ops::ControlFlow;
pub struct NotCopy(pub u32);
pub fn push_notcopy() -> u32 {
    match Just::new(NotCopy(5)).branch() {
        ControlFlow::Continue(v) => v.0,
        ControlFlow::Break(_) => 0,
    }
}
