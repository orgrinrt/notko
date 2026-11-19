// `Arr<N>` does not duplicate what it holds, since that would copy the whole
// storage. A crate owning an `Elem` member could otherwise state `Dup` for it
// on `Arr<N>`, through the member parameter. Duplication on a uniform carrier
// comes through `DupAll` alone, so the impl conflicts with the blanket one.

use notko_hlist::{Arr, Dup, Elem};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

impl<const N: usize> Dup<Pos> for Arr<N> {
    fn dup(c: &[i32; N]) -> [i32; N] {
        *c
    }
}

fn main() {}
