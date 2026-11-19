//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Concat`: that appending does what appending does.
//!
//! Every check here is a type equality, so the whole file is decided at build
//! time and the `#[test]` functions exist only so a reader running the suite
//! sees names rather than an empty list. A wrong `Out` is a compile error in
//! this file, not a failing assertion.
//!
//! Ungated: `Concat` needs neither feature, so it is checked in both
//! configurations.

use notko_hlist::{Concat, Cons, Empty};

/// Type equality, expressed so the compiler decides it.
///
/// The reflexive impl is the only one, so `A: SameAs<B>` holds exactly when
/// the two are the same type after normalisation. Calling `same::<X, Y>()`
/// with a wrong pair fails to build, which is the assertion.
trait SameAs<T> {}
impl<T> SameAs<T> for T {}

fn same<A: SameAs<B>, B>() {}

struct A;
struct B;
struct C;
struct D;

type Cat<L, R> = <L as Concat<R>>::Out;

type One = Cons<A, Empty>;
type Two = Cons<A, Cons<B, Empty>>;
type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
type Right = Cons<C, Cons<D, Empty>>;

#[test]
fn the_empty_list_is_an_identity_on_both_sides() {
    same::<Cat<Empty, Three>, Three>();
    same::<Cat<Three, Empty>, Three>();
    same::<Cat<Empty, Empty>, Empty>();
}

#[test]
fn order_is_preserved_and_the_left_side_comes_first() {
    same::<Cat<Two, Right>, Cons<A, Cons<B, Cons<C, Cons<D, Empty>>>>>();
    // The other way round is a different list, which is the whole content of
    // "order is preserved". Asserting only the first equality would pass for
    // an implementation that sorted, deduplicated or reversed.
    same::<Cat<Right, Two>, Cons<C, Cons<D, Cons<A, Cons<B, Empty>>>>>();
}

#[test]
fn appending_associates() {
    same::<Cat<Cat<One, Two>, Right>, Cat<One, Cat<Two, Right>>>();
    same::<Cat<Cat<Empty, Two>, Empty>, Two>();
    same::<Cat<Cat<Three, Empty>, Three>, Cat<Three, Three>>();
}

#[test]
fn nothing_is_deduplicated() {
    // Appending a list to one that already holds the same types gives a list
    // holding them twice. A set would collapse this and a list must not.
    same::<Cat<Two, Two>, Cons<A, Cons<B, Cons<A, Cons<B, Empty>>>>>();
    same::<Cat<One, One>, Cons<A, Cons<A, Empty>>>();
}

#[test]
fn a_head_that_is_itself_a_list_is_an_ordinary_member() {
    // The walk goes along the spine and never into a head, so appending does
    // not flatten. Were it to descend, this would be a list of four.
    same::<Cat<Cons<Two, Empty>, Cons<Three, Empty>>, Cons<Two, Cons<Three, Empty>>>();
}

#[test]
fn the_right_side_is_not_walked_and_need_not_be_a_list() {
    // `Concat` recurses on the left only, so the right is whatever it is. That
    // is not a hole to be closed: `<Empty as Concat<T>>::Out = T` is what makes
    // the empty list an identity, and bounding the right would cost that
    // without buying anything, since a non-list right produces a list nothing
    // else in the crate will accept.
    same::<Cat<Empty, u8>, u8>();
    same::<Cat<One, u8>, Cons<A, u8>>();
}

#[test]
fn appending_one_cell_at_a_time_reaches_the_same_list() {
    // The associativity check above compares two groupings of the same three
    // lists. This compares a fold from the left against the literal answer,
    // which is the shape a consumer's builder actually produces.
    type Step1 = Cat<Empty, One>;
    type Step2 = Cat<Step1, Cons<B, Empty>>;
    type Step3 = Cat<Step2, Cons<C, Empty>>;
    same::<Step3, Three>();
}
