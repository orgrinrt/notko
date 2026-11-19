// One list through two carriers that hold the same field type for its member
// is two product types, since a product names its carrier. The carrier is
// never inferred from what the fields hold.

use notko_hlist::{Cons, Elem, Empty, Form, PCons, PNil, PerMember, Project, Shape, Val};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Read;
impl Shape for Read {
    type Is = PerMember;
}
impl Form<Pos> for Read {
    type Of = i32;
}

fn main() {
    let a: <Cons<Pos, Empty> as Project<Val>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Pos, Empty> as Project<Read>>::Prod = a;
}
