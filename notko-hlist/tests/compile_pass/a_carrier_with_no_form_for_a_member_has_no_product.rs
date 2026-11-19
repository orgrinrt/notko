// The twin: the carrier states a form for `Heat` too.

use notko_hlist::{Carrier, Cons, Empty, PCons, PNil, Project};

struct Pos;
struct Heat;

struct Read;
impl Carrier<Pos> for Read {
    type Of = i32;
}
impl Carrier<Heat> for Read {
    type Of = u8;
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
