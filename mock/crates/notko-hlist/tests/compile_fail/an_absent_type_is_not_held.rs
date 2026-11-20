// A type that is nowhere in the list is not held by it. Without this the
// positive membership cases pass equally well for a `Contains` implemented
// for everything, which is the shape `#[marker]` makes easy to write.

use notko_hlist::{Cons, Contains, Empty};

struct A;
struct B;
struct Absent;

type Two = Cons<A, Cons<B, Empty>>;

fn holds<L: Contains<X>, X>() {}

fn main() {
    holds::<Two, Absent>();
}
