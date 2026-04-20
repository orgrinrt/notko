//! Demonstrates the downstream-extension pattern: a third-party crate
//! defines its own ZST tier marker implementing [`Tier`], and optionally
//! uses the core rewrite machinery from its own proc-macro attribute.

use notko_macros_core::tiers::{Cold, CustomTier, Hot, Strategy, Tier, Warm};

/// A hypothetical third-party tier. In a real downstream crate this would
/// come paired with either a `notko-optimizers/trace.rs` config file (for
/// consumption through notko-macros' built-in attribute) or a sibling
/// proc-macro crate publishing its own attribute.
pub struct Trace;
impl Tier for Trace {
    const NAME: &'static str = "trace";
    const STRATEGY: Strategy = Strategy::Cold;
    const INLINE: bool = false;
}

#[test]
fn builtin_markers_carry_the_right_names() {
    assert_eq!(Hot::NAME, "hot");
    assert_eq!(Warm::NAME, "warm");
    assert_eq!(Cold::NAME, "cold");
}

#[test]
fn builtin_markers_have_expected_strategies() {
    assert_eq!(Hot::STRATEGY, Strategy::Hot);
    assert_eq!(Warm::STRATEGY, Strategy::Passthrough);
    assert_eq!(Cold::STRATEGY, Strategy::Cold);
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
    assert_eq!(Strategy::from_name(Warm::NAME), Some(Strategy::Passthrough));
    assert_eq!(Strategy::from_name(Cold::NAME), Some(Strategy::Cold));
    assert_eq!(Strategy::from_name("unknown"), None);
}

#[test]
fn builtin_lookup_ignores_unknown_names() {
    assert!(CustomTier::builtin(Hot::NAME).is_some());
    assert!(CustomTier::builtin(Trace::NAME).is_none());
}
