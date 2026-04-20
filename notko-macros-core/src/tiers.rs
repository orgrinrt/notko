//! Tier markers + rewrite-strategy model.
//!
//! Tiers are ZST marker types implementing the [`Tier`] trait. [`Hot`],
//! [`Warm`], and [`Cold`] are the shipped markers; third-party crates can
//! implement [`Tier`] on their own ZSTs to register additional built-ins
//! in their own code.
//!
//! Following the shared-principles convention:
//! - ZST markers for discrimination, not string literals at comparison
//!   sites.
//! - `Named::NAME` single source of truth — the [`Tier::NAME`] const is the
//!   only place a tier's string identity lives.
//!
//! # Extension in downstream crates
//!
//! ```ignore
//! use notko_macros_core::tiers::{Tier, Strategy};
//!
//! pub struct Trace;
//! impl Tier for Trace {
//!     const NAME: &'static str = "trace";
//!     const STRATEGY: Strategy = Strategy::Cold;
//!     const INLINE: bool = false;
//! }
//! ```
//!
//! The new marker is usable at type level inside the downstream crate.
//! Making it usable from `#[optimize_for(trace)]` across crates still
//! requires either the config-file path (`notko-optimizers/trace.rs`) or
//! authoring a new attribute macro in a sibling proc-macro crate. The
//! shared trait keeps every tier — built-in or third-party — identifying
//! itself through the same contract.

/// Marker trait implemented by each tier ZST.
///
/// # Required associated items
///
/// - [`NAME`](Self::NAME) — string identity used in attribute arguments
///   and config-file `based_on` fields.
/// - [`STRATEGY`](Self::STRATEGY) — which rewrite strategy this tier
///   selects.
/// - [`INLINE`](Self::INLINE) — whether to emit `#[inline]` on the
///   rewritten function by default (callers can override via a custom
///   `CustomTier`).
pub trait Tier {
    const NAME: &'static str;
    const STRATEGY: Strategy;
    const INLINE: bool;
}

/// Hot tier — minimum-overhead happy path. In release + `internal` feature:
/// rewrites to `Just<T>` with Err → panic. Otherwise: `Outcome<T, E>`.
pub struct Hot;
impl Tier for Hot {
    const NAME: &'static str = "hot";
    const STRATEGY: Strategy = Strategy::Hot;
    const INLINE: bool = true;
}

/// Warm tier — passthrough. Preserves the source `Result<T, E>` signature.
pub struct Warm;
impl Tier for Warm {
    const NAME: &'static str = "warm";
    const STRATEGY: Strategy = Strategy::Passthrough;
    const INLINE: bool = false;
}

/// Cold tier — always `Outcome<T, E>`. `diagnose!(...)` calls preserved.
pub struct Cold;
impl Tier for Cold {
    const NAME: &'static str = "cold";
    const STRATEGY: Strategy = Strategy::Cold;
    const INLINE: bool = false;
}

/// Rewrite strategy picked for a given tier (built-in or custom).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strategy {
    /// No rewrite.
    Passthrough,
    /// Hot: in debug/standalone, wrap in `Outcome`; in release/internal,
    /// strip to `Just<T>` + panic-on-Err.
    Hot,
    /// Cold: wrap in `Outcome` always.
    Cold,
}

impl Strategy {
    /// Resolve a strategy name used in config-file `based_on` fields.
    ///
    /// Accepts exactly the [`Tier::NAME`] of one of the shipped built-in
    /// markers. Unknown names return `None`.
    pub fn from_name(name: &str) -> Option<Self> {
        if name == Hot::NAME {
            Some(Strategy::Hot)
        } else if name == Warm::NAME {
            Some(Strategy::Passthrough)
        } else if name == Cold::NAME {
            Some(Strategy::Cold)
        } else {
            None
        }
    }

    /// Default `inline` flag for a built-in strategy.
    pub fn default_inline(self) -> bool {
        match self {
            Strategy::Hot => Hot::INLINE,
            Strategy::Passthrough => Warm::INLINE,
            Strategy::Cold => Cold::INLINE,
        }
    }
}

/// Parameters for a resolved tier (built-in or custom-file-sourced).
#[derive(Clone, Debug)]
pub struct CustomTier {
    /// Which built-in strategy this tier uses.
    pub strategy: Strategy,
    /// If true, emit `#[inline]` on the rewritten function.
    pub inline: bool,
    /// Optional override of the panic message format for hot-strategy tiers.
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
    /// Construct a `CustomTier` from a built-in tier marker.
    pub fn from_marker<T: Tier>() -> Self {
        Self {
            strategy: T::STRATEGY,
            inline: T::INLINE,
            panic_fmt: None,
            source_path: None,
        }
    }

    /// Resolve a tier name against the built-in ZST markers.
    /// Returns `None` for unrecognised names; callers then fall back to the
    /// config-file discovery path.
    pub fn builtin(name: &str) -> Option<Self> {
        if name == Hot::NAME {
            Some(Self::from_marker::<Hot>())
        } else if name == Warm::NAME {
            Some(Self::from_marker::<Warm>())
        } else if name == Cold::NAME {
            Some(Self::from_marker::<Cold>())
        } else {
            None
        }
    }
}
