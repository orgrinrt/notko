//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `At`: which member sits at a position.
//!
//! The counterpart to [`Contains`](crate::Contains) and the other half of
//! what a list can be asked. Membership says a type is in there and hides
//! where; this says which type is at a place and hides nothing, because the
//! place is what was asked with.
//!
//! Neither replaces the other. A bound wanting "somewhere in this list" takes
//! membership and stays indifferent to order, and one wanting "the second
//! one" takes this and is an ordering claim. Reaching for this where
//! membership would do pins a position nothing needed pinned, and a list
//! reordered later then breaks a consumer that never cared.

use crate::list::{Cons, List};
use crate::position::{Here, There};

/// The member of `Self` at position `P`.
///
/// ```
/// use notko_hlist::{At, Cons, Empty, Here, There};
///
/// struct A;
/// struct B;
/// struct C;
///
/// type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
///
/// // B, and the compiler is what says so.
/// type Second = <Three as At<There<Here>>>::Member;
/// # let _: core::marker::PhantomData<Second> = core::marker::PhantomData::<B>;
/// ```
///
/// [`Empty`](crate::Empty) implements this for no position at all, which is
/// what makes an index past the end a build that does not succeed rather than
/// a value nobody checked. The walk runs out of cells and the error names the
/// empty list, so the message points at the end of the list and not at the
/// position, which is the honest way round: the position is what the consumer
/// wrote and the list is what could not answer it.
///
/// The impls are closed without a seal, unlike everything else here. `At` is
/// this crate's trait and [`Cons`] is this crate's type, so no other crate can
/// write one, and the supertrait keeps `Self` a list besides. What a seal
/// would add is protection against this crate writing a third impl, which is
/// not what sealing is for.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no member at `{P}`",
    note = "A member is at `Here` in a `Cons`, and at `There<P>` when the tail has one at `P`. The usual causes are an index past the end of the list, which leaves `Empty` holding the position, and a position that is not built from `Here` and `There`."
)]
pub trait At<P>: List {
    /// The type sitting there.
    type Member;
}

impl<H, T: List> At<Here> for Cons<H, T> {
    type Member = H;
}

impl<H, P, T: At<P>> At<There<P>> for Cons<H, T> {
    type Member = <T as At<P>>::Member;
}
