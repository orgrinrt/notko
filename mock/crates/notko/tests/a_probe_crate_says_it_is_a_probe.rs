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
    // The mock directory, two levels above this crate, which holds `research/`.
    let root = &Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
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

/// Every `path = "..."` a manifest line sets, with whether the line is an
/// inline dependency table, which must name a crate rather than a file.
fn paths_named(text: &str) -> Vec<(String, bool)> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| {
            let at = l.find("path")?;
            let rest = l[at + "path".len() ..].trim_start().strip_prefix('=')?;
            let rest = rest.trim_start().strip_prefix('"')?;
            let end = rest.find('"')?;
            Some((rest[.. end].to_string(), l.contains('{')))
        })
        .collect()
}

/// A probe that cannot resolve its own manifest cannot be rebuilt, and a
/// design citing it then cites something nobody can check. Moving the crates
/// under `mock/crates/` left seven sketches pointing at directories that were
/// gone, and nothing failed until a reviewer ran `cargo metadata` on them.
#[test]
fn every_path_a_research_manifest_names_exists() {
    let found = probe_manifests();
    let mut bad = Vec::new();
    let mut deps = 0;
    for p in &found {
        let text = std::fs::read_to_string(p).expect("a tracked manifest reads");
        let dir = p.parent().expect("a manifest sits in a directory");
        for (rel, is_dep) in paths_named(&text) {
            let target = dir.join(&rel);
            let ok = if is_dep {
                deps += 1;
                target.join("Cargo.toml").is_file()
            } else {
                target.exists()
            };
            if !ok {
                bad.push(format!("{}: {}", p.display(), rel));
            }
        }
    }
    assert!(
        deps > 0,
        "no manifest names a path dependency, so this read nothing"
    );
    assert!(
        bad.is_empty(),
        "a research manifest names a path that does not resolve:\n{}",
        bad.join("\n")
    );
}

#[test]
fn the_path_reader_can_fail() {
    let read = paths_named("notko = { path = \"../../..\", features = [\"x\"] }\n");
    assert_eq!(read, vec![("../../..".to_string(), true)]);
    let read = paths_named("path = \"src/lib.rs\"\n");
    assert_eq!(read, vec![("src/lib.rs".to_string(), false)]);
    assert!(
        paths_named("# notko = { path = \"gone\" }\n").is_empty(),
        "a commented line reads as a path"
    );
    let gone = Path::new(env!("CARGO_MANIFEST_DIR")).join("../notko-hlist-does-not-exist");
    assert!(
        !gone.join("Cargo.toml").is_file(),
        "a missing directory reads as a crate"
    );
}
