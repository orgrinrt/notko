//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Has` and `Vals`: a member's field of a product, found from the member.
//!
//! [`PlaceOf`] finds the place of a member in a list of types. `Has` walks a
//! product of that list to the same place and hands out the field there, through
//! whichever carrier the product is over. [`Vals`] is the product of values at
//! one row, read and written by naming the member.
//!
//! # Module layout
//!
//! A const twin and a plain twin, both written once in `has_items!`. See
//! `carrier.rs` for why.

use crate::carrier::{Carrier, Elem, Val};
use crate::list::Cons;
use crate::place_of::PlaceOf;
use crate::position::{Here, Place, There};
use crate::project::Project;

macro_rules! has_items {
    ($($c:ident)?) => {
        /// The field of member `M` in a product of `Self`, at the place `P`.
        ///
        /// `P` is the place [`PlaceOf`] answers, inferred the same way, and with the
        /// same refusals: no place when the member is absent, and an ambiguous
        /// one when it is present twice.
        ///
        /// ```
        /// use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Has, Project, Val};
        ///
        /// struct Pos;
        /// impl Elem for Pos {
        ///     type Value = i32;
        /// }
        /// struct Heat;
        /// impl Elem for Heat {
        ///     type Value = u8;
        /// }
        /// type Lanes = Cons<Pos, Cons<Heat, Empty>>;
        ///
        /// fn heat<L: Has<Heat, P>, P: notko_hlist::Place>(row: &L::Prod<Val>) -> u8 {
        ///     *L::pick::<Val>(row)
        /// }
        ///
        /// let row: <Lanes as Project>::Prod<Val> = PCons { head: 7, tail: PCons { head: 3, tail: PNil } };
        /// assert_eq!(heat::<Lanes, _>(&row), 3);
        /// ```
        ///
        /// Closed the way `PlaceOf` is: an impl from outside has to name a place
        /// where `PlaceOf` already holds, and the impls below cover every one.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` has no field for `{M}`",
            note = "A field is found where `PlaceOf` finds the member: `Here` for the head of a `Cons`, `There<P>` when the tail holds it at `P`. The usual cause is a member the list does not hold, and if the compiler reports `type annotations needed` instead, the list holds it at more than one place."
        )]
        pub $($c)? trait Has<M: Elem, P: Place>: PlaceOf<M, P> + Project {
            /// The field of `M` in `p`.
            fn pick<C: Carrier>(p: &Self::Prod<C>) -> &C::Of<M>;

            /// The field of `M` in `p`, to write.
            fn pick_mut<C: Carrier>(p: &mut Self::Prod<C>) -> &mut C::Of<M>;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<M: Elem, T: $([$c])? Project> Has<M, Here> for Cons<M, T> {
            fn pick<C: Carrier>(p: &Self::Prod<C>) -> &C::Of<M> {
                &p.head
            }

            fn pick_mut<C: Carrier>(p: &mut Self::Prod<C>) -> &mut C::Of<M> {
                &mut p.head
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<H: Elem, M: Elem, P: Place, T: $([$c])? Has<M, P>> Has<M, There<P>> for Cons<H, T> {
            fn pick<C: Carrier>(p: &Self::Prod<C>) -> &C::Of<M> {
                T::pick::<C>(&p.tail)
            }

            fn pick_mut<C: Carrier>(p: &mut Self::Prod<C>) -> &mut C::Of<M> {
                T::pick_mut::<C>(&mut p.tail)
            }
        }

        impl<L: Project> Vals<L> {
            /// The value of `member`.
            pub $($c)? fn get<M: Elem, P: Place>(&self, member: M) -> M::Value
            where
                L: $([$c])? Has<M, P>,
            {
                // The marker says which member and nothing else. Forgotten
                // rather than dropped, since dropping a generic value is not
                // something a const fn may do, and a marker has nothing to drop.
                core::mem::forget(member);
                *L::pick::<Val>(&self.0)
            }

            /// Writes `v` as the value of `member`.
            pub $($c)? fn set<M: Elem, P: Place>(&mut self, member: M, v: M::Value)
            where
                L: $([$c])? Has<M, P>,
            {
                core::mem::forget(member);
                *L::pick_mut::<Val>(&mut self.0) = v;
            }
        }
    };
}

/// The values of a list at one row, read and written by member.
///
/// The member is passed as a value, the marker itself, so a call reads as the
/// member it means and its place is inferred from it.
///
/// ```
/// use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Vals};
///
/// struct Pos;
/// impl Elem for Pos {
///     type Value = i32;
/// }
/// struct Heat;
/// impl Elem for Heat {
///     type Value = u8;
/// }
///
/// let mut row = Vals::<Cons<Pos, Cons<Heat, Empty>>>(PCons {
///     head: 7,
///     tail: PCons {
///         head: 3,
///         tail: PNil,
///     },
/// });
/// row.set(Heat, 4);
/// assert_eq!(row.get(Pos), 7);
/// assert_eq!(row.get(Heat), 4);
/// ```
///
/// The marker is taken by value and forgotten, so a unit marker costs nothing
/// at the call, and a marker with a `Drop` impl does not have it run: a member
/// is a name, and a name with something to release is not one.
pub struct Vals<L: Project>(pub L::Prod<Val>);

#[cfg(feature = "const")]
has_items!(const);

#[cfg(not(feature = "const"))]
has_items!();
