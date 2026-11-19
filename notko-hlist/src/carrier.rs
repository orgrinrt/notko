//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Carrier`, `Uniform` and `Elem`: what a product of a list holds for each
//! member.
//!
//! A list of types says nothing about values, and a consumer holding values for
//! one wants them in more than one shape: one value per member for a row, a
//! slice per member for a column, an array per member for owned storage, or a
//! form chosen per member for a read list, a value here and a borrow there. The
//! shape is the carrier, and [`Project`](crate::Project) builds the product for
//! a whole list through whichever carrier is asked for.
//!
//! A carrier is a typestate: a marker whose type, lifetimes included, is the
//! state, with nothing about it decided at run time. It declares its [`Shape`]
//! once. A uniform carrier holds every [`Elem`] member the same way over its
//! value, through [`Uniform`], and nothing else. A per-member carrier states
//! exactly one form for each member it holds, through one impl of [`Form<M>`].
//! [`Carrier<M>`], what a product reads, is this crate's alone and follows the
//! shape, so a form stated for a carrier of the other shape, by whichever
//! crate, is refused, and a carrier is uniform or per member and never both.
//!
//! A member of a uniform carrier is a marker that names its value type,
//! [`Elem`], rather than the value type itself. Two members may carry the same
//! value type and still be two members, which a list of bare value types could
//! not say. A per-member carrier asks nothing of its members beyond its own
//! impls.
//!
//! # Module layout
//!
//! The capabilities [`ByRow`], [`ByRowMut`] and [`Dup`] come in a const twin
//! and a plain twin like `Cardinal` does, and here both are written once, in
//! `carrier_items!`, which takes the constness as its one argument. The
//! declarations and the bodies are the same on both paths, so a second file
//! holding the same text with `const` struck out would only be a place for the
//! two to drift apart.

use crate::seal::Token;

/// A member of a list, naming the one value type a uniform carrier holds it
/// over.
///
/// The marker itself carries nothing and is usually a unit struct, so a call
/// site can pass it as a value to say which member it means.
///
/// ```
/// use notko_hlist::Elem;
///
/// struct Pos;
/// impl Elem for Pos {
///     type Value = i32;
/// }
/// ```
///
/// `'static` on both, and `Copy` on the value, because a product is read and
/// written by row and every row is a fresh copy of the values in it. A value
/// that borrows would tie every product holding it to that borrow, and none of
/// the uniform carriers wants that. A borrow for a frame goes on a carrier of
/// its own instead, `Read<'f>` holding `&'f T` for the member, and the list
/// names no lifetime at all.
#[diagnostic::on_unimplemented(
    message = "`{Self}` names no value type",
    note = "A member of a product is a marker implementing `Elem`, whose `Value` is what the product carries for it. A bare value type is not a member: wrap it in a marker of its own, so that two members carrying the same type stay two members."
)]
pub trait Elem: 'static {
    /// The value carried for this member.
    type Value: Copy + 'static;
}

/// The shape of a carrier, declared once in its type: [`Uniformly`] or
/// [`PerMember`].
///
/// This is what makes a carrier's forms a function of its type alone. A
/// uniform carrier holds every [`Elem`] member over its value and nothing else,
/// a per-member carrier holds exactly the members it states a [`Form`] for, and
/// the shape decides which of the two routes [`Carrier`] reads. A form stated
/// for a carrier of the other shape is refused, so neither can be given an
/// exception by a crate that owns a member.
#[diagnostic::on_unimplemented(
    message = "`{Self}` declares no shape",
    note = "A carrier declares its shape: `Uniformly` for one that implements `Uniform`, `PerMember` for one that states a `Form<M>` for each member it holds."
)]
pub trait Shape {
    /// The shape, [`Uniformly`] or [`PerMember`].
    type Is: shape::Kind;
}

/// The shape of a carrier holding every [`Elem`] member the same way.
pub struct Uniformly;

/// The shape of a carrier stating a [`Form`] for each member it holds.
pub struct PerMember;

/// The two shapes, closed.
mod shape {
    /// Implemented for [`Uniformly`](super::Uniformly) and
    /// [`PerMember`](super::PerMember) and nothing else, since a third shape
    /// would be a route [`Carrier`](super::Carrier) does not read.
    pub trait Kind {}
    impl Kind for super::Uniformly {}
    impl Kind for super::PerMember {}
}

/// A carrier holding every [`Elem`] member the same way, over its value.
///
/// `Of<E>` is the type held for member `E`. The four carriers in this crate,
/// [`Val`], [`Col`], [`ColMut`] and [`Arr`], are uniform, and they are what the
/// row capabilities and the walks between products work over.
pub trait Uniform: Shape<Is = Uniformly> {
    /// What is held for member `E`.
    type Of<E: Elem>;
}

