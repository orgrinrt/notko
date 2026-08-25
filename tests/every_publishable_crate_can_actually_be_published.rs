//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The manifest properties `cargo publish` will not let past, checked here
//! instead of at the registry.
//!
//! A publish is the one thing in this repository that cannot be undone and
//! cannot be rehearsed: the first one for a crate is also the first time
//! anything resolves its dependencies against the registry rather than against
//! the working copy. So the conditions it would fail on are read off the
//! manifests, where a wrong one costs a test run instead of a yanked version.

use std::path::{Path, PathBuf};

/// Every manifest in the workspace, the root's included.
///
/// Read off the members list rather than by walking the directory, so a member
/// added later is checked without anybody remembering to add it here, and a
/// directory that is not a member is not checked at all.
fn manifests() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let name = format!("Cargo.{}", "toml");
    let text = std::fs::read_to_string(root.join(&name)).expect("workspace manifest");
    let members = text
        .split("members = [")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .expect("a members list");

    let mut out = vec![root.join(&name)];
    for entry in members.split(',') {
        let dir = entry.trim().trim_matches('"').trim();
        if dir.is_empty() || dir == "." {
            continue;
        }
        let candidate = root.join(dir).join(&name);
        assert!(
            candidate.is_file(),
            "the members list names {dir}, which has no manifest"
        );
        out.push(candidate);
    }
    assert!(out.len() > 1, "the members list was not read: {out:?}");
    out.sort();
    out
}

/// The `[dependencies]` and `[build-dependencies]` lines, which are the ones a
/// publish resolves. `[dev-dependencies]` are stripped from the published
/// manifest, so a path with no version there is fine and is how a proc-macro
/// crate tests against the crate that consumes it.
fn published_dependency_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut published = false;
    for (i, line) in text.lines().enumerate() {
        let t = line.trim();
        if t.starts_with('[') {
            published = matches!(t, "[dependencies]" | "[build-dependencies]")
                || t.ends_with(".dependencies]") && !t.contains("dev-dependencies");
            continue;
        }
        if published && !t.is_empty() && !t.starts_with('#') {
            out.push((i + 1, t.to_string()));
        }
    }
    out
}

#[test]
fn a_path_dependency_carries_the_version_the_registry_will_resolve() {
    // A path with no version beside it publishes as nothing at all: cargo
    // strips the path and has no requirement left to write, so the consumer
    // installs a crate missing a dependency it needs. Naming both means the
    // working copy is used here and the registry is used there, which is the
    // only arrangement where both work.
    let mut complaints = Vec::new();
    for manifest in manifests() {
        let text = std::fs::read_to_string(&manifest).unwrap();
        for (line, dep) in published_dependency_lines(&text) {
            if dep.contains("path =") && !dep.contains("version =") {
                complaints.push(format!("{}:{line}: {dep}", manifest.display()));
            }
        }
    }
    assert!(
        complaints.is_empty(),
        "a path dependency publishes as nothing unless a version sits beside \
         it:\n{}",
        complaints.join("\n")
    );
}

#[test]
fn nothing_published_depends_on_a_git_revision() {
    // crates.io refuses a git dependency outright, so one reaching a manifest
    // is a publish that fails at the last step of the sequence, after the
    // crates before it in the order have already gone out and cannot be taken
    // back.
    let mut complaints = Vec::new();
    for manifest in manifests() {
        let text = std::fs::read_to_string(&manifest).unwrap();
        for (line, dep) in published_dependency_lines(&text) {
            if dep.contains("git =") {
                complaints.push(format!("{}:{line}: {dep}", manifest.display()));
            }
        }
    }
    assert!(
        complaints.is_empty(),
        "crates.io refuses these:\n{}",
        complaints.join("\n")
    );
}

#[test]
fn the_reader_finds_the_dependencies_and_skips_the_dev_ones() {
    // The control for the reader above. Without it a parser that matched no
    // section would report every manifest clean for ever.
    let sample = "\
[package]\n\
name = \"x\"\n\
\n\
[dependencies]\n\
a = { path = \"../a\", version = \"1\" }\n\
b = \"2\"\n\
\n\
[dev-dependencies]\n\
c = { path = \"..\" }\n\
";
    let found = published_dependency_lines(sample);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].1.starts_with("a ="), "{found:?}");
    assert!(found[1].1.starts_with("b ="), "{found:?}");
    assert!(
        !found.iter().any(|(_, d)| d.starts_with("c =")),
        "a dev dependency was read as a published one: {found:?}"
    );

    let offending = "[dependencies]\nd = { path = \"../d\" }\n";
    let found = published_dependency_lines(offending);
    assert_eq!(found.len(), 1);
    assert!(found[0].1.contains("path =") && !found[0].1.contains("version ="));
}
