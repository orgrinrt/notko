// The twin: the same list on both sides.

use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, Val};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

fn main() {
    let a: <Cons<Pos, Empty> as Project<Val>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Pos, Empty> as Project<Val>>::Prod = a;
}
