//! Const-callable parallels of `core::ops::Try` and `core::ops::FromResidual`.
//!
//! `core::ops::Try` is not `pub const trait` as of 2026-05 nightly, so
//! `?` on `Just<T>`, `Maybe<T>`, `Outcome<T, E>`, and `Bool` cannot be
//! used in const context. This module ships a substrate-internal
//! parallel surface: explicit `match x.branch() { ... }` calls work in
//! const fn bodies.
//!
//! `?` syntax itself stays non-const because rustc desugars `?`
//! directly to `core::ops::Try::branch`. Adopting `ConstTry` does not
//! change `?`-syntax behaviour. When core eventually lifts `Try` to
//! `pub const trait Try`, this bridge can be removed and existing
//! `impl Try` blocks rewritten to `impl const Try` (no consumer-API
//! migration required since `?` desugaring switches automatically).
//!
//! Both traits are gated behind feature `const` (default-on). Without
//! the feature, the traits exist as regular `pub trait`s; impls drop
//! the `const` keyword. This lets notko consumers on stable Rust opt
//! out of the const-trait machinery via `default-features = false`.
//!
//! The const-variant impls in `just.rs` / `maybe.rs` / `outcome.rs`
//! carry an extra `T: Copy` bound (and `E: Copy` for Outcome). The
//! restriction exists because const fn cannot evaluate destructors
//! for arbitrary generic `T` under current rustc nightly. Consumers
//! pushing non-Copy types through `branch` / `from_output` reach for
//! the non-const variant via `default-features = false`. Substrate
//! consumers (Bool, USize, Cap, Bits, NUSize) are all Copy, so the
//! restriction is invisible at the typical call site.
//!
//! # Module layout
//!
//! The const-trait declarations use the `pub const trait` keyword and
//! `#[feature(const_trait_impl)]`, both still unstable as of rustc
//! 1.96 nightly. Rustc parses cfg-gated items at the inline-mod level
//! before evaluating cfg-attrs, so `#[cfg(feature = "const")] mod x {
//! pub const trait Foo { ... } }` fires a feature-gate diagnostic
//! when the feature is off. The fix is file-level gating: the
//! const-path lives in `consttry_const_path.rs` loaded only when the
//! feature is on; the plain-path lives in `consttry_plain_path.rs`
//! loaded only when off. Cfg on the `mod` declaration controls
//! whether the file is opened at all.

#[cfg(feature = "const")]
#[path = "consttry_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "consttry_plain_path.rs"]
mod plain_path;

#[cfg(feature = "const")]
pub use const_path::{ConstFromResidual, ConstTry};

#[cfg(not(feature = "const"))]
pub use plain_path::{ConstFromResidual, ConstTry};
