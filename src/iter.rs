//! [`IteratorExt`]: adapter from `core::iter::Iterator::next` to [`Maybe`].
//!
//! Bridges the std `Iterator::next() -> Option<Item>` boundary to the
//! stack's [`Maybe`] vocabulary. Every consumer that ships a custom
//! `Iterator` impl reaches for `lint:allow(no-bare-option)` because the
//! trait method signature names `Option`. The adapter lets call sites
//! use `Maybe` without paying that cost; the std signature stays where
//! it must (the `Iterator` impl body itself).

use crate::Maybe;

/// Blanket adapter on every [`Iterator`] that returns the next value as
/// a [`Maybe`] instead of an [`Option`].
///
/// Use at call sites that consume an iterator and want to stay in the
/// substrate's vocabulary:
///
/// ```
/// use notko::{Maybe, iter::IteratorExt};
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
