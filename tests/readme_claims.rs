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
            // `=0.0.1` is the same number wearing an exact-pin requirement, and
            // a readme telling a reader to pin has to be able to show them how.
            // The pin is the reader's business; whether the number is current
            // is this test's.
            let found = found.strip_prefix('=').unwrap_or(found);
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
            // The first thing that is not a flag. `cargo add --build notko-build`
            // is how a build dependency is added, and reading the first word
            // flat would test the string `--build` against the member list and
            // report the crate missing.
            let named = rest
                .split_whitespace()
                .find(|word| !word.starts_with('-'))
                .unwrap_or("");
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

/// Every registry link naming one of ours names one that exists.
///
/// `cargo add` lines are checked above and they are the minority. The badges at
/// the top of each file point at crates.io and docs.rs by name, and the prose
/// links the four crates to each other by name, and all of those rot the same
/// way a stale install line does: silently, on somebody else's screen, months
/// after the rename that caused it.
///
/// Only names beginning `notko` are checked. Everything else is a link to
/// somebody else's crate, and this repository has no business asserting that
/// one exists.
#[test]
fn every_registry_link_to_one_of_ours_names_a_crate_that_exists() {
    // The four shapes these files use. Each is a prefix, and the crate name is
    // whatever follows it up to the first character that cannot be in one.
    const PREFIXES: [&str; 4] = [
        "https://crates.io/crates/",
        "https://docs.rs/",
        "https://img.shields.io/crates/v/",
        "https://img.shields.io/docsrs/",
    ];

    let names: Vec<String> = crates().into_iter().map(|(_, name)| name).collect();

    let mut found = 0usize;
    let mut wrong = Vec::new();
    for (path, text) in readmes() {
        for (at, line) in text.lines().enumerate() {
            for prefix in PREFIXES {
                for piece in line.split(prefix).skip(1) {
                    let named: String = piece
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                        .collect();
                    if !named.starts_with("notko") {
                        continue;
                    }
                    found += 1;
                    if !names.contains(&named) {
                        wrong.push(format!(
                            "{}:{}: {prefix}{named}, and this workspace has {names:?}",
                            path.display(),
                            at + 1,
                        ));
                    }
                }
            }
        }
    }

    // Without this the whole thing passes by finding nothing, which is what it
    // would do if a prefix above were mistyped or the badge block changed
    // shape. Four crates carry two registry badges each, so eight is the floor
    // before a single prose link is counted.
    assert!(
        found >= 8,
        "matched {found} registry links across the readmes, so the prefixes no \
         longer describe what these files contain",
    );

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
    //
    // Read off the members rather than typed out. The list used to be a literal
    // followed by an assertion that it had four entries, which is the literal
    // agreeing with itself: a crate added to the workspace and left out of the
    // literal was never checked, and nothing said so.
    let crates: Vec<PathBuf> = std::iter::once(root.clone())
        .chain(crates().into_iter().map(|(dir, _)| dir))
        .filter(|d| d.join("Cargo.toml").is_file())
        .collect();
    assert!(
        crates.len() > 1,
        "the workspace members were not read: {crates:?}"
    );

    let mut checked_any_file = false;

    for dir in &crates {
        let where_it_is = dir.file_name().unwrap().to_string_lossy().to_string();
        let toml = fs::read_to_string(dir.join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("{where_it_is} has no readable manifest: {e}"));
        // The package name, not the directory. A clone can sit anywhere and
        // under any name, and a message naming the directory sends the reader
        // to a crate that is not what it is called.
        let label = toml
            .split_once("\nname = \"")
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(n, _)| n.to_string())
            .unwrap_or(where_it_is);

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

            let why = reaches_for_the_repository(&body, &stripped);
            assert!(
                why.is_empty(),
                "{label}/{file} is published and {}. An unpacked tarball has \
                 none of that. Either drop it from `include` because it is a \
                 check about this repository, or stop it depending on the \
                 repository because it is a check about the crate.",
                why.join(", and ")
            );
        }

        // And the other direction, which the loop above cannot see. A test that
        // belongs in the package and is simply not named ships nothing,
        // silently: `cargo package` does not miss it, `cargo test` in a
        // checkout runs it, and the only observable difference is in a tarball
        // nobody unpacks until a consumer does.
        //
        // This runs per crate rather than per shipped file, because a crate
        // naming no test at all is exactly where an unnamed one hides, and
        // nesting it under the shipped files would skip those crates entirely.
        for entry in fs::read_dir(dir.join("tests"))
            .into_iter()
            .flatten()
            .flatten()
        {
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
            assert!(
                !reaches_for_the_repository(&body, &stripped).is_empty(),
                "{label}/{name} is a test about the crate and is not named in \
                 `include`, so it does not ship. Either name it, or make it \
                 plainly a check about this repository."
            );
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

/// Why a test body could not run from an unpacked package, if it could not.
///
/// One classifier, read in both directions, and the two directions do not
/// tolerate the same gaps. Forward, over a file that ships, a missing rule is
/// a false negative: the check merely fails to catch something. Reverse, over
/// a file that does not ship, the same missing rule is a false positive, and
/// it fails the build over a file that is correctly left out.
///
/// So a rule here is positive evidence that the file reaches outside the
/// package, never the absence of evidence that it does not. Every one of them
/// names something a tarball demonstrably has no answer for.
fn reaches_for_the_repository(body: &str, stripped: &[String]) -> Vec<String> {
    let mut why = Vec::new();

    for needle in ABSENT {
        if body.contains(needle) {
            why.push(format!("reaches for `{needle}`"));
        }
    }

    for dep in stripped {
        let underscored = dep.replace('-', "_");
        if body.contains(&format!("use {underscored}::"))
            || body.contains(&format!("{underscored}::"))
        {
            why.push(format!("names `{dep}`, which cargo strips on publish"));
        }
    }

    // `CARGO_MANIFEST_DIR` is the package root in a tarball, so anything above
    // it is the repository and is not there.
    if body.contains("CARGO_MANIFEST_DIR") && body.contains(".parent()") {
        why.push(
            "walks above `CARGO_MANIFEST_DIR`, which in a package has nothing above it".into(),
        );
    }

    // A fixture tree is a repository artifact. `include` names files, so a
    // directory of fixtures beside a test does not travel with it.
    if body.contains("tests/fixtures") {
        why.push("reads a fixture tree, which a package does not carry".into());
    }

    // A compile-fail harness is the same thing under another name: a directory
    // of sources plus the diagnostics they must produce, none of which is a
    // file `include` could sensibly name one at a time.
    if body.contains("compile_fail(") {
        why.push("drives a compile-fail tree, which a package does not carry".into());
    }

    // Spawning the toolchain against the tree is a check about the tree.
    if body.contains("env!(\"CARGO\")") {
        why.push("builds something with cargo, which needs the repository".into());
    }

    // Asking git what is tracked needs a repository, and an unpacked tarball
    // is a directory. `cargo package` does not carry `.git`, so this is
    // positive evidence rather than a guess about intent.
    if body.contains("\"ls-files\"") || body.contains("ls-tree") {
        why.push("asks git what is tracked, which a package has no answer for".into());
    }

    // `research/` is the audit trail. `include` names files, so no probe under
    // it travels with the package.
    if body.contains("\"research\"") || body.contains("research/") {
        why.push("reads the research tree, which a package does not carry".into());
    }

    // One `../` climbs from `tests/` to the package root and is fine. A second
    // leaves the package.
    for macro_name in ["include_str!", "include_bytes!", "include!"] {
        let mut rest = body;
        while let Some(at) = rest.find(macro_name) {
            let after = &rest[at + macro_name.len() ..];
            let Some(open) = after.find('"') else { break };
            let tail = &after[open + 1 ..];
            let Some(close) = tail.find('"') else { break };
            let path = &tail[.. close];
            if path.matches("../").count() > 1 {
                why.push(format!(
                    "includes `{path}`, which climbs out of the package"
                ));
            }
            rest = &tail[close ..];
        }
    }

    why
}

/// Each rule fires on the shape it names, and none of them fires on a test
/// that genuinely belongs in the package.
///
/// The reverse direction is what makes this necessary. It asserts that a file
/// left out of `include` reaches outside the package, so a rule that is merely
/// missing turns a correctly-excluded file into a red suite. Every positive
/// here is a real file in this repository reduced to the line that placed it.
#[test]
fn the_classifier_fires_on_each_shape_and_on_nothing_else() {
    let stripped = vec!["notko-macros".to_string()];
    let fires = |body: &str| !reaches_for_the_repository(body, &stripped).is_empty();

    assert!(
        fires("let m = \"the workspace manifest\";"),
        "ABSENT needle"
    );
    assert!(
        fires("assert!(toml.contains(\"members = [\"));"),
        "ABSENT needle"
    );
    assert!(fires("use notko_macros::profile;"), "stripped dependency");
    assert!(
        fires("let root = Path::new(env!(\"CARGO_MANIFEST_DIR\")).parent().unwrap();"),
        "walking above the package root"
    );
    assert!(
        fires("Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"tests/fixtures/consumer\")"),
        "a fixture tree"
    );
    assert!(fires("Command::new(env!(\"CARGO\"))"), "spawning cargo");
    assert!(
        fires("const R: &str = include_str!(\"../../README.md\");"),
        "climbing out of the package"
    );

    // And the shapes that belong in a package, so this is not simply always
    // positive. Each is a line a real shipped test here contains.
    assert!(!fires("use notko::{Just, Maybe, Outcome};"));
    assert!(!fires("const R: &str = include_str!(\"../README.md\");"));
    assert!(!fires(
        "let d = PathBuf::from(env!(\"CARGO_MANIFEST_DIR\"));"
    ));
    assert!(!fires(
        "assert_eq!(Maybe::Is(1).cmp(&Maybe::Isnt), Ordering::Greater);"
    ));

    // The reason list is the message, so it has to say something.
    let why = reaches_for_the_repository("use notko_macros::profile;", &stripped);
    assert_eq!(why.len(), 1);
    assert!(
        why[0].contains("notko-macros"),
        "the reason names nothing: {why:?}"
    );
}

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

/// The doc-include gate names every feature the readme's own examples need.
///
/// The readme is compiled as a doctest through a `#[cfg(doctest, ...)]` item in
/// the crate root. That gate has to name every feature the blocks reach for, or
/// the configuration where one is missing compiles a block against a feature it
/// does not have and fails on the absence rather than on anything being wrong.
///
/// It went wrong exactly that way: the gate named `macros`, for the block using
/// `#[profile]`, and a later block started using `?` on `Outcome`, which needs
/// `try_trait_v2`. Under `--features macros` alone the readme then did not
/// build, and the default run and the all-features run were both green, so
/// nothing anybody normally runs said so.
#[test]
fn the_doc_include_gate_names_every_feature_the_readme_reaches_for() {
    let lib = fs::read_to_string(root().join("src/lib.rs")).expect("no crate root");
    let readme = fs::read_to_string(root().join("README.md")).expect("no readme");

    let gate = lib
        .lines()
        .find(|l| l.contains("#[cfg(") && l.contains("doctest"))
        .expect("the readme is not included as a doctest at all, which is the whole check");

    // What each construct in a runnable block needs, and why. Only `rust`
    // fences count: a shell block is prose as far as the doctest is concerned.
    let needs: [(&str, &str); 2] = [
        // `?` on these types goes through `Try`, which is the unstable impl.
        ("?;", "try_trait_v2"),
        // the attribute macro itself
        ("#[profile", "macros"),
    ];

    // `Some(true)` inside a block rustdoc compiles, `Some(false)` inside one it
    // does not, `None` between them. Two states are not enough: a closing fence
    // carries no language either, so reading it as an opening one puts the prose
    // after a shell block into the collected text and leaves the rust blocks out.
    let mut runnable = String::new();
    let mut inside: Option<bool> = None;
    for line in readme.lines() {
        if let Some(rest) = line.trim().strip_prefix("```") {
            inside = match inside {
                // a fence with no language, or one naming rust, is compiled
                None => Some(rest.is_empty() || rest.starts_with("rust")),
                Some(_) => None,
            };
            continue;
        }
        if inside == Some(true) {
            runnable.push_str(line);
            runnable.push('\n');
        }
    }
    assert!(
        !runnable.is_empty(),
        "no runnable block was found in the readme, so this check has nothing to read"
    );

    for (construct, feature) in needs {
        if !runnable.contains(construct) {
            continue;
        }
        assert!(
            gate.contains(&format!("feature = \"{feature}\"")),
            "a readme example uses `{construct}`, which `{feature}` provides, and the \
             doc-include gate does not name it:\n  {}\n\
             Under a configuration with the other features and not this one, the \
             readme is compiled and cannot build.",
            gate.trim()
        );
    }

    // The control. Without it the loop above holds vacuously the moment the
    // needles stop matching, which is exactly how the defect got in.
    assert!(
        runnable.contains("?;") || runnable.contains("#[profile"),
        "neither needle appears in any runnable block, so nothing was checked"
    );
}

/// The `#![cfg_attr(..., feature(...))]` gates in the crate root, by name.
fn unstable_gates() -> Vec<String> {
    let text = fs::read_to_string(root().join("src/lib.rs")).expect("the crate root");
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if !l.starts_with("#![cfg_attr(") {
                return None;
            }
            let inner = l.split("feature(").nth(1)?;
            Some(inner.split(')').next()?.to_string())
        })
        .collect()
}

