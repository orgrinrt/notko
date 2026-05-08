//! [`NonZeroable`]: trait for types with a zero sentinel and a nonzero
//! guarantee form.

use crate::Maybe;

/// A type where the underlying representation has a distinguishable zero
/// and an impl that guarantees nonzero at the type level.
///
/// Arvo impls this on its `UFixed` / `IFixed` nonzero flavours. Downstream
/// consumers take `T: NonZeroable` instead of `core::num::NonZeroU*` when
/// the underlying storage shape should vary by caller.
///
/// # Inner bound
///
/// `Inner: Clone` is the minimum the trait requires. Every shipped impl
/// (the 12 `core::num::NonZero{U,I}*` types below, plus arvo's nonzero
/// `UFixed` / `IFixed` flavours) has `Inner: Copy`, which satisfies
/// `Clone` trivially. The relaxed bound matches the sibling
/// [`crate::Boundable`] trait so consumer code that takes both `T:
/// NonZeroable` and `U: Boundable` does not have to wrestle with
/// asymmetric inner-type bounds.
///
/// # Trait-first usage
///
/// ```
/// use notko::{Maybe, NonZeroable};
///
/// fn only_positive<T: NonZeroable<Inner = u32>>(raw: u32) -> Maybe<T> {
///     T::try_new(raw)
/// }
/// ```
pub trait NonZeroable: Sized {
    /// Underlying scalar (`u8`, `u32`, `i64`, ...).
    type Inner: Clone;

    /// Try to construct from a raw value. Returns [`Maybe::Isnt`] if the
    /// value is zero.
    fn try_new(value: Self::Inner) -> Maybe<Self>;

    /// Extract the underlying value. Guaranteed nonzero.
    fn value(self) -> Self::Inner;
}

// Built-in impls for the canonical `core::num::NonZero*` types.
// Notko ships these so consumers reaching for `T: NonZeroable` can
// pass the standard nonzero primitives directly without writing a
// newtype. Domain newtypes (e.g. arvo's nonzero UFixed flavours)
// add their own NonZeroable impls on top.
macro_rules! impl_nonzeroable_for_core {
    ($($nz:ty => $inner:ty),* $(,)?) => {
        $(
            impl NonZeroable for $nz {
                type Inner = $inner;

                #[inline]
                fn try_new(value: Self::Inner) -> Maybe<Self> {
                    match <$nz>::new(value) {
                        Some(nz) => Maybe::Is(nz),
                        None => Maybe::Isnt,
                    }
                }

                #[inline]
                fn value(self) -> Self::Inner {
                    self.get()
                }
            }
        )*
    };
}

impl_nonzeroable_for_core! {
    core::num::NonZeroU8 => u8,
    core::num::NonZeroU16 => u16,
    core::num::NonZeroU32 => u32,
    core::num::NonZeroU64 => u64,
    core::num::NonZeroU128 => u128,
    core::num::NonZeroUsize => usize,
    core::num::NonZeroI8 => i8,
    core::num::NonZeroI16 => i16,
    core::num::NonZeroI32 => i32,
    core::num::NonZeroI64 => i64,
    core::num::NonZeroI128 => i128,
    core::num::NonZeroIsize => isize,
}
