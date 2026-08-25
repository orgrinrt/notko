//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

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
//! and a guaranteed-nonzero form." A downstream crate implements it
//! on its own newtypes, a nonzero flavour of a fixed-point type say,
//! without coordinating with this crate.
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
//! A const assertion per `NonZeroable` impl verifies
//! `size_of::<Slot<T>> == size_of::<T>` at compile time, so a layout
//! regression on any of them is a build error rather than something
//! that shows up at an FFI boundary. Those assertions cover the
//! primitives this crate implements the trait for; a consumer adding
//! its own `NonZeroable` type is asserting its own layout.
//!
//! ## Composition with domain wrappers
//!
//! The +1 / -1 shift that exposes 0-indexed semantics over a
//! `NonZeroX` payload is a wrapper around `Slot<T>` at the domain
//! layer, not something `Slot` does. The shift is that wrapper's
//! contract, and this crate stays arithmetic-free.
//!
//! ## Limitations on downstream NicheFilled types
//!
//! `NicheFilled` is sealed in notko. Downstream crates that want a
//! `Slot<TheirCustomNonZeroType>` cannot extend `NicheFilled` and
//! must use `core::num::NonZero{U,I}*` or a reference type as the
//! payload. Wrapping one of those is the shape that is actually
//! reached for, so the seal has not bound anything in practice.

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
    /// the `into_*` form.
    ///
    /// The bound is there because a const fn has to be able to drop
    /// what it consumes, and the property that names is `Destruct`
    /// rather than `Copy`. Naming it directly would need the const
    /// feature, and this method exists in both configurations, so it
    /// takes the bound that works in both. `Copy` implies `Destruct`,
    /// so nothing unsound gets through; the cost is that a non-`Copy`
    /// payload is refused here while every other method on `Slot`
    /// takes it.
    ///
    /// That payload is reachable, so the cost is real rather than
    /// theoretical. `NicheFilled` is sealed and `NonZeroable` is not,
    /// and what a `Slot<T>` admits is the two together: the sealed
    /// list covers `&T`, `&mut T`, `NonNull<T>`, the function pointer
    /// shapes and the `core::num::NonZero*` family, and a crate may
    /// implement `NonZeroable` for a reference to a type of its own.
    /// `&mut T` is the one member of that list that is not `Copy`, so
    /// a `Slot<&mut Theirs>` builds, borrows and reports, and refuses
    /// this one method.
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

// Layout assertions over every type this crate implements `NonZeroable`
// for. Drift between the claimed layout and the one rustc realizes is a
// build error rather than something an FFI boundary discovers later.
mod layout_assertions {
    use super::Slot;
    use core::mem::size_of;

    // The list mirrors `impl_nonzeroable_for_core!` in `nonzero.rs`. A
    // hand-written subset of it drifts silently, since every assertion that
    // is present still passes while the ones nobody typed cover nothing.
    macro_rules! assert_slot_layout {
        ($($nz:ty => $inner:ty),* $(,)?) => {
            $(const _: () = assert!(size_of::<Slot<$nz>>() == size_of::<$inner>());)*
        };
    }

    crate::nonzero::for_each_core_nonzero!(assert_slot_layout);
}
