//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What this crate sells is that somebody else can build their own tier
//! attribute macro on it. That is only true if nothing it emits names this
//! crate unless the author asked for it.
//!
//! The suite used to build every `CustomTier` by hand and call `rewrite_fn`,
//! which covers exactly the surface that already worked. The discovery route
//! was never entered, and it hardcoded both the crate path and the gate
//! feature, so a third party's macro emitted code demanding `notko` be in
//! scope in consumers that had never heard of it, with nothing to say so until
//! one of them failed to compile.

use notko_macros_core::discover::Discovery;
use notko_macros_core::rewrite::rewrite_fn;
use proc_macro2::Span;
use quote::quote;
use syn::{ItemFn, parse_quote, parse2};

/// What somebody else's macro would set up.
fn theirs() -> Discovery {
    Discovery {
        krate: parse_quote!(::widget),
        gate_feature: "widget-release".to_string(),
        dir: "widget-tiers".to_string(),
        env_var: "WIDGET_TIERS_PATH".to_string(),
        marker: "@widget-tier".to_string(),
        docs: "the widget book".to_string(),
    }
}

fn emitted(d: &Discovery, tier: &str, f: ItemFn) -> String {
    let resolved = d
        .resolve(tier, Span::call_site())
        .expect("the tier resolves");
    rewrite_fn(resolved, f)
        .expect("the rewrite succeeds")
        .to_string()
}

#[test]
fn a_built_in_tier_emits_through_the_crate_the_author_named() {
    let f: ItemFn = parse2(quote! {
        fn load(x: u32) -> Result<u32, Oops> { Ok(x) }
    })
    .unwrap();
    let out = emitted(&theirs(), "Cold", f);
    assert!(
        out.contains(":: widget"),
        "the emission does not name their crate: {out}"
    );
    assert!(
        !out.contains(":: notko"),
        "the emission names this crate in somebody else's macro: {out}"
    );
}

#[test]
fn the_release_gate_is_the_feature_the_author_named() {
    let f: ItemFn = parse2(quote! {
        fn load(x: u32) -> Result<u32, Oops> { Ok(x) }
    })
    .unwrap();
    let out = emitted(&theirs(), "Hot", f);
    assert!(
        out.contains("widget-release"),
        "the release arm is gated on the wrong feature: {out}"
    );
    assert!(
        !out.contains("\"internal\""),
        "the release arm is gated on this crate's own feature: {out}"
    );
}

#[test]
fn the_default_still_names_this_crate() {
    // The control on both above. A `Discovery` that emitted nothing, or one
    // whose fields were ignored, would pass them and break notko's own macro.
    let f: ItemFn = parse2(quote! {
        fn load(x: u32) -> Result<u32, Oops> { Ok(x) }
    })
    .unwrap();
    let out = emitted(&Discovery::default(), "Cold", f);
    assert!(
        out.contains(":: notko"),
        "the default stopped naming this crate: {out}"
    );
    assert!(
        !out.contains(":: widget"),
        "the default named somebody else: {out}"
    );
}

#[test]
fn a_tier_file_of_their_own_resolves_and_carries_their_identity() {
    // The file route, which is the one that was hardcoded. It reads
    // `CARGO_MANIFEST_DIR`, so the directory is created under this crate and
    // removed after, rather than the environment being mutated.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("widget-tiers");
    std::fs::create_dir_all(&dir).expect("the tier directory");
    let file = dir.join("Audit.rs");
    std::fs::write(
        &file,
        "//! @widget-tier\n//! based_on = \"Cold\"\n//! inline = true\n",
    )
    .expect("the tier file");

    let d = theirs();
    let resolved = d.resolve("Audit", Span::call_site());

    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_dir(&dir);

    let tier = resolved.expect("a tier file of their own resolves");
    let out = rewrite_fn(
        tier,
        parse2(quote! { fn load(x: u32) -> Result<u32, Oops> { Ok(x) } }).unwrap(),
    )
    .expect("the rewrite succeeds")
    .to_string();
    assert!(
        out.contains(":: widget"),
        "the file route lost their crate: {out}"
    );
    assert!(
        !out.contains(":: notko"),
        "the file route named this crate: {out}"
    );
}

#[test]
fn the_marker_a_tier_file_must_carry_is_the_author_s() {
    // And the refusal names their marker rather than ours, since ours means
    // nothing to their reader.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("widget-tiers-2");
    std::fs::create_dir_all(&dir).expect("the tier directory");
    let file = dir.join("Audit.rs");
    std::fs::write(&file, "//! @notko-optimiser\n//! based_on = \"Cold\"\n")
        .expect("the tier file");

    let mut d = theirs();
    d.dir = "widget-tiers-2".to_string();
    let err = d
        .resolve("Audit", Span::call_site())
        .expect_err("the wrong marker is refused");
    let text = err.to_string();

    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_dir(&dir);

    assert!(
        text.contains("@widget-tier"),
        "the refusal names the wrong marker: {text}"
    );
}

#[test]
fn a_name_that_is_no_tier_says_where_the_author_documents_them() {
    let err = theirs()
        .resolve("Nonesuch", Span::call_site())
        .expect_err("an unknown tier is refused");
    let text = err.to_string();
    assert!(
        text.contains("widget-tiers"),
        "the refusal names the wrong directory: {text}"
    );
    assert!(
        text.contains("WIDGET_TIERS_PATH"),
        "the refusal names the wrong variable: {text}"
    );
    assert!(
        text.contains("the widget book"),
        "the refusal sends them to our docs: {text}"
    );
    // Not a bare search for the name: the crate-local path this reports is
    // rooted at `CARGO_MANIFEST_DIR`, which on this machine sits under a
    // directory called `notko`, so a bare search reads the checkout's location
    // as a leak. What must not appear is any of this crate's own answers to
    // the three questions above.
    for ours in ["notko-optimisers", "NOTKO_OPTIMISERS_PATH", "notko-macros README"] {
        assert!(
            !text.contains(ours),
            "the refusal gives this crate's `{ours}` to somebody else's user: {text}"
        );
    }
}
