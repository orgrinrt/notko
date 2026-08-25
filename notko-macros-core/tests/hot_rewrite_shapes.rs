//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What the hot rewrite may and may not change.
//!
//! The hot strategy emits two versions of one function, a debug one and a
//! release one, and they are compiled from the same source under different
//! `cfg`. So every shape it does not handle identically in both arms is a
//! program that means one thing when you test it and another when you ship it,
//! with nothing to say so.
//!
//! That makes "leaves it alone" the interesting assertion here, more than
//! "rewrites it correctly", and it is why most of what follows is about input
//! the rewriter should decline.

use notko_macros_core::rewrite::rewrite_fn;
use notko_macros_core::tiers::{CustomTier, Hot};

/// The emitted pair, as text, normalised so a match is about tokens rather
/// than about how `quote!` happened to space them.
fn emitted(src: &str) -> String {
    let func: syn::ItemFn = syn::parse_str(src).expect("the fixture parses");
    let out = rewrite_fn(CustomTier::from_marker::<Hot>(), func).expect("the rewrite runs");
    out.to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The release arm alone, which is the one that diverges.
///
/// The two arms are emitted in order, debug first, each under its own `cfg`.
/// Splitting on the second `# [cfg` is what separates them.
fn release_arm(src: &str) -> String {
    let all = emitted(src);
    let at = all.rfind("# [cfg").expect("two cfg-gated arms are emitted");
    all[at..].to_string()
}

/// How many `cfg` gates the emission carries, for the shapes where the answer
/// is meant to be none.
fn arms(emitted: &str) -> usize {
    emitted.matches("# [cfg").count()
}

fn debug_arm(src: &str) -> String {
    let all = emitted(src);
    let at = all.rfind("# [cfg").expect("two cfg-gated arms are emitted");
    all[..at].to_string()
}

#[test]
fn the_documented_two_arm_shape_collapses_to_an_unwrap() {
    // The one thing the rewrite is for, and the control for everything below:
    // without this passing, every refusal test could be satisfied by a rewrite
    // that does nothing at all.
    let rel = release_arm(
        r#"fn f(r: Result<u32, E>) -> Result<u32, E> {
            match r { Ok(n) => n + 1, Err(e) => return Err(e) }
        }"#,
    );
    assert!(rel.contains("unwrap"), "{rel}");
    assert!(rel.contains("Just"), "{rel}");
}

#[test]
fn a_guard_is_never_dropped() {
    // A guard decides which arm runs. Discarding one keeps the arm and loses
    // the condition, so the release build takes a branch the debug build did
    // not, from the same source, with no diagnostic anywhere.
    let src = r#"fn f(r: Result<u32, E>) -> Result<&'static str, E> {
        match r {
            Ok(n) if n > 100 => Ok("big"),
            Ok(_) => Ok("small"),
            Err(e) => Err(e),
        }
    }"#;
    let rel = release_arm(src);
    assert!(
        rel.contains("> 100") || rel.contains("100"),
        "the guard is gone from the release arm:\n{rel}"
    );
}

#[test]
fn a_match_with_more_arms_than_it_understands_is_left_alone() {
    // Three arms is not the two-arm shape the rewrite documents. Keeping one
    // of them and silently discarding the rest is not a partial rewrite, it is
    // a different program.
    let src = r#"fn f(r: Result<u32, E>) -> Result<&'static str, E> {
        match r {
            Ok(0) => Ok("zero"),
            Ok(_) => Ok("some"),
            Err(e) => Err(e),
        }
    }"#;
    let rel = release_arm(src);
    assert!(rel.contains("zero"), "the first arm is gone:\n{rel}");
    assert!(rel.contains("some"), "the second arm is gone:\n{rel}");
}

#[test]
fn a_match_with_no_err_arm_is_left_alone() {
    // `Ok` and nothing else is not a `Result` match at all; it is somebody
    // matching a type of their own that happens to spell a variant `Ok`.
    let src = r#"fn f(s: Status) -> Result<u32, E> {
        match s { Status::Ok(n) => Ok(n), Status::Pending => Ok(0) }
    }"#;
    let rel = release_arm(src);
    assert!(rel.contains("Pending"), "an arm was discarded:\n{rel}");
}

#[test]
fn a_variant_of_the_consumers_own_type_is_not_our_ok() {
    // `Status::Ok` is not `core::result::Result::Ok`, and a rewriter that
    // compares only the last path segment cannot tell. The consumer then gets
    // an error naming a `notko` path they never typed, in a function whose
    // body mentions no `notko` type.
    let rel = release_arm(r#"fn f() -> Result<Status, E> { Ok(Status::Ok(3)) }"#);
    assert!(
        rel.contains("Status :: Ok") || rel.contains("Status::Ok"),
        "the consumer's own variant was rewritten:\n{rel}"
    );
}

#[test]
fn a_return_type_that_is_not_a_result_is_emitted_once_and_unchanged() {
    // Two arms exist so the release one can drop a check the debug one keeps.
    // With nothing to rewrite there is no second version to emit, and emitting
    // one anyway is how the signatures came to disagree: the release arm read
    // `-> u32` as an ok type and wrapped it, the debug arm left it bare, and
    // the caller compiled under `cargo test` and not under `cargo build
    // --release`.
    let src = r#"fn f() -> u32 { 7 }"#;
    let all = emitted(src);
    assert_eq!(
        arms(&all),
        0,
        "a function with nothing to rewrite was split in two:\n{all}"
    );
    assert!(!all.contains("Just"), "a wrapper appeared:\n{all}");
    assert!(all.contains("-> u32"), "the signature moved:\n{all}");
    assert!(all.contains("7"), "the body moved:\n{all}");
}

#[test]
fn a_result_alias_carrying_one_argument_leaves_the_body_alone_too() {
    // `type Result<T> = core::result::Result<T, MyError>` is ordinary, and the
    // rewriter cannot resolve it: a path with one argument says nothing about
    // what the second one is. Declining is fine. Declining the signature and
    // rewriting the body anyway is not, because it produces a function that
    // returns one type and constructs another.
    let src = r#"fn f() -> Result<u32> { Ok(1) }"#;
    let all = emitted(src);
    assert_eq!(arms(&all), 0, "an alias was split in two:\n{all}");
    assert!(!all.contains("Just"), "the signature was rewritten:\n{all}");
    assert!(
        !all.contains("notko") && !all.contains("unwrap"),
        "the body was rewritten under a signature that was not:\n{all}"
    );
    assert!(all.contains("Ok (1)"), "the body moved:\n{all}");
}

#[test]
fn the_two_arms_are_emitted_under_opposite_gates() {
    // The control for `release_arm` and `debug_arm` themselves. Both helpers
    // split on a marker, and a split that silently found the wrong boundary
    // would make every assertion above read the same text twice.
    let all = emitted(r#"fn f() -> Result<u32, E> { Ok(1) }"#);
    assert_eq!(all.matches("# [cfg").count(), 2, "{all}");
    assert!(all.contains("debug_assertions"), "{all}");
    assert_ne!(
        debug_arm(r#"fn f() -> Result<u32, E> { Ok(1) }"#),
        release_arm(r#"fn f() -> Result<u32, E> { Ok(1) }"#),
    );
}
