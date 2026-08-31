// `Length<N>` does not bound `N`, deliberately: the two paths bound it
// differently and a bound on the trait would push that difference into every
// consumer signature. What keeps that from being a weakening is the sealing,
// and this is the case that would exploit it.
//
// Were `Length` open, this impl would compile, `u8` implements no `Cardinal`
// at all, and a generic function bounded on `L: Length<N>` could no longer get
// a zero back without re-declaring the bound in the spelling of whichever
// configuration it was written for.

use notko_hlist::Length;

struct Mine;

impl Length<u8> for Mine {
    const LEN: u8 = 3;
}

fn main() {}
