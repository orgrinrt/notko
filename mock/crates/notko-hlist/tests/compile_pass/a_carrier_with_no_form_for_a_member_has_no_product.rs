// The twin: the carrier states a form for `Heat` too.

use notko_hlist::{Form, PerMember, Shape, Cons, Empty, PCons, PNil, Project};

struct Pos;
struct Heat;

struct Read;
impl Shape for Read {
    type Is = PerMember;
}
impl Form<Pos> for Read {
    type Of = i32;
}
impl Form<Heat> for Read {
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
