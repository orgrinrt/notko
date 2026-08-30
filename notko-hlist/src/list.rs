//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The two shapes a list is built from, and the sealed marker saying that is
//! what something is.
//!
//! Nothing else in this crate declares a type. Every other item is a trait
//! saying something about a list assembled out of these two.

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The empty list, and the end of every non-empty one.
///
/// Carries nothing, means nothing on its own, and is the base case every
/// recursion in this crate terminates on.
pub struct Empty;

/// A cell: head `H` in front of the list `T`.
///
/// The tuple in the phantom is what makes the cell covariant in both, and
/// what keeps a `Cons` over a borrowed head from outliving it. Neither `H`
/// nor `T` is bounded here on purpose: a head is any type at all, including
/// one that is not itself a list, and `Cons<A, u8>` is a type that exists.
/// What it is not is a [`List`], and that is where the error arrives.
pub struct Cons<H, T>(core::marker::PhantomData<(H, T)>);

/// Built from [`Empty`] and [`Cons`], all the way down.
///
/// Sealed, so this crate's two impls are the only ones there will ever be.
/// That is what every other trait here rests on, and the reason is not
/// tidiness: `Contains`, `ContainsAll` and `Length` are claims about a list,
/// and a claim anybody may implement for their own type is a claim that
/// proves nothing. Bound on `L: Contains<X>` and you want it to mean `X` is
/// in there, rather than that somebody wrote an empty impl saying so.
///
/// The same sealing is what lets the bounds on those traits stay off their
/// declarations and sit on the impls, where they can differ between the const
/// path and the plain one without that difference reaching a consumer's
/// signature.
///
/// It costs the ability to bring your own list type, and that is not what the
/// crate is for: the intended shape is aliasing the cell and the leaf into
/// your own vocabulary, which keeps them these two types.
///
/// ```
/// use notko_hlist::{Cons, Empty, List};
///
/// struct A;
/// struct B;
///
/// fn takes_a_list<L: List>() {}
/// takes_a_list::<Cons<A, Cons<B, Empty>>>();
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a list",
    note = "A list is `Empty`, or `Cons<H, T>` where `T` is itself a list. A head may be anything, a tail may not, so the usual cause is a cell whose tail is a bare type. The trait is sealed, so a type of your own cannot become one."
)]
pub trait List: sealed::Sealed {}

impl sealed::Sealed for Empty {}
impl List for Empty {}

impl<H, T: List> sealed::Sealed for Cons<H, T> {}
impl<H, T: List> List for Cons<H, T> {}
