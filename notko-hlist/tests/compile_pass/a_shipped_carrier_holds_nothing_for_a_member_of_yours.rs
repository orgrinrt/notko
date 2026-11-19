// The twin: the same form, on a per-member carrier of the crate's own.

use notko_hlist::{Cons, Empty, Form, Has, PCons, PNil, PerMember, Project, Shape};

struct Res;

struct Mine;
impl Shape for Mine {
    type Is = PerMember;
}
impl Form<Res> for Mine {
    type Of = &'static str;
}

fn main() {
    let p: <Cons<Res, Empty> as Project<Mine>>::Prod = PCons { head: "a", tail: PNil };
    let _ = <Cons<Res, Empty> as Has<Mine, Res, _>>::pick(&p);
}
