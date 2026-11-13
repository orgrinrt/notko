// A list of bare value types has no product: each member is a marker naming
// its value, so two members carrying the same type stay two members.

use notko_hlist::{Cons, Elem, Empty, Project};

struct Pos;
impl Elem for Pos {
    type Value = u8;
}

fn has_product<L: Project>() {}

fn main() {
    has_product::<Cons<u8, Empty>>();
    let _ = Pos;
}
