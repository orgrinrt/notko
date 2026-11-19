// The twin: the same carrier on both sides.

use notko_hlist::{Cons, Elem, Empty, Form, PCons, PNil, PerMember, Project, Shape};

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
    let a: <Cons<Pos, Empty> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Pos, Empty> as Project<Read>>::Prod = a;
}