/// The form a per-member carrier `Self` holds for member `M`.
///
/// An associated type, so one impl per carrier and member and one form per
/// pair: coherence refuses a second. A product through `Self` exists for a
/// list exactly where every member has one.
///
/// ```
/// use core::marker::PhantomData;
///
/// use notko_hlist::{
///     Cons,
///     Empty,
///     Form,
///     Has,
///     PCons,
///     PNil,
///     PerMember,
///     Project,
///     Shape,
/// };
///
/// struct Seq(u32);
/// struct Count;
/// struct Store;
///
/// /// What a unit reads for the frame `'f`.
/// struct Read<'f>(PhantomData<&'f ()>);
/// impl<'f> Shape for Read<'f> {
///     type Is = PerMember;
/// }
/// impl<'f> Form<Count> for Read<'f> {
///     type Of = u32;
/// }
/// impl<'f> Form<Store> for Read<'f> {
///     type Of = &'f Seq;
/// }
///
/// type Reads = Cons<Count, Cons<Store, Empty>>;
///
/// let seq = Seq(5);
/// let p: <Reads as Project<Read<'_>>>::Prod = PCons {
///     head: 3,
///     tail: PCons {
///         head: &seq,
///         tail: PNil,
///     },
/// };
/// assert_eq!(<Reads as Has<Read<'_>, Store, _>>::pick(&p).0, 5);
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` states no form for `{M}`",
    note = "A per-member carrier declares `Shape<Is = PerMember>` and states one `impl Form<{M}>` for each member it holds. A uniform carrier states none: what it holds is `Uniform::Of`, for `Elem` members only."
)]
pub trait Form<M>: Shape<Is = PerMember> {
    /// What is held for `M`.
    type Of;
}

/// Carrier `Self` holds `Of` for member `M`: what a product reads.
///
/// Implemented by this crate alone, for every carrier, through its [`Shape`]:
/// a uniform carrier holds `Uniform::Of<M>` for an [`Elem`] member, a
/// per-member carrier holds its `Form<M>::Of`, and nothing else holds anything.
/// Sealed, so no crate that owns a member can make a carrier hold a form for it
/// that the carrier's own shape does not give.
#[diagnostic::on_unimplemented(
    message = "`{Self}` holds no form for `{M}`",
    note = "A uniform carrier holds every `Elem` member and nothing else, and a per-member carrier holds exactly the members it states a `Form<{M}>` for. A product through it exists only where every member of the list is held."
)]
pub trait Carrier<M> {
    /// What is held for `M`.
    type Of;

    /// The seal. Never needs calling.
    #[doc(hidden)]
    fn sealed(_: Token);
}

/// The route [`Carrier`] takes for a carrier of shape `K`. Sealed.
#[doc(hidden)]
pub trait Route<M, K> {
    /// What is held for `M`.
    type Of;

    /// The seal. Never needs calling.
    #[doc(hidden)]
    fn sealed(_: Token);
}

#[diagnostic::do_not_recommend]
impl<C: Uniform, E: Elem> Route<E, Uniformly> for C {
    type Of = C::Of<E>;

    fn sealed(_: Token) {}
}

#[diagnostic::do_not_recommend]
impl<C: Form<M>, M> Route<M, PerMember> for C {
    type Of = <C as Form<M>>::Of;

    fn sealed(_: Token) {}
}

// Not recommended, so that a missing form is reported through `Carrier`'s own
// note rather than as the route behind it.
#[diagnostic::do_not_recommend]
impl<C: Shape + Route<M, <C as Shape>::Is>, M> Carrier<M> for C {
    type Of = <C as Route<M, <C as Shape>::Is>>::Of;

    fn sealed(_: Token) {}
}

/// One value per member.
pub struct Val;

/// A shared column per member, borrowed for `'a`.
pub struct Col<'a>(core::marker::PhantomData<&'a ()>);

/// An exclusive column per member, borrowed for `'a`.
pub struct ColMut<'a>(core::marker::PhantomData<&'a mut ()>);

/// An owned array of `N` per member.
pub struct Arr<const N: usize>;

// No impl in this crate is offered as a suggestion when a bound goes
// unsatisfied, for the reason `at.rs` gives: the suggestion prints the impl's
// source path, which in a consumer is its cargo checkout, and a downstream
// compile-fail fixture holding it passes on one machine only.
#[diagnostic::do_not_recommend]
impl Uniform for Val {
    type Of<E: Elem> = E::Value;
}

#[diagnostic::do_not_recommend]
impl<'a> Uniform for Col<'a> {
    type Of<E: Elem> = &'a [E::Value];
}

