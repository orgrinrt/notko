// The twin: the shared columns in place of the exclusive ones.

use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, SubOf};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

type Lanes = Cons<Pos, Empty>;

fn main() {
    let mut a: <Lanes as Project>::Prod<notko_hlist::Arr<2>> = PCons { head: [0; 2], tail: PNil };
    let _ = <Lanes as SubOf<Lanes, _>>::sub(&Lanes::cols::<2>(&mut a));
}
