//! Expected to fail: a product through a carrier that states no form for one
//! of the list's members.

use consumer::{Pos, Read};
use model::{Cons, Elem, Empty, PCons, PNil, Project};

struct Heat;
impl Elem for Heat {
    type Value = u8;
}

fn main() {
    let _p: <Cons<Pos, Cons<Heat, Empty>> as Project<Read<'static>>>::Prod = PCons {
        head: 1,
        tail: PCons {
            head: 2,
            tail: PNil,
        },
    };
}
