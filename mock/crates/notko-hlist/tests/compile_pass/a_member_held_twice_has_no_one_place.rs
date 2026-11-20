// The twin: the same list with its second member another type, so `A` has
// one place.

use notko_hlist::{Cons, Empty, PlaceOf, Place};

struct A;
struct B;

type Two = Cons<A, Cons<B, Empty>>;

fn place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    place::<Two, A, _>();
    let _ = B;
}
