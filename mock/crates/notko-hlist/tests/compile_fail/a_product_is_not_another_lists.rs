// Two lists whose products hold the same field types are two product types,
// since a product names its members.

use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, Val};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}
struct Depth;
impl Elem for Depth {
    type Value = i32;
}

fn main() {
    let a: <Cons<Pos, Empty> as Project<Val>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Depth, Empty> as Project<Val>>::Prod = a;
}
