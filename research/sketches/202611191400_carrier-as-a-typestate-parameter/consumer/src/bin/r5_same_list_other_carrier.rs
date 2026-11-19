//! Expected to fail: one list through two carriers that both hold an `i32`
//! for its member is still two product types, since the product names its
//! carrier.

use consumer::{Pos, Read};
use model::{Cons, Empty, PCons, PNil, Project, Val};

fn main() {
    let a: <Cons<Pos, Empty> as Project<Val>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
    let _b: <Cons<Pos, Empty> as Project<Read<'static>>>::Prod = a;
}
