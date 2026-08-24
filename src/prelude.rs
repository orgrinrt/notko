//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Re-exports of the foundational vocabulary.
//!
//! ```ignore
//! use notko::prelude::*;
//!
//! fn lookup(k: u32) -> Maybe<u32> { Maybe::Isnt }
//! fn compute() -> Outcome<u32, ()> { Outcome::Ok(42) }
//! ```

pub use crate::HasTrivialCtor;
pub use crate::Just;
pub use crate::NonZeroable;
pub use crate::Outcome;
pub use crate::Slot;
pub use crate::cmp::PartialOrdExt;
pub use crate::iter::IteratorExt;
pub use crate::lend::{Exhausted, Fill, Lend};
pub use crate::sink::{BulkPush, Emit, Push};
pub use crate::{BoundError, Boundable};
pub use crate::{Maybe, MaybeNull, NicheFilled};
// ConstTry / ConstFromResidual are intentionally not in the prelude.
// They are substrate-internal const-callable parallels of core's Try /
// FromResidual; consumers usually access fallibility through `?`
// (which desugars to core::ops::Try, not ConstTry).
