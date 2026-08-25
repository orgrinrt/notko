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
    // Two halves, and only one of them is a class.
    //
    // The branch-dependency check below is structural: any README telling a
    // reader to install from a git branch is making this claim, however it is
    // worded, and the check finds it without knowing the wording.
    //
    // The phrase list is not a class and should not be read as one. It is six
    // spellings that were actually in these files, kept so those exact
    // regressions cannot come back. A seventh wording sails through, and the
    // structural half is what catches it, so adding to this list is worth
    // doing when a new phrasing turns up and is not a substitute for the check
    // underneath.
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

/// The tests this crate publishes have to run from an unpacked tarball, where
/// the repository is not there.
///
/// Two kinds of test live under `tests/`. Most check the crate and belong in
/// the package. This file checks the repository: it reads the workspace
/// manifest's member list, which cargo's generated manifest does not carry, so
/// shipping it produces a crate whose own suite panics on the first line.
/// `cargo package` will not catch that, because packaging compiles the tests
/// and never runs them.
#[test]
fn a_shipped_test_does_not_reach_for_the_repository() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("no manifest");

    let include = toml
        .split_once("include = [")
        .expect("the manifest names no `include`, so everything ships and this check is moot")
        .1
        .split_once(']')
        .expect("`include` is not closed")
        .0;

    let shipped: Vec<&str> = include
        .split('"')
        .filter(|s| s.starts_with("tests/") && s.ends_with(".rs"))
        .collect();

    assert!(
        !shipped.is_empty(),
        "no test file is named in `include`, so this check would hold over an \
         empty set. If that is deliberate, delete this test rather than \
         leaving it to pass on nothing."
    );

    // What a tarball does not have: the workspace manifest, and the repository
    // git would answer questions about.
    const ABSENT: [&str; 2] = ["workspace manifest", "members = ["];

    for file in &shipped {
        let body = fs::read_to_string(manifest_dir.join(file))
            .unwrap_or_else(|e| panic!("`include` names {file}, which is not readable: {e}"));

        for needle in ABSENT {
            assert!(
                !body.contains(needle),
                "{file} is published and reaches for `{needle}`, which an \
                 unpacked tarball does not have. Either drop it from `include` \
                 because it is a check about this repository, or stop it \
                 depending on the repository because it is a check about the \
                 crate."
            );
        }
    }

    // The control. Without it the loop passes on any needle nothing contains,
    // including a typo, and this file is the proof the needles are findable in
    // a real test body.
    let own = fs::read_to_string(file!()).expect("this file is not readable");
    for needle in ABSENT {
        assert!(
            own.contains(needle),
            "the needle `{needle}` was not found even in a file that plainly \
             uses it, so the loop above would clear every file it looked at"
        );
    }
}
