// `ContainsAll` is every member, not most of them. Two of the three named here
// are present, which is the case worth pinning: a recursion that stopped at the
// first hit, or that only checked the head of the subset, would accept this.

use notko_hlist::{Cons, ContainsAll, Empty};

struct A;
struct B;
struct C;
struct Absent;

type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
type Wanted = Cons<A, Cons<Absent, Cons<C, Empty>>>;

fn holds_all<L: ContainsAll<M>, M>() {}

fn main() {
    holds_all::<Three, Wanted>();
}