#[diagnostic::do_not_recommend]
impl<'a> Uniform for ColMut<'a> {
    type Of<E: Elem> = &'a mut [E::Value];
}

#[diagnostic::do_not_recommend]
impl<const N: usize> Uniform for Arr<N> {
    type Of<E: Elem> = [E::Value; N];
}

#[diagnostic::do_not_recommend]
impl Shape for Val {
    type Is = Uniformly;
}

#[diagnostic::do_not_recommend]
impl<'a> Shape for Col<'a> {
    type Is = Uniformly;
}

#[diagnostic::do_not_recommend]
impl<'a> Shape for ColMut<'a> {
    type Is = Uniformly;
}

#[diagnostic::do_not_recommend]
impl<const N: usize> Shape for Arr<N> {
    type Is = Uniformly;
}

macro_rules! carrier_items {
    ($($c:ident)?) => {
        /// A carrier that can be read at a row.
        ///
        /// The row is a slice index, so it is a `usize`: the carriers here are
        /// core's slices and arrays and that is the number they are indexed by.
        /// A row past the end panics the way slice indexing does.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` is not read by row",
            note = "The carriers read by row are `Col`, `ColMut` and `Arr<N>`. `Val` is one value per member and has no rows."
        )]
        pub $($c)? trait ByRow: Uniform {
            /// The value of member `E` at row `i`.
            fn at<E: Elem>(c: &Self::Of<E>, i: usize) -> E::Value;
        }

        /// A carrier that can be written at a row.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` is not written by row",
            note = "The carriers written by row are `ColMut` and `Arr<N>`. A `Col` is shared and is read only."
        )]
        pub $($c)? trait ByRowMut: ByRow {
            /// Writes `v` as the value of member `E` at row `i`.
            fn set<E: Elem>(c: &mut Self::Of<E>, i: usize, v: E::Value);
        }

        /// A carrier whose item for member `M` can be duplicated.
        ///
        /// A value is copied and a shared column is borrowed again, so a
        /// product over either can be taken apart into a smaller one while the
        /// original stays usable. An exclusive column cannot be, and has no impl.
        /// Per member, because a per-member carrier knows for each of its forms
        /// whether that form can be duplicated.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` cannot duplicate what it holds for `{M}`",
            note = "Of the uniform carriers, `Val` and `Col` duplicate every member. A `ColMut` duplicated would be two exclusive borrows of one column, and an `Arr<N>` would be a copy of the whole storage, so a part of either is cut through `Arrays::cols` first. A carrier of your own states `Dup` per member."
        )]
        pub $($c)? trait Dup<M>: Carrier<M> {
            /// Another of what `c` is, for the same member.
            fn dup(c: &<Self as Carrier<M>>::Of) -> <Self as Carrier<M>>::Of;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<'a> ByRow for Col<'a> {
            fn at<E: Elem>(c: &Self::Of<E>, i: usize) -> E::Value {
                c[i]
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<'a> ByRow for ColMut<'a> {
            fn at<E: Elem>(c: &Self::Of<E>, i: usize) -> E::Value {
                c[i]
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<'a> ByRowMut for ColMut<'a> {
            fn set<E: Elem>(c: &mut Self::Of<E>, i: usize, v: E::Value) {
                c[i] = v;
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<const N: usize> ByRow for Arr<N> {
            fn at<E: Elem>(c: &Self::Of<E>, i: usize) -> E::Value {
                c[i]
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<const N: usize> ByRowMut for Arr<N> {
            fn set<E: Elem>(c: &mut Self::Of<E>, i: usize, v: E::Value) {
                c[i] = v;
            }
        }

        /// A uniform carrier that duplicates what it holds for every member.
        ///
        /// No member parameter, so no crate but this one can say it of a
        /// carrier it does not own, and through the blanket impl below no crate
        /// can state `Dup` for a single member of a uniform carrier either: that
        /// impl conflicts with the blanket one. `Val` and `Col` are the two.
        pub $($c)? trait DupAll: Uniform {
            /// Another of what `c` is, for the same member.
            fn dup_all<E: Elem>(c: &Self::Of<E>) -> Self::Of<E>;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C: $([$c])? DupAll, E: Elem> Dup<E> for C {
            fn dup(c: &<Self as Carrier<E>>::Of) -> <Self as Carrier<E>>::Of {
                C::dup_all::<E>(c)
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl DupAll for Val {
            fn dup_all<E: Elem>(c: &E::Value) -> E::Value {
                *c
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<'a> DupAll for Col<'a> {
            fn dup_all<E: Elem>(c: &&'a [E::Value]) -> &'a [E::Value] {
                c
            }
        }
    };
}

#[cfg(feature = "const")]
carrier_items!(const);

#[cfg(not(feature = "const"))]
carrier_items!();
