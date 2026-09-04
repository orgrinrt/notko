//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

#![no_std]
// Five unstable features across two optional sets, and neither set is reachable
// on stable. `try_trait_v2` and its residual companion carry `Try` for the
// three carriers; the three under `const` carry the const paths, where
// `const_destruct` is what lets a value be dropped in a const context and is
// the reason the const carriers can take ownership at all. All five are on the
// stabilisation pipeline and none has a known soundness hole.
#![cfg_attr(feature = "try_trait_v2", feature(try_trait_v2))]
#![cfg_attr(feature = "try_trait_v2", feature(try_trait_v2_residual))]
#![cfg_attr(feature = "const", feature(const_trait_impl))]
#![cfg_attr(feature = "const", feature(const_destruct))]
#![cfg_attr(feature = "const", feature(const_convert))]

//! notko: foundation primitives.
//!
//! Finnish notko: hollow, trough.
//!
//! Core's carriers differ by what they hold, `Option<T>` for absence and
//! `Result<T, E>` for an error with a payload, and the three here differ by
//! what a branch costs instead. [`Just<T>`] has no error case at all and is
//! `#[repr(transparent)]` over the value, so with `try_trait_v2` on there is
//! nothing for `?` to branch to. [`Maybe<T>`] handles ordinary absence, and
//! [`Outcome<T, E>`] takes an error carrying data. Each keeps a matching api on
//! purpose, so moving a function across is usually a type change with the body
//! left alone, although the guarantees are not equal.
//!
//! It's `#![no_std]`, no alloc, no platform deps, and in the default build no
//! dependencies at all. The `macros` feature is the one exception, since it
//! pulls the proc-macro crate in and that one uses std and `syn`, only at
//! compile time though, so none of it lands in the binary.
//!
//! Around the three there's a smaller set for the boundaries, where either the
//! bytes or the value are the contract: [`Slot<T>`] and [`MaybeNull<T>`] for a
//! layout somebody outside the crate is relying on, [`Boundable`] and
//! [`NonZeroable`] for an invariant checked once at construction, and
//! [`sink::Push`], [`sink::Emit`] and [`lend::Lend`] for handing data into
//! storage that is somebody else's.
//!
//! # Cost per call site
//!
//! [`Just`] / [`Maybe`] / [`Outcome`] put the hot, warm and cold split at
//! the control-flow level, so what a branch costs is picked where the call
//! is written instead of being fixed by the type:
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
//! internal-release builds where invariants are proven by construction. The
//! primitives work on their own, so the macro is optional.
//!
//! # ABI stability
//!
//! [`Maybe<T>`] takes part in the compiler's niche filling, so where the payload
//! has an invalid bit pattern of its own (a function pointer, `&T`, `&mut T`, a
//! `NonZero*`, a `NonNull<T>`, and the rest of that family) it comes out the same
//! size and alignment as the `T`, with [`Maybe::Isnt`] sitting in that pattern,
//! null for the pointers and zero for the `NonZero*`. That's the layout
//! `Option<T>` already gets for the same shapes, which makes `Maybe` a drop-in
//! across an FFI boundary whenever the payload is shaped like a pointer.
//!
//! The size parity is pinned by a `const` assertion per shape in `maybe.rs`, so a
//! future compiler dropping niche filling for user enums while keeping the
//! `Option` guarantee breaks the build rather than the ABI.
//!
//! Do note there's no such thing where both variants carry a value, as in
//! [`Outcome<T, E>`], since nothing can be folded into anything. If you want an
//! exact result layout across the boundary, wrap the payload in your own
//! `#[repr(C)]` struct instead of leaning on `repr(Rust)`'s tagged union.
//!
//! `Maybe` sits in a public API position for vocabulary reasons rather than
//! layout ones. `Option<&T>`, `Option<NonNull<T>>`, `Option<NonZero*>` and
//! `Option<fn>` all carry documented layout guarantees of their own, and
//! `Maybe` carries the same niche-filled layout for the same shapes. What you
//! get out of it is one word for presence across the whole surface, rather
//! than a representation `core` could not have given.
//!
//! # Where `Option` stays
//!
//! The core types are still there and still what the core traits ask for in their
//! own signatures, `fn next() -> Option<Self::Item>`, `fn partial_cmp() ->
//! Option<Ordering>`, `fn fmt() -> fmt::Result`, and nothing here changes that.
//! Those are the places `Option` cannot be avoided, so that's where it stays, and
//! [`iter::IteratorExt`] and [`cmp::PartialOrdExt`] are the bridge at the call
//! site rather than an attempt to move the boundary.

// The README's `rust` blocks are compiled as doctests. Only those: the shell
// blocks are prose as far as this is concerned, and changing a fence would drop
// the check with nothing saying so.
//
// Without this the examples are the one part of the documentation nothing
// verifies, which is the part a reader is most likely to copy. Adding it found
// two that did not build: one calling functions it never defined, and one
// importing `profile` without the feature that provides it.
//
// Gated on every feature the blocks reach for, for the same reason `try_smoke`
// carries `required-features`: a block that names something a feature provides
// fails on that feature's absence rather than on anything being wrong. One
// block shows `#[profile]`, which `macros` provides; another uses `?` on
// `Outcome`, which `try_trait_v2` provides. The gate names both, so
// `cargo test --all-features` is the run that exercises the examples and no
// narrower configuration compiles them at all.
#[cfg(all(doctest, feature = "macros", feature = "try_trait_v2"))]
#[doc = include_str!("../README.md")]
struct Readme;

pub mod bounded;
pub mod cmp;
pub mod consttry;
pub mod ctor;
pub mod iter;
pub mod just;
pub mod lend;
pub mod maybe;
pub mod nonzero;
pub mod outcome;
pub mod prelude;
pub mod sink;
pub mod slot;

pub use bounded::{BoundError, Boundable};
pub use consttry::{ConstFromResidual, ConstTry};
pub use ctor::HasTrivialCtor;
pub use just::{Just, JustIter};
pub use maybe::{Maybe, MaybeIter, MaybeNull, NicheFilled};
pub use nonzero::NonZeroable;
#[cfg(feature = "macros")]
pub use notko_macros::profile;
pub use outcome::Outcome;
pub use slot::Slot;
