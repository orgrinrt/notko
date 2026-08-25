//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Where `?` stops working as you move down the ladder.
//!
//! Its own target because `?` needs `try_trait_v2`, which is off by default,
//! and `required-features` is the only thing that actually skips a target.
//!
//! The one case here is a gap rather than a refusal, and it is written as a
//! build that must fail so it cannot be forgotten. `tests/try_smoke.rs` covers
//! what does work: `?` on each of the three, back into its own kind.

#[test]
fn the_gap_is_still_there() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail_try/*.rs");
}
