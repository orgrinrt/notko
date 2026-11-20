// A member the list holds twice has two places, and a bound leaving the place
// to inference cannot pick one. That ambiguity is the refusal a consumer gets
// for naming one member twice where it meant one place.

use notko_hlist::{Cons, Empty, PlaceOf, Place};

struct A;
struct B;

type Two = Cons<A, Cons<A, Empty>>;

fn place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    place::<Two, A, _>();
    let _ = B;
}
