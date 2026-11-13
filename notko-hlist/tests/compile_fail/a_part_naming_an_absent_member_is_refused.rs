// A part names only members of the whole. One it names that the whole lacks
// leaves the cut with nowhere to read the field from.

use notko_hlist::{Cons, Elem, Empty, List, Project, SubOf};

struct A;
impl Elem for A {
    type Value = u8;
}
struct B;
impl Elem for B {
    type Value = u8;
}
struct Absent;
impl Elem for Absent {
    type Value = u8;
}

type Whole = Cons<A, Cons<B, Empty>>;

fn part<S: Project, L: SubOf<S, Ps>, Ps: List>() {}

fn main() {
    part::<Whole, Cons<Absent, Empty>, _>();
}
