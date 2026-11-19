// The twin: the same carrier on both sides.

use notko_hlist::{Carrier, Cons, Elem, Empty, PCons, PNil, Project};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Read;
impl Carrier<Pos> for Read {
    type Of = i32;
}

fn main() {
    let a: <Cons<Pos, Empty> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Pos, Empty> as Project<Read>>::Prod = a;
}
