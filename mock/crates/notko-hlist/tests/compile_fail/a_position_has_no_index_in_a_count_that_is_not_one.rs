// A position is only a position in a count that counts, and `Position<N>`
// leaves `N` unbounded on the declaration for the reason `Length<N>` does: the
// two paths bound it differently and a bound on the trait would push that
// difference into every consumer signature.
//
// `Here` is a legal position and the seal is satisfied, so this is the case
// that reaches the trait's own note rather than the sealing error the sibling
// case produces. What refuses it is the impls carrying the cardinal bound.

use notko_hlist::{Here, Position};

struct NotACount;

fn index_of<P: Position<N>, N>() {}

fn main() {
    index_of::<Here, NotACount>();
}
