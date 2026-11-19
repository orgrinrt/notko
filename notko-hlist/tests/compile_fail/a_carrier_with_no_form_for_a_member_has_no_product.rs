// A product holds a field for every member, so a carrier stating no form for
// one of them has no product for the list.

use notko_hlist::{Carrier, Cons, Empty, PCons, PNil, Project};

struct Pos;
struct Heat;

struct Read;
impl Carrier<Pos> for Read {
    type Of = i32;
}

fn main() {
    let _p: <Cons<Pos, Cons<Heat, Empty>> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PCons {
            head: 2,
            tail: PNil,
        },
    };
}
