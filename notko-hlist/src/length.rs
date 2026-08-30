//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Length`: how many cells a list has, counted in the consumer's own type.
//!
//! The count is `N` rather than a `usize`, because the crates that wanted
//! this already had a number type of their own and a length that did not
//! speak it is a length they have to convert at every use.
//!
//! # The count is bounded on the impls and not on the trait
//!
//! `Length<N>` names `N` without constraining it, and the `Cardinal` bound
//! sits on the two impls instead. The reason is that the two paths bound `N`
//! differently, `const Cardinal` against `Cardinal`, so a bound on the trait
//! would propagate that difference into every consumer signature mentioning
//! `Length`, and a generic function written for one configuration would not
//! compile in the other.
//!
//! **This only holds together because the trait is sealed.** `Length` has
//! [`List`](crate::List) as a supertrait, `List` is sealed, and the two impls
//! here are therefore the only impls of `Length` that can exist anywhere. So
//! "a count that carries a length is a cardinal" is a fact about the whole
//! program rather than about this file, and a consumer bounding on
//! `L: Length<N>` gets it without writing it.
//!
//! Unsealed it would be a real weakening: anyone could write
//! `impl Length<u8> for Theirs` with `u8` implementing nothing, and a generic
//! function would have to re-declare the cardinal bound to get a zero back,
//! in the spelling of whichever configuration it was written for.
//!
//! # What differs between the two paths
//!
//! `len()` is in both and means the same thing. `LEN` is in the const path
//! only, and it is the one worth having: an associated constant is resolved
//! once by the compiler where a call is resolved once per call site and then
//! usually inlined to the same thing. The plain path cannot have it, because
//! building it means calling `Cardinal::succ` in a constant, and off the
//! const path `succ` is not const-callable.
//!
//! # Module layout
//!
//! Same file-level cfg pattern as `cardinal`, and for the same reason.

#[cfg(feature = "const")]
#[path = "length_const_path.rs"]
mod const_path;

#[cfg(not(feature = "const"))]
#[path = "length_plain_path.rs"]
mod plain_path;

#[cfg(feature = "const")]
pub use const_path::Length;
#[cfg(not(feature = "const"))]
pub use plain_path::Length;
