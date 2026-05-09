//! Re-exports of the foundational vocabulary.
//!
//! ```ignore
//! use notko::prelude::*;
//!
//! fn lookup(k: u32) -> Maybe<u32> { Maybe::Isnt }
//! fn compute() -> Outcome<u32, ()> { Outcome::Ok(42) }
//! ```

pub use crate::cmp::PartialOrdExt;
pub use crate::iter::IteratorExt;
pub use crate::{BoundError, Boundable};
pub use crate::HasTrivialCtor;
pub use crate::Just;
pub use crate::{Maybe, MaybeNull, NicheFilled};
pub use crate::NonZeroable;
pub use crate::Outcome;
pub use crate::Slot;
// ConstTry / ConstFromResidual are intentionally not in the prelude.
// They are substrate-internal const-callable parallels of core's Try /
// FromResidual; consumers usually access fallibility through `?`
// (which desugars to core::ops::Try, not ConstTry).
