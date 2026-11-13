// The twin: the marker naming `u8` in place of `u8`.

use notko_hlist::{Cons, Elem, Empty, Project};

struct Pos;
impl Elem for Pos {
    type Value = u8;
}

fn has_product<L: Project>() {}

fn main() {
    has_product::<Cons<Pos, Empty>>();
    let _ = Pos;
}
