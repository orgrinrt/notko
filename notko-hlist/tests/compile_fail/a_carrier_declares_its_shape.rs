// A carrier's forms follow from its shape, so a form stated for a carrier
// that has declared none has nothing to follow from.

use notko_hlist::Form;

struct Pos;

struct Read;
impl Form<Pos> for Read {
    type Of = i32;
}

fn main() {}
