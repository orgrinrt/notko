//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Plain-trait `Cardinal` declaration. See `cardinal.rs` for the cfg-gated
//! module layout rationale.

/// A type that counts, as a zero and a successor.
///
/// Plain-feature variant: identical surface to the const-feature variant
/// minus the `const` keyword. Consumers on stable Rust opt into this form via
/// `default-features = false`, and what it costs them is `Length::LEN`, since
/// a constant cannot be built by a call that is not const.
///
/// ```
/// use notko_hlist::Cardinal;
///
/// #[derive(Clone, Copy, PartialEq, Debug)]
/// struct Count(usize);
///
/// impl Cardinal for Count {
///     const ZERO: Self = Count(0);
///
///     fn succ(self) -> Self {
///         Count(self.0 + 1)
///     }
/// }
///
/// assert_eq!(Count::ZERO.succ().succ().succ(), Count(3));
/// ```
///
/// # Why `Cardinal`
///
/// Not `Countable`, because `-able` says the thing can be counted and this
/// type is the count itself. Not `Natural`, which is exactly precise and is the
/// problem: it claims a mathematical primitive, and that belongs to whichever
/// crate owns the numerics rather than to a list.
pub trait Cardinal: Sized {
    /// The count of nothing.
    const ZERO: Self;

    /// One more than this count.
    ///
    /// Takes its argument by value, so a count may be a type that owns
    /// something. Nothing here calls it more than once per cell.
    fn succ(self) -> Self;
}
