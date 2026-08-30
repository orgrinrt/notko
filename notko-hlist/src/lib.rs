//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

#![no_std]
// Two unstable features behind two optional sets, neither reachable on stable.
// `const_trait_impl` carries the const path of `Cardinal`, which is what makes
// a length computable at compile time rather than at each call.
// `marker_trait_attr` carries membership, where the head match and the
// recursive tail match genuinely overlap and coherence has to be told that the
// overlap is intended.
#![cfg_attr(feature = "const", feature(const_trait_impl))]
#![cfg_attr(feature = "membership", feature(marker_trait_attr))]

//! A heterogeneous type-level list, and the structural facts about one.
//!
//! Zero deps, no std, no alloc, and nothing here exists at runtime: a list is
//! [`Empty`] or [`Cons<H, T>`], both of them uninhabited-shaped markers, and
//! everything the crate says about a list it says through traits the compiler
//! resolves.
//!
//! # Contents
//!
//! - [`Empty`] and [`Cons<H, T>`]: the leaf and the cell.
//! - [`List`]: that something is one, sealed so nothing else can be. Every
//!   other trait here has it as a supertrait, which is what makes a bound on
//!   one of them a fact rather than a claim.
//! - [`Cardinal`]: what a count is, as zero and a successor. A consumer
//!   implements it for its own number type, which is the whole reason the
//!   count is a parameter here rather than a type this crate picked.
//! - [`Length<N>`]: the cardinality of a list, in the consumer's count type.
//! - [`Contains<X>`] and [`ContainsAll<L>`]: membership of one type, and of
//!   every member of another list.
//! - [`Concat<L>`]: append, as a type-level function.
//!
//! # Why a crate rather than a copy
//!
//! Three repositories had independently declared `Empty` and `Cons<H, T>`
//! byte for byte, then regrown the same operations under different names, so
//! a list built by one of them could not be handed to another and no error
//! anywhere said why. What is here is the union of what they had grown.
//!
//! # Naming the list in your own vocabulary
//!
//! The generic names are meant to appear at the definition and almost nowhere
//! else. Alias them:
//!
//! ```
//! use notko_hlist::{Cons, Empty};
//!
//! type Scalar = Empty;
//! type Axis<H, T> = Cons<H, T>;
//!
//! struct Rows;
//! struct Cols;
//!
//! type Matrix = Axis<Rows, Axis<Cols, Scalar>>;
//! ```
//!
//! A shape with no axes being the leaf is not a pun: it is a scalar.
//!
//! # What is not here
//!
//! **No value-level fold.** Reducing a list with an identity and an
//! associative combine needs the algebra, and the algebra is numerics
//! territory rather than this crate's. What is here is the structural folds,
//! which need none: length, concatenation, membership.
//!
//! # Features
//!
//! Both are on by default and both need nightly, so a consumer on stable
//! takes `default-features = false` and gets the list, `Concat`, and the
//! runtime spelling of `Length`.
//!
//! | Feature | What it adds | Why it is not stable |
//! |---|---|---|
//! | `const` | [`Cardinal`] becomes a const trait and [`Length`] gains `LEN`, a compile-time constant | `const_trait_impl` |
//! | `membership` | [`Contains`] and [`ContainsAll`] | `marker_trait_attr` |
//!
//! Without `const` the count is still there and still correct, as
//! [`Length::len`], computed rather than named. Without `membership` there is
//! no way to ask whether a list holds a type, because the two impls that
//! answer it overlap by construction.

// The README's `rust` block is compiled as a doctest, because the example a
// reader is most likely to copy is otherwise the one part of the crate nothing
// verifies. Gated on the features it reaches for, since it declares a const
// impl and bounds on membership, and off them it would fail for the wrong
// reason.
#[cfg(all(doctest, feature = "const", feature = "membership"))]
#[doc = include_str!("../README.md")]
pub struct Readme;

mod cardinal;
mod concat;
mod length;
mod list;
#[cfg(feature = "membership")]
mod membership;

pub use cardinal::Cardinal;
pub use concat::Concat;
pub use length::Length;
pub use list::{Cons, Empty, List};
#[cfg(feature = "membership")]
pub use membership::{Contains, ContainsAll};
