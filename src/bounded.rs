//! [`Boundable`] — trait for types bounded to `[MIN, MAX]`.

use crate::Maybe;

/// A type that carries a bounded integer-like value in `[MIN, MAX]`.
///
/// Implementors guarantee that every constructed instance holds a value
/// satisfying `MIN <= value <= MAX`. Out-of-range values are rejected at
/// construction ([`Self::try_new`]).
///
/// Arvo impls this trait on its `UFixed` / `IFixed` newtypes. Downstream
/// consumers take `T: Boundable` (+ associated bounds) rather than concrete
/// types — monomorphisation picks the storage shape at each call site.
///
/// # Trait-first usage
///
/// ```ignore
/// fn clamp_into<T: Boundable<Inner = u32>>(value: u32) -> Maybe<T> {
///     T::try_new(value)
/// }
/// ```
pub trait Boundable: Sized {
    /// Underlying scalar the bound is expressed in (`u8`, `u32`, `i64`, …).
    type Inner: Copy;

    /// Minimum value permitted.
    const MIN: Self::Inner;

    /// Maximum value permitted.
    const MAX: Self::Inner;

    /// Try to construct from a raw value. Returns [`Maybe::Isnt`] if out of
    /// range.
    fn try_new(value: Self::Inner) -> Maybe<Self>;

    /// Extract the underlying value.
    fn value(self) -> Self::Inner;
}
