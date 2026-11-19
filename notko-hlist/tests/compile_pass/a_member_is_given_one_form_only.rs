// The twin: the one form, stated once.

use notko_hlist::Carrier;

struct Pos;

struct Read;
impl Carrier<Pos> for Read {
    type Of = i32;
}

fn main() {}
