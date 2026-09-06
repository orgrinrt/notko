//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `At`: which member a position names.
//!
//! Ungated, because `At` needs neither feature: its two impls do not overlap,
//! so there is no marker attribute, and the associated type is resolved by the
//! solver rather than computed in a constant, so there is no const path. The
//! index that goes with a position is the other half and is a count, so it is
//! pinned in the two counting targets beside `Length`.
//!
//! Nothing here runs a walk. Every assertion below is an equality between two
//! types, checked by [`at_is`] failing to build when they differ, which is the
//! only shape the property has: `At` answers a type and a type is not a value
//! to compare at runtime.

mod lists;

use lists::*;
use notko_hlist::{At, Concat, Cons, Empty, There};

/// Compiles exactly when the member of `L` at `P` is `M`.
///
/// The bound is the assertion and the empty body is the point: an associated
/// type equality in a `where` clause is checked where the function is
/// declared, so instantiating it in a constant is a build failure on any
/// mismatch and costs nothing at runtime.
const fn at_is<L, P, M>()
where
    L: At<P, Member = M>,
{
    // The list is named in the body as well as in the bound. Without this the
    // parameter appears only in a `where` clause, which reads to a linter as a
    // parameter nobody uses, and the reading is exactly backwards: `L` is the
    // thing under test and the bound is the whole assertion.
    let _ = core::marker::PhantomData::<L>;
}

// ---------------------------------------------------------------------------
// Decided at build time. A wrong one of these does not run.
// ---------------------------------------------------------------------------

/// The front, in every list that has one.
const _: () = at_is::<L1, P0, A>();
const _: () = at_is::<L2, P0, A>();
const _: () = at_is::<L5, P0, A>();

/// Every position of a five-cell list, which is where an off-by-one lives if
/// there is one.
const _: () = at_is::<L5, P0, A>();
const _: () = at_is::<L5, P1, B>();
const _: () = at_is::<L5, P2, C>();
const _: () = at_is::<L5, P3, D>();
const _: () = at_is::<L5, P4, E>();

/// A shorter list answers the positions it has and the same ones answer the
/// same members, so a list is not renumbered by being longer.
const _: () = at_is::<L3, P0, A>();
const _: () = at_is::<L3, P1, B>();
const _: () = at_is::<L3, P2, C>();

/// A list is not a set: the same type at three positions is three answers and
/// not one, which is the fact `Contains` deliberately cannot see.
const _: () = at_is::<Repeated, P0, A>();
const _: () = at_is::<Repeated, P1, A>();
const _: () = at_is::<Repeated, P2, A>();

/// A head that is itself a list is one member and is answered whole. Were the
/// walk to descend into heads, position one here would be `B`.
const _: () = at_is::<Nested, P0, L3>();
const _: () = at_is::<Nested, P1, L2>();

/// A head carrying a lifetime is an ordinary member and is nameable.
const _: () = at_is::<WithBorrowing, P0, Borrowing<'static>>();
const _: () = at_is::<WithBorrowing, P1, A>();

/// Deep enough that the recursion is doing real work. `Eight` repeats
/// `A B C D E A B C`, so the eighth cell is where the pattern restarts and the
/// last cell of `L32` is the one an off-by-one at either end puts outside.
const _: () = at_is::<L8, P7, C>();
const _: () = at_is::<L32, P0, A>();
const _: () = at_is::<L32, P7, C>();
const _: () = at_is::<L32, Onwards<P0>, A>();
const _: () = at_is::<L32, P31, C>();

/// The tail of a cell is what answers every position but the front, which is
/// the recursion stated as a fact rather than read off the impl.
const _: () = at_is::<Cons<E, L3>, P0, E>();
const _: () = at_is::<Cons<E, L3>, P1, A>();
const _: () = at_is::<Cons<E, L3>, P2, B>();

/// Appending preserves order on both sides, positionally. `L2` is `A B` and
/// `L3` is `A B C`, so the join sits between position one and position two and
/// the right-hand side is not renumbered into the left.
const _: () = at_is::<<L2 as Concat<L3>>::Out, P1, B>();
const _: () = at_is::<<L2 as Concat<L3>>::Out, P2, A>();
const _: () = at_is::<<L2 as Concat<L3>>::Out, P4, C>();

/// Appending onto the empty list is the right-hand side unchanged, so every
/// position of it is where it was.
const _: () = at_is::<<Empty as Concat<L3>>::Out, P2, C>();

// ---------------------------------------------------------------------------
// Run-time restatements. The constants above are the real check; these exist
// so a reader running the suite sees the target report rather than an empty
// test list, and so a regression names something.
// ---------------------------------------------------------------------------

#[test]
fn a_member_is_reached_through_a_generic_bound() {
    // The same equality from inside a function generic over all three, which
    // is the shape a consumer writes. It resolves here for the same reason the
    // constants above do, and it is worth pinning separately because a bound
    // that only works on concrete types would satisfy every constant and no
    // consumer.
    fn member_of<L: At<P, Member = M>, P, M>(value: M) -> M {
        let _ = core::marker::PhantomData::<L>;
        value
    }

    let _: A = member_of::<L5, P0, A>(A);
    let _: E = member_of::<L5, P4, E>(E);
    let _: C = member_of::<L32, P31, C>(C);
}

#[test]
fn a_position_names_a_place_and_carries_no_value() {
    // A position is a type-level object like a list is. If `There` ever grew
    // a field the walk would still work and the crate would have stopped being
    // what it says it is.
    assert_eq!(core::mem::size_of::<P0>(), 0);
    assert_eq!(core::mem::size_of::<P7>(), 0);
    assert_eq!(core::mem::size_of::<P31>(), 0);
    assert_eq!(core::mem::size_of::<There<Empty>>(), 0);
}
