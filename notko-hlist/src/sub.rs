//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `SubOf`: one list's product cut out of another's.
//!
//! A consumer holding a product of a wide list hands a part of it to code that
//! names a narrower one. Every member of the narrower list is found in the wider
//! one through [`Has`], at the places `Ps`, one place per member in the
//! narrower list's order, all inferred.
//!
//! # Module layout
//!
//! A const twin and a plain twin, both written once in `sub_items!`. See
//! `carrier.rs` for why.

use crate::carrier::{ByRowMut, Dup, Elem, Val};
use crate::has::Has;
use crate::list::{Cons, Empty};
use crate::position::Place;
use crate::project::{PCons, PNil, Project};

macro_rules! sub_items {
    ($($c:ident)?) => {
        /// Every member of `Self` is in `S`, at the places `Ps`.
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
        /// let wide: <Wide as Project>::Prod<Val> =
        ///     PCons { head: 1, tail: PCons { head: 2, tail: PCons { head: 3, tail: PNil } } };
        /// let narrow = <Narrow as SubOf<Wide, _>>::sub::<Val>(&wide);
        /// assert_eq!((narrow.head, narrow.tail.head), (3, 1));
        /// ```
        ///
        /// Order does not carry, the way it does not for `ContainsAll`: the
        /// narrower list names its members in any order, and its product comes
        /// out in that order. A member it names twice is found twice.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` is not a part of `{S}`",
            note = "Each member of `{Self}` is found in `{S}` through `Has`, so the one that failed is not named here. Bound on `PlaceOf<M, _>` for the individual member to find out which, and check that both lists have a product, which takes every member implementing `Elem`."
        )]
        pub $($c)? trait SubOf<S: Project, Ps>: Project {
            /// The narrower product, each field duplicated out of `s`.
            fn sub<C: $([$c])? Dup>(s: &S::Prod<C>) -> Self::Prod<C>;

            /// Writes the values `v` into `s` at row `i`, each at its member's
            /// field.
            fn put_row<C: $([$c])? ByRowMut>(s: &mut S::Prod<C>, i: usize, v: &Self::Prod<Val>);

            /// Writes the values `v` into the values `s`, each at its member's
            /// field.
            fn put_vals(s: &mut S::Prod<Val>, v: &Self::Prod<Val>);
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<S: Project> SubOf<S, Empty> for Empty {
            fn sub<C: $([$c])? Dup>(_: &S::Prod<C>) -> PNil {
                PNil
            }

            fn put_row<C: $([$c])? ByRowMut>(_: &mut S::Prod<C>, _: usize, _: &PNil) {}

            fn put_vals(_: &mut S::Prod<Val>, _: &PNil) {}
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<S, H, T, P, Ps> SubOf<S, Cons<P, Ps>> for Cons<H, T>
        where
            S: $([$c])? Has<H, P>,
            H: Elem,
            P: Place,
            T: $([$c])? SubOf<S, Ps>,
        {
            fn sub<C: $([$c])? Dup>(s: &S::Prod<C>) -> Self::Prod<C> {
                PCons {
                    head: C::dup::<H>(S::pick::<C>(s)),
                    tail: T::sub::<C>(s),
                }
            }

            fn put_row<C: $([$c])? ByRowMut>(s: &mut S::Prod<C>, i: usize, v: &Self::Prod<Val>) {
                C::set::<H>(S::pick_mut::<C>(s), i, v.head);
                T::put_row::<C>(s, i, &v.tail);
            }

            fn put_vals(s: &mut S::Prod<Val>, v: &Self::Prod<Val>) {
                *S::pick_mut::<Val>(s) = v.head;
                T::put_vals(s, &v.tail);
            }
        }
    };
}

#[cfg(feature = "const")]
sub_items!(const);

#[cfg(not(feature = "const"))]
sub_items!();
