//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Contains` and `ContainsAll`: what a list holds and what it does not.
//!
//! Every positive case here is a bound the compiler has to discharge, so the
//! file failing to build is the failure. The negative cases cannot live here
//! at all, because a bound that does not hold is a build error rather than a
//! value: those are in `tests/compile_fail/` and are run by
//! `the_refusals_hold`.
//!
//! Both halves are needed. A suite of positives alone passes for a `Contains`
//! implemented for everything, which is the shape a `#[marker]` trait makes
//! easy to write by accident.

use notko_hlist::{Concat, Cons, Contains, ContainsAll, Empty};

/// A bound written the way a consumer writes it. Instantiating it is the
/// assertion; there is nothing to check at run time.
fn holds<L: Contains<X>, X>() {}

fn holds_all<L: ContainsAll<M>, M>() {}

struct A;
struct B;
struct C;
struct D;
struct E;
struct Absent;

type One = Cons<A, Empty>;
type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
type Five = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Empty>>>>>;

/// The same head twice, which is where the head match and the tail match both
/// apply to one list. Under `#[marker]` the solver may pick either and the
/// program means the same thing; without it this is the coherence error the
/// attribute exists to permit.
type Duplicated = Cons<A, Cons<B, Cons<A, Empty>>>;

/// A head that is itself a list. Membership walks the spine and never into a
/// head, so `Nested` holds `Three` and does not hold `A`.
type Nested = Cons<Three, Cons<One, Empty>>;

#[test]
fn a_single_cell_holds_its_head() {
    holds::<One, A>();
}

#[test]
fn every_position_is_reachable() {
    // First, middle and last, because a walk that stops one short passes for
    // everything but the last and a walk that never recurses passes for
    // nothing but the first.
    holds::<Three, A>();
    holds::<Three, B>();
    holds::<Three, C>();
    holds::<Five, A>();
    holds::<Five, C>();
    holds::<Five, E>();
}

#[test]
fn a_head_appearing_twice_is_held_once_as_far_as_this_is_concerned() {
    // Both impls apply. That the bound discharges at all is the assertion:
    // without `#[marker]` this list is where coherence refuses.
    holds::<Duplicated, A>();
    holds::<Duplicated, B>();
}

#[test]
fn membership_walks_the_spine_and_not_the_heads() {
    holds::<Nested, Three>();
    holds::<Nested, One>();
    // That `Nested` does not hold `A` is the other half and cannot be written
    // here. `tests/compile_fail/membership_does_not_descend_into_a_head.rs`.
}

#[test]
fn every_list_holds_every_member_of_nothing() {
    // The base impl is a blanket over lists, so this says nothing about which
    // list it is and a bound of `ContainsAll<Empty>` is not worth writing. It
    // is pinned because it is what makes the recursion terminate, so a later
    // narrowing of the base impl has to be a decision rather than an accident.
    holds_all::<Empty, Empty>();
    holds_all::<One, Empty>();
    holds_all::<Three, Empty>();
    // The blanket stops at lists, which is the half that took a correction:
    // `u8` holds every member of nothing in the sense that the sentence is
    // vacuously true, and letting it say so would have made the trait
    // implementable for a type that is not a list.
    // `tests/compile_fail/a_bare_type_holds_nothing_at_all.rs`.
}

#[test]
fn a_list_holds_every_member_of_itself() {
    holds_all::<One, One>();
    holds_all::<Three, Three>();
    holds_all::<Five, Five>();
    holds_all::<Duplicated, Duplicated>();
}

#[test]
fn a_subset_may_be_named_in_any_order_and_may_repeat() {
    // Each member is checked on its own, so the second list is a bag rather
    // than a subsequence.
    holds_all::<Five, Cons<C, Cons<A, Empty>>>();
    holds_all::<Five, Cons<E, Cons<D, Cons<C, Cons<B, Cons<A, Empty>>>>>>();
    holds_all::<Five, Cons<A, Cons<A, Cons<A, Empty>>>>();
}

#[test]
fn appending_can_only_add_members() {
    // The two operations agree, which is the property a consumer composing
    // bundles actually relies on: a set grown by concatenation still holds
    // everything either side held.
    type Left = Cons<A, Cons<B, Empty>>;
    type Right = Cons<C, Cons<D, Empty>>;
    type Both = <Left as Concat<Right>>::Out;

    holds_all::<Both, Left>();
    holds_all::<Both, Right>();
    holds::<Both, A>();
    holds::<Both, D>();
}

#[test]
fn a_deep_list_stays_within_the_default_recursion_limit() {
    // Membership recurses once per cell, and a consumer gets the default limit
    // before it reaches for the raised one the diagnostics mention. Thirty-two
    // is comfortably inside it, and the last cell is the one that costs the
    // full walk.
    type Eight<T> = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<A, Cons<B, Cons<C, T>>>>>>>>;
    type Deep = Eight<Eight<Eight<Cons<Absent, Empty>>>>;

    holds::<Deep, Absent>();
    holds_all::<Deep, Cons<Absent, Cons<E, Empty>>>();
}
