//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

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
/// (the 12 `core::num::NonZero{U,I}*` types below) has `Inner: Copy`, which satisfies
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
// Shipped so a consumer reaching for `T: NonZeroable` can pass the
// standard nonzero primitives directly rather than writing a newtype
// first. A domain newtype adds its own impl on top.
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

/// The twelve `core::num::NonZero*` types and the primitive each wraps, in one
/// place.
///
/// Three things need this list and they need it in different shapes: the trait
/// impls here, the niche seal in `maybe`, and the layout assertions in `slot`.
/// Written out three times it drifts, and the drift is silent, because a
/// thirteenth type added to one list leaves the other two passing over the
/// twelve they still name. So the list lives here and the shapes come to it:
/// pass a macro that accepts `Ty => inner` pairs and it is invoked with all
/// twelve.
macro_rules! for_each_core_nonzero {
    ($callback:ident) => {
        $callback! {
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
    };
}
pub(crate) use for_each_core_nonzero;

for_each_core_nonzero!(impl_nonzeroable_for_core);
