//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What the readmes claim about installing these crates, against what the
//! manifests say.
//!
//! An install line is the first thing anybody runs and the last thing anybody
//! re-reads. It went wrong twice here in one week: the badges pointed at a
//! registry the crate was not on, and then the git dependency that replaced
//! them outlived the release that made it wrong. Both were true when written
//! and neither announced that it had stopped being.

use std::fs;
use std::path::{Path, PathBuf};

/// The workspace root, from this file rather than from the working directory,
/// which differs between `cargo test` and a runner invoking the binary.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every crate in the workspace, as its directory and its package name.
fn crates() -> Vec<(PathBuf, String)> {
    let manifest = fs::read_to_string(root().join("Cargo.toml")).expect("workspace manifest");
    let members = manifest
        .split("members = [")
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .expect("a members list");

    members
        .split(',')
        .filter_map(|entry| {
            let dir = entry.trim().trim_matches('"').trim();
            if dir.is_empty() {
                return None;
            }
            let path = if dir == "." { root() } else { root().join(dir) };
            let name = package_name(&path.join("Cargo.toml"));
            Some((path, name))
        })
        .collect()
}

/// The `name` under `[package]`, read as a line rather than parsed, which is
/// enough for four manifests we write ourselves.
fn package_name(manifest: &Path) -> String {
    let text = fs::read_to_string(manifest).unwrap_or_else(|_| panic!("{}", manifest.display()));
    text.lines()
        .find_map(|line| line.strip_prefix("name = "))
        .map(|value| value.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| panic!("no package name in {}", manifest.display()))
}

/// The version every crate here shares, from `[workspace.package]`.
fn workspace_version() -> String {
    let text = fs::read_to_string(root().join("Cargo.toml")).expect("workspace manifest");
    let after = text
        .split("[workspace.package]")
        .nth(1)
        .expect("workspace.package");
    after
        .lines()
        .find_map(|line| line.strip_prefix("version = "))
        .map(|value| value.trim().trim_matches('"').to_string())
        .expect("a workspace version")
}

/// Every readme in the workspace, as its path and its text.
fn readmes() -> Vec<(PathBuf, String)> {
    crates()
        .into_iter()
        .filter_map(|(dir, _)| {
            let path = dir.join("README.md");
            fs::read_to_string(&path).ok().map(|text| (path, text))
        })
        .collect()
}

#[test]
fn every_crate_ships_a_readme() {
    // `readme` is one of the files cargo always packages, so a crate without
    // one publishes with a blank page on the registry.
    for (dir, name) in crates() {
        assert!(
            dir.join("README.md").is_file(),
            "{name} has no README.md, and crates.io renders the readme as the landing page",
        );
    }
}

#[test]
fn no_readme_says_the_crate_is_unpublished() {
    // The whole class rather than the sentence that was there, because the next
    // one will be phrased differently. Anything pointing a reader at the git
    // repository for an install is the same claim in another coat.
    const UNPUBLISHED: [&str; 6] = [
        "not on crates.io",
        "isn't on crates.io",
        "is not on crates.io",
        "yet, so both come off the repository",
        "once it publishes",
        "once they publish",
    ];

    let mut wrong = Vec::new();
    for (path, text) in readmes() {
        let lower = text.to_lowercase();
        for claim in UNPUBLISHED {
            if lower.contains(claim) {
                wrong.push(format!("{}: {claim:?}", path.display()));
            }
        }
        for (at, line) in text.lines().enumerate() {
            if line.contains("github.com/orgrinrt/notko.git") && line.contains("branch") {
                wrong.push(format!(
                    "{}:{}: a branch dependency",
                    path.display(),
                    at + 1
                ));
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "these readmes install from the repository rather than the registry:\n{}",
        wrong.join("\n"),
    );
}

#[test]
fn every_version_a_readme_names_is_the_one_the_workspace_ships() {
    // A pinned version in a readme is a copy of the manifest's, and a copy of a
    // number is a number that will disagree.
    let version = workspace_version();
    let names: Vec<String> = crates().into_iter().map(|(_, name)| name).collect();

    let mut wrong = Vec::new();
    for (path, text) in readmes() {
        for (at, line) in text.lines().enumerate() {
            let Some((left, right)) = line.split_once(" = \"") else {
                continue;
            };
            let named = left.trim();
            if !names.iter().any(|name| name == named) {
                continue;
            }
            let Some(found) = right.split('"').next() else {
                continue;
            };
            if found != version {
                wrong.push(format!(
                    "{}:{}: {named} at {found:?}, and the workspace ships {version:?}",
                    path.display(),
                    at + 1,
                ));
            }
        }
    }

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_crate_a_readme_tells_you_to_add_is_one_that_exists() {
    // A rename leaves the old name in the install line, where it fails for the
    // reader and for nobody else.
    let names: Vec<String> = crates().into_iter().map(|(_, name)| name).collect();

    let mut wrong = Vec::new();
    for (path, text) in readmes() {
        for (at, line) in text.lines().enumerate() {
            let Some(rest) = line.trim().strip_prefix("cargo add ") else {
                continue;
            };
            let named = rest.split_whitespace().next().unwrap_or("");
            if !names.iter().any(|name| name == named) {
                wrong.push(format!(
                    "{}:{}: `cargo add {named}`, and this workspace has {names:?}",
                    path.display(),
                    at + 1,
                ));
            }
        }
    }

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_readme_the_manifest_names_is_the_one_that_is_there() {
    // `readme = "README.md"` naming a file that does not exist is not an error
    // at build time and is a blank landing page at publish time.
    for (dir, name) in crates() {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("a manifest");
        let Some(named) = manifest
            .lines()
            .find_map(|line| line.strip_prefix("readme = "))
            .map(|value| value.trim().trim_matches('"').to_string())
        else {
            continue;
        };
        assert!(
            dir.join(&named).is_file(),
            "{name}'s manifest names {named}, which is not beside it",
        );
    }
}
