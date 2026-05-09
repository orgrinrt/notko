//! `HasTrivialCtor`: type ships a no-arg constructor.
//!
//! Granular contract trait. A type that impls `HasTrivialCtor` declares
//! it has a `fn new() -> Self` taking no arguments. Useful for marker
//! and phantom-data wrappers that need a uniform construction
//! convention (e.g., `Column::<Player>::new()`,
//! `Virtual::<Tick>::new()`) without the consumer learning a bespoke
//! per-type spelling.
//!
//! Reusable across the workspace. Independent of any specific
//! framework. Any wrapper, marker, or unit-shaped type that wants the
//! convention impls this trait.
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
