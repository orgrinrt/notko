// A part is cut out of a product by duplicating each field it names, and an
// exclusive column duplicated is two `&mut` to one slice. `ColMut` has no
// `Dup`, so the cut refuses it.

use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, SubOf};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

type Lanes = Cons<Pos, Empty>;

fn main() {
    let mut a: <Lanes as Project>::Prod<notko_hlist::Arr<2>> = PCons { head: [0; 2], tail: PNil };
    let _ = <Lanes as SubOf<Lanes, _>>::sub(&Lanes::cols_mut::<2>(&mut a));
}
