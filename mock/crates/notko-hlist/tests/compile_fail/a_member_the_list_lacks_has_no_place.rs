// A member nowhere in the list has no place in it. Without this the positive
// cases pass equally well for a `PlaceOf` answering `Here` for everything.

use notko_hlist::{Cons, Empty, PlaceOf, Place};

struct A;
struct B;
struct Absent;

type Two = Cons<A, Cons<B, Empty>>;

fn place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    place::<Two, Absent, _>();
}
