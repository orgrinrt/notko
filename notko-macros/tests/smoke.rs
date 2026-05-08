//! Smoke tests for `#[profile(Tier)]`.
//!
//! Built-in tiers are covered here in the debug + not-internal mode: Hot and
//! Cold both rewrite to `Outcome<T, E>`; Warm is passthrough.
//!
//! Release+internal hot-path behavior (Just<T> + panic-on-Err) is harder to
//! exercise from a test binary, but covered by compile-only verification that
//! the cfg branches don't collide. A dedicated integration test crate with
//! its own `internal` feature would be needed to drive the release path.

#![feature(try_trait_v2)]

use notko::{Outcome, Just};
use notko_macros::profile;

#[derive(Debug, PartialEq, Eq)]
struct Oops;

// ---- Hot tier (debug mode) ----

#[profile(Hot)]
fn hot_ok(x: u32) -> Result<u32, Oops> {
    Ok(x + 1)
}

#[profile(Hot)]
fn hot_err(x: u32) -> Result<u32, Oops> {
    if x == 0 {
        return Err(Oops);
    }
    Ok(x)
}

#[test]
fn hot_returns_outcome_ok() {
    let o: Outcome<u32, Oops> = hot_ok(41);
    assert_eq!(o, Outcome::Ok(42));
}

#[test]
fn hot_returns_outcome_err() {
    let o: Outcome<u32, Oops> = hot_err(0);
    assert_eq!(o, Outcome::Err(Oops));
}

// ---- Cold tier ----

#[profile(Cold)]
fn cold_ok(x: u32) -> Result<u32, Oops> {
    Ok(x * 2)
}

#[profile(Cold)]
fn cold_err(x: u32) -> Result<u32, Oops> {
    if x == 0 {
        return Err(Oops);
    }
    Ok(x)
}

#[test]
fn cold_returns_outcome_ok() {
    let o: Outcome<u32, Oops> = cold_ok(21);
    assert_eq!(o, Outcome::Ok(42));
}

#[test]
fn cold_returns_outcome_err() {
    let o: Outcome<u32, Oops> = cold_err(0);
    assert_eq!(o, Outcome::Err(Oops));
}

// ---- Warm tier ----

#[profile(Warm)]
fn warm_ok(x: u32) -> Result<u32, Oops> {
    Ok(x)
}

#[test]
fn warm_is_passthrough() {
    let r: Result<u32, Oops> = warm_ok(42);
    assert_eq!(r, Ok(42));
}

// ---- custom tier via notko-optimizers/Trace.rs ----
//
// The test fixture at notko-macros/notko-optimizers/Trace.rs declares
// based_on = "Cold", so `#[profile(Trace)]` should rewrite the function
// the same way cold would.

#[profile(Trace)]
fn trace_ok(x: u32) -> Result<u32, Oops> {
    Ok(x + 100)
}

#[test]
fn custom_trace_tier_resolves_and_rewrites_like_cold() {
    let o: Outcome<u32, Oops> = trace_ok(42);
    assert_eq!(o, Outcome::Ok(142));
}

// ---- Suppress unused warnings; Just is re-exported via tests but the hot
// release path isn't exercised here. ----
#[allow(dead_code)]
fn _unused_just_marker() -> Just<u32> {
    Just::new(0)
}
