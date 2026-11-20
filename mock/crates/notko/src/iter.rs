//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! [`IteratorExt`]: adapter from `core::iter::Iterator::next` to [`Maybe`].
//!
//! Bridges `Iterator::next() -> Option<Item>` over to [`Maybe`]. The trait
//! signature names `Option` and there's nothing to be done about that, so
//! the `Option` stays inside the `Iterator` impl where it has to be, and
//! this lets the call site go back to `Maybe`.

use crate::Maybe;

/// Blanket adapter on every [`Iterator`] that returns the next value as
/// a [`Maybe`] instead of an [`Option`].
///
/// Use at call sites that consume an iterator and want to stay in the
/// vocabulary of this crate:
///
/// ```
/// use notko::Maybe;
/// use notko::iter::IteratorExt;
///
/// let mut it = [1, 2, 3].into_iter();
/// match it.next_maybe() {
///     Maybe::Is(x) => assert_eq!(x, 1),
///     Maybe::Isnt => unreachable!(),
/// }
/// ```
///
/// The adapter does not replace the `Iterator` impl itself; it sits on
/// top of `Iterator::next` via a blanket impl over `I: Iterator`.
pub trait IteratorExt: Iterator {
    /// Advance the iterator, returning the next value as [`Maybe`].
    ///
    /// Equivalent to `self.next().into()`. Inlined; codegen identical
    /// to a direct `next()` call followed by an `Option` to `Maybe`
    /// conversion (which is itself a `match`, niche-filled at any
    /// pointer-shaped `Item`).
    #[inline]
    fn next_maybe(&mut self) -> Maybe<Self::Item> {
        self.next().into()
    }
}

impl<I: Iterator + ?Sized> IteratorExt for I {}
