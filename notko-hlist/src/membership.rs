//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Contains` and `ContainsAll`: what a list holds.
//!
//! Both are marker traits, and for [`Contains`] that is load-bearing rather
//! than decorative. A cell contains its own head, and it also contains
//! whatever its tail contains, and those two impls genuinely overlap for a
//! list whose head appears again further down. `#[marker]` is how coherence
//! is told the overlap is intended: the impls carry nothing, so which of them
//! the solver picks cannot change what the program means.
//!
//! That attribute is `marker_trait_attr` and is unstable, which is the whole
//! reason this module sits behind a feature rather than in the crate
//! unconditionally.

use crate::list::{Cons, Empty, List};

/// The list holds `X` somewhere in it.
///
/// ```
/// use notko_hlist::{Cons, Contains, Empty};
///
/// struct A;
/// struct B;
/// type Both = Cons<A, Cons<B, Empty>>;
///
/// fn needs<L: Contains<B>>() {}
/// needs::<Both>();
/// ```
///
/// Depth is not visible in the bound and is not meant to be: what a call site
/// says is that the type is in there, and where it sits is the list's
/// business. There is no position, no index and no witness to pass around,
/// which is the difference between this shape and the selector-with-an-index
/// shape that works without the marker attribute.
#[marker]
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not hold `{X}`",
    note = "Membership is decided by walking the cells, so `{Self}` has to be a list built from `Empty` and `Cons<H, T>`, and `{X}` has to be one of its heads. If the compiler reports `overflow evaluating the requirement` instead, the list is deeper than the default recursion limit and the crate root wants `#![recursion_limit = \"1024\"]`."
)]
pub trait Contains<X>: List {}

// The head match. Coexists with the tail match below only because of
// `#[marker]`: a list whose head is `X` and whose tail also holds `X`
// satisfies both.
impl<H, T: List> Contains<H> for Cons<H, T> {}

// The tail match, which is the recursion.
impl<H, T, X> Contains<X> for Cons<H, T> where T: Contains<X> + List {}

/// The list holds every member of `L`.
///
/// ```
/// use notko_hlist::{Cons, ContainsAll, Empty};
///
/// struct A;
/// struct B;
/// struct C;
/// type Three = Cons<A, Cons<B, Cons<C, Empty>>>;
/// type Some = Cons<C, Cons<A, Empty>>;
///
/// fn needs<L: ContainsAll<Some>>() {}
/// needs::<Three>();
/// ```
///
/// Order does not carry: the subset may name its members in any order, and
/// may name one of them twice, because each is checked on its own.
///
/// The base impl is a blanket over lists, so **every** list holds every member
/// of the empty one. That is what makes the recursion terminate, and it means
/// a bound of `ContainsAll<Empty>` says nothing and is not worth writing. The
/// blanket stops at lists rather than covering every type: the sentence is
/// vacuously true of anything, and letting it be said would make this trait
/// implementable for a type that is not a list.
#[marker]
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not hold every member of `{L}`",
    note = "Each member of `{L}` is checked on its own, so the one that failed is not named here. Bound on `Contains<T>` for the individual type to find out which. Check also that `{Self}` is a list and that `{L}` is one, since both sides are walked. If the compiler reports `overflow evaluating the requirement` instead, the crate root wants `#![recursion_limit = \"1024\"]`."
)]
pub trait ContainsAll<L>: List {}

impl<S: List> ContainsAll<Empty> for S {}

impl<S, H, T> ContainsAll<Cons<H, T>> for S where S: Contains<H> + ContainsAll<T> + List {}
