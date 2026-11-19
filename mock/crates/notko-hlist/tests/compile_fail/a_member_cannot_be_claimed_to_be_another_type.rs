// The member at a position is the one that is there, and a bound saying
// otherwise is refused.
//
// This is the property a consumer indexing a heterogeneous run of values rests
// on: a slot typed as one thing and pointed at a position holding another does
// not compile. Without a case naming it, every positive assertion in
// `a_member_sits_at_a_position` would pass equally well for an `At` whose
// `Member` were an unconstrained parameter.

use notko_hlist::{At, Cons, Empty, Here, There};

struct A;
struct B;
struct C;

type Three = Cons<A, Cons<B, Cons<C, Empty>>>;

fn member_is<L: At<P, Member = M>, P, M>() {}

fn main() {
    // Position one is `B`.
    member_is::<Three, There<Here>, C>();
}
