//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The refusals, as builds that fail.
//!
//! Four of these lived as commented-out functions and as sentences in doc
//! comments, which is a refusal nothing pins: a bound loosened later restores
//! the illegal state and every remaining test still passes, because none of
//! them names it.
//!
//! A build failure is expressible, so it is expressed. Each case sits in
//! `tests/compile_fail/` with the diagnostic it must produce beside it in a
//! `.stderr` file, and `TRYBUILD=overwrite cargo test --test
//! the_seals_hold_from_outside` regenerates those when a compiler message
//! legitimately changes wording.
//!
//! This one is a check about the repository rather than about the crate: it
//! reads a fixture tree, which a package does not carry.

#[test]
fn every_refusal_still_refuses() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
