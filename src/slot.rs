//! `Slot<T>`. Transparent niche-filled `Maybe<T>` wrapper.
//!
//! `Slot<T>` is a `#[repr(transparent)]` newtype over `Maybe<T>` for
//! types `T: NonZeroable + NicheFilled`. The newtype signals at the
//! type level that the wrapped Maybe lives in a niche-filled layout:
//! one zero pattern of `T` is reserved for `Maybe::Isnt`, so
//! `Slot<T>` has the same size and alignment as `T`.
//!
//! ## Why both bounds
//!
//! `NonZeroable` and `NicheFilled` carry adjacent but distinct
//! contracts:
//!
//! `NonZeroable` (open trait) says "this type has a zero sentinel
//! and a guaranteed-nonzero form." Downstream crates implement
//! `NonZeroable` on their own newtypes (e.g. arvo's nonzero
//! flavours of UFixed / IFixed) without needing to coordinate with
//! notko.
//!
//! `NicheFilled` (sealed trait) says "rustc's niche-filling
//! optimisation actually realizes the bit-pattern-zero niche for
//! this type." It enumerates the exact set notko has verified:
//! `&T`, `&mut T`, `NonNull<T>`, `core::num::NonZero{U,I}*`, and
//! function-pointer arities 0..=8. Sealing prevents drift.
//!
//! `NonZeroable` alone does not guarantee niche-fill. `NicheFilled`
//! alone does not guarantee a public nonzero contract. `Slot<T>`
//! requires both: NonZeroable so the type-level "presence" semantics
//! match the contract a niche occupies; NicheFilled so the layout
//! claim holds.
//!
//! Per-instantiation `_LAYOUT_ASSERT` constants verify
//! `size_of::<Slot<T>> == size_of::<T>` at compile time. Adding a
//! new `Slot<T>` instantiation that fails the assert is a build
//! error, not a silent layout regression.
//!
//! ## Composition with domain wrappers
//!
//! Consumers needing the +1 / -1 shift to expose 0-indexed semantics
//! over a `NonZeroX` payload (e.g. `arvo`'s `NUSize` over
//! `Slot<NonZeroUSize>`) wrap `Slot<T>` again at the domain layer.
//! The shift is the wrapper's contract, not Slot's. notko stays
//! arithmetic-free.
//!
//! ## Limitations on downstream NicheFilled types
//!
//! `NicheFilled` is sealed in notko. Downstream crates that want a
//! `Slot<TheirCustomNonZeroType>` cannot extend `NicheFilled` and
//! must use `core::num::NonZero{U,I}*` or a reference type as the
//! payload. Tracked as a future cross-repo design question; for
//! immediate use cases (arvo's `NUSize` over `NonZeroUSize`) the
//! seal is not load-bearing.

use crate::{Maybe, NicheFilled, NonZeroable};

/// A niche-filled `Maybe<T>` wrapper for `T: NonZeroable + NicheFilled`.
///
/// Layout: identical to `T` (`#[repr(transparent)]` over `Maybe<T>`,
/// which niche-fills when `T: NicheFilled`).
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Slot<T: NonZeroable + NicheFilled>(Maybe<T>);

impl<T: NonZeroable + NicheFilled> Slot<T> {
    /// The absent value. Equivalent to `Slot(Maybe::Isnt)`.
    pub const NONE: Self = Self(Maybe::Isnt);

    /// Construct a present `Slot` carrying `value`.
    pub const fn some(value: T) -> Self {
        Self(Maybe::Is(value))
    }

    /// Borrow the underlying `Maybe<T>`. The notko convention for
    /// `as_*` methods on Maybe-shaped wrappers (cf. `MaybeNull::as_maybe`)
    /// is borrow-projection.
    pub const fn as_maybe(&self) -> &Maybe<T> {
        &self.0
    }

    /// Consume to the underlying `Maybe<T>`. By-value extraction is
    /// the `into_*` form; this requires `T: Copy` because const fn
    /// cannot evaluate destructors of generic `T` under current
    /// rustc nightly.
    pub const fn into_maybe(self) -> Maybe<T>
    where
        T: Copy,
    {
        self.0
    }

    /// True when the slot carries a value.
    pub const fn is_some(&self) -> bool {
        matches!(&self.0, Maybe::Is(_))
    }

    /// True when the slot is absent.
    pub const fn is_none(&self) -> bool {
        matches!(&self.0, Maybe::Isnt)
    }
}

impl<T: NonZeroable + NicheFilled> Default for Slot<T> {
    fn default() -> Self {
        Self::NONE
    }
}

// Per-instantiation layout assertions. Each named instantiation
// pins `size_of::<Slot<T>> == size_of::<T>` at compile time. New
// instantiations downstream consumers actually exercise (e.g.
// arvo's `Slot<NonZeroUSize>`) should add a matching const here.
// Drift between Slot's claimed layout and rustc's realized layout
// surfaces as a build error.
mod layout_assertions {
    use super::Slot;
    use core::mem::size_of;
    use core::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize};

    const _SLOT_NONZERO_USIZE: () = assert!(size_of::<Slot<NonZeroUsize>>() == size_of::<usize>());
    const _SLOT_NONZERO_U8: () = assert!(size_of::<Slot<NonZeroU8>>() == size_of::<u8>());
    const _SLOT_NONZERO_U16: () = assert!(size_of::<Slot<NonZeroU16>>() == size_of::<u16>());
    const _SLOT_NONZERO_U32: () = assert!(size_of::<Slot<NonZeroU32>>() == size_of::<u32>());
    const _SLOT_NONZERO_U64: () = assert!(size_of::<Slot<NonZeroU64>>() == size_of::<u64>());
    const _SLOT_NONZERO_I8: () = assert!(size_of::<Slot<NonZeroI8>>() == size_of::<i8>());
    const _SLOT_NONZERO_I16: () = assert!(size_of::<Slot<NonZeroI16>>() == size_of::<i16>());
    const _SLOT_NONZERO_I32: () = assert!(size_of::<Slot<NonZeroI32>>() == size_of::<i32>());
    const _SLOT_NONZERO_I64: () = assert!(size_of::<Slot<NonZeroI64>>() == size_of::<i64>());
}
