//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The AST rewriting behind [`notko-macros`]' `#[profile(Tier)]`, as an ordinary
//! library.
//!
//! It is here because a proc-macro crate cannot export anything that is not a
//! macro, so there was nowhere else to put the rewrite engine, and once it had to
//! exist separately it may as well be public: a fallibility-tier attribute of
//! your own can build on this rather than doing the rewrite again. Four modules
//! carry it, the tier vocabulary in [`tiers`], the parse in [`parse`], the
//! lookup that turns a tier name into something to rewrite with in [`discover`],
//! and the rewrite itself in [`rewrite`].
//!
//! Do note that [`discover::Discovery`] names the crate the rewritten body will
//! reach for and the feature its release arm is gated on, and both of those
//! become requirements on your users rather than on you, so an attribute of your
//! own writes its own six fields rather than spreading the default, which is
//! notko's. The crate README has that and an example of both routes.
//!
//! [`notko-macros`]: https://crates.io/crates/notko-macros

pub mod discover;
pub mod parse;
pub mod rewrite;
pub mod tiers;
