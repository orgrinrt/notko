//! `Slot<T: NonZeroable>`. Transparent niche-filled `Maybe<T>` wrapper.
//!
//! `Slot<T>` is a `#[repr(transparent)]` newtype over `Maybe<T>` for
//! types `T: NonZeroable`. The newtype signals at the type level that
//! the wrapped Maybe lives in a niche-filled layout: one zero pattern
//! of `T` is reserved for `Maybe::Isnt`, so `Slot<T>` has the same
//! size and alignment as `T`.
//!
//! Compared to bare `Maybe<T>`:
//!
//! - Layout is identical to `T` (no discriminant byte) when `T:
//!   NonZeroable`. Bare `Maybe<T>` only achieves this layout when the
//!   Rust niche-filling optimiser recognises `T`'s niche; the `Slot`
//!   newtype documents the property at the API surface.
//! - Construction takes `T` directly (which is already nonzero by the
//!   `NonZeroable` contract), so the absence-or-present semantics
//!   stay clean and the consumer doesn't have to remember the niche.
//!
//! Consumers that need the +1 / -1 shift to expose 0-indexed semantics
//! over a `NonZeroX` payload (e.g., `arvo`'s `NUSize` over `Slot<NonZeroUSize>`)
//! wrap `Slot<T>` again at the domain layer; the shift is the wrapper's
//! contract, not Slot's.
//!
//! `Slot<T>` does NOT implement arithmetic, ordering, or any operation
//! that interprets the inner value. It is a typed container; consumers
//! call `as_maybe()` to access the wrapped `Maybe<T>` for Maybe-shaped
//! pattern matching.

use crate::{Maybe, NonZeroable};

/// A niche-filled `Maybe<T>` wrapper for `T: NonZeroable`.
///
/// Layout: identical to `T` (`#[repr(transparent)]` over `Maybe<T>`,
/// which itself niche-fills when `T: NicheFilled`, which `T: NonZeroable`
/// implies through `core::num::NonZero<...>`-style nonzero contracts).
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Slot<T: NonZeroable>(Maybe<T>);

impl<T: NonZeroable> Slot<T> {
    /// The absent value. Equivalent to `Slot(Maybe::Isnt)`.
    pub const NONE: Self = Self(Maybe::Isnt);

    /// Construct a present `Slot` carrying `value`.
    pub const fn some(value: T) -> Self {
        Self(Maybe::Is(value))
    }

    /// Project to the underlying `Maybe<T>`.
    pub const fn as_maybe(self) -> Maybe<T>
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

impl<T: NonZeroable> Default for Slot<T> {
    fn default() -> Self {
        Self::NONE
    }
}
