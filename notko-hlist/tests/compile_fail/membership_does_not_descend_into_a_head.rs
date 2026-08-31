// Membership walks the spine and never into a head, so a list whose head is
// itself a list holds that list and does not hold that list's members. The
// positive half is in `the_list_holds_what_it_holds`; this is the half that
// cannot be written as a passing bound.

use notko_hlist::{Cons, Contains, Empty};

struct A;
struct B;

type Inner = Cons<A, Cons<B, Empty>>;
type Nested = Cons<Inner, Empty>;

fn holds<L: Contains<X>, X>() {}

fn main() {
    holds::<Nested, A>();
}
