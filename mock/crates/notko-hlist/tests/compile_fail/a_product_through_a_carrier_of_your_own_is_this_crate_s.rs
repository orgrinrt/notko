// A carrier of the consumer's own puts a local type in `Project`'s parameter,
// so the orphan rule lets an impl through it be written for any list, with any
// product it likes. That would make the product of a list something other than
// the nest the list says. The seal is what refuses it: `Project::sealed` takes
// a token no other crate can name, so the impl cannot supply every item.

use notko_hlist::{Cons, Empty, Project};

struct Mine;
struct Read;

impl Project<Read> for Cons<Mine, Empty> {
    type Prod = u8;
}

fn main() {}
