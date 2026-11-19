// A uniform carrier duplicates every member through `DupAll` or none, so a
// duplication stated for one member is refused at the carrier's shape, the
// way a form is. `Arr<N>` duplicates none.

use notko_hlist::{Arr, DupForm, Elem};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

impl<const N: usize> DupForm<Pos> for Arr<N> {
    fn dup_form(c: &[i32; N]) -> [i32; N] {
        *c
    }
}

fn main() {}
