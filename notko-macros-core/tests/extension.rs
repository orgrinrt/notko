//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Demonstrates the downstream-extension pattern: a third-party crate
//! defines its own ZST tier marker implementing [`Tier`], and optionally
//! uses the core rewrite machinery from its own proc-macro attribute.

use notko_macros_core::tiers::{Cold, CustomTier, Hot, Strategy, Tier, Warm};

/// A hypothetical third-party tier. In a real downstream crate this would
/// come paired with either a `notko-optimisers/Trace.rs` config file (for
/// consumption through notko-macros' built-in attribute) or a sibling
/// proc-macro crate publishing its own attribute.
pub struct Trace;
impl Tier for Trace {
    const INLINE: bool = false;
    const NAME: &'static str = "Trace";
    const STRATEGY: Strategy = Strategy::Cold;
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
/// What keeps the two in step is the test below it, which reads the readme and
/// asserts every line of the block above appears somewhere in it. The check runs
/// one way and by substring, so a line dropped from the readme is caught and a
/// stale line left in it is not, and nothing constrains their order or whether
/// they sit together.
#[test]
fn the_readme_authoring_example_still_builds() {
    let input: syn::ItemFn = syn::parse_quote! {
        fn load(path: &str) -> Result<u32, std::io::Error> {
            Ok(path.len() as u32)
        }
    };
    // readme-example:start
    let mut tier = CustomTier::from_marker::<Hot>()
        .with_crate(syn::parse_quote!(::my_runtime))
        .with_gate_feature("my_release_arm");
    tier.panic_fmt = Some("asserted invariant violated: {err:?}".into());
    // readme-example:end
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

/// The readme's example and the compiled one above are the same lines.
///
/// The test above compiles a tier construction, and the readme prints one. They
/// were two copies of the same code with nothing tying them together, so the
/// readme could name a field that had been renamed and stay green: the note
/// where this comment is used to say exactly that, and say it to a reader who
/// had no reason to be reading a test file.
///
/// So the lines between the sentinels above are the single source, and this
/// asserts every one of them appears in the readme. Editing one side alone is
/// what turns it red, whichever side that is.
///
/// Both files ship in the package, so this runs from an unpacked tarball as
/// well as from a checkout.
///
/// What it does not cover: prose. The readme can describe the example wrongly
/// in the paragraph underneath and this stays green, because the check is over
/// code and the paragraph is not code.
#[test]
fn the_readme_example_is_the_one_compiled_above() {
    const THIS_FILE: &str = include_str!("extension.rs");
    const README: &str = include_str!("../README.md");

    let example: Vec<&str> = THIS_FILE
        .split("// readme-example:start")
        .nth(1)
        .and_then(|rest| rest.split("// readme-example:end").next())
        .expect("the sentinels in the test above")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    // A sentinel pair that matched an empty region would make every assertion
    // below vacuous, and vacuous is what this whole test exists to stop being.
    assert!(
        example.len() >= 4,
        "the sentinels caught {} lines, so they have moved or the example shrank",
        example.len(),
    );

    let missing: Vec<&str> = example
        .iter()
        .copied()
        .filter(|line| !README.contains(line))
        .collect();

    assert!(
        missing.is_empty(),
        "the readme's authoring example no longer carries these lines from the \
         test that compiles it:\n{}",
        missing.join("\n"),
    );
}

/// The control for `the_readme_authoring_example_still_builds`.
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
    let out = notko_macros_core::rewrite::rewrite_fn(CustomTier::from_marker::<Hot>(), input)
        .expect("a hot rewrite at the defaults")
        .to_string();

    assert!(out.contains(":: notko"), "{out}");
    assert!(out.contains("\"internal\""), "{out}");
}

/// Which strategies read which of the two fields the readme singles out.
///
/// The readme says one of them is the hot strategy's alone and the other is
/// read by the three strategies that write a type, and a sweep is the only
/// honest way to say that: two tests at `Hot` establish nothing about the other
/// three, and a paragraph asserting all four while the suite covers one is how
/// the wrong claim got written down in the first place.
///
/// The name says three rather than every, because the body asserts `Passthrough`
/// names nothing and a name claiming every strategy would be the same overreach
/// one rung up, where nothing checks it.
#[test]
fn the_crate_path_is_read_by_three_strategies_and_the_gate_by_hot_alone() {
    fn emitted(strategy: Strategy) -> String {
        let input: syn::ItemFn = syn::parse_quote! {
            fn load(path: &str) -> Result<u32, std::io::Error> {
                Ok(path.len() as u32)
            }
        };
        let tier = CustomTier {
            strategy,
            inline: false,
            panic_fmt: None,
            source_path: None,
            ..CustomTier::from_marker::<Hot>()
        };
        notko_macros_core::rewrite::rewrite_fn(tier, input)
            .expect("a rewrite at every strategy")
            .to_string()
    }

    for s in [Strategy::Hot, Strategy::Warm, Strategy::Cold] {
        assert!(
            emitted(s).contains(":: notko"),
            "{s:?} does not name the crate"
        );
    }
    // Passthrough writes the function back untouched, so it names nothing, and
    // saying "every strategy" without this would be saying it about four.
    assert!(!emitted(Strategy::Passthrough).contains(":: notko"));

    assert!(emitted(Strategy::Hot).contains("\"internal\""));
    for s in [Strategy::Warm, Strategy::Cold, Strategy::Passthrough] {
        assert!(
            !emitted(s).contains("\"internal\""),
            "{s:?} reads the gate feature"
        );
    }
}
