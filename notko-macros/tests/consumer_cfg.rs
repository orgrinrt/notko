//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What `#[profile]`'s output does on the far side of the crate boundary.
//!
//! The macro emits `cfg(feature = "internal")`, and a `cfg` inside an
//! attribute macro is evaluated against the features of the crate the macro
//! expanded into, not the one that defined it. So the consumer is the crate
//! that has to declare the feature, and nothing inside this repository can
//! observe that: every test target here belongs to a crate that already
//! declares it.
//!
//! Hence a fixture crate and a real build of it. The two claims are that a
//! consumer declaring the feature builds silently, and that one which does not
//! is warned rather than left to wonder why `--release` changed its return
//! types.

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/consumer")
}

/// Copy the fixture somewhere of our own, optionally without its `[features]`
/// table, build it, and return cargo's stderr.
///
/// The copy is why this does not touch the fixture in the repository: the
/// three arms run in parallel, and an arm rewriting a file the others are
/// reading is a race, not a test.
fn build(with_feature_declared: bool, extra: &[&str]) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the macro crate has a parent");
    let src = fixture();

    let scratch = std::env::temp_dir().join(format!(
        "notko-consumer-cfg-{}-{}-{}",
        std::process::id(),
        with_feature_declared,
        extra.join("_").replace(['-', ' '], "")
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(scratch.join("src")).unwrap();

    let mut manifest = std::fs::read_to_string(src.join("Cargo.toml")).unwrap();
    // The fixture names the crate by a relative path, which stops meaning the
    // same thing once the fixture is somewhere else.
    manifest = manifest.replace(
        "path = \"../../../..\"",
        &format!("path = {:?}", root.display().to_string()),
    );
    if !with_feature_declared {
        manifest = manifest
            .split_once("[features]")
            .expect("the fixture declares [features]")
            .0
            .to_string();
    }
    std::fs::write(scratch.join("Cargo.toml"), manifest).unwrap();
    std::fs::copy(src.join("src/lib.rs"), scratch.join("src/lib.rs")).unwrap();

    let out = Command::new(env!("CARGO"))
        .args(["build", "--manifest-path"])
        .arg(scratch.join("Cargo.toml"))
        .args(extra)
        .output()
        .expect("cargo did not run");

    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let succeeded = out.status.success();
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(
        succeeded,
        "the fixture must build in every arm; it did not:\n{stderr}"
    );
    stderr
}

#[test]
fn a_consumer_declaring_the_feature_builds_without_a_warning() {
    let stderr = build(true, &[]);
    assert!(
        !stderr.contains("unexpected `cfg`"),
        "declaring the feature should have made the cfg a known value:\n{stderr}"
    );
}

#[test]
fn a_consumer_that_does_not_declare_it_is_warned() {
    // The half that makes the pair a test rather than a build. Without it,
    // an arm that silently stopped emitting the cfg at all would still pass
    // the assertion above.
    let stderr = build(false, &[]);
    assert!(
        stderr.contains("unexpected `cfg` condition value: `internal`"),
        "a consumer with no such feature should be told where the cfg came from:\n{stderr}"
    );
}

#[test]
fn the_release_arm_the_feature_selects_also_builds() {
    let stderr = build(true, &["--features", "internal", "--release"]);
    assert!(
        !stderr.contains("unexpected `cfg`"),
        "the arm the feature exists to select should build as quietly as the other:\n{stderr}"
    );
}
