//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `HasTrivialCtor`: type ships a no-arg constructor.
//!
//! Granular contract trait. A type that impls `HasTrivialCtor` is saying
//! it has a `fn new() -> Self` taking no arguments. Handy for markers and
//! phantom wrappers, where you want `Type::<T>::new()` to mean the same
//! thing everywhere instead of every wrapper inventing its own spelling.
//!
//! Independent of any specific framework. Any wrapper, marker, or
//! unit-shaped type that wants the convention impls this trait.
//!
//! # Module layout
//!
//! Same file-level cfg pattern as `consttry`: rustc parses cfg-gated
//! items inside an inline mod before evaluating cfg-attrs, so the
//! const-path and plain-path live in separate files. The `mod`
//! declaration's cfg controls whether the file is opened at all.

#[cfg(feature = "const")]
#[path = "ctor_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "ctor_plain_path.rs"]
mod plain_path;

#[cfg(feature = "const")]
pub use const_path::HasTrivialCtor;
#[cfg(not(feature = "const"))]
pub use plain_path::HasTrivialCtor;
