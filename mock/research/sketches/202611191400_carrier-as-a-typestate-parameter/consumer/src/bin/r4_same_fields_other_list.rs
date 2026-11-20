//! Expected to fail: two lists whose products hold the same field types are
//! still two product types, since the product names its members.

use consumer::Pos;
use model::{Cons, Elem, Empty, PCons, PNil, Project, Val};

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
