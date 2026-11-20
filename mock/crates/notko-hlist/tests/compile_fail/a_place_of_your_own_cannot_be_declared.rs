// `Place` is sealed, so the two types this crate builds positions from are the
// only ones there will ever be.
//
// Declaring one more is what a consumer would do to index a list by something
// that is not a path of `Here` and `There`, and it is refused at the
// declaration, before any list is asked about it. The sibling cases that name a
// position of their own are refused one step later, at the bound on `At`, and
// say only that the type is not a `Place`; this is the case that reaches the
// seal itself.

struct Rogue;

impl notko_hlist::Place for Rogue {}

fn main() {}
