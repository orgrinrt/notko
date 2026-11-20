// NEGATIVE CONTROL for a_. Naming `Destruct` with NO gate must be refused, and the
// refusal must cite the tracking issue. If this compiles, the feature is already
// stable and the gate is drift.
fn f<T: core::marker::Destruct>(_t: T) {}
fn main() {}
