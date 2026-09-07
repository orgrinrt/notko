//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Cardinal`, `Length` and `Position` on the const path: every count is a
//! constant, so most of the file is decided by the compiler and a wrong number
//! is a build failure rather than a failing run.
//!
//! Both counts are here because both are the same question, how many cells,
//! answered about a list and about a position, and both land in the consumer's
//! own number type through the same three `Cardinal` impls. Which member sits
//! at a position is a different question and is in
//! `a_member_sits_at_a_position`, which needs no count and no feature.
//!
//! The plain path counts the same lists in `a_count_counts_without_const`, off
//! the same shared module, and the bound spellings both paths must accept are
//! in `both_paths_take_the_same_bounds`. Three targets rather than one because
//! `const impl` is gated at parse time: a `#[cfg]` around it does not save an
//! ungated target, since the span is recorded before cfg-stripping runs, so
//! `required-features` is what actually skips this one.

#![feature(const_trait_impl)]

mod lists;

use lists::*;
use notko_hlist::{At, Cardinal, Empty, Here, Length, Position, There};

// ---------------------------------------------------------------------------
// Three count types, because the count being the consumer's choice is the
// reason `Length` carries a parameter at all. A list has a length in each of
// them at once and picking one is the call site's business.
// ---------------------------------------------------------------------------

/// The ordinary case: counts up forever.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Count(usize);

/// A count that stops at four, which is a legal `Cardinal` and a reminder that
/// nothing here assumes the successor is injective. What `succ` does is the
/// consumer's, and a saturating count is a real shape: a capacity that reports
/// "four or more" is a thing people build.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Saturating(u8);

/// A count that is not a number. `Length` never adds, compares or orders, so
/// unary is enough to satisfy it, and a type that can only be built by
/// succeeding from zero proves that.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Tally(u32);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);

    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

const impl Cardinal for Saturating {
    const ZERO: Self = Saturating(0);

    fn succ(self) -> Self {
        Saturating(if self.0 >= 4 { 4 } else { self.0 + 1 })
    }
}

const impl Cardinal for Tally {
    const ZERO: Self = Tally(0);

    fn succ(self) -> Self {
        Tally(self.0 + 1)
    }
}

// ---------------------------------------------------------------------------
// Decided at build time. A wrong one of these does not run.
// ---------------------------------------------------------------------------

/// Zero through five, which is where an off-by-one lives if there is one.
const _: () = assert!(<L0 as Length<Count>>::LEN.0 == 0);
const _: () = assert!(<L1 as Length<Count>>::LEN.0 == 1);
const _: () = assert!(<L2 as Length<Count>>::LEN.0 == 2);
const _: () = assert!(<L3 as Length<Count>>::LEN.0 == 3);
const _: () = assert!(<L4 as Length<Count>>::LEN.0 == 4);
const _: () = assert!(<L5 as Length<Count>>::LEN.0 == 5);

/// A list is not a set.
const _: () = assert!(<Repeated as Length<Count>>::LEN.0 == 3);

/// A head that is a list counts once. Were the walk to descend into heads this
/// would be five.
const _: () = assert!(<Nested as Length<Count>>::LEN.0 == 2);

/// A head carrying a lifetime still counts.
const _: () = assert!(<WithBorrowing as Length<Count>>::LEN.0 == 2);

/// Deep enough to matter. The eight is pinned as well as the thirty-two,
/// because the long list is built by nesting the short one and a wrong `Eight`
/// would give a wrong `L32` that still looks like a round number.
const _: () = assert!(<L8 as Length<Count>>::LEN.0 == 8);
const _: () = assert!(<L32 as Length<Count>>::LEN.0 == 32);

/// The saturating count stops where it says it does, and the list it counts is
/// longer than the stop. A length reports what the count type says, not what
/// the list would have been in a different one.
const _: () = assert!(<L3 as Length<Saturating>>::LEN.0 == 3);
const _: () = assert!(<L5 as Length<Saturating>>::LEN.0 == 4);
const _: () = assert!(<L32 as Length<Saturating>>::LEN.0 == 4);

