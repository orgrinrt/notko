// The twin: a marker naming `u8` in place of `u8`.

use notko_hlist::{Elem, Uniform, Val};

struct Pos;
impl Elem for Pos {
    type Value = u8;
}

fn main() {
    let _: <Val as Uniform>::Of<Pos> = 1;
}
