//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Plain-path `Position` declaration. See `position.rs` for the cfg-gated
//! module layout rationale.

use crate::cardinal::Cardinal;
use crate::position::{Here, There, sealed};

/// How far along a position is, in the consumer's own count.
///
/// The same arrangement [`Length`](crate::Length) has and for the same
/// reason: a crate that picked the number type would hand every consumer a
/// conversion at each use. `Here` is zero and `There<P>` is one more than
/// `P`, so the count is the depth and nothing else.
///
/// ```
/// use notko_hlist::{Cardinal, Here, Position, There};
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
/// assert_eq!(<There<There<Here>> as Position<Count>>::index(), Count(2));
/// ```
///
/// Plain-path variant: the count is a call, one `succ` per cell, because
/// building it as a constant means calling `Cardinal::succ` in a constant and
/// off the const path `succ` is not const-callable.
///
/// Sealed through the two markers, so these two impls are the only ones there
/// will ever be. Without that a consumer could declare a position of its own,
/// give it whatever index it liked, and hand it to a bound that would then be
/// saying nothing: [`At`](crate::At) would answer no type for it, and a
/// consumer that read the count alone would index wherever it was told.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no index in `{N}`",
    note = "A position is `Here`, or `There<P>` where `P` is itself a position, and the count `{N}` has to implement `Cardinal`. The trait is sealed, so a marker of your own cannot become one."
)]
pub trait Position<N>: sealed::Sealed {
    /// How many cells in, computed one cell at a time.
    fn index() -> N;
}

impl<N: Cardinal> Position<N> for Here {
    fn index() -> N {
        N::ZERO
    }
}

impl<N: Cardinal, P: Position<N>> Position<N> for There<P> {
    fn index() -> N {
        <P as Position<N>>::index().succ()
    }
}