/// One list, three counts, at once. This is what parameterising bought.
const _: () = assert!(<L4 as Length<Count>>::LEN.0 == 4);
const _: () = assert!(<L4 as Length<Tally>>::LEN.0 == 4);
const _: () = assert!(<L4 as Length<Saturating>>::LEN.0 == 4);

/// A position's index, zero through seven, which is the other place an
/// off-by-one lives. `Here` is nought rather than one, so the index is what a
/// consumer hands a slice and not what it hands a human.
const _: () = assert!(<P0 as Position<Count>>::INDEX.0 == 0);
const _: () = assert!(<P1 as Position<Count>>::INDEX.0 == 1);
const _: () = assert!(<P2 as Position<Count>>::INDEX.0 == 2);
const _: () = assert!(<P3 as Position<Count>>::INDEX.0 == 3);
const _: () = assert!(<P7 as Position<Count>>::INDEX.0 == 7);

/// Deep enough that the recursion is doing real work, and `Onwards` is pinned
/// under it for the reason `Eight` is: a wrong eight gives a wrong thirty-one
/// that still looks like a number somebody meant.
const _: () = assert!(<Onwards<P0> as Position<Count>>::INDEX.0 == 8);
const _: () = assert!(<P31 as Position<Count>>::INDEX.0 == 31);

/// An index and a length are the same walk from opposite ends, so the last
/// position of a list is one under its length. Nothing enforces that and
/// nothing could: they are two traits over two kinds of thing, and this is
/// where the pair is held to it.
const _: () = assert!(<P31 as Position<Count>>::INDEX.0 + 1 == <L32 as Length<Count>>::LEN.0);
const _: () = assert!(<P7 as Position<Count>>::INDEX.0 + 1 == <L8 as Length<Count>>::LEN.0);

/// A position counts in whichever type it is asked in, and the saturating one
/// stops where it says it does, exactly as a length does. An index that
/// saturates is a real shape and a wrong one to hand a slice, which is the
/// consumer's call rather than this crate's.
const _: () = assert!(<P3 as Position<Saturating>>::INDEX.0 == 3);
const _: () = assert!(<P7 as Position<Saturating>>::INDEX.0 == 4);
const _: () = assert!(<P3 as Position<Tally>>::INDEX.0 == 3);

// ---------------------------------------------------------------------------
// Run-time restatements. The constants above are the real check; these exist so
// a reader running the suite sees the numbers rather than an empty test list,
// and so a regression reports a value rather than only a line.
// ---------------------------------------------------------------------------

#[test]
fn len_reads_the_same_constant() {
    // The provided method exists so that code written against the plain path
    // compiles here unchanged. It has to be the same number.
    assert_eq!(<L0 as Length<Count>>::len(), <L0 as Length<Count>>::LEN);
    assert_eq!(<L3 as Length<Count>>::len(), <L3 as Length<Count>>::LEN);
    assert_eq!(<L32 as Length<Count>>::len(), <L32 as Length<Count>>::LEN);
    assert_eq!(
        <L5 as Length<Saturating>>::len(),
        <L5 as Length<Saturating>>::LEN
    );
}

#[test]
fn the_constants_are_the_numbers_they_should_be() {
    assert_eq!(<L0 as Length<Count>>::LEN, Count(0));
    assert_eq!(<L5 as Length<Count>>::LEN, Count(5));
    assert_eq!(<Repeated as Length<Count>>::LEN, Count(3));
    assert_eq!(<Nested as Length<Count>>::LEN, Count(2));
    assert_eq!(<L8 as Length<Count>>::LEN, Count(8));
    assert_eq!(<L32 as Length<Count>>::LEN, Count(32));
    assert_eq!(<L5 as Length<Saturating>>::LEN, Saturating(4));
    assert_eq!(<L4 as Length<Tally>>::LEN, Tally(4));
}

