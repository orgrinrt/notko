// A uniform carrier holds every member over its value, and that is all it
// does. Another form for one member would be an exception a reader of the
// carrier could not see. A form is stated only for a carrier whose shape is
// per member, so the exception is refused at its shape.

use notko_hlist::{Elem, Form, Shape, Uniform, Uniformly};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Mine;
impl Shape for Mine {
    type Is = Uniformly;
}
impl Uniform for Mine {
    type Of<E: Elem> = E::Value;
}
impl Form<Pos> for Mine {
    type Of = u8;
}

fn main() {}
