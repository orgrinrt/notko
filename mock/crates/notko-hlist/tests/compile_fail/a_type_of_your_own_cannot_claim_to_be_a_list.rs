// `List` is sealed, so the two impls in the crate are the only ones there are.
// Without this every other guarantee here is forgeable: a downstream impl of
// `List` puts a foreign type inside the blanket that `ContainsAll` rests on
// and inside the reach of anything else bounded on it.

use notko_hlist::List;

struct Mine;

impl List for Mine {}

fn main() {}
