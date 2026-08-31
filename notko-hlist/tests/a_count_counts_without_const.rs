//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Cardinal` and `Length` on the plain path: the same lengths over the same
//! lists as `a_count_counts`, arrived at by calling rather than by being named.
//!
//! Every assertion there that can be expressed here is expressed here, so the
//! two configurations are not tested to different standards.
//!
//! The whole file is behind a crate-level `cfg`, which is how a target is
//! excluded from a configuration: `required-features` can require a feature
//! and there is no spelling for requiring its absence. Nothing here uses gated
//! syntax, so unlike the other direction the cfg is enough.

#![cfg(not(feature = "const"))]

mod lists;

use lists::*;
use notko_hlist::{Cardinal, Empty, Length};

/// The ordinary case: counts up forever.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Count(usize);

/// A count that stops at four, which is a legal `Cardinal` and a reminder that
/// nothing here assumes the successor is injective.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Saturating(u8);

/// A count that is not a number. `Length` never adds, compares or orders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Tally(u32);

impl Cardinal for Count {
    const ZERO: Self = Count(0);

    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

impl Cardinal for Saturating {
    const ZERO: Self = Saturating(0);

    fn succ(self) -> Self {
        Saturating(if self.0 >= 4 { 4 } else { self.0 + 1 })
    }
}

impl Cardinal for Tally {
    const ZERO: Self = Tally(0);

    fn succ(self) -> Self {
        Tally(self.0 + 1)
    }
}

#[test]
fn zero_through_five() {
    assert_eq!(<L0 as Length<Count>>::len(), Count(0));
    assert_eq!(<L1 as Length<Count>>::len(), Count(1));
    assert_eq!(<L2 as Length<Count>>::len(), Count(2));
    assert_eq!(<L3 as Length<Count>>::len(), Count(3));
    assert_eq!(<L4 as Length<Count>>::len(), Count(4));
    assert_eq!(<L5 as Length<Count>>::len(), Count(5));
}

#[test]
fn a_list_is_not_a_set() {
    assert_eq!(<Repeated as Length<Count>>::len(), Count(3));
}

#[test]
fn a_head_that_is_a_list_counts_once() {
    assert_eq!(<Nested as Length<Count>>::len(), Count(2));
}

#[test]
fn a_head_that_borrows_still_counts() {
    assert_eq!(<WithBorrowing as Length<Count>>::len(), Count(2));
}

#[test]
fn thirty_two_cells_inside_the_default_recursion_limit() {
    // The eight as well as the thirty-two: the long list is the short one
    // nested, so a wrong `Eight` gives a wrong `L32` that still looks like a
    // round number.
    assert_eq!(<L8 as Length<Count>>::len(), Count(8));
    assert_eq!(<L32 as Length<Count>>::len(), Count(32));
}

#[test]
fn the_count_type_decides_what_the_number_does() {
    assert_eq!(<L3 as Length<Saturating>>::len(), Saturating(3));
    assert_eq!(<L5 as Length<Saturating>>::len(), Saturating(4));
    assert_eq!(<L32 as Length<Saturating>>::len(), Saturating(4));
}

#[test]
fn one_list_has_a_length_in_every_count_at_once() {
    assert_eq!(<L4 as Length<Count>>::len(), Count(4));
    assert_eq!(<L4 as Length<Tally>>::len(), Tally(4));
    assert_eq!(<L4 as Length<Saturating>>::len(), Saturating(4));
}

#[test]
fn a_zero_is_a_zero_in_every_count() {
    assert_eq!(<Empty as Length<Count>>::len(), Count::ZERO);
    assert_eq!(<Empty as Length<Saturating>>::len(), Saturating::ZERO);
    assert_eq!(<Empty as Length<Tally>>::len(), Tally::ZERO);
}

#[test]
fn a_generic_bound_takes_every_list_and_every_count() {
    // The same body as the const path's, in the same spelling. That it needs
    // no `const` anywhere is the parity `both_paths_take_the_same_bounds`
    // asserts; this is it instantiated.
    fn length_of<L: Length<N>, N: Cardinal>() -> N {
        <L as Length<N>>::len()
    }

    assert_eq!(length_of::<L0, Count>(), Count(0));
    assert_eq!(length_of::<L5, Count>(), Count(5));
    assert_eq!(length_of::<L32, Count>(), Count(32));
    assert_eq!(length_of::<Repeated, Tally>(), Tally(3));
    assert_eq!(length_of::<L5, Saturating>(), Saturating(4));
}

#[test]
fn succ_from_zero_is_the_whole_of_a_cardinal() {
    assert_eq!(Count::ZERO.succ().succ().succ(), Count(3));
    assert_eq!(Tally::ZERO.succ(), Tally(1));
    assert_eq!(
        Saturating::ZERO.succ().succ().succ().succ().succ().succ(),
        Saturating(4)
    );
}
