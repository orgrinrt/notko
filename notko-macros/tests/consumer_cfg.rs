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
    let (ok, stderr) = try_build(with_feature_declared, extra, None);
    assert!(
        ok,
        "the fixture must build in every arm; it did not:\n{stderr}"
    );
    stderr
}

/// Build the fixture, optionally with a body of our own, and report whether it
/// built rather than insisting that it did.
///
/// The pair of arms is the whole point of the hot strategy, so the interesting
/// question is not whether one of them builds. It is whether they agree, and a
/// helper that panics on failure can only ever ask about the arm that works.
fn try_build(with_feature_declared: bool, extra: &[&str], body: Option<&str>) -> (bool, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the macro crate has a parent");
    let src = fixture();

    // A counter, not a description. Every attempt at deriving the name from
    // what the build is doing has collided with something: two tests running
    // the same flags over different sources, then two running the same flags
    // over the same source. Both raced, one deleting the directory the other
    // was compiling into, and both fail intermittently rather than always,
    // which is the worst way for a suite to be wrong.
    //
    // Nothing needs to read this name, so nothing needs it to mean anything.
    static NTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = NTH.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let scratch =
        std::env::temp_dir().join(format!("notko-consumer-cfg-{}-{nth}", std::process::id()));
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
    match body {
        Some(text) => std::fs::write(scratch.join("src/lib.rs"), text).unwrap(),
        None => {
            std::fs::copy(src.join("src/lib.rs"), scratch.join("src/lib.rs")).unwrap();
        },
    }

    let out = Command::new(env!("CARGO"))
        .args(["build", "--manifest-path"])
        .arg(scratch.join("Cargo.toml"))
        .args(extra)
        // Its own, inside the scratch. Every one of these builds a crate under
        // the same name, so sharing a target directory means they overwrite
        // each other's artifacts: the arms then report whichever body won the
        // race, and the result changes between runs of the same suite.
        .env("CARGO_TARGET_DIR", scratch.join("target"))
        .output()
        .expect("cargo did not run");

    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let succeeded = out.status.success();
    let _ = std::fs::remove_dir_all(&scratch);
    (succeeded, stderr)
}

/// Whether a body builds in each arm, as a pair.
///
/// `(debug, release)`. The two are what the strategy promises to keep equal,
/// so they are returned together and asserted together: a body that builds in
/// one and not the other is the defect, whichever way round it falls.
fn both_arms(body: &str) -> (bool, bool, String) {
    let (debug, d_err) = try_build(true, &[], Some(body));
    let (release, r_err) = try_build(true, &["--features", "internal", "--release"], Some(body));
    (
        debug,
        release,
        format!("debug:\n{d_err}\n\nrelease:\n{r_err}"),
    )
}

/// The fixture's own body with an extra function appended.
fn fixture_plus(extra: &str) -> String {
    let base = std::fs::read_to_string(fixture().join("src/lib.rs")).expect("the fixture body");
    format!("{base}\n{extra}\n")
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

#[test]
fn the_question_mark_operator_builds_in_both_arms_or_neither() {
    // The release arm narrows the return type to `Just<T>`, which is not what
    // `?` needs. It used to compile in the arm that is tested and fail in the
    // arm that ships, so the first anybody heard of it was their own release
    // build, from inside a macro they cannot see into.
    let (debug, release, why) = both_arms(&fixture_plus(
        r#"
#[profile(Cold)]
pub fn inner(x: u32) -> Result<u32, Oops> { Ok(x) }

#[profile(Hot)]
pub fn through(x: u32) -> Result<u32, Oops> {
    let v = inner(x)?;
    Ok(v * 2)
}
"#,
    ));
    assert_eq!(debug, release, "the arms disagree about `?`:\n{why}");
    assert!(
        debug,
        "`?` builds in neither arm, which is a different defect:\n{why}"
    );
}

#[test]
fn an_error_without_debug_is_refused_by_both_arms_or_neither() {
    // The release arm's panic reads the error with `{err:?}`. The debug arm
    // never formats it, so an error type carrying no `Debug` compiled in the
    // arm that is tested and did not compile in the arm that ships.
    //
    // Refused by both is the right answer here rather than accepted by both:
    // the strategy's whole contract is that a failure panics, and a panic that
    // cannot say what failed is not worth having. What matters is that the
    // author is told at the attribute rather than at their release build.
    let (debug, release, why) = both_arms(&fixture_plus(
        r#"
pub struct Opaque;

#[profile(Hot)]
pub fn opaque(x: u32) -> Result<u32, Opaque> {
    if x == 0 { return Err(Opaque); }
    Ok(x)
}
"#,
    ));
    assert_eq!(
        debug, release,
        "the arms disagree about an error type with no `Debug`:\n{why}"
    );
    assert!(
        !debug,
        "an error with no `Debug` is accepted, so the panic cannot report it"
    );
}

#[test]
fn an_ordinary_body_still_builds_in_both_arms() {
    // The control on the two above. Refusing everything, or accepting
    // everything, would satisfy one of them and break the macro.
    let (debug, release, why) = both_arms(&fixture_plus(""));
    assert!(
        debug && release,
        "the plain fixture stopped building:\n{why}"
    );
}

#[test]
fn the_release_arm_warns_about_nothing() {
    // `return Err(e)` became `return ::core::panic!(..)`, a `return` wrapping
    // a diverging expression, so every consumer's release build carried an
    // unreachable-expression warning pointing at the attribute. The fixture
    // has exactly that shape.
    let stderr = build(true, &["--features", "internal", "--release"]);
    assert!(
        !stderr.contains("unreachable"),
        "the release arm warns on a body it was written for:\n{stderr}"
    );
}

#[test]
fn an_impl_trait_error_builds_in_both_arms_or_neither() {
    // The guard that makes the arms agree about `Debug` is a closure taking
    // the error type by name, and `impl Trait` is not allowed in a closure
    // parameter. So emitting it against one refused the debug arm and left the
    // release arm building, which is the divergence the guard exists to close,
    // pointing the other way. It shipped that way for one review round and was
    // caught by a reviewer rather than by this file, which is why it is here.
    let (debug, release, why) = both_arms(&fixture_plus(
        r#"
#[profile(Hot)]
pub fn opaque(x: u32) -> Result<u32, impl core::fmt::Debug> {
    if x == 0 { return Err(Oops); }
    Ok(x)
}
"#,
    ));
    assert_eq!(
        debug, release,
        "the arms disagree about an `impl Trait` error:\n{why}"
    );
    assert!(debug, "an `impl Trait` error builds in neither arm:\n{why}");
}
