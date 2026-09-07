//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! That a consumer writes the same bounds whichever path it is built against.
//!
//! Ungated on purpose, and that is the whole assertion. Nothing here runs; the
//! target failing to build in exactly one configuration is the failure, which
//! is the only shape this property has, since a bound that does not hold is a
//! compile error rather than a value.
//!
//! It is worth its own target because the two paths genuinely bound the count
//! differently, `const Cardinal` against `Cardinal`. That difference is kept
//! off the trait declaration and confined to the impls, and the sealing on
//! `List` is what makes confining it safe rather than a weakening. Were the
//! bound on the trait, `length_of` below would need `N: const Cardinal` here
//! and `N: Cardinal` there, and no consumer could write one signature.

mod lists;

use lists::*;
use notko_hlist::{At, Cardinal, Concat, Cons, Empty, Length, List, Place, Position};

// The count-carrying signatures below are never called. A generic function's
// body and bounds are checked where it is declared, so declaring one is the
// assertion in full, and calling it would need a concrete count, which is the
// one thing that cannot be written once for both paths. Each path's own target
// instantiates them.

/// The count, from a bound naming only `Cardinal`.
#[allow(dead_code)]
fn length_of<L: Length<N>, N: Cardinal>() -> N {
    <L as Length<N>>::len()
}

/// A list, with nothing said about counting it. `List` is the bound a consumer
/// reaches for when it wants to accept a list and does not care how long it is.
fn takes_a_list<L: List>() {}

/// Appending, from a bound naming nothing but `Concat`.
fn appended<L: Concat<R>, R>() -> core::marker::PhantomData<<L as Concat<R>>::Out> {
    core::marker::PhantomData
}

/// A cardinal used through the trait alone, with no impl in sight.
#[allow(dead_code)]
fn stepped_twice<N: Cardinal>() -> N {
    N::ZERO.succ().succ()
}

/// A count that carries a length must be a cardinal, and this is where that is
/// checked without naming one: the body reads `N::ZERO`, which only resolves
/// because `Length`'s impls carry the bound. Sealing is what makes the compiler
/// able to see that from the outside.
#[allow(dead_code)]
fn a_length_implies_a_cardinal<L: Length<N>, N: Cardinal>() -> N {
    let _ = <L as Length<N>>::len();
    N::ZERO
}

/// The index, from a bound naming only `Position` and `Cardinal`. The count is
/// bounded the same two ways on the two paths, exactly as `Length`'s is, and
/// it is kept off the trait declaration for the same reason.
#[allow(dead_code)]
fn index_of<P: Position<N>, N: Cardinal>() -> N {
    <P as Position<N>>::index()
}

/// The member at a position, from a bound naming nothing but `At`. No count
/// appears, which is the arrangement: what sits there is the list's fact and
/// how far along it is is the position's, so a consumer wanting only the type
/// never names a number type it does not care about.
fn member_of<L: At<P, Member = M>, P: Place, M>(value: M) -> M {
    // Named in the body as well as in the bound, since a parameter appearing
    // only in a `where` clause reads to a linter as one nobody uses, and here
    // the bound is the whole point of the signature.
    let _ = core::marker::PhantomData::<L>;
    value
}

/// Both halves at once, which is the shape a consumer indexing a flat run of
/// values writes: the bound proves the type and hands over the number to reach
/// it with.
#[allow(dead_code)]
fn read_at<L: At<P, Member = M>, P: Position<N>, M, N: Cardinal>(values: &[M]) -> N {
    let _ = values;
    let _ = core::marker::PhantomData::<L>;
    <P as Position<N>>::index()
}

#[test]
fn the_bounds_above_compile_in_this_configuration() {
    // Instantiating what can be instantiated without a concrete count. The
    // count-carrying ones are exercised in each path's own target, because a
    // `Cardinal` impl is exactly what cannot be written once for both.
    takes_a_list::<Empty>();
    takes_a_list::<L0>();
    takes_a_list::<L5>();
    takes_a_list::<L32>();
    takes_a_list::<Nested>();
    takes_a_list::<WithBorrowing>();

    let _ = appended::<L2, L3>();
    let _ = appended::<Empty, L5>();
    let _ = appended::<Cons<A, Empty>, Empty>();

    // `At` carries no count, so unlike the others it can be instantiated here.
    let _: A = member_of::<L5, P0, A>(A);
    let _: E = member_of::<L5, P4, E>(E);
    let _: C = member_of::<L32, P31, C>(C);
}

#[test]
fn nothing_here_has_a_size() {
    // A list is a type-level object. If a cell ever grew a field the length
    // machinery would still work and the crate would have stopped being what
    // it says it is, so the claim is pinned rather than asserted in prose.
    assert_eq!(core::mem::size_of::<Empty>(), 0);
    assert_eq!(core::mem::size_of::<L1>(), 0);
    assert_eq!(core::mem::size_of::<L32>(), 0);
    assert_eq!(core::mem::size_of::<WithBorrowing>(), 0);
    assert_eq!(core::mem::size_of::<Nested>(), 0);
}
