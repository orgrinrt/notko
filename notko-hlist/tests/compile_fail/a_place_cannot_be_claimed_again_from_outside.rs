// `PlaceOf` is closed by the `At` equality and by coherence: `Mine` is the
// consumer's own, so the orphan rule lets the impl be written, and it then
// overlaps the crate's own impl at `Here`.

use notko_hlist::{Cons, Empty, Here, Place, PlaceOf};

struct Mine;

#[allow(dead_code)]
trait Placed<M, P: Place> {}

#[allow(unused_imports)]
use PlaceOf as _;

impl PlaceOf<Mine, Here> for Cons<Mine, Empty> {}

fn main() {}
