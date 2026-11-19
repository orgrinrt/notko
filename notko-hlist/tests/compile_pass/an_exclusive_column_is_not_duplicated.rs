// The twin: the shared columns in place of the exclusive ones.

use notko_hlist::{Arrays, Col, Cons, Elem, Empty, PCons, PNil, Project, SubOf};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

type Lanes = Cons<Pos, Empty>;

fn main() {
    let a: <Lanes as Project<notko_hlist::Arr<2>>>::Prod = PCons { head: [0; 2], tail: PNil };
    let _ = <Lanes as SubOf<Col<'_>, Lanes, _>>::sub(&<Lanes as Arrays<2>>::cols(&a));
}
