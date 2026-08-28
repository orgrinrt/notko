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

/// The toolchain the `.stderr` files were blessed on.
///
/// Read from `rust-toolchain.toml` rather than written twice, so bumping the
/// pin cannot leave this behind.
fn the_pinned_channel() -> String {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/rust-toolchain.toml"))
            .expect("this repository pins a toolchain");
    manifest
        .lines()
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("channel")?
                .trim_start()
                .strip_prefix('=')?
                .trim()
                .trim_matches('"')
                .to_owned()
                .into()
        })
        .expect("the pin names a channel")
}

/// The compiler actually running, as `rustc -V` reports it.
fn the_running_compiler() -> String {
    let out = std::process::Command::new(std::env::var("RUSTC").unwrap_or("rustc".into()))
        .arg("-V")
        .output()
        .expect("rustc could not be run");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

#[test]
fn every_refusal_still_refuses() {
    // A diagnostic's wording is not stable across compiler versions, and these
    // files hold the wording. Stable 1.94 says "the associated item `NONE`
    // exists" where the pinned nightly says "the associated function or
    // constant `NONE` exists", so every case reports a mismatch there while
    // every refusal still holds. That is the harness being wrong about the
    // toolchain, not the crate being wrong about anything, and it was red in
    // exactly the configuration the readme tells a stable reader to use.
    //
    // So the assertion is skipped off the pin and says so. What is not skipped
    // is `the_refusals_are_pinned_to_a_toolchain_that_is_named`, which holds
    // everywhere.
    let pinned = the_pinned_channel();
    let running = the_running_compiler();
    let on_the_pin = running.contains(pinned.trim_start_matches("nightly-"))
        || (pinned.starts_with("nightly") && running.contains("nightly"));
    if !on_the_pin {
        eprintln!(
            "skipped: these diagnostics are blessed on {pinned} and this is {running}. \
             the refusals are unaffected; run on the pin to check their wording."
        );
        return;
    }
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}

#[test]
fn the_refusals_are_pinned_to_a_toolchain_that_is_named() {
    // The check above skips itself off the pin, so the pin has to exist and
    // has to be readable, or the skip becomes unconditional and silent.
    let pinned = the_pinned_channel();
    assert!(!pinned.is_empty(), "the toolchain pin names no channel");
    assert!(
        !the_running_compiler().is_empty(),
        "rustc reported no version, so the skip above cannot decide anything"
    );
}

#[test]
fn every_case_carries_the_diagnostic_it_must_produce() {
    // A `.rs` with no `.stderr` beside it is a case trybuild will happily
    // create the file for on its first run, which turns whatever the compiler
    // said that day into the expectation. This is the check that a case was
    // blessed deliberately, and it holds on every toolchain.
    let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/compile_fail"));
    let mut missing = Vec::new();
    let mut cases = 0usize;
    for entry in std::fs::read_dir(dir).expect("the fixture directory") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            cases += 1;
            if !path.with_extension("stderr").is_file() {
                missing.push(path.display().to_string());
            }
        }
    }
    assert!(
        cases >= 5,
        "the fixture directory holds {cases} cases, too few to be the set"
    );
    assert!(
        missing.is_empty(),
        "a case with no blessed diagnostic:\n{}",
        missing.join("\n")
    );
}
