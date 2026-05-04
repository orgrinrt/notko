#![no_std]
#![cfg_attr(feature = "try_trait_v2", feature(try_trait_v2))]
#![cfg_attr(feature = "const", feature(const_trait_impl))]

//! notko: foundation primitives for the hilavitkutin stack.
//!
//! Finnish *notko*: hollow, trough. The ground every downstream crate sits on.
//!
//! Zero deps, no std, no alloc. Ships the scalar-level vocabulary used in
//! place of `core::option::Option`, `core::result::Result`, and bare integer
//! primitives across arvo, hilavitkutin, and vehje.
//!
//! # Contents
//!
//! - [`Just<T>`]: infallible value wrapper with a zero-cost `Try` impl.
//! - [`Maybe<T>`]: presence (Option replacement). Niche-fills for pointer-shaped `T` so
//!   `Maybe<unsafe extern "C" fn(...)>`, `Maybe<&T>`, `Maybe<NonNull<T>>`, etc. are one
//!   pointer wide with `Isnt` as the null bit pattern, matching `Option<T>`'s layout for
//!   the same shapes. Size parity is const-asserted in `maybe.rs`.
//! - [`Outcome<T, E>`]: fallible (Result replacement). Layout is tagged-union with
//!   platform-standard field ordering; no `repr(C)` forcing. For FFI-critical two-variant
//!   results, wrap the payload in a dedicated `#[repr(C)]` struct.
//! - [`Slot<T>`]: niche-filled `Maybe<T>` wrapper for `T: NonZeroable + NicheFilled`,
//!   `#[repr(transparent)]` so layout matches `T`.
//! - [`Boundable`]: trait for "this type is bounded to `[MIN, MAX]`".
//! - [`NonZeroable`]: trait for "this type has a zero sentinel and a
//!   nonzero guarantee form".
//! - [`ConstTry`] / [`ConstFromResidual`]: const-callable parallels of
//!   `core::ops::Try` / `FromResidual`. Gated behind the `const` cargo
//!   feature (default-on).
//!
//! # Three tiers of fallibility
//!
//! [`Just`] / [`Maybe`] / [`Outcome`] mirror arvo's Hot / Warm / Cold
//! numeric strategy at the control-flow level:
//!
//! | Tier | Type | Cold path |
//! |------|------|-----------|
//! | Hot  | [`Just<T>`]       | None: no branch. `?` compiles away. |
//! | Warm | [`Maybe<T>`]      | One-bit discriminant, no payload. |
//! | Cold | [`Outcome<T, E>`] | Full error payload + branch. |
//!
//! The companion `#[profile(Hot | Warm | Cold)]` proc-macro (see the
//! `notko-macros` crate, re-exported at the root under the `macros`
//! feature) rewrites a function's return type between builds:
//! `Outcome<T, E>` in debug and standalone consumers, `Just<T>` in
//! internal-release builds where invariants are proven by construction.
//! The primitives are usable without the macro; the macro is an optional
//! accelerator.
//!
//! # ABI stability
//!
//! [`Maybe<T>`] participates in Rust's niche-filling optimisation. For payload
//! types with a niche (function pointers, `&T`, `&mut T`, `NonZero*`,
//! `NonNull<T>`, and similar), `Maybe<T>` has identical size and alignment
//! to `T` itself, with [`Maybe::Isnt`] represented by `T`'s invalid bit
//! pattern (null for pointers, zero for `NonZero*`). This is the same
//! layout `Option<T>` gets for those shapes; `Maybe<T>` is a drop-in
//! FFI-compatible replacement whenever the payload is pointer-shaped.
//!
//! Size parity is pinned by compile-time `assert!` in `maybe.rs`. If a
//! future rustc drops niche-filling for user enums while keeping
//! `Option`-specific guarantees, those assertions fail compilation and
//! the stack learns about it immediately.
//!
//! For fully general payload types (both variants carry values, as in
//! `Outcome<T, E>`), there is no single-pointer representation. Code
//! that needs a specific C ABI result layout should wrap the payload
//! in a dedicated `#[repr(C)]` struct rather than rely on
//! `Outcome`'s default Rust-repr tagged-union layout.
//!
//! The stack's plugin dispatch and FFI surfaces previously insisted
//! that bare `Option` / `Result` could not appear because `core`
//! niche-filling for `Option` was "not a stable contract". That is
//! inaccurate: `Option<&T>`, `Option<NonNull<T>>`, `Option<NonZero*>`,
//! and `Option<fn>` all carry documented layout guarantees. With
//! `Maybe` now equipped with the same niche-filled layout for the same
//! shapes, `Maybe` replaces `Option` in public API positions for
//! vocabulary reasons (one word for presence, uniform across the
//! stack), not layout reasons.
//!
//! # Sanctioned use of std primitives
//!
//! The std types still exist and are still what std trait method signatures
//! require (`fn next() -> Option<Self::Item>`, `fn partial_cmp() ->
//! Option<Ordering>`, `fn fmt() -> fmt::Result`). Those impls are the only
//! sanctioned use of std primitives in stack code.

pub mod bounded;
pub mod cmp;
pub mod consttry;
pub mod iter;
pub mod just;
pub mod maybe;
pub mod nonzero;
pub mod outcome;
pub mod prelude;
pub mod slot;

pub use bounded::Boundable;
pub use consttry::{ConstFromResidual, ConstTry};
pub use just::Just;
pub use maybe::{NicheFilled, Maybe, MaybeNull};
pub use nonzero::NonZeroable;
pub use outcome::Outcome;
pub use slot::Slot;

#[cfg(feature = "macros")]
pub use notko_macros::profile;
