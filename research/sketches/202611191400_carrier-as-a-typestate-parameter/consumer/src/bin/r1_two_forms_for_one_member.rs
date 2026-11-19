//! Expected to fail: a carrier stating two forms for one member.

use consumer::Pos;
use model::Carrier;

struct Two;
impl Carrier<Pos> for Two {
    type Of = i32;
}
impl Carrier<Pos> for Two {
    type Of = u8;
}

fn main() {}
