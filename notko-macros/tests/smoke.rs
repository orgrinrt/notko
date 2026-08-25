//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Smoke tests for `#[profile(Tier)]`, in both configurations it rewrites for.
//!
//! Cold rewrites to `Outcome<T, E>` and Warm to `Maybe<T>`, whatever the
//! configuration. Hot is the one that moves: `Outcome<T, E>` by default, and
//! `Just<T>` with the error arm panicking under `--features internal` in a
//! build with `debug_assertions` off.
//!
//! Both arms are asserted here rather than one of them. The cfg the macro
//! emits is evaluated in the crate the macro expanded into, which is this test
//! crate, so the same `cfg` decides which assertions are compiled.
//!
//! Run the second arm with:
//!
//! ```text
//! cargo test -p notko-macros --features internal --release
//! ```

#[cfg(all(feature = "internal", not(debug_assertions)))]
use notko::Just;
use notko::Outcome;
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

#[cfg(any(not(feature = "internal"), debug_assertions))]
mod hot_default {
    use super::*;

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
}

#[cfg(all(feature = "internal", not(debug_assertions)))]
mod hot_internal_release {
    use super::*;

    #[test]
    fn hot_returns_just() {
        let j: Just<u32> = hot_ok(41);
        assert_eq!(j.into_inner(), 42);
    }

    #[test]
    #[should_panic(expected = "Oops")]
    fn hot_panics_on_the_error_arm() {
        // The whole point of the tier: there is no error to return, so the
        // arm that would have produced one aborts instead.
        let _: Just<u32> = hot_err(0);
    }
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

// ---- custom tier via notko-optimisers/Trace.rs ----
//
// The test fixture at notko-macros/notko-optimisers/Trace.rs declares
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

// ---- Warm tier ----
//
// The tier table names Warm as `Maybe<T>`, so the attribute produces one. The
// error is discarded rather than carried, which is what choosing this tier
// means: a caller who wants the error asks for Cold.

#[profile(Warm)]
fn warm_ok(x: u32) -> Result<u32, Oops> {
    Ok(x + 1)
}

#[profile(Warm)]
fn warm_err(x: u32) -> Result<u32, Oops> {
    if x == 0 {
        return Err(Oops);
    }
    Ok(x)
}

mod warm {
    use super::*;
    use notko::Maybe;

    #[test]
    fn warm_returns_maybe_is() {
        // The annotation is the assertion: were the rewrite still passthrough,
        // this would be a `Result` and would not compile as a `Maybe`.
        let m: Maybe<u32> = warm_ok(41);
        assert_eq!(m, Maybe::Is(42));
    }

    #[test]
    fn warm_discards_the_error() {
        let m: Maybe<u32> = warm_err(0);
        assert_eq!(m, Maybe::Isnt);
    }

    #[test]
    fn warm_keeps_the_ok_path_through_an_early_return() {
        let m: Maybe<u32> = warm_err(7);
        assert_eq!(m, Maybe::Is(7));
    }
}
