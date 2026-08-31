// Q: is `const_destruct` a live gate name on the pinned nightly, or has it been
// renamed/stabilised out from under us (the `const_convert` -> `const_from` shape)?
// PASS = compiles clean. E0635 would mean the name is dead on this pin.
#![feature(const_destruct)]
fn main() {}
