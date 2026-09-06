//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Here` and `There`: where in a list, as a type.
//!
//! A position is the second thing [`At`](crate::At) needs and the crate has
//! no number to say it with. `Length` counts into the consumer's own type
//! through [`Cardinal`](crate::Cardinal) and a position cannot do that,
//! because it has to be a type before anything is counted: it appears in a
//! bound, where a value cannot go.
//!
//! So a position is the two Peano constructors again, this time as types.
//! [`Here`] is the front cell and [`There<P>`] is one further along, and a
//! list's third member sits at `There<There<Here>>`. That is the shape every
//! type-level index takes, and the alternative, a `const N: usize` on the
//! trait, needs arithmetic in the impl's own parameter to walk the tail,
//! which is a feature this crate does not take.
//!
//! [`Position<N>`] then reads the same position back as a count, so a
//! consumer holding a flat run of values indexes it with the number the
//! bound already proved. Both halves are needed together: what sits there is
//! the list's fact and how far along it is is the position's, and a consumer
//! reading a slice wants each from the side that owns it.
//!
//! # Module layout
//!
//! Same file-level cfg pattern as `cardinal`, and for the same reason. The
//! two markers are ordinary structs and sit here, since neither path changes
//! them.

#[cfg(feature = "const")]
#[path = "position_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "position_plain_path.rs"]
mod plain_path;

#[cfg(feature = "const")]
pub use const_path::Position;
#[cfg(not(feature = "const"))]
pub use plain_path::Position;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The front of the list.
///
/// Carries nothing and means nothing on its own, the way [`Empty`](crate::Empty)
/// does: it is the base case both the walk in [`At`](crate::At) and the count
/// in [`Position`] terminate on.
pub struct Here;

/// One cell further along than `P`.
///
/// The phantom is the plain one rather than `fn() -> P`, so `There` is
/// covariant in what it wraps, which costs nothing here because a position is
/// never constructed and its parameter is never a borrow. `P` is unbounded on
/// the declaration for the reason `Cons` leaves its own two unbounded:
/// `There<u8>` is a type that exists, what it is not is a [`Position`], and
/// that is where the error arrives.
pub struct There<P>(core::marker::PhantomData<P>);

impl sealed::Sealed for Here {}
impl<P> sealed::Sealed for There<P> {}
