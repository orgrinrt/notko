// `PlaceOf` carries `At` at the same place, so a place answers for the member `At`
// names there and no other. Asked for `B` at the front of a list whose front
// is `A`, it refuses rather than answering the place it was handed.

use notko_hlist::{Cons, Empty, PlaceOf, Here, Place};

struct A;
struct B;

type Two = Cons<A, Cons<B, Empty>>;

fn at_place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    at_place::<Two, B, Here>();
}