#[test]
fn index_reads_the_same_constant() {
    // The provided method is here for the same reason `len` is, so it has to
    // be the same number.
    assert_eq!(
        <P0 as Position<Count>>::index(),
        <P0 as Position<Count>>::INDEX
    );
    assert_eq!(
        <P31 as Position<Count>>::index(),
        <P31 as Position<Count>>::INDEX
    );
    assert_eq!(
        <P7 as Position<Saturating>>::index(),
        <P7 as Position<Saturating>>::INDEX
    );
}

#[test]
fn the_front_is_nought_and_a_step_is_one_more() {
    // The whole of what a position means, spelled out, so the two impls are
    // pinned by something other than the numbers they add up to.
    assert_eq!(<Here as Position<Count>>::INDEX, Count::ZERO);
    assert_eq!(
        <There<Here> as Position<Count>>::INDEX,
        <Here as Position<Count>>::INDEX.succ()
    );
    assert_eq!(
        <There<There<Here>> as Position<Count>>::INDEX,
        <There<Here> as Position<Count>>::INDEX.succ()
    );
}

#[test]
fn a_zero_is_a_zero_in_every_count() {
    assert_eq!(<Empty as Length<Count>>::LEN, Count::ZERO);
    assert_eq!(<Empty as Length<Saturating>>::LEN, Saturating::ZERO);
    assert_eq!(<Empty as Length<Tally>>::LEN, Tally::ZERO);
}

#[test]
fn a_generic_bound_takes_every_list_and_every_count() {
    // The bound is declared in `both_paths_take_the_same_bounds`, which is
    // where the parity is asserted. This is it instantiated, which the other
    // file cannot do because instantiating needs a concrete count and a count
    // is where the two paths differ.
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
fn a_generic_bound_takes_every_position_and_every_count() {
    // The same shape as `length_of` and for the same reason: a bound that only
    // works on concrete positions would satisfy every constant above and no
    // consumer.
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
    // `At` and `Position` are split so a consumer can take each from the side
    // that owns it, and nothing above checks that the two agree: the member is
    // pinned in `a_member_sits_at_a_position` and the index is pinned here, in
    // two coordinate systems that happen to line up. This is the law that says
    // they are one system. An off-by-one in either walk moves one and not the
    // other, and this is the only case that would report it.
    //
    // The name is what a member and an index can both be compared through,
    // since a type cannot be indexed for at runtime. `run` is the list written
    // out in order by hand, which is what keeps this from restating either
    // impl.
    fn agree<L: At<P, Member = M>, P: Position<Count>, M>(run: &[&str]) {
        // The list is named in the body as well as in the bound, since a
        // parameter appearing only in a bound reads to a linter as one nobody
        // uses, and here the bound is half of what is being asserted.
        let _ = core::marker::PhantomData::<L>;
        let i = <P as Position<Count>>::INDEX.0;
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

    // Long enough that an off-by-one has somewhere to hide. `Eight` repeats
    // `A B C D E A B C`, so the run is written out once and laid down four
    // times rather than typed thirty-two times and miscounted.
    let block = ["A", "B", "C", "D", "E", "A", "B", "C"];
    let mut thirty_two = [""; 32];
    for (i, slot) in thirty_two.iter_mut().enumerate() {
        *slot = block[i % block.len()];
    }
    agree::<L32, P0, A>(&thirty_two);
    agree::<L32, P7, C>(&thirty_two);
    agree::<L32, P31, C>(&thirty_two);
}

#[test]
fn succ_from_zero_is_the_whole_of_a_cardinal() {
    // The trait is two items and nothing else, so this is its entire surface.
    assert_eq!(Count::ZERO.succ().succ().succ(), Count(3));
    assert_eq!(Tally::ZERO.succ(), Tally(1));
    assert_eq!(
        Saturating::ZERO.succ().succ().succ().succ().succ().succ(),
        Saturating(4)
    );
}

#[test]
fn a_cardinal_may_also_be_stepped_at_compile_time() {
    // What the const path buys, at its smallest: the successor in a constant.
    const THREE: Count = Count::ZERO.succ().succ().succ();
    const STOPPED: Saturating = Saturating::ZERO.succ().succ().succ().succ().succ();
    assert_eq!(THREE, Count(3));
    assert_eq!(STOPPED, Saturating(4));
}
