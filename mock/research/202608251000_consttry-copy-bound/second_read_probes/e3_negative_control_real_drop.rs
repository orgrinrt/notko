// NEGATIVE CONTROL for e2_. Same shape, but the field is a bare `Moose` rather than
// `ManuallyDrop<Moose>`, so real drop glue for a non-const `Drop` type is reachable.
// This MUST still be refused. If it compiles, e2_ proves nothing, because the
// compiler would be accepting every drop rather than reasoning about ManuallyDrop.
#![feature(const_destruct)]
#![feature(const_trait_impl)]
struct Moose;
impl Drop for Moose { fn drop(&mut self) {} }
struct RealDropper<T>(T);
impl<T> const Drop for RealDropper<T> { fn drop(&mut self) {} }
const fn foo(_var: RealDropper<Moose>) {}
const _: () = foo(RealDropper(Moose));
fn main() {}
