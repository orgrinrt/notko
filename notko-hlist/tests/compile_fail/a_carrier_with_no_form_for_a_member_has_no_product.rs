// A product holds a field for every member, so a carrier stating no form for
// one of them has no product for the list.

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

fn main() {
    let _p: <Cons<Pos, Cons<Heat, Empty>> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PCons {
            head: 2,
            tail: PNil,
        },
    };
}
