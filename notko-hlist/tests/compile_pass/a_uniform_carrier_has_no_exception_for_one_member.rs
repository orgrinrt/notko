// The twin: the same form for `Pos` on a carrier that is per member instead.

use notko_hlist::{Carrier, Elem};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Mine;
impl Carrier<Pos> for Mine {
    type Of = u8;
}

fn main() {}
