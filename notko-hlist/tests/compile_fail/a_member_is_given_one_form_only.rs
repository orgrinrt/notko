// A carrier states one form for a member. A second impl for the same member
// would make what the product holds for it depend on which impl is read, so
// coherence refuses it.

use notko_hlist::Carrier;

struct Pos;

struct Read;
impl Carrier<Pos> for Read {
    type Of = i32;
}
impl Carrier<Pos> for Read {
    type Of = u8;
}

fn main() {}
