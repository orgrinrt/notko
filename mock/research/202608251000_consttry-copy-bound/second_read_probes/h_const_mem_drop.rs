// core ships `pub const fn drop<T>(_x: T) where T: [const] Destruct`
// (core/src/mem/mod.rs:998, rustc_const_unstable(const_destruct)).
// Q: is that const-callable from a downstream crate on this pin?
#![feature(const_trait_impl, const_destruct)]
struct HasDrop;
impl const Drop for HasDrop { fn drop(&mut self) {} }
const fn go() { core::mem::drop(HasDrop); }
const _: () = go();
fn main() {}
