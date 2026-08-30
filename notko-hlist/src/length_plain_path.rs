//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Plain-path `Length`. See `length.rs` for the cfg-gated module layout
//! rationale and for what the two paths differ on.

use crate::cardinal::Cardinal;
use crate::list::{Cons, Empty, List};

/// How many cells the list has, as a value of the count type `N`.
///
/// Plain-feature variant. `LEN` is absent here and `len()` is what remains:
/// a constant would have to be built by calling `Cardinal::succ` inside it,
/// and off the const path `succ` is an ordinary method.
///
/// ```
/// use notko_hlist::{Cardinal, Cons, Empty, Length};
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
/// struct A;
/// struct B;
/// type Two = Cons<A, Cons<B, Empty>>;
///
/// assert_eq!(<Two as Length<Count>>::len(), Count(2));
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no length in `{N}`",
    note = "A length is defined for lists built from `Empty` and `Cons<H, T>`, and only in a count type implementing `Cardinal`. Check that `{N}` implements `Cardinal`, and that the tail of every cell is itself a list rather than a bare type."
)]
pub trait Length<N>: List {
    /// The count, computed one cell at a time.
    fn len() -> N;
}

impl<N: Cardinal> Length<N> for Empty {
    fn len() -> N {
        N::ZERO
    }
}

impl<N: Cardinal, H, T: Length<N>> Length<N> for Cons<H, T> {
    fn len() -> N {
        <T as Length<N>>::len().succ()
    }
}
