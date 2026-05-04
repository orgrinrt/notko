//! Tier-symmetric combinator coverage for [`Just`].
//!
//! `Just<T>`'s combinators degenerate to the trivial branch (the
//! infallible side) of `Maybe` / `Outcome`. These tests pin that
//! degeneracy so future drift surfaces as a test failure.

use notko::{Just, Maybe, Outcome};

#[test]
fn predicate_pair_is_constant() {
    let j = Just::new(42_i32);
    assert!(j.is_ok());
    assert!(!j.is_err());
    assert!(j.is_some());
    assert!(!j.is_none());
}

#[test]
fn inspect_observes_and_passes_through() {
    let mut seen = 0_i32;
    let j = Just::new(7_i32).inspect(|v| seen = *v);
    assert_eq!(seen, 7);
    assert_eq!(j.into_inner(), 7);
}

#[test]
fn and_then_runs_closure() {
    let j = Just::new(3_i32).and_then(|v| Just::new(v * 2));
    assert_eq!(j.into_inner(), 6);
}

#[test]
fn or_and_or_else_keep_self() {
    let j = Just::new(1_i32).or(Just::new(99));
    assert_eq!(j.into_inner(), 1);

    let j = Just::new(1_i32).or_else(|| Just::new(99));
    assert_eq!(j.into_inner(), 1);
}

#[test]
fn unwrap_variants_return_inner() {
    assert_eq!(Just::new(42_i32).unwrap_or(0), 42);
    assert_eq!(Just::new(42_i32).unwrap_or_else(|| 0), 42);
    assert_eq!(Just::new(42_i32).unwrap_or_default(), 42);
    assert_eq!(Just::new(42_i32).expect("never"), 42);
}

#[test]
fn map_or_variants_apply_f() {
    let mut default_called = 0;
    let v = Just::new(3_i32).map_or(0, |x| x * 2);
    assert_eq!(v, 6);

    let v = Just::new(3_i32).map_or_else(
        || {
            default_called += 1;
            0
        },
        |x| x * 4,
    );
    assert_eq!(v, 12);
    assert_eq!(default_called, 0);
}

#[test]
fn ok_projects_to_maybe_is() {
    match Just::new(7_i32).ok() {
        Maybe::Is(v) => assert_eq!(v, 7),
        Maybe::Isnt => panic!("Just::ok must project to Maybe::Is"),
    }
}

#[test]
fn ok_or_variants_project_to_outcome_ok() {
    match Just::new(7_i32).ok_or("never") {
        Outcome::Ok(v) => assert_eq!(v, 7),
        Outcome::Err(_) => panic!("Just::ok_or must project to Outcome::Ok"),
    }
    match Just::new(7_i32).ok_or_else(|| "never") {
        Outcome::Ok(v) => assert_eq!(v, 7),
        Outcome::Err(_) => panic!("Just::ok_or_else must project to Outcome::Ok"),
    }
}

#[test]
fn into_iter_yields_once() {
    let j = Just::new(99_i32);
    let mut iter = j.into_iter();
    assert_eq!(iter.next(), Some(99));
    assert_eq!(iter.next(), None);
}

#[test]
fn ref_into_iter_borrows() {
    let j = Just::new(11_i32);
    let mut sum = 0;
    for v in &j {
        sum += *v;
    }
    assert_eq!(sum, 11);
    assert_eq!(j.into_inner(), 11);
}

#[test]
fn iter_size_hint_and_len() {
    let j = Just::new(5_i32);
    let mut iter = j.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&5));
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn as_mut_borrow_mutates_inner() {
    let mut j = Just::new(1_i32);
    {
        let r = j.as_mut();
        *r.into_inner() = 42;
    }
    assert_eq!(j.into_inner(), 42);
}
