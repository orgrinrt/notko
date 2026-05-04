//! [`PartialOrdExt`]: adapter from `core::cmp::PartialOrd::partial_cmp` to [`Maybe`].
//!
//! Same shape as [`crate::iter::IteratorExt`]. Bridges
//! `PartialOrd::partial_cmp -> Option<Ordering>` to the stack's
//! [`Maybe`] vocabulary at call sites.

use core::cmp::Ordering;

use crate::Maybe;

/// Blanket adapter on every [`PartialOrd`] that returns the comparison
/// as a [`Maybe<Ordering>`] instead of `Option<Ordering>`.
///
/// Use at call sites that compare two values and want to stay in the
/// substrate's vocabulary:
///
/// ```
/// use notko::{Maybe, cmp::PartialOrdExt};
/// use core::cmp::Ordering;
///
/// let a = 1.0_f64;
/// let b = 2.0_f64;
/// match a.partial_cmp_maybe(&b) {
///     Maybe::Is(Ordering::Less) => {},
///     _ => unreachable!(),
/// }
///
/// let nan = f64::NAN;
/// assert!(matches!(a.partial_cmp_maybe(&nan), Maybe::Isnt));
/// ```
///
/// The adapter does not replace the `PartialOrd` impl; it sits on top
/// of `PartialOrd::partial_cmp` via a blanket impl over
/// `T: PartialOrd<U>`.
pub trait PartialOrdExt<Rhs: ?Sized = Self>: PartialOrd<Rhs> {
    /// Compare `self` to `other`, returning the [`Ordering`] as a
    /// [`Maybe`].
    ///
    /// Equivalent to `self.partial_cmp(other).into()`. Inlined.
    #[inline]
    fn partial_cmp_maybe(&self, other: &Rhs) -> Maybe<Ordering> {
        self.partial_cmp(other).into()
    }
}

impl<T: PartialOrd<Rhs> + ?Sized, Rhs: ?Sized> PartialOrdExt<Rhs> for T {}
