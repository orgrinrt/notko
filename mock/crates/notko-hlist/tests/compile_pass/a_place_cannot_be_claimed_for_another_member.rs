// The twin: the member `At` names at the front.

use notko_hlist::{Cons, Empty, Here, Place, PlaceOf};

struct A;
struct B;

type Two = Cons<A, Cons<B, Empty>>;

fn at_place<L: PlaceOf<M, P>, M, P: Place>() {}

fn main() {
    at_place::<Two, A, Here>();
}
