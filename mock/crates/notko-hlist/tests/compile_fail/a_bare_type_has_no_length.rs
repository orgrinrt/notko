// `Length` is implemented for `Empty` and `Cons` and for nothing else, so a
// type that is not a list has no length. The blanket base impl of
// `ContainsAll` does apply to any type, and this is what keeps that from
// spreading: a bare type is still not a list.

#![feature(const_trait_impl)]

use notko_hlist::{Cardinal, Length};

#[derive(Clone, Copy)]
struct Count(usize);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);
    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

fn main() {
    let _ = <u8 as Length<Count>>::len();
}
