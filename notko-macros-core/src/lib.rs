//! AST-rewrite primitives for notko-macros' `#[lower_by(Tier)]`.
//!
//! Proc-macro crates cannot export non-macro items, so this crate houses the
//! reusable building blocks as a normal library. Third-party proc-macro
//! crates can depend on `notko-macros-core` to author their own attribute
//! macros without reimplementing the rewrite engine.
//!
//! See the crate README for the public API map and an example of authoring
//! a custom tier attribute.

pub mod discover;
pub mod parse;
pub mod rewrite;
pub mod tiers;
