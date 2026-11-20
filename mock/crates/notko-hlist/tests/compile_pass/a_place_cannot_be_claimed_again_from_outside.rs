// The twin: the same impl, of a trait of the consumer's own.

use notko_hlist::{Cons, Empty, Here, Place, PlaceOf};

struct Mine;

#[allow(dead_code)]
trait Placed<M, P: Place> {}

#[allow(unused_imports)]
use PlaceOf as _;

impl Placed<Mine, Here> for Cons<Mine, Empty> {}

fn main() {}
