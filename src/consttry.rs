//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-callable parallels of `core::ops::Try` and `core::ops::FromResidual`.
//!
//! An explicit `match x.branch() { ... }` on `Just<T>`, `Maybe<T>`,
//! `Outcome<T, E>` or `Bool` works in a const fn body through these
//! traits. `?` on those types works too, but that route goes through
//! `core::ops::Try` rather than through here, and costs more gates: it
//! needs `try_trait_v2`, which is off by default, plus `const_try` and
//! `const_try_residual` on top. These traits need only
//! `const_trait_impl`, and they come with the default feature set.
//!
//! Owning the surface is also what keeps the crate independent of how
//! new a nightly is, and of how the const shape of `Try` upstream
//! settles. There is a second reason it could not simply be borrowed:
//! orphan rules forbid implementing `core`'s `Residual` for
//! `Infallible`, which is why `Just` carries a residual of its own.
//!
//! Both traits are gated behind feature `const` (default-on). Without
//! the feature, the traits exist as regular `pub trait`s; impls drop
//! the `const` keyword. This lets notko consumers on stable Rust opt
//! out of the const-trait machinery via `default-features = false`.
//!
//! The const-variant impls carry a `[const] Destruct` bound, which is
//! what const evaluation actually asks for: a value it can drop. That
//! admits every type without a destructor, and for a runtime caller the
//! bound disappears, so those get every type at all. A type with a real
//! `Drop` is refused in const context and accepted outside it, which is
//! the line the language draws rather than one this crate invented.
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
