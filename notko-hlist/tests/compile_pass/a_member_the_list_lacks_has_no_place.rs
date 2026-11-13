// The twin: the same call, for a member the list holds.

use notko_hlist::{Cons, Empty, PlaceOf, Place};

struct A;
struct B;
struct Absent;

type Two = Cons<A, Cons<B, Empty>>;

fn place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    place::<Two, B, _>();
    let _ = Absent;
}
