// Isolates necessity. notko already gates const_trait_impl (src/lib.rs:9). Does that
// alone let a generic const fn drop its T, making const_destruct redundant?
// Expected: no, still E0493. Together with d_ this shows const_destruct is the
// feature that carries the capability, not a convenience on top of const_trait_impl.
#![feature(const_trait_impl)]
const fn consume<T>(_t: T) {}
fn main() {}
