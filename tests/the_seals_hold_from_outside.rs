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

/// The toolchain rustup has actually selected, as it names it.
///
/// The toolchain name rather than `rustc -V`, because the version string
/// carries the commit date and the toolchain carries the release date, and
/// they differ by a day. Comparing the pin against the wrong one never matches.
///
/// It comes back with the host triple appended, so callers compare with
/// `starts_with` rather than for equality.
fn the_running_toolchain() -> String {
    let out = std::process::Command::new("rustup")
        .args(["show", "active-toolchain"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("rustup could not be run");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned()
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
    // The date in a nightly's name is the day it was cut and `rustc -V` prints
    // the day its commit landed, which is the day before. So a first attempt at
    // this matched the pin's date against the version string, never matched,
    // and fell through to "am I on any nightly at all", which ran the
    // assertion on every nightly including ones the files were not blessed on.
    //
    // The toolchain file is what selects the compiler, so asking rustup which
    // toolchain is in force is the question with an answer.
    let pinned = the_pinned_channel();
    let running = the_running_toolchain();
    if !running.starts_with(&pinned) {
        // Loud, because a skip that reports `ok` is a suite claiming to have
        // checked something it did not look at.
        panic!(
            "these diagnostics are blessed on {pinned} and this is {running}.\n\
             the refusals are unaffected by the wording, so this is the harness \
             refusing to report a pass it did not earn.\n\
             run on the pin, or pass --skip every_refusal_still_refuses."
        );
    }
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}

#[test]
fn the_refusals_are_pinned_to_a_toolchain_that_is_named() {
    // The check above skips itself off the pin, so the pin has to exist and
    // has to be readable, or the skip becomes unconditional and silent.
    let pinned = the_pinned_channel();
    assert!(!pinned.is_empty(), "the toolchain pin names no channel");
    let running = the_running_toolchain();
    assert!(
        !running.is_empty(),
        "rustup named no toolchain, so nothing above can decide"
    );
    // Not that we are on the pin: this one holds on every toolchain, and it is
    // the check above that decides whether the wording may be asserted. What
    // has to hold here is that both halves of that decision have an answer, or
    // the comparison is between two empty strings and matches everything.
    assert!(!running.is_empty(), "rustup named no toolchain");
    assert!(
        running.contains('-') || running.starts_with("stable"),
        "rustup's answer does not look like a toolchain name: {running}"
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
