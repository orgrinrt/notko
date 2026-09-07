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
use crate::position::{Here, Place, There};

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
/// The two impls are what there is, and [`Place`] on the parameter is what
/// holds them to it. Being this crate's trait over this crate's type is not
/// enough on its own: `P` sits after `Self` and is free, so a downstream
/// crate filling it with a type of its own satisfies the orphan rule, and
/// `impl<H, T: List> At<Yours> for Cons<H, T>` is then a blanket over every
/// list, written from outside, answering whatever it likes. That construction
/// compiles against an unbounded `P` and is a case in
/// `tests/compile_fail/` now that the bound refuses it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no member at `{P}`",
    note = "A member is at `Here` in a `Cons`, and at `There<P>` when the tail has one at `P`. The usual causes are an index past the end of the list, which leaves `Empty` holding the position, and a position that is not built from `Here` and `There`. If the compiler reports `overflow evaluating the requirement` instead, the walk is one step per cell and the list is deeper than the default recursion limit, so the crate root wants `#![recursion_limit = \"1024\"]`."
)]
pub trait At<P: Place>: List {
    /// The type sitting there.
    type Member;
}

// Neither impl is offered as a suggestion when the bound goes unsatisfied. The
// note above already says where a member sits and what the usual causes are,
// which is the whole of what a reader needs, and `help: the trait is
// implemented for `Cons<H, T>`` adds nothing to it.
//
// What it does add is a path. The compiler points at the impl's own source, and
// in a consumer that reaches this crate over git that is an absolute path
// through somebody's home directory and the checkout revision the pin resolved
// to. Inside this repository the same diagnostic is workspace-relative and
// reads fine, so the defect is invisible where it is written and lands on
// whoever commits a compile-fail case downstream: the file is green on one
// machine and red on every other, and it reddens again on the next update for a
// reason unrelated to any code.
#[diagnostic::do_not_recommend]
impl<H, T: List> At<Here> for Cons<H, T> {
    type Member = H;
}

#[diagnostic::do_not_recommend]
impl<H, P: Place, T: At<P>> At<There<P>> for Cons<H, T> {
    type Member = <T as At<P>>::Member;
}
