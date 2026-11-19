// The twin: the one form, stated once.

use notko_hlist::{Form, PerMember, Shape};

struct Pos;

struct Read;
impl Shape for Read {
    type Is = PerMember;
}
impl Form<Pos> for Read {
    type Of = i32;
}

fn main() {}
