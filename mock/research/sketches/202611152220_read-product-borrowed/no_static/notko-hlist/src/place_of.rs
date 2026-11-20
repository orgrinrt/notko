//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `PlaceOf`: where a member sits, found from the member.
//!
//! [`At`] goes from a place to the member there. This goes the other way, and a
//! consumer cannot get it out of `At` by bounding on `L: At<P, Member = M>` with
//! `P` left to inference, because the solver picks between `At`'s two impls
//! before it looks at what the member is, and with both candidates open it
//! answers that the place is ambiguous. So the member is a parameter here, and
//! the two impls differ in the place they answer with: `Here` when the member is
//! the head, `There<P>` when the tail holds it at `P`. They do not overlap, and
//! neither needs `#[marker]`, so this sits in every configuration of the crate.
//!
//! The place is then a type the rest of the crate already reads: [`Position`]
//! counts it, and [`Has`](crate::Has) walks a product to it.
//!
//! [`Position`]: crate::Position

use crate::at::At;
use crate::list::{Cons, List};
use crate::position::{Here, Place, There};

/// The list holds `M` at the place `P`.
///
/// ```
/// use notko_hlist::{Cons, Empty, Here, PlaceOf, There};
///
/// struct A;
/// struct B;
/// struct C;
/// type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
///
/// fn place_of<M, P: notko_hlist::Place, L: PlaceOf<M, P>>()
/// -> core::marker::PhantomData<P> {
///     core::marker::PhantomData
/// }
///
/// // The place is inferred from the member, and it is `There<There<Here>>`.
/// let _: core::marker::PhantomData<There<There<Here>>> =
///     place_of::<C, _, Three>();
/// ```
///
/// `At<P, Member = M>` is the supertrait, so whatever bounds on `PlaceOf` also
/// knows what `At` says at that place, and the two can never disagree. That law
/// is also what closes the trait without a seal of its own: an impl from outside
/// has to name a list and a place where `At` already answers `M`, and every such
/// pair is one the impls below cover, so it would overlap them.
///
/// A list holding `M` twice holds it at two places, and a bound leaving `P` to
/// inference is then ambiguous and does not build. That is the refusal a
/// consumer gets for naming a member twice where one place was meant, and it
/// arrives at the bound rather than as a wrong answer.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no place for `{M}`",
    note = "A member is at `Here` when it is the head of a `Cons`, and at `There<P>` when the tail holds it at `P`. The usual cause is a member the list does not hold. If the compiler reports `type annotations needed` instead, the list holds the member at more than one place. If it reports `overflow evaluating the requirement`, the walk is one step per cell and the list is deeper than the default recursion limit, so the crate root wants `#![recursion_limit = \"1024\"]`."
)]
pub trait PlaceOf<M, P: Place>: At<P, Member = M> {}

// Neither impl is offered as a suggestion, for the reason `At`'s are not: the
// note says where a member sits, and the suggestion would print this file's path
// as a consumer's checkout has it.
#[diagnostic::do_not_recommend]
impl<M, T: List> PlaceOf<M, Here> for Cons<M, T> {}

#[diagnostic::do_not_recommend]
impl<H, M, P: Place, T: PlaceOf<M, P>> PlaceOf<M, There<P>> for Cons<H, T> {}
