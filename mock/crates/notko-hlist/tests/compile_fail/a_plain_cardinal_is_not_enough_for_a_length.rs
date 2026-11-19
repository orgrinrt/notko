// On the const path the count has to be const-implemented, not merely
// implemented: `LEN` is a constant and building it calls `succ` inside one. A
// plain impl satisfies `Cardinal` and still cannot carry a length, and the
// error naming that is worth pinning, because the two impls differ by one
// keyword and read as the same thing.

#![feature(const_trait_impl)]

use notko_hlist::{Cardinal, Cons, Empty, Length};

#[derive(Clone, Copy)]
struct Count(usize);

impl Cardinal for Count {
    const ZERO: Self = Count(0);
    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

struct A;

type One = Cons<A, Empty>;

fn main() {
    let _ = <One as Length<Count>>::len();
}
