//! [`NonZeroable`] — trait for types with a zero sentinel and a nonzero
//! guarantee form.

use crate::Maybe;

/// A type where the underlying representation has a distinguishable zero
/// and an impl that guarantees nonzero at the type level.
///
/// Arvo impls this on its `UFixed` / `IFixed` nonzero flavours. Downstream
/// consumers take `T: NonZeroable` instead of `core::num::NonZeroU*` when
/// the underlying storage shape should vary by caller.
///
/// # Trait-first usage
///
/// ```ignore
/// fn only_positive<T: NonZeroable<Inner = u32>>(raw: u32) -> Maybe<T> {
///     T::try_new(raw)
/// }
/// ```
pub trait NonZeroable: Sized {
    /// Underlying scalar (`u8`, `u32`, `i64`, …).
    type Inner: Copy;

    /// Try to construct from a raw value. Returns [`Maybe::Isnt`] if the
    /// value is zero.
    fn try_new(value: Self::Inner) -> Maybe<Self>;

    /// Extract the underlying value. Guaranteed nonzero.
    fn value(self) -> Self::Inner;
}
