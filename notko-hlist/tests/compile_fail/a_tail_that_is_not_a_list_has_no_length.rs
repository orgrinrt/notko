// A cell is only as good as its tail. Nothing bounds `T` where `Cons` is
// declared, deliberately, so `Cons<A, u8>` is a type that exists; what it does
// not have is a length, and the error arrives at the use rather than at the
// declaration.

#![feature(const_trait_impl)]

use notko_hlist::{Cardinal, Cons, Length};

#[derive(Clone, Copy)]
struct Count(usize);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);
    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

struct A;

type Malformed = Cons<A, u8>;

fn main() {
    let _ = <Malformed as Length<Count>>::len();
}
