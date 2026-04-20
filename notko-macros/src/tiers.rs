//! Tier and rewrite-strategy model.

/// The three built-in fallibility tiers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tier {
    Hot,
    Warm,
    Cold,
}

impl Tier {
    /// Parse a tier name against the built-in set.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "hot" => Some(Tier::Hot),
            "warm" => Some(Tier::Warm),
            "cold" => Some(Tier::Cold),
            _ => None,
        }
    }

    /// The default rewrite strategy for each tier.
    pub fn strategy(self) -> Strategy {
        match self {
            Tier::Hot => Strategy::Hot,
            Tier::Warm => Strategy::Passthrough,
            Tier::Cold => Strategy::Cold,
        }
    }
}

/// Rewrite strategy picked for a given tier (or custom tier via discover.rs).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Strategy {
    /// No rewrite.
    Passthrough,
    /// Hot-path: in debug/standalone, wrap in `Outcome`; in release/internal,
    /// strip to `Just<T>` + panic-on-Err.
    Hot,
    /// Cold-path: wrap in `Outcome` always.
    Cold,
}

/// Parameters for a custom tier sourced from a `notko-optimizers/<X>.rs` file.
#[derive(Clone, Debug)]
pub struct CustomTier {
    /// Which built-in strategy this tier is based on.
    pub strategy: Strategy,
    /// If true, emit `#[inline]` on the rewritten function.
    pub inline: bool,
    /// Optional override of the panic message format for hot-strategy tiers.
    /// Default: `"hot path invariant violated: {err:?}"`.
    pub panic_fmt: Option<String>,
    /// Absolute path to the source file (for potential `include!` of its
    /// helper module by the rewrite layer). None for built-in tiers.
    ///
    /// Currently unread; reserved for the notko-build cross-crate
    /// accumulation path and future helper-module injection.
    #[allow(dead_code)]
    pub source_path: Option<std::path::PathBuf>,
}

impl CustomTier {
    pub fn from_builtin(tier: Tier) -> Self {
        Self {
            strategy: tier.strategy(),
            inline: matches!(tier, Tier::Hot),
            panic_fmt: None,
            source_path: None,
        }
    }
}
