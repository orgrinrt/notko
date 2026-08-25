// e_ only DEFINED the const fn. Strengthened: actually construct and drop the value
// inside a forced const evaluation, so the check cannot pass by never running.
#![feature(const_destruct)]
#![feature(const_trait_impl)]
use std::mem::ManuallyDrop;
struct Moose;
impl Drop for Moose { fn drop(&mut self) {} }
struct ConstDropper<T>(ManuallyDrop<T>);
impl<T> const Drop for ConstDropper<T> { fn drop(&mut self) {} }
const fn foo(_var: ConstDropper<Moose>) {}
const _: () = foo(ConstDropper(ManuallyDrop::new(Moose)));
fn main() {}
