//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Cardinal`: a type that counts, as zero and a successor.
//!
//! The count is a parameter rather than a type this crate picked, and the
//! reason is the orphan rule rather than taste. A number type worth counting
//! with lives in a crate downstream of this one, so an impl written here
//! would have a foreign type in it and an impl written there would have a
//! foreign trait and a foreign type both, if the list types were also
//! foreign. Declaring the trait here and leaving the impl to whoever owns the
//! number is the one arrangement that is legal: their type, our trait.
//!
//! # Module layout
//!
//! Same file-level cfg pattern as `notko`'s `ctor`: rustc parses cfg-gated
//! items inside an inline mod before evaluating cfg-attrs, so the const path
//! and the plain path live in separate files and the `mod` declaration's cfg
//! decides which one is opened at all.

#[cfg(feature = "const")]
#[path = "cardinal_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "cardinal_plain_path.rs"]
mod plain_path;

#[cfg(feature = "const")]
pub use const_path::Cardinal;
#[cfg(not(feature = "const"))]
pub use plain_path::Cardinal;