#[test]
fn the_readme_counts_the_unstable_features_the_crate_root_gates() {
    // The readme said three where the root gated five, and it named two the
    // default set does not turn on while omitting two it does. Every word of
    // it was true when written; what changed underneath was the const set
    // growing, and prose does not notice that.
    let gates = unstable_gates();
    let readme = fs::read_to_string(root().join("README.md")).expect("the readme");

    let numbers = [
        (1, "one"),
        (2, "two"),
        (3, "three"),
        (4, "four"),
        (5, "five"),
        (6, "six"),
        (7, "seven"),
        (8, "eight"),
    ];
    let spelled = numbers
        .iter()
        .find(|(n, _)| *n == gates.len())
        .map(|(_, w)| *w)
        .unwrap_or_else(|| panic!("no spelling for {} gates", gates.len()));
    assert!(
        readme.contains(&format!("{spelled} unstable features")),
        "the crate root gates {} features and the readme does not say {spelled}: {gates:?}",
        gates.len()
    );

    for gate in &gates {
        assert!(
            readme.contains(&format!("`{gate}`")),
            "`{gate}` is gated in the crate root and named nowhere in the readme"
        );
    }
}

#[test]
fn the_msrv_says_which_build_it_is_the_msrv_of() {
    // `rust-version` is one number and this crate has two builds. The default
    // set needs nightly, so the number is the floor for the build with the
    // defaults off, and a reader who takes it for the default one gets a
    // feature-gate error rather than the version error the key exists to give.
    // Verified against the toolchain it names, with the defaults off.
    let manifest = fs::read_to_string(root().join("Cargo.toml")).expect("workspace manifest");
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("rust-version"))
        .expect("rust-version is set, per the publishing rules");
    let version = line
        .split('"')
        .nth(1)
        .expect("rust-version carries a version string");

    let readme = fs::read_to_string(root().join("README.md")).expect("the readme");
    assert!(
        readme.contains(version),
        "the manifest claims {version} and the readme never mentions it, so \
         nobody reading the crate learns which build it is the floor for"
    );
    assert!(
        readme.contains("default-features = false") || readme.contains("turn the defaults off"),
        "the readme does not say how to reach the build {version} applies to"
    );
}
