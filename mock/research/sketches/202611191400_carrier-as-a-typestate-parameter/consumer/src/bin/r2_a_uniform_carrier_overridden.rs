//! Expected to fail: a uniform carrier quietly given another form for one
//! member, so that what it holds for `Pos` would depend on which impl is read.

use consumer::Pos;
use model::{Carrier, Elem, Uniform};

struct Mine;
impl Uniform for Mine {
    type Of<E: Elem> = E::Value;
}
impl Carrier<Pos> for Mine {
    type Of = u8;
}

fn main() {}
