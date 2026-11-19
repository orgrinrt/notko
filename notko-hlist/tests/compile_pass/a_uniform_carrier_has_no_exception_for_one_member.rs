// The twin: the same form for `Pos` on a carrier that is per member instead.

use notko_hlist::{Elem, Form, PerMember, Shape};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Mine;
impl Shape for Mine {
    type Is = PerMember;
}
impl Form<Pos> for Mine {
    type Of = u8;
}

fn main() {}
