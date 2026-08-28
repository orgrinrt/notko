//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! A crate under `research/` never goes to a registry, and says so itself.
//!
//! The probes there are copies of real crates, held still so a panel could
//! check one thing about them. Two of them are called `notko` and carry this
//! crate's own version, which means a tool walking the tree for manifests
//! finds three crates by that name, two of them stale copies that have already
//! drifted from the real one.
//!
//! Nothing here is in the workspace's member list and nothing ships in a
//! tarball, so today the only cost is a reader's confusion. That is the whole
//! reason to say it in the file rather than to rely on where the file sits.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Whether a manifest actually sets `publish = false`, rather than mentioning
/// it.
///
/// A `contains` over the whole text says yes to a commented-out line, and a
/// commented-out one is the exact state this is here to catch: it is what a
/// manifest looks like on the way to losing the key. Found by the control
/// below, which passed against a manifest with the line commented out.
fn refuses_to_publish(text: &str) -> bool {
    text.lines().any(|l| {
        let l = l.trim();
        !l.starts_with('#') && l.starts_with("publish") && l.contains("false")
    })
}

/// Every manifest under `research/`, from git rather than from a directory
/// walk, so an untracked scratch copy is not mistaken for one of these.
fn probe_manifests() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "research"])
        .output()
        .expect("git could not be run");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.ends_with("/Cargo.toml") || *l == "research/Cargo.toml")
        .map(|l| root.join(l))
        .collect()
}

#[test]
fn every_manifest_under_research_refuses_to_publish() {
    let found = probe_manifests();
    let mut bad = Vec::new();
    for p in &found {
        let text = std::fs::read_to_string(p).expect("a tracked manifest reads");
        if !refuses_to_publish(&text) {
            bad.push(p.display().to_string());
        }
    }
    assert!(
        bad.is_empty(),
        "a probe crate does not refuse to publish:\n{}",
        bad.join("\n")
    );
}

#[test]
fn no_probe_shares_this_crate_s_name_without_saying_what_it_is() {
    // Two of them are called `notko`, which is allowed: a probe copying the
    // shipped crate in order to check something about it wants the same name,
    // and renaming it would change what was checked. What is not allowed is
    // one of them being reachable as a publishable crate under that name.
    let mut shared = 0usize;
    for p in probe_manifests() {
        let text = std::fs::read_to_string(&p).expect("a tracked manifest reads");
        let names_us = text
            .lines()
            .any(|l| l.trim_start().starts_with("name") && l.contains("\"notko\""));
        if names_us {
            shared += 1;
            assert!(
                refuses_to_publish(&text),
                "{} declares itself `notko` and does not refuse to publish",
                p.display()
            );
        }
    }
    assert!(
        shared > 0,
        "no probe names this crate, so this test read nothing"
    );
}

#[test]
fn the_checks_above_can_fail() {
    // The reader found manifests at all, and the needle it looks for is one a
    // manifest can genuinely lack. Without this both tests above pass over an
    // empty list.
    let found = probe_manifests();
    assert!(
        found.len() >= 4,
        "the reader found {} manifests under research",
        found.len()
    );
    let ours = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("this crate's own manifest reads");
    assert!(
        !refuses_to_publish(&ours),
        "the reader matches this crate too, so it cannot distinguish a probe"
    );
    // And the reader is not a substring scan. A commented-out key is what a
    // manifest looks like on the way to losing it, and it read as present.
    assert!(
        !refuses_to_publish("# publish = false\n"),
        "a commented key reads as set"
    );
    assert!(
        !refuses_to_publish("name = \"x\"\n"),
        "a manifest with no key reads as set"
    );
    assert!(
        refuses_to_publish("publish = false\n"),
        "a manifest that sets it reads as unset"
    );
    assert!(
        refuses_to_publish("publish=false\n"),
        "one spelling of it is missed"
    );
}
