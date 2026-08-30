//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-path `Length`. See `length.rs` for the cfg-gated module layout
//! rationale and for what the two paths differ on.

use crate::cardinal::Cardinal;
use crate::list::{Cons, Empty, List};

/// How many cells the list has, as a value of the count type `N`.
///
/// Implemented for [`Empty`] and, recursively, for [`Cons`], so every list
/// built from those two has a length in every count type that implements
/// [`Cardinal`]. A list is not required to pick one: `Length<A>` and
/// `Length<B>` both hold for the same list, and which one a call site means
/// falls out of what it asked for.
///
/// ```
/// # #![feature(const_trait_impl)]
/// use notko_hlist::{Cardinal, Cons, Empty, Length};
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
/// struct A;
/// struct B;
/// type Two = Cons<A, Cons<B, Empty>>;
///
/// const N: Count = <Two as Length<Count>>::LEN;
/// assert_eq!(N, Count(2));
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no length in `{N}`",
    note = "A length is defined for lists built from `Empty` and `Cons<H, T>`, and only in a count type implementing `Cardinal`. Check that `{N}` implements `Cardinal`, and that the tail of every cell is itself a list rather than a bare type."
)]
pub trait Length<N>: List {
    /// The count, resolved by the compiler.
    const LEN: N;

    /// The count, as a call.
    ///
    /// Reads `LEN`, so it is the same number arrived at the same way. It
    /// exists so that code written against the plain path compiles here
    /// unchanged.
    fn len() -> N {
        <Self as Length<N>>::LEN
    }
}

impl<N: const Cardinal> Length<N> for Empty {
    const LEN: N = N::ZERO;
}

impl<N: const Cardinal, H, T: Length<N>> Length<N> for Cons<H, T> {
    const LEN: N = <T as Length<N>>::LEN.succ();
}
