// A uniform carrier holds every member over its value, and that is all it
// does. Another form for one member would be an exception a reader of the
// carrier could not see, so it conflicts with the blanket impl.

use notko_hlist::{Carrier, Elem, Uniform};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Mine;
impl Uniform for Mine {
    type Of<E: Elem> = E::Value;
}
impl Carrier<Pos> for Mine {
    type Of = u8;
}

fn main() {}
