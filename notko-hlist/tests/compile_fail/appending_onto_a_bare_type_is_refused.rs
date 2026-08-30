// `Concat` walks the left-hand side, so the left has to be a list. The right
// is not walked and may be anything, which the crate says out loud and the
// positive suite pins; this is the side that is genuinely constrained.

use notko_hlist::{Concat, Cons, Empty};

struct A;

type One = Cons<A, Empty>;

fn main() {
    let _: core::marker::PhantomData<<u8 as Concat<One>>::Out> = core::marker::PhantomData;
}
