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

/// The tests any crate here publishes have to run from an unpacked tarball,
/// where neither the repository nor its sibling crates are there.
///
/// Two kinds of test live under a `tests/`. Most check the crate and belong in
/// the package. Some check the repository: this file reads the workspace
/// manifest's member list, which cargo's generated manifest does not carry, so
/// shipping it produces a crate whose own suite panics on the first line.
///
/// The second class is subtler and is what this check missed while covering
/// only the root crate. `notko-macros` shipped a test importing `notko`, which
/// it reaches through a dev-dependency carrying a path and no version. Cargo
/// strips exactly those on publish, so the tarball's test could not resolve the
/// import at all.
///
/// Neither shows up in `cargo package`, because packaging compiles the tests
/// and never runs them, and the second does not even compile.
#[test]
fn a_shipped_test_does_not_reach_for_the_repository() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Every crate in the workspace, not just the one this file sits in. The
    // defect this check exists for recurred in a sibling while the root stayed
    // clean, and a check that looks at one crate reports on one crate.
    let crates: Vec<PathBuf> = std::iter::once(root.clone())
        .chain(
            ["notko-macros", "notko-macros-core", "notko-build"]
                .iter()
                .map(|c| root.join(c)),
        )
        .collect();
    assert_eq!(crates.len(), 4, "the crate list is not what it says it is");

    let mut checked_any_file = false;

    for dir in &crates {
        let label = dir.file_name().unwrap().to_string_lossy().to_string();
        let toml = fs::read_to_string(dir.join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("{label} has no readable manifest: {e}"));

        let Some(include) = toml
            .split_once("include = [")
            .map(|(_, rest)| rest.split_once(']').expect("`include` is not closed").0)
        else {
            panic!(
                "{label} names no `include`, so everything in it ships, \
                 repository checks included"
            );
        };

        let shipped: Vec<&str> = include
            .split('"')
            .filter(|s| s.starts_with("tests/") && s.ends_with(".rs"))
            .collect();

        // A glob under `tests/` defeats the whole check: it matches every file
        // including the ones that must not ship, and it reads as one entry to
        // anything comparing strings, so the loop below silently finds nothing
        // wrong. It is also the thing being forbidden on its own merits, since
        // deciding per file is the point.
        let globbed: Vec<&&str> = shipped.iter().filter(|e| e.contains('*')).collect();
        assert!(
            globbed.is_empty(),
            "{label}'s `include` reaches into `tests/` with a glob: {globbed:?}\n\
             A glob cannot tell a crate test from a repository check, so it ships \
             both. Name the files that belong in the package instead."
        );

        // Crates cargo removes from the published manifest: a dev-dependency
        // with a path and no version. A shipped test importing one of these
        // does not compile in the tarball, and nothing before the consumer's
        // first `cargo test` says so.
        let stripped = stripped_dev_deps(&toml);

        for file in &shipped {
            let body = fs::read_to_string(dir.join(file)).unwrap_or_else(|e| {
                panic!("{label}'s `include` names {file}, which is not readable: {e}")
            });
            checked_any_file = true;

            for needle in ABSENT {
                assert!(
                    !body.contains(needle),
                    "{label}/{file} is published and reaches for `{needle}`, which \
                     an unpacked tarball does not have. Either drop it from \
                     `include` because it is a check about this repository, or \
                     stop it depending on the repository because it is a check \
                     about the crate."
                );
            }

        // And the other direction, which the check above cannot see. A test
        // that belongs in the package and is simply not named ships nothing,
        // silently: `cargo package` does not miss it, `cargo test` in a
        // checkout runs it, and the only observable difference is in a tarball
        // nobody unpacks until a consumer does.
        //
        // The classifier is the same pair of rules the loop below applies,
        // read the other way round. A file reaching for the repository or for
        // a stripped dependency is a check about this repository and belongs
        // out of `include`. Anything else is a check about the crate and
        // belongs in it.
        for entry in fs::read_dir(dir.join("tests")).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = format!("tests/{}", path.file_name().unwrap().to_string_lossy());
            if shipped.contains(&name.as_str()) {
                continue;
            }
            let Ok(body) = fs::read_to_string(&path) else {
                continue;
            };
            let about_the_repository = ABSENT.iter().any(|n| body.contains(n))
                || stripped.iter().any(|dep| {
                    let u = dep.replace('-', "_");
                    body.contains(&format!("use {u}::")) || body.contains(&format!("{u}::"))
                });
            assert!(
                about_the_repository,
                "{label}/{name} is a test about the crate and is not named in \
                 `include`, so it does not ship. Either name it, or make it \
                 plainly a check about this repository."
            );
        }

            for dep in &stripped {
                let underscored = dep.replace('-', "_");
                let imports = body.contains(&format!("use {underscored}::"))
                    || body.contains(&format!("{underscored}::"));
                assert!(
                    !imports,
                    "{label}/{file} is published and imports `{dep}`, which \
                     reaches it only through a dev-dependency carrying a path \
                     and no version. Cargo strips those on publish, so this \
                     test cannot compile from the tarball. Either drop it from \
                     `include`, or give the dependency a registry version."
                );
            }
        }
    }

    assert!(
        checked_any_file,
        "no crate named a test file in `include`, so every assertion above held \
         over an empty set"
    );

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

/// What a tarball does not have: the workspace manifest, and the repository git
/// would answer questions about.
const ABSENT: [&str; 2] = ["workspace manifest", "members = ["];

/// Dependency names under `[dev-dependencies]` that carry a path and no
/// version, which is exactly the set cargo drops when it publishes.
fn stripped_dev_deps(toml: &str) -> Vec<String> {
    let Some((_, rest)) = toml.split_once("[dev-dependencies]") else {
        return Vec::new();
    };
    let section = rest.split("\n[").next().unwrap_or(rest);
    section
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (name, spec) = line.split_once('=')?;
            if spec.contains("path") && !spec.contains("version") {
                Some(name.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn a_path_only_dev_dependency_is_recognised_as_stripped() {
    let found = stripped_dev_deps(
        "[dev-dependencies]\n\
         # a comment\n\
         local = { path = \"..\" }\n\
         pinned = { path = \"../x\", version = \"0.0.1\" }\n\
         registry = \"3\"\n\
         \n\
         [profile.release]\n\
         local_but_out_of_section = { path = \"..\" }\n",
    );
    assert_eq!(
        found,
        vec!["local".to_string()],
        "a versioned path dep, a registry dep, or a line past the section end \
         was picked up, or the path-only one was missed"
    );
    assert!(
        stripped_dev_deps("[dependencies]\nlocal = { path = \"..\" }").is_empty(),
        "a normal dependency was read as a dev-dependency"
    );
}

