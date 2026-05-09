//! HasTrivialCtor: const + non-const construction.

#![cfg_attr(feature = "const", feature(const_trait_impl))]

#[cfg(feature = "const")]
#[path = "ctor_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "ctor_plain_path.rs"]
mod plain_path;
