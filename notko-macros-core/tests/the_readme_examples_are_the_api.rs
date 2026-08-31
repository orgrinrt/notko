//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The two authoring examples in this crate's readme, compiled.
//!
//! `notko` and `notko-hlist` both pull their readme in as a doctest, so an
//! example that stopped compiling there fails the suite. This crate cannot do
//! that: the first example is a `#[proc_macro_attribute]`, which only a
//! proc-macro crate may declare, so the block would fail for a reason that has
//! nothing to do with whether the api it walks is still there.
//!
//! What is left is the api path itself, which is the part that rots. The two
//! tests below build what the readme builds, through the same calls in the same
//! order, so a rename or a changed signature lands here rather than on whoever
//! copied the block. The wrapping that cannot be compiled outside a proc-macro
//! crate is the `#[proc_macro_attribute]` line and the `TokenStream`
//! conversions, and neither of those is ours to break.
//!
//! It found one already: the readme's first block named `CustomTier` and `Hot`
//! with no `use` line for either.

use notko_macros_core::discover::Discovery;
use notko_macros_core::rewrite::rewrite_fn;
use notko_macros_core::tiers::{CustomTier, Hot};
use proc_macro2::Span;
use quote::quote;
use syn::{ItemFn, parse2, parse_quote};

/// The function every arm here rewrites. Spelled with both arguments written
/// out, since that is the only shape the rewrite fires on.
fn subject() -> ItemFn {
    parse2(quote! {
        fn load(x: u32) -> Result<u32, Oops> { Ok(x) }
    })
    .expect("the subject parses")
}

#[test]
fn the_hand_built_tier_example_still_builds_and_rewrites() {
    // Byte for byte what `## Authoring a third-party attribute macro` does,
    // minus the `TokenStream` hop at either end.
    let mut tier = CustomTier::from_marker::<Hot>()
        .with_crate(parse_quote!(::my_runtime))
        .with_gate_feature("my_release_arm");
    tier.panic_fmt = Some("asserted invariant violated: {err:?}".into());

    let out = rewrite_fn(tier, subject())
        .expect("the rewrite succeeds")
        .to_string();

    // The two things the example is actually demonstrating: that the crate the
    // author named is the one that gets emitted, and that this crate's own name
    // does not follow it into somebody else's macro.
    assert!(
        out.contains(":: my_runtime"),
        "the emission does not name the crate the example asked for: {out}"
    );
    assert!(
        !out.contains(":: notko"),
        "the emission names notko, which the example never asked for: {out}"
    );
}

#[test]
fn the_discovery_example_still_names_every_field() {
    // `## Letting a consumer write their own tiers`. Every field is written out
    // in the readme deliberately, so that a field added here breaks the example
    // rather than silently inheriting notko's answer, and this arm is what makes
    // that promise checkable: it does not spread a default either.
    let mine = Discovery {
        krate:        parse_quote!(::my_runtime),
        gate_feature: "my_release_arm".to_string(),
        dir:          "my-tiers".to_string(),
        env_var:      "MY_TIERS_PATH".to_string(),
        marker:       "@my-tier".to_string(),
        docs:         "the my-macros readme".to_string(),
    };

    let tier = mine
        .resolve("Cold", Span::call_site())
        .expect("a built-in tier resolves without any file on disk");
    let out = rewrite_fn(tier, subject())
        .expect("the rewrite succeeds")
        .to_string();

    assert!(
        out.contains(":: my_runtime"),
        "resolution dropped the crate the example named: {out}"
    );
}

#[test]
fn the_readme_does_not_name_a_module_this_crate_lacks() {
    // The structural half, and it catches what the two above cannot: a path in
    // readme prose rather than in a compiled block. Module granularity is all a
    // grep can honestly claim, so a renamed function inside a module still gets
    // past this one and is what the arms above are for.
    //
    // Read off `lib.rs` rather than typed out. A literal here is a second copy of
    // the module list, and a second copy is one that disagrees: a module removed
    // from the crate and left in the literal makes this arm pass by checking the
    // readme against a crate that no longer exists.
    let lib = include_str!("../src/lib.rs");
    let modules: Vec<String> = lib
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub mod "))
        .filter_map(|rest| rest.split(';').next())
        .map(|m| m.trim().to_string())
        .collect();
    assert!(
        modules.len() >= 4,
        "the public modules were not read off lib.rs, only {modules:?} came back"
    );

    let readme = include_str!("../README.md");
    let mut wrong = Vec::new();
    for (at, line) in readme.lines().enumerate() {
        let mut rest = line;
        while let Some(found) = rest.find("notko_macros_core::") {
            rest = &rest[found + "notko_macros_core::".len()..];
            let named: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !named.is_empty() && !modules.contains(&named) {
                wrong.push(format!("README.md:{}: `{named}`", at + 1));
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the readme names paths under modules this crate does not have, and \
         the public modules are {modules:?}:\n{}",
        wrong.join("\n"),
    );
}
