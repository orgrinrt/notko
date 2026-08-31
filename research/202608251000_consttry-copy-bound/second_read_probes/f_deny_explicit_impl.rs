// Soundness-relevant: core marks Destruct `#[rustc_deny_explicit_impl]`, so a user
// must not be able to assert it for a type that is not in fact droppable.
// NEGATIVE CONTROL: this MUST be refused. If it compiles, the marker is forgeable.
#![feature(const_trait_impl, const_destruct)]
use core::marker::Destruct;
struct Mine;
impl Destruct for Mine {}
fn main() {}
