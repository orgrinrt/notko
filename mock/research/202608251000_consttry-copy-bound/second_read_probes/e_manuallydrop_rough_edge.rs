// The one rough edge the tracking thread actually reports: gamozolabs 2025-04-29
// says a `ManuallyDrop` wrapper with a `const Drop` impl is wrongly refused,
// oli-obk 2025-04-29 agrees it should work and says a fix branch exists.
// Q: is that gap still live on the pinned nightly? Snippet is theirs, verbatim.
#![feature(const_destruct)]
#![feature(const_trait_impl)]
use std::mem::ManuallyDrop;
struct Moose;
impl Drop for Moose { fn drop(&mut self) {} }
struct ConstDropper<T>(ManuallyDrop<T>);
impl<T> const Drop for ConstDropper<T> { fn drop(&mut self) {} }
const fn foo(_var: ConstDropper<Moose>) {}
fn main() {}
