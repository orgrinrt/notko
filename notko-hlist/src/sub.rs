//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `SubOf`, `PutRow` and `PutVals`: one list's product cut out of another's,
//! and written back into it.
//!
//! A consumer holding a product of a wide list hands a part of it to code that
//! names a narrower one. Every member of the narrower list is found in the wider
//! one through [`Has`], at the places `Ps`, one place per member in the
//! narrower list's order, all inferred. The carrier is a parameter of each
//! trait, for the reason [`Has`] gives.
//!
//! # Module layout
//!
//! A const twin and a plain twin, both written once in `sub_items!`. See
//! `carrier.rs` for why.

use crate::carrier::{ByRowMut, Dup, Elem, Uniform, Val};
use crate::has::Has;
use crate::list::{Cons, Empty, List};
use crate::position::Place;
use crate::project::{PCons, PNil, Project};
use crate::seal::Token;

macro_rules! sub_items {
    ($($c:ident)?) => {
        /// Every member of `Self` is in `S`, at the places `Ps`, and its product
        /// through `C` is cut out of `S`'s.
        ///
        /// `Ps` is a [`List`], and the trait is sealed the way [`Project`] is.
        ///
        /// ```
        /// use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, SubOf, Val};
        ///
        /// struct A;
        /// impl Elem for A {
        ///     type Value = u8;
        /// }
        /// struct B;
        /// impl Elem for B {
        ///     type Value = u16;
        /// }
        /// struct C;
        /// impl Elem for C {
        ///     type Value = u32;
        /// }
        /// type Wide = Cons<A, Cons<B, Cons<C, Empty>>>;
        /// type Narrow = Cons<C, Cons<A, Empty>>;
        ///
        /// let wide: <Wide as Project<Val>>::Prod =
        ///     PCons { head: 1, tail: PCons { head: 2, tail: PCons { head: 3, tail: PNil } } };
        /// let narrow = <Narrow as SubOf<Val, Wide, _>>::sub(&wide);
        /// assert_eq!((narrow.head, narrow.tail.head), (3, 1));
        /// ```
        ///
        /// Order does not carry, the way it does not for `ContainsAll`: the
        /// narrower list names its members in any order, and its product comes
        /// out in that order. A member it names twice is found twice. Each field
        /// is duplicated through [`Dup`], so a carrier that cannot duplicate what
        /// it holds for a member has no cut for a list naming that member.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` is not a part of `{S}` through `{C}`",
            note = "Each member of `{Self}` is found in `{S}` through `Has` and duplicated through `Dup<M>`, so the one that failed is not named here. Bound on `PlaceOf<M, _>` for the individual member to find out which, and check that the carrier duplicates what it holds for each."
        )]
        pub $($c)? trait SubOf<C, S: Project<C>, Ps: List>: Project<C> {
            /// The seal. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: Token);

            /// The narrower product, each field duplicated out of `s`.
            fn sub(s: &<S as Project<C>>::Prod) -> <Self as Project<C>>::Prod;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C, S: Project<C>> SubOf<C, S, Empty> for Empty {
            fn sealed(_: Token) {}

            fn sub(_: &<S as Project<C>>::Prod) -> PNil {
                PNil
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C, S, H, T, P, Ps> SubOf<C, S, Cons<P, Ps>> for Cons<H, T>
        where
            S: $([$c])? Has<C, H, P>,
            C: $([$c])? Dup<H>,
            P: Place,
            Ps: List,
            T: $([$c])? SubOf<C, S, Ps>,
        {
            fn sealed(_: Token) {}

            fn sub(s: &<S as Project<C>>::Prod) -> <Self as Project<C>>::Prod {
                PCons {
                    head: C::dup(<S as Has<C, H, P>>::pick(s)),
                    tail: T::sub(s),
                }
            }
        }

        /// The values of `Self` at one row, written into `S`'s product through
        /// the uniform carrier `C`, each at its member's field.
        ///
        /// ```
        /// use notko_hlist::{Arr, Cons, Elem, Empty, PCons, PNil, Project, PutRow};
        ///
        /// struct A;
        /// impl Elem for A {
        ///     type Value = u8;
        /// }
        /// struct B;
        /// impl Elem for B {
        ///     type Value = u16;
        /// }
        /// type Wide = Cons<A, Cons<B, Empty>>;
        /// type Narrow = Cons<B, Empty>;
        ///
        /// let mut wide: <Wide as Project<Arr<2>>>::Prod = PCons { head: [0; 2], tail: PCons { head: [0; 2], tail: PNil } };
        /// <Narrow as PutRow<Arr<2>, Wide, _>>::put_row(&mut wide, 1, &PCons { head: 7, tail: PNil });
        /// assert_eq!(wide.tail.head, [0, 7]);
        /// ```
        ///
        /// Sealed the way [`Project`] is.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` cannot be written into `{S}` through `{C}`",
            note = "Each member of `{Self}` is found in `{S}` through `Has`, and every member implements `Elem`, since a row is a product of values."
        )]
        pub $($c)? trait PutRow<C: Uniform, S: Project<C>, Ps: List>: Project<Val> {
            /// The seal. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: Token);

            /// Writes the values `v` into `s` at row `i`, each at its member's
            /// field.
            fn put_row(s: &mut <S as Project<C>>::Prod, i: usize, v: &<Self as Project<Val>>::Prod)
            where
                C: $([$c])? ByRowMut;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C: Uniform, S: Project<C>> PutRow<C, S, Empty> for Empty {
            fn sealed(_: Token) {}

            fn put_row(_: &mut <S as Project<C>>::Prod, _: usize, _: &PNil)
            where
                C: $([$c])? ByRowMut,
            {
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C, S, H, T, P, Ps> PutRow<C, S, Cons<P, Ps>> for Cons<H, T>
        where
            C: Uniform,
            S: $([$c])? Has<C, H, P>,
            H: Elem,
            P: Place,
            Ps: List,
            T: $([$c])? PutRow<C, S, Ps>,
        {
            fn sealed(_: Token) {}

            fn put_row(s: &mut <S as Project<C>>::Prod, i: usize, v: &<Self as Project<Val>>::Prod)
            where
                C: $([$c])? ByRowMut,
            {
                C::set::<H>(<S as Has<C, H, P>>::pick_mut(s), i, v.head);
                T::put_row(s, i, &v.tail);
            }
        }

        /// The values of `Self` written into the values of `S`, each at its
        /// member's field.
        ///
        /// Sealed the way [`Project`] is.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` cannot be written into the values of `{S}`",
            note = "Each member of `{Self}` is found in `{S}` through `Has`, and every member implements `Elem`."
        )]
        pub $($c)? trait PutVals<S: Project<Val>, Ps: List>: Project<Val> {
            /// The seal. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: Token);

            /// Writes the values `v` into the values `s`, each at its member's
            /// field.
            fn put_vals(s: &mut <S as Project<Val>>::Prod, v: &<Self as Project<Val>>::Prod);
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<S: Project<Val>> PutVals<S, Empty> for Empty {
            fn sealed(_: Token) {}

            fn put_vals(_: &mut <S as Project<Val>>::Prod, _: &PNil) {}
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<S, H, T, P, Ps> PutVals<S, Cons<P, Ps>> for Cons<H, T>
        where
            S: $([$c])? Has<Val, H, P>,
            H: Elem,
            P: Place,
            Ps: List,
            T: $([$c])? PutVals<S, Ps>,
        {
            fn sealed(_: Token) {}

            fn put_vals(s: &mut <S as Project<Val>>::Prod, v: &<Self as Project<Val>>::Prod) {
                *<S as Has<Val, H, P>>::pick_mut(s) = v.head;
                T::put_vals(s, &v.tail);
            }
        }
    };
}

#[cfg(feature = "const")]
sub_items!(const);

#[cfg(not(feature = "const"))]
sub_items!();
