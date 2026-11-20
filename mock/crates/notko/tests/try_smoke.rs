//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Smoke tests for the three-tier fallibility ladder.
//!
//! Validates the headline behaviour: `?` on `Just` is a no-op extraction,
//! `?` on `Maybe` short-circuits on `Isnt`, `?` on `Outcome` short-circuits
//! on `Err`.

// Declared in `Cargo.toml` with `required-features = ["try_trait_v2"]`, so cargo
// skips this target when the feature is off. A `#![cfg]` at the crate root
// instead compiled it to an empty binary reporting zero tests, which reads as
// "nothing to check here" rather than "not applicable to this configuration".
//
// No `#![feature(try_trait_v2)]`. The gate belongs to notko's own build; a
// consumer using `?` on these types needs no nightly feature of its own, and
// naming it here wrongly implied otherwise.

use notko::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct Oops;

fn hot_chain(j: Just<u32>) -> Just<u32> {
    let value = j?;
    Just::new(value + 1)
}

fn warm_chain(m: Maybe<u32>) -> Maybe<u32> {
    let value = m?;
    Maybe::Is(value + 1)
}

fn cold_chain(o: Outcome<u32, Oops>) -> Outcome<u32, Oops> {
    let value = o?;
    Outcome::Ok(value + 1)
}

#[test]
fn just_question_mark_is_noop() {
    let out = hot_chain(Just::new(41));
    assert_eq!(out.into_inner(), 42);
}

#[test]
fn maybe_question_mark_is_continues() {
    assert_eq!(warm_chain(Maybe::Is(41)), Maybe::Is(42));
}

#[test]
fn maybe_question_mark_breaks_on_isnt() {
    assert_eq!(warm_chain(Maybe::Isnt), Maybe::Isnt);
}

#[test]
fn outcome_question_mark_continues() {
    assert_eq!(cold_chain(Outcome::Ok(41)), Outcome::Ok(42));
}

#[test]
fn outcome_question_mark_breaks_on_err() {
    assert_eq!(cold_chain(Outcome::Err(Oops)), Outcome::Err(Oops));
}

#[test]
fn maybe_map() {
    let m: Maybe<u32> = Maybe::Is(7);
    assert_eq!(m.map(|v| v * 2), Maybe::Is(14));
    let n: Maybe<u32> = Maybe::Isnt;
    assert_eq!(n.map(|v| v * 2), Maybe::Isnt);
}

#[test]
fn outcome_map_and_map_err() {
    let o: Outcome<u32, Oops> = Outcome::Ok(7);
    assert_eq!(o.map(|v| v * 2), Outcome::Ok(14));
    let e: Outcome<u32, Oops> = Outcome::Err(Oops);
    assert_eq!(e.map_err(|_| 99u32), Outcome::Err(99));
}

/// `?` on a fallible value, inside a function narrowed to [`Just`].
///
/// This used to be a catalogued gap, written as a build that must fail: `Just`
/// carried `FromResidual<JustResidual>` and nothing else, so the most ordinary
/// line anybody writes in a fallible function did not compile there.
///
/// It is where the hot strategy's two arms parted. The attribute emits the
/// function twice, once returning `Outcome` and once returning `Just`, and
/// `let v = f()?;` compiled in the arm that is tested and not in the arm that
/// ships. The consumer met it on their own release build.
///
/// The panic is what the release arm already does with a written-out `Err`, so
/// the operator now means there what it has always meant there.
mod the_ladder_composes {
    use notko::{Just, Maybe, Outcome};

    #[derive(Debug, PartialEq)]
    struct Oops;

    fn fallible(ok: bool) -> Outcome<u32, Oops> {
        if ok { Outcome::Ok(7) } else { Outcome::Err(Oops) }
    }

    fn optional(some: bool) -> Maybe<u32> {
        if some { Maybe::Is(7) } else { Maybe::Isnt }
    }

    fn through_outcome(ok: bool) -> Just<u32> {
        let n = fallible(ok)?;
        Just::new(n * 2)
    }

    fn through_maybe(some: bool) -> Just<u32> {
        let n = optional(some)?;
        Just::new(n * 2)
    }

    #[test]
    fn the_value_passes_through() {
        assert_eq!(through_outcome(true).into_inner(), 14);
        assert_eq!(through_maybe(true).into_inner(), 14);
    }

    #[test]
    #[should_panic(expected = "hot path invariant violated")]
    fn an_error_panics_rather_than_propagating() {
        // The narrowed function has nowhere to propagate to, which is the
        // whole of what narrowing means. Panicking is what the strategy does
        // with a failure it was told could not happen.
        let _ = through_outcome(false);
    }

    #[test]
    #[should_panic(expected = "hot path invariant violated")]
    fn an_absence_panics_too() {
        let _ = through_maybe(false);
    }
}
