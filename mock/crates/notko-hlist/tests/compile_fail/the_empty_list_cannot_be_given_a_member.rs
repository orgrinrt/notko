// `Empty` answering a place, written from outside.
//
// The other half of the same hole, and the one that falsifies a specific claim
// rather than a general one: `At`'s documentation says the empty list
// implements it for no position at all, which is what makes an index past the
// end of a list a build that does not succeed. With a free parameter that
// sentence was not true, since `Empty` and `At` are both foreign here while the
// place is local, so this impl satisfied the orphan rule and gave the end of
// every list a member.

use notko_hlist::{At, Empty};

struct Rogue;
struct Lie;

impl At<Rogue> for Empty {
    type Member = Lie;
}

fn main() {}
