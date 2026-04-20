#![no_std]
#![cfg_attr(feature = "try_trait_v2", feature(try_trait_v2))]

//! notko — foundation primitives for the hilavitkutin stack.
//!
//! Finnish *notko*: hollow, trough. The ground every downstream crate sits on.
//!
//! Zero deps, no std, no alloc. Ships the scalar-level vocabulary used in
//! place of `core::option::Option`, `core::result::Result`, and bare integer
//! primitives across arvo, hilavitkutin, and clause.
//!
//! # Contents
//!
//! - [`Just<T>`] — infallible value wrapper with a zero-cost `Try` impl.
//! - [`Maybe<T>`] — presence (Option replacement). ABI-stable `repr(C)`.
//! - [`Outcome<T, E>`] — fallible (Result replacement). ABI-stable `repr(C)`.
//! - [`Boundable`] — trait for "this type is bounded to `[MIN, MAX]`".
//! - [`NonZeroable`] — trait for "this type has a zero sentinel and a
//!   nonzero guarantee form".
//!
//! # Three tiers of fallibility
//!
//! [`Just`] / [`Maybe`] / [`Outcome`] mirror arvo's Hot / Warm / Cold
//! numeric strategy at the control-flow level:
//!
//! | Tier | Type | Cold path |
//! |------|------|-----------|
//! | Hot  | [`Just<T>`]       | None — no branch. `?` compiles away. |
//! | Warm | [`Maybe<T>`]      | One-bit discriminant, no payload. |
//! | Cold | [`Outcome<T, E>`] | Full error payload + branch. |
//!
//! An `#[optimize_for(...)]` proc-macro (sibling crate, TBD) rewrites a
//! function's return type between builds: `Outcome<T, E>` in debug /
//! standalone consumers, `Just<T>` in internal-release builds where
//! invariants are proven by construction. The primitives are usable
//! without the macro — the macro is an optional accelerator.
//!
//! # ABI stability
//!
//! [`Maybe`] and [`Outcome`] are `#[repr(C)]`; layout is stable across
//! compilations and ABI boundaries. `core::option::Option<T>` depends on
//! niche optimisation that is not a stable contract. The stack's plugin
//! dispatch and FFI surfaces require stable layout, so bare `Option` /
//! `Result` cannot appear in public API positions.
//!
//! # Sanctioned use of std primitives
//!
//! The std types still exist and are still what std trait method signatures
//! require (`fn next() -> Option<Self::Item>`, `fn partial_cmp() ->
//! Option<Ordering>`, `fn fmt() -> fmt::Result`). Those impls are the only
//! sanctioned use of std primitives in stack code.

pub mod bounded;
pub mod just;
pub mod maybe;
pub mod nonzero;
pub mod outcome;
pub mod prelude;

pub use bounded::Boundable;
pub use just::Just;
pub use maybe::Maybe;
pub use nonzero::NonZeroable;
pub use outcome::Outcome;
