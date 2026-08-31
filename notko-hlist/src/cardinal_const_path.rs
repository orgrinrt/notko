//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-trait `Cardinal` declaration. See `cardinal.rs` for the cfg-gated
//! module layout rationale.

/// A type that counts, as a zero and a successor.
///
/// The two Peano constructors and nothing else. `Length` walks a list and
/// applies `succ` once per cell, so a count is whatever the consumer's
/// number type says it is: a `usize` newtype, a saturating fixed-width
/// integer, a type-level numeral.
///
/// Const-feature variant: `succ` is const-callable, which is what lets a
/// length be an associated constant rather than a function call. Implement it
/// with `const impl Cardinal for YourCount`.
///
/// ```
/// # #![feature(const_trait_impl)]
/// use notko_hlist::Cardinal;
///
/// #[derive(Clone, Copy, PartialEq, Debug)]
/// struct Count(usize);
///
/// const impl Cardinal for Count {
///     const ZERO: Self = Count(0);
///
///     fn succ(self) -> Self {
///         Count(self.0 + 1)
///     }
/// }
///
/// const THREE: Count = Count::ZERO.succ().succ().succ();
/// assert_eq!(THREE, Count(3));
/// ```
///
/// # Why `Cardinal`
///
/// Not `Countable`, because `-able` says the thing can be counted and this
/// type **is** the count. Not `Natural`, which is exactly precise and is the
/// problem: it claims a mathematical primitive, and that belongs to whichever
/// crate owns the numerics rather than to a list.
pub const trait Cardinal: Sized {
    /// The count of nothing.
    const ZERO: Self;

    /// One more than this count.
    ///
    /// Takes its argument by value, so a count may be a type that owns
    /// something. Nothing here calls it more than once per cell.
    fn succ(self) -> Self;
}
