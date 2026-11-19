// The twin: the carrier states its form, and the product is the crate's own.

use notko_hlist::{Carrier, Cons, Empty, PCons, PNil, Project};

struct Mine;
struct Read;
impl Carrier<Mine> for Read {
    type Of = u8;
}

fn main() {
    let _p: <Cons<Mine, Empty> as Project<Read>>::Prod = PCons {
        head: 1,
        tail: PNil,
    };
}
