//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

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
//! - `Named::NAME` single source of truth: the [`Tier::NAME`] const is the
//!   only place a tier's string identity lives.
//!
//! # Extension in downstream crates
//!
//! ```
//! use notko_macros_core::tiers::{Tier, Strategy};
//!
//! pub struct Trace;
//! impl Tier for Trace {
//!     const NAME: &'static str = "Trace";
//!     const STRATEGY: Strategy = Strategy::Cold;
//!     const INLINE: bool = false;
//! }
//! ```
//!
//! The new marker is usable at type level inside the downstream crate.
//! Making it usable from `#[profile(Trace)]` across crates still requires
//! either the config-file path (`notko-optimisers/Trace.rs`) or authoring
//! a new attribute macro in a sibling proc-macro crate. The shared trait
//! keeps every tier, built-in or third-party, identifying itself through
//! the same contract.

use syn::Path;

/// Marker trait implemented by each tier ZST.
///
/// # Required associated items
///
/// - [`NAME`](Self::NAME): string identity used in attribute arguments
///   and config-file `based_on` fields.
/// - [`STRATEGY`](Self::STRATEGY): which rewrite strategy this tier
///   selects.
/// - [`INLINE`](Self::INLINE): whether to emit `#[inline]` on the
///   rewritten function by default (callers can override via a custom
///   `CustomTier`).
pub trait Tier {
    const NAME: &'static str;
    const STRATEGY: Strategy;
    const INLINE: bool;
}

/// Hot tier. Minimum-overhead happy path. In release + `internal` feature:
/// rewrites to `Just<T>` with Err mapped to panic. Otherwise: `Outcome<T, E>`.
pub struct Hot;
impl Tier for Hot {
    const NAME: &'static str = "Hot";
    const STRATEGY: Strategy = Strategy::Hot;
    const INLINE: bool = true;
}

/// Warm tier: rewrites to `Maybe<T>`, which is what the tier names.
pub struct Warm;
impl Tier for Warm {
    const NAME: &'static str = "Warm";
    const STRATEGY: Strategy = Strategy::Warm;
    const INLINE: bool = false;
}

/// Cold tier: always `Outcome<T, E>`. `diagnose!(...)` calls preserved.
pub struct Cold;
impl Tier for Cold {
    const NAME: &'static str = "Cold";
    const STRATEGY: Strategy = Strategy::Cold;
    const INLINE: bool = false;
}

/// Rewrite strategy picked for a given tier (built-in or custom).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strategy {
    /// No rewrite.
    Passthrough,
    /// Warm: wrap in `Maybe<T>` always, discarding the error.
    Warm,
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
            Some(Strategy::Warm)
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
            Strategy::Warm => Warm::INLINE,
            // Reachable only through a custom tier asking for it by name; no
            // built-in marker carries it, so there is no marker to read.
            Strategy::Passthrough => false,
            Strategy::Cold => Cold::INLINE,
        }
    }
}

/// The cargo feature a hot rewrite's release arm is gated on when nothing
/// else is asked for.
pub const DEFAULT_GATE_FEATURE: &str = "internal";

/// The crate a rewrite emits through when nothing else is asked for.
///
/// A function rather than a constant, because a [`Path`] is not constructible
/// in const position.
#[must_use]
pub fn default_krate() -> Path {
    syn::parse_quote!(::notko)
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
    /// The crate path the rewrite emits its types through, as in
    /// `::notko::Outcome`.
    ///
    /// A rewrite has to name some crate for the type it rewrites to, and the
    /// name it picks becomes a requirement on whoever uses the attribute built
    /// on it. Fixing that name here would mean a third party's macro silently
    /// demanding `notko` be in scope in its own users' crates, under exactly
    /// that spelling, with nothing to say so until one of them failed to
    /// compile.
    pub krate: Path,
    /// The cargo feature that selects the release arm of a hot rewrite.
    ///
    /// Emitted as `cfg(feature = ...)` and therefore evaluated in the crate
    /// the attribute expands into, never here. One name for every framework in
    /// a dependency graph means two of them cannot be switched apart, so the
    /// name belongs to whoever builds the attribute.
    pub gate_feature: String,
}

impl CustomTier {
    /// Construct a `CustomTier` from a built-in tier marker.
    pub fn from_marker<T: Tier>() -> Self {
        Self {
            strategy: T::STRATEGY,
            inline: T::INLINE,
            panic_fmt: None,
            source_path: None,
            krate: default_krate(),
            gate_feature: DEFAULT_GATE_FEATURE.to_string(),
        }
    }

    /// Emit through a different crate than [`default_krate`].
    #[must_use]
    pub fn with_crate(mut self, krate: Path) -> Self {
        self.krate = krate;
        self
    }

    /// Gate the release arm on a different cargo feature than
    /// [`DEFAULT_GATE_FEATURE`].
    #[must_use]
    pub fn with_gate_feature(mut self, feature: impl Into<String>) -> Self {
        self.gate_feature = feature.into();
        self
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
