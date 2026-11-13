// The twin: the part names a member the whole holds.

use notko_hlist::{Cons, Elem, Empty, Project, SubOf};

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

fn part<S: Project, L: SubOf<S, Ps>, Ps>() {}

fn main() {
    part::<Whole, Cons<B, Empty>, _>();
    let _ = Absent;
}
