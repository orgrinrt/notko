// `ContainsAll` walks the list it is given, so that one has to be a list. The
// base impl is a blanket over `Self`, which makes it easy to read the trait as
// unconstrained on both sides; it is not, and this is the side that is
// constrained.
//
// It is also the only case reaching `ContainsAll`'s own diagnostic. Where a
// member is merely missing the error resolves through `Contains` instead, so
// without this fixture that note's wording is never checked.

use notko_hlist::{Cons, ContainsAll, Empty};

struct A;
struct B;

type Two = Cons<A, Cons<B, Empty>>;

fn holds_all<L: ContainsAll<M>, M>() {}

fn main() {
    holds_all::<Two, u8>();
}
