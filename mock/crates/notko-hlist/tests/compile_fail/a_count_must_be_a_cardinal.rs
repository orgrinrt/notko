// The count is the consumer's type and the contract on it is `Cardinal`. A
// type that does not implement it cannot be a length, however number-shaped it
// looks, because there is nothing to start from and nothing to step with.

use notko_hlist::{Cons, Empty, Length};

struct A;

type One = Cons<A, Empty>;

fn main() {
    let _ = <One as Length<usize>>::len();
}
