//! Smoke tests for the three-tier fallibility ladder.
//!
//! Validates the headline behaviour: `?` on `Just` is a no-op extraction,
//! `?` on `Maybe` short-circuits on `Isnt`, `?` on `Outcome` short-circuits
//! on `Err`.

#![cfg(feature = "try_trait_v2")]
#![feature(try_trait_v2)]

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
