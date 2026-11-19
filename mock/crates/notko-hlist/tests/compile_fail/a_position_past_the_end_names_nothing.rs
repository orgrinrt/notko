// A position past the last cell names no member. Without this the positive
// cases pass equally well for an `At` whose walk falls off the end and answers
// something, and the whole reason for indexing through the type system is that
// running off the end is a build failure rather than a value.
//
// The error names `Empty`, which is what the walk reached, rather than the
// position, which is what was written. That is the honest way round: the
// position is legal and the list is what could not answer it.

use notko_hlist::{At, Cons, Empty, Here, There};

struct A;
struct B;

type Two = Cons<A, Cons<B, Empty>>;

fn member_of<L: At<P>, P>() {}

fn main() {
    member_of::<Two, There<There<Here>>>();
}
