// A blanket `At` impl written from outside, over every list, at a place of the
// writer's own.
//
// This is the case the crate's trait and the crate's type do not refuse on
// their own. `At`'s parameter sits after `Self` and is free, so filling it with
// a local type satisfies the orphan rule, and what follows is an impl covering
// every `Cons` in the graph, answering whatever member it likes, written by
// somebody who is not this crate. It compiled until `Place` went on the
// parameter, so the case is here to keep the bound from being taken back off.

use notko_hlist::{At, Cons, List};

struct Rogue;
struct Lie;

impl<H, T: List> At<Rogue> for Cons<H, T> {
    type Member = Lie;
}

fn main() {}
