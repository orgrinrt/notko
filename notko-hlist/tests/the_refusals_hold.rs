//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The refusals, as builds that fail.
//!
//! Everything this crate guarantees is a refusal: a type is not in the list, a
//! tail is not a list, a count is not a cardinal. None of those can be written
//! as a passing assertion, because the illegal state has no value to inspect,
//! so a positive suite alone would pass equally well for a `Contains`
//! implemented for everything.
//!
//! Each case sits in `tests/compile_fail/` with the diagnostic it must produce
//! beside it in a `.stderr` file, and `TRYBUILD=overwrite cargo test --test
//! the_refusals_hold` regenerates those when a compiler message legitimately
//! changes wording. The `.stderr` files also hold the crate's own
//! `on_unimplemented` notes, which is the only place those are checked.
//!
//! This one is a check about the repository rather than about the crate: it
//! reads a fixture tree and the toolchain pin, neither of which a package
//! carries.

/// The toolchain the `.stderr` files were blessed on.
///
/// Read from `rust-toolchain.toml` rather than written twice, so bumping the
/// pin cannot leave this behind. The file is at the workspace root, one level
/// above this crate.
fn the_pinned_channel() -> String {
    let manifest = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../rust-toolchain.toml"
    ))
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
    // A diagnostic's wording is not stable across compiler versions and these
    // files hold the wording, so off the pin every case reports a mismatch
    // while every refusal still holds. That is the harness being wrong about
    // the toolchain rather than the crate being wrong about anything.
    //
    // So the assertion is skipped off the pin and says so, loudly, because a
    // skip reporting `ok` is a suite claiming to have checked something it did
    // not look at.
    let pinned = the_pinned_channel();
    let running = the_running_toolchain();
    if !running.starts_with(&pinned) {
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
        cases >= 12,
        "the fixture directory holds {cases} cases, too few to be the set"
    );
    assert!(
        missing.is_empty(),
        "a case with no blessed diagnostic:\n{}",
        missing.join("\n")
    );
}

#[test]
fn the_blessed_diagnostics_carry_this_crate_s_own_notes() {
    // The `on_unimplemented` notes are the part of the surface a consumer
    // meets when they get it wrong, and nothing else checks their wording: a
    // note deleted from the source shows up here as a `.stderr` that no longer
    // matches, but only if a case actually reaches one.
    //
    // So this asserts the fixtures cover all five traits that carry a note,
    // which is what makes the blessed files a check on the notes rather than
    // on rustc's generic phrasing alone.
    let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/compile_fail"));
    let mut blessed = String::new();
    for entry in std::fs::read_dir(dir).expect("the fixture directory") {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_some_and(|e| e == "stderr") {
            blessed.push_str(&std::fs::read_to_string(&path).expect("a blessed diagnostic"));
        }
    }
    assert!(
        !blessed.is_empty(),
        "no blessed diagnostics at all, so this checks nothing"
    );
    for fragment in [
        "does not hold",
        "does not hold every member of",
        "has no length in",
        "cannot append",
        "is not a list",
        // The sealing is the load-bearing one and rustc words it, not us, so
        // this is the check that a fixture actually reaches a sealed impl
        // rather than failing earlier for some other reason.
        "is a \"sealed trait\"",
    ] {
        assert!(
            blessed.contains(fragment),
            "no blessed diagnostic reaches the note containing {fragment:?}, \
             so that note's wording is unchecked"
        );
    }
}
