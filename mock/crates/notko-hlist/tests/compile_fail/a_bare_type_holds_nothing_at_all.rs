// The `ContainsAll` base impl is a blanket, and this is where it stops. "`u8`
// holds every member of the empty list" is vacuously true and saying it would
// make the trait hold for a type that is not a list, which is the whole thing
// the sealing is for.

use notko_hlist::{ContainsAll, Empty};

fn holds_all<L: ContainsAll<M>, M>() {}

fn main() {
    holds_all::<u8, Empty>();
}
