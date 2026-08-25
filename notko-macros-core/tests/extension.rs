//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Demonstrates the downstream-extension pattern: a third-party crate
//! defines its own ZST tier marker implementing [`Tier`], and optionally
//! uses the core rewrite machinery from its own proc-macro attribute.

use notko_macros_core::tiers::{Cold, CustomTier, Hot, Strategy, Tier, Warm};

/// A hypothetical third-party tier. In a real downstream crate this would
/// come paired with either a `notko-optimizers/Trace.rs` config file (for
/// consumption through notko-macros' built-in attribute) or a sibling
/// proc-macro crate publishing its own attribute.
pub struct Trace;
impl Tier for Trace {
    const NAME: &'static str = "Trace";
    const STRATEGY: Strategy = Strategy::Cold;
    const INLINE: bool = false;
}

#[test]
fn custom_tier_from_third_party_marker() {
    let t = CustomTier::from_marker::<Trace>();
    assert_eq!(t.strategy, Strategy::Cold);
    assert!(!t.inline);
    assert!(t.panic_fmt.is_none());
    assert!(t.source_path.is_none());
}

#[test]
fn strategy_from_name_matches_tier_name_consts() {
    // Strategy::from_name accepts the NAME const of each built-in tier,
    // confirming the ZST markers are the single source of truth.
    assert_eq!(Strategy::from_name(Hot::NAME), Some(Strategy::Hot));
    assert_eq!(Strategy::from_name(Warm::NAME), Some(Strategy::Warm));
    assert_eq!(Strategy::from_name(Cold::NAME), Some(Strategy::Cold));
    assert_eq!(Strategy::from_name("unknown"), None);
}

#[test]
fn builtin_lookup_ignores_unknown_names() {
    assert!(CustomTier::builtin(Hot::NAME).is_some());
    assert!(CustomTier::builtin(Trace::NAME).is_none());
}

/// The readme's authoring example, minus the proc-macro wrapper it cannot have
/// here.
///
/// That example is not doctested, because `#[proc_macro_attribute]` only exists
/// in a proc-macro crate and this is not one. Everything inside it is ordinary
/// code, though, and everything inside it is what goes stale: a renamed field, a
/// changed type, a strategy that stops existing. So the body lives here and the
/// wrapper stays prose.
///
/// Keep the two in step. A reader copying the readme is copying this.
#[test]
fn the_readme_authoring_example_still_builds() {
    let input: syn::ItemFn = syn::parse_quote! {
        fn load(path: &str) -> Result<u32, std::io::Error> {
            Ok(path.len() as u32)
        }
    };
    let tier = CustomTier {
        strategy:     Strategy::Hot,
        inline:       true,
        panic_fmt:    Some("asserted invariant violated: {err:?}".into()),
        source_path:  None,
        krate:        syn::parse_quote!(::my_runtime),
        gate_feature: "my_release_arm".to_string(),
    };
    let out = notko_macros_core::rewrite::rewrite_fn(tier, input)
        .expect("the readme's example rewrites")
        .to_string();

    // The two fields the readme singles out are the two a reader is most
    // likely to leave at the default by accident, so this is the assertion
    // that says they were read at all.
    assert!(out.contains("my_runtime"), "{out}");
    assert!(out.contains("my_release_arm"), "{out}");
    // And the default the readme warns about is not also present, which the
    // two above cannot see: an emitter naming both would pass them.
    assert!(!out.contains(":: notko"), "{out}");
    assert!(!out.contains("\"internal\""), "{out}");
}

/// The control for the test above.
///
/// Its last two assertions say the defaults are absent, and an emitter that
/// named neither crate nor feature would satisfy them by writing nothing at
/// all. So this is the same rewrite left at the defaults, asserting the two
/// spellings the other one refuses do appear when nobody overrides them.
#[test]
fn the_defaults_are_what_the_readme_says_they_are() {
    let input: syn::ItemFn = syn::parse_quote! {
        fn load(path: &str) -> Result<u32, std::io::Error> {
            Ok(path.len() as u32)
        }
    };
    let out = notko_macros_core::rewrite::rewrite_fn(
        CustomTier::from_marker::<Hot>(),
        input,
    )
    .expect("a hot rewrite at the defaults")
    .to_string();

    assert!(out.contains(":: notko"), "{out}");
    assert!(out.contains("\"internal\""), "{out}");
}
