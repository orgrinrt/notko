// Q: what does the feature actually buy? Same shape as c_, with the bound.
// Expected: compiles. c_ + d_ together isolate the capability to this feature.
#![feature(const_trait_impl, const_destruct)]
use core::marker::Destruct;
const fn consume<T: [const] Destruct>(_t: T) {}
struct HasDrop;
impl const Drop for HasDrop { fn drop(&mut self) {} }
const fn use_it() { consume(HasDrop); consume(5u8); }
fn main() { use_it(); }
