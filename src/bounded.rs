//! [`Boundable`]: trait for types bounded to `[MIN, MAX]`.
//! [`BoundError`]: rejection reason returned from [`Boundable::try_new`].

use crate::Outcome;

/// Why a value was rejected by [`Boundable::try_new`].
///
/// The variant names whether the offending value was below the
/// minimum or above the maximum, and carries both the value and the
/// bound it crossed. Callers can produce diagnostics without reaching
/// back for `MIN` / `MAX` themselves.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub enum BoundError<I> {
    /// Value was less than `MIN`. Carries the offending value and the
    /// `MIN` bound it failed.
    Below {
        /// The rejected value.
        value: I,
        /// The minimum permitted value (`MIN`).
        min: I,
    },
    /// Value was greater than `MAX`. Carries the offending value and
    /// the `MAX` bound it failed.
    Above {
        /// The rejected value.
        value: I,
        /// The maximum permitted value (`MAX`).
        max: I,
    },
}

/// A type that carries a bounded integer-like value in `[MIN, MAX]`.
///
/// Implementors guarantee that every constructed instance holds a value
/// satisfying `MIN <= value <= MAX`. Out-of-range values are rejected at
/// construction ([`Self::try_new`]) with a [`BoundError`] naming the
/// offending value and the bound it crossed.
///
/// Arvo impls this trait on its `UFixed` / `IFixed` newtypes. Downstream
/// consumers take `T: Boundable` (+ associated bounds) rather than concrete
/// types. Monomorphisation picks the storage shape at each call site.
///
/// # Inner bound
///
/// `Inner: Clone` is required so [`BoundError`] can carry the rejected
/// value alongside the bound it crossed without forcing the caller to
/// give up ownership. The vast majority of impls have `Inner: Copy`,
/// which satisfies `Clone` trivially.
///
/// # Trait-first usage
///
/// ```
/// use notko::{BoundError, Boundable, Outcome};
///
/// fn clamp_into<T: Boundable<Inner = u32>>(value: u32) -> Outcome<T, BoundError<u32>> {
///     T::try_new(value)
/// }
/// ```
pub trait Boundable: Sized {
    /// Underlying scalar the bound is expressed in (`u8`, `u32`, `i64`, ...).
    type Inner: Clone;

    /// Minimum value permitted.
    const MIN: Self::Inner;

    /// Maximum value permitted.
    const MAX: Self::Inner;

    /// Try to construct from a raw value.
    ///
    /// On success returns the constructed value as [`Outcome::Ok`].
    /// On rejection returns [`Outcome::Err`] carrying a [`BoundError`]
    /// naming the offending value and the bound it crossed.
    fn try_new(value: Self::Inner) -> Outcome<Self, BoundError<Self::Inner>>;

    /// Extract the underlying value.
    fn value(self) -> Self::Inner;
}
