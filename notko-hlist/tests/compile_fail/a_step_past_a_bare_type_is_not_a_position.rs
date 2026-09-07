// `There<P>` leaves `P` unbounded on the declaration, for the reason `Cons`
// leaves its own two unbounded: `There<u8>` is a type that exists. What it is
// not is a position, and this is where that arrives.
//
// The counterpart to `a_tail_that_is_not_a_list_has_no_length`, one step over:
// a position is built all the way down to `Here` or it is not built at all, and
// a walk that reaches a bare type has nowhere to take its next step from.

#![feature(const_trait_impl)]

use notko_hlist::{Cardinal, Position, There};

#[derive(Clone, Copy)]
struct Count(usize);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);
    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

fn main() {
    let _ = <There<u8> as Position<Count>>::index();
}
