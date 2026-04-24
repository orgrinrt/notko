//! Re-exports of the foundational vocabulary.
//!
//! ```ignore
//! use notko::prelude::*;
//!
//! fn lookup(k: u32) -> Maybe<u32> { Maybe::Isnt }
//! fn compute() -> Outcome<u32, ()> { Outcome::Ok(42) }
//! ```

pub use crate::Boundable;
pub use crate::Just;
pub use crate::{Maybe, MaybeNull};
pub use crate::NonZeroable;
pub use crate::Outcome;
