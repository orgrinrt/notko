// The twin: the carrier states its form, and the product is the crate's own.

use notko_hlist::{Cons, Empty, Form, PCons, PNil, PerMember, Project, Shape};

struct Mine;
struct Read;
impl Shape for Read {
    type Is = PerMember;
}
impl Form<Mine> for Read {
    type Of = u8;
}

fn main() {
    let _p: <Cons<Mine, Empty> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
}
