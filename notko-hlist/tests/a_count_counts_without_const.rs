//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Cardinal`, `Length` and `Position` on the plain path: the same counts over
//! the same lists and positions as `a_count_counts`, arrived at by calling
//! rather than by being named.
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
use notko_hlist::{At, Cardinal, Empty, Here, Length, Position, There};

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

#[test]
fn a_position_says_how_far_along_it_is() {
    // Zero through seven, then deep, which is where an off-by-one lives.
    // `Here` is nought rather than one, so the index is what a consumer hands
    // a slice and not what it hands a human.
    assert_eq!(<P0 as Position<Count>>::index(), Count(0));
    assert_eq!(<P1 as Position<Count>>::index(), Count(1));
    assert_eq!(<P2 as Position<Count>>::index(), Count(2));
    assert_eq!(<P3 as Position<Count>>::index(), Count(3));
    assert_eq!(<P7 as Position<Count>>::index(), Count(7));
    assert_eq!(<Onwards<P0> as Position<Count>>::index(), Count(8));
    assert_eq!(<P31 as Position<Count>>::index(), Count(31));
}

#[test]
fn the_front_is_nought_and_a_step_is_one_more() {
    // The whole of what a position means, spelled out, so the two impls are
    // pinned by something other than the numbers they add up to.
    assert_eq!(<Here as Position<Count>>::index(), Count::ZERO);
    assert_eq!(
        <There<Here> as Position<Count>>::index(),
        <Here as Position<Count>>::index().succ()
    );
    assert_eq!(
        <There<There<Here>> as Position<Count>>::index(),
        <There<Here> as Position<Count>>::index().succ()
    );
}

#[test]
fn the_last_position_of_a_list_is_one_under_its_length() {
    // An index and a length are the same walk from opposite ends. Nothing
    // enforces that and nothing could: they are two traits over two kinds of
    // thing, and this is where the pair is held to it.
    assert_eq!(
        <P31 as Position<Count>>::index().succ(),
        <L32 as Length<Count>>::len()
    );
    assert_eq!(
        <P7 as Position<Count>>::index().succ(),
        <L8 as Length<Count>>::len()
    );
}

#[test]
fn a_position_counts_in_whichever_type_it_is_asked_in() {
    // The saturating one stops where it says it does, exactly as a length
    // does. An index that saturates is a real shape and a wrong one to hand a
    // slice, which is the consumer's call rather than this crate's.
    assert_eq!(<P3 as Position<Saturating>>::index(), Saturating(3));
    assert_eq!(<P7 as Position<Saturating>>::index(), Saturating(4));
    assert_eq!(<P3 as Position<Tally>>::index(), Tally(3));
}

#[test]
fn a_generic_bound_takes_every_position_and_every_count() {
    // The same shape as `length_of` above and for the same reason: a bound
    // that only works on concrete positions would satisfy every assertion here
    // and no consumer.
    fn index_of<P: Position<N>, N: Cardinal>() -> N {
        <P as Position<N>>::index()
    }

    assert_eq!(index_of::<P0, Count>(), Count(0));
    assert_eq!(index_of::<P31, Count>(), Count(31));
    assert_eq!(index_of::<P7, Saturating>(), Saturating(4));
    assert_eq!(index_of::<P3, Tally>(), Tally(3));
}

#[test]
fn the_two_halves_of_a_position_mean_the_same_cell() {
    // The same law the const path pins, through the call rather than the
    // constant. It is worth having on both paths because `At` is shared and
    // `Position` is not: the member is answered by one impl in either
    // configuration and the index by a different one, so agreeing on the const
    // path is not agreeing here.
    //
    // The name is what a member and an index can both be compared through,
    // since a type cannot be indexed for at runtime, and `run` is the list
    // written out in order by hand, which is what keeps this from restating
    // either impl.
    fn agree<L: At<P, Member = M>, P: Position<Count>, M>(run: &[&str]) {
        // The list is named in the body as well as in the bound, since a
        // parameter appearing only in a bound reads to a linter as one nobody
        // uses, and here the bound is half of what is being asserted.
        let _ = core::marker::PhantomData::<L>;
        let i = <P as Position<Count>>::index().0;
        let member = core::any::type_name::<M>();
        assert!(
            member.ends_with(run[i]),
            "the position counts to {i}, where the list has {}, and the member \
             at it is {member}",
            run[i]
        );
    }

    let five = ["A", "B", "C", "D", "E"];
    agree::<L5, P0, A>(&five);
    agree::<L5, P1, B>(&five);
    agree::<L5, P4, E>(&five);

    let block = ["A", "B", "C", "D", "E", "A", "B", "C"];
    let mut thirty_two = [""; 32];
    for (i, slot) in thirty_two.iter_mut().enumerate() {
        *slot = block[i % block.len()];
    }
    agree::<L32, P0, A>(&thirty_two);
    agree::<L32, P7, C>(&thirty_two);
    agree::<L32, P31, C>(&thirty_two);
}
