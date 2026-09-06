// A marker of your own is not a position, however plausible its index.
//
// `Position` is sealed through `Here` and `There`, and this is what the seal is
// for. Without it a consumer could declare a marker, give it whatever index it
// liked, and hand it to a bound that would then prove nothing: `At` answers no
// member for it, so the type is unchecked, and a consumer reading the count
// alone would index wherever it was told.

use notko_hlist::{Cardinal, Position};

#[derive(Clone, Copy)]
struct Count(usize);

impl Cardinal for Count {
    const ZERO: Self = Count(0);

    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

struct Ninth;

impl Position<Count> for Ninth {
    const INDEX: Count = Count(9);
}

fn main() {}
