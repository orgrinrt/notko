//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Concat`: append, as a function the compiler evaluates.

use crate::list::{Cons, Empty, List};

/// Every member of `Self`, then every member of `L`.
///
/// A type-level function, so `<A as Concat<B>>::Out` is a list and there is
/// no value anywhere. Order is preserved on both sides and nothing is
/// deduplicated: appending a list to one that already holds the same type
/// gives a list holding it twice, which is what membership then answers
/// about.
///
/// ```
/// use notko_hlist::{Concat, Cons, Empty};
///
/// struct A;
/// struct B;
///
/// type Left = Cons<A, Empty>;
/// type Right = Cons<B, Empty>;
///
/// // Cons<A, Cons<B, Empty>>
/// type Both = <Left as Concat<Right>>::Out;
/// # let _: core::marker::PhantomData<Both> = core::marker::PhantomData;
/// ```
///
/// Appending onto [`Empty`] is the right-hand side unchanged, so the empty
/// list is an identity on both sides and the whole thing associates. Nothing
/// enforces that; it falls out of the two impls, and the tests pin it.
#[diagnostic::on_unimplemented(
    message = "cannot append `{L}` onto `{Self}`",
    note = "Concat walks the left-hand side, so `{Self}` has to be a list built from `Empty` and `Cons<H, T>`. The right-hand side is not walked and may be anything, which is why the error names the left one."
)]
pub trait Concat<L>: List {
    /// The appended list.
    type Out;
}

impl<L> Concat<L> for Empty {
    type Out = L;
}

impl<H, T, L> Concat<L> for Cons<H, T>
where
    T: Concat<L> + List,
{
    type Out = Cons<H, <T as Concat<L>>::Out>;
}
