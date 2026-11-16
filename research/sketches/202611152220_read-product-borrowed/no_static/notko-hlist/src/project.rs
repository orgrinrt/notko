//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Project`: a list of members as a product of what a carrier holds for each.
//!
//! [`Cons`] holds nothing, so a consumer wanting one value per member, or one
//! column per member, had to build the matching nest of structs by hand and walk
//! it by hand. `Project` is that nest, derived from the list: `Cons<H, T>`
//! becomes [`PCons`] holding `C::Of<H>` in front of the tail's product, and
//! `Empty` becomes [`PNil`]. Every operation on it recurses on the list's
//! structure and nothing else.
//!
//! # Whole-product operations are in place
//!
//! A product over a long list is as deep as the list, and moving one by value
//! recurses with a frame per level holding the rest of it. Deep enough, in a
//! debug build, that overflows a thread's default stack. So the operations on a
//! whole product of arrays take it by reference and write field by field:
//! [`Project::copy_from`] copies into an existing product, and [`Project::made`]
//! builds one inside a `MaybeUninit` through [`Project::splat_in`].
//!
//! # Module layout
//!
//! The trait comes in a const twin and a plain twin, and both are written once in
//! `project_items!`, which takes the constness as its argument. See `carrier.rs`
//! for why.

use crate::carrier::{Arr, ByRow, ByRowMut, Carrier, Col, ColMut, Elem, Val};
use crate::list::{Cons, Empty, List};

/// The seal on [`Project`]: a token only this crate can name.
mod seal {
    /// Passed to `Project::sealed`. Public in a private
    /// module, so a signature can carry it and nothing outside can write one.
    pub struct Token;
}

/// A cell of a product: what carrier `C` holds for member `H`, then the rest.
pub struct PCons<C: Carrier, H: Elem, T> {
    /// What `C` holds for the head member.
    pub head: C::Of<H>,
    /// The product of the tail.
    pub tail: T,
}

/// The end of a product.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PNil;

impl<C: Carrier, H: Elem, T: Default> Default for PCons<C, H, T>
where
    C::Of<H>: Default,
{
    fn default() -> Self {
        PCons {
            head: Default::default(),
            tail: T::default(),
        }
    }
}

impl<C: Carrier, H: Elem, T: Clone> Clone for PCons<C, H, T>
where
    C::Of<H>: Clone,
{
    fn clone(&self) -> Self {
        PCons {
            head: self.head.clone(),
            tail: self.tail.clone(),
        }
    }
}

impl<C: Carrier, H: Elem, T: Copy> Copy for PCons<C, H, T> where C::Of<H>: Copy {}

impl<C: Carrier, H: Elem, T: PartialEq> PartialEq for PCons<C, H, T>
where
    C::Of<H>: PartialEq,
{
    fn eq(&self, o: &Self) -> bool {
        self.head == o.head && self.tail == o.tail
    }
}

impl<C: Carrier, H: Elem, T: Eq> Eq for PCons<C, H, T> where C::Of<H>: Eq {}

impl<C: Carrier, H: Elem, T: core::fmt::Debug> core::fmt::Debug for PCons<C, H, T>
where
    C::Of<H>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PCons")
            .field("head", &self.head)
            .field("tail", &self.tail)
            .finish()
    }
}

macro_rules! project_items {
    ($($c:ident)?) => {
        /// A list of [`Elem`] members, projected to a product of what a carrier
        /// holds for each.
        ///
        /// ```
        /// use notko_hlist::{Arr, Cons, Elem, Empty, PCons, PNil, Project, Val};
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
        /// let row: <Lanes as Project>::Prod<Val> = PCons { head: 7, tail: PCons { head: 3, tail: PNil } };
        /// let mut cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row);
        ///
        /// Lanes::write_row::<Arr<4>>(&mut cols, 2, &PCons { head: -1, tail: PCons { head: 9, tail: PNil } });
        /// assert_eq!(cols.head, [7, 7, -1, 7]);
        /// assert_eq!(Lanes::read_row::<Arr<4>>(&cols, 2).tail.head, 9);
        /// ```
        ///
        /// No impl outside this crate exists or can. `Project` has a required
        /// method, `Project::sealed`, taking a token whose type is unnameable
        /// outside this crate, so an impl elsewhere cannot be written whatever its
        /// `Self`. That closure is what [`Project::made`] rests its `unsafe` on:
        /// the two impls below are every impl there is.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` has no product",
            note = "A product is defined for lists built from `Empty` and `Cons<H, T>` whose every member implements `Elem`, so the usual cause is a member that names no value type, or a tail that is not a list."
        )]
        pub $($c)? trait Project: List {
            /// The product through carrier `C`: [`PCons`] per cell, [`PNil`]
            /// at the end.
            type Prod<C: Carrier>;

            /// The seal. Takes a token nothing outside this crate can name, so no
            /// impl outside it can supply this method. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: seal::Token);

            /// The values at row `i`, one per member.
            fn read_row<C: $([$c])? ByRow>(p: &Self::Prod<C>, i: usize) -> Self::Prod<Val>;

            /// Writes the values `v` at row `i`, one per member.
            fn write_row<C: $([$c])? ByRowMut>(p: &mut Self::Prod<C>, i: usize, v: &Self::Prod<Val>);

            /// A shared column over each member's array.
            fn cols<const N: usize>(p: &Self::Prod<Arr<N>>) -> Self::Prod<Col<'_>>;

            /// An exclusive column over each member's array.
            fn cols_mut<const N: usize>(p: &mut Self::Prod<Arr<N>>) -> Self::Prod<ColMut<'_>>;

            /// Copies `src` into `dst`, field by field.
            ///
            /// In place rather than by value: see the module docs on depth.
            fn copy_from<const N: usize>(dst: &mut Self::Prod<Arr<N>>, src: &Self::Prod<Arr<N>>);

            /// Writes every field of `dst`, each member's array filled with that
            /// member's value from `row`.
            ///
            /// Every field, the head and then the tail's, down to `PNil`. That is
            /// the promise [`Project::made`] reads `dst` as initialised on, and
            /// the two impls in this crate are the only ones that make it.
            fn splat_in<const N: usize>(
                dst: &mut core::mem::MaybeUninit<Self::Prod<Arr<N>>>,
                row: &Self::Prod<Val>,
            );

            /// A product of arrays with every row equal to `row`, built in place.
            fn made<const N: usize>(row: &Self::Prod<Val>) -> Self::Prod<Arr<N>> {
                let mut p = core::mem::MaybeUninit::uninit();
                Self::splat_in::<N>(&mut p, row);
                // SAFETY: every impl of `Project` is one of the two below, since
                // an impl must supply `sealed`, whose `seal::Token` argument no
                // other crate can name. Both `splat_in`s write every field of the product, the
                // head and then the tail through the tail's own `splat_in`, down
                // to `PNil`, so `p` is initialised here.
                unsafe { p.assume_init() }
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl Project for Empty {
            type Prod<C: Carrier> = PNil;

            fn sealed(_: seal::Token) {}

            fn read_row<C: $([$c])? ByRow>(_: &PNil, _: usize) -> PNil {
                PNil
            }

            fn write_row<C: $([$c])? ByRowMut>(_: &mut PNil, _: usize, _: &PNil) {}

            fn cols<const N: usize>(_: &PNil) -> PNil {
                PNil
            }

            fn cols_mut<const N: usize>(_: &mut PNil) -> PNil {
                PNil
            }

            fn copy_from<const N: usize>(_: &mut PNil, _: &PNil) {}

            fn splat_in<const N: usize>(dst: &mut core::mem::MaybeUninit<PNil>, _: &PNil) {
                dst.write(PNil);
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<H: Elem, T: $([$c])? Project> Project for Cons<H, T> {
            type Prod<C: Carrier> = PCons<C, H, T::Prod<C>>;

            fn sealed(_: seal::Token) {}

            fn read_row<C: $([$c])? ByRow>(p: &Self::Prod<C>, i: usize) -> Self::Prod<Val> {
                PCons {
                    head: C::at::<H>(&p.head, i),
                    tail: T::read_row::<C>(&p.tail, i),
                }
            }

            fn write_row<C: $([$c])? ByRowMut>(p: &mut Self::Prod<C>, i: usize, v: &Self::Prod<Val>) {
                C::set::<H>(&mut p.head, i, v.head);
                T::write_row::<C>(&mut p.tail, i, &v.tail);
            }

            fn cols<const N: usize>(p: &Self::Prod<Arr<N>>) -> Self::Prod<Col<'_>> {
                PCons {
                    head: &p.head,
                    tail: T::cols::<N>(&p.tail),
                }
            }

            fn cols_mut<const N: usize>(p: &mut Self::Prod<Arr<N>>) -> Self::Prod<ColMut<'_>> {
                PCons {
                    head: &mut p.head,
                    tail: T::cols_mut::<N>(&mut p.tail),
                }
            }

            fn copy_from<const N: usize>(dst: &mut Self::Prod<Arr<N>>, src: &Self::Prod<Arr<N>>) {
                dst.head = src.head;
                T::copy_from::<N>(&mut dst.tail, &src.tail);
            }

            fn splat_in<const N: usize>(
                dst: &mut core::mem::MaybeUninit<Self::Prod<Arr<N>>>,
                row: &Self::Prod<Val>,
            ) {
                let p = dst.as_mut_ptr();
                // SAFETY: `p` points at storage for the whole product, so the
                // head's field pointer is in bounds and writing through it
                // initialises the head. The tail's field pointer is then handed
                // on as `MaybeUninit` of the tail's own product type, which has
                // that type's layout, and the tail's `splat_in` writes all of it.
                unsafe {
                    core::ptr::addr_of_mut!((*p).head).write([row.head; N]);
                    let tail = &mut *(core::ptr::addr_of_mut!((*p).tail)
                        as *mut core::mem::MaybeUninit<T::Prod<Arr<N>>>);
                    T::splat_in::<N>(tail, &row.tail);
                }
            }
        }
    };
}

#[cfg(feature = "const")]
project_items!(const);

#[cfg(not(feature = "const"))]
project_items!();

/// A product of exclusive columns cut in two, row-wise.
///
/// Apart from [`Project`] and plain on both paths, because it cannot be const:
/// it takes the product by value and hands its fields on, and the const checker
/// on this toolchain reads a by-value parameter of a generic type as dropped at
/// the end of the body whether or not every field was moved out. Lifting that
/// takes `const_precise_live_drops`, which is not a vetted feature. Cutting a
/// column for threads is a runtime act in any case.
///
/// ```
/// use notko_hlist::{Arr, Cons, Elem, Empty, PCons, PNil, Project, Split};
///
/// struct Pos;
/// impl Elem for Pos {
///     type Value = i32;
/// }
/// type Lanes = Cons<Pos, Empty>;
///
/// let mut cols: <Lanes as Project>::Prod<Arr<4>> = PCons {
///     head: [1, 2, 3, 4],
///     tail: PNil,
/// };
/// let (front, back) = Lanes::split_mut(Lanes::cols_mut::<4>(&mut cols), 1);
/// front.head[0] = 10;
/// back.head[0] = 20;
/// assert_eq!(cols.head, [10, 20, 3, 4]);
/// ```
///
/// Closed through `Project`: its supertrait cannot be implemented outside this
/// crate, so neither can this. No `unsafe` rests on it.
pub trait Split: Project {
    /// Every member's column cut at `mid`, the rows before it in the first
    /// product and the rest in the second.
    ///
    /// Panics when `mid` is past the end of a column, as `split_at_mut` does.
    fn split_mut<'a>(
        p: Self::Prod<ColMut<'a>>,
        mid: usize,
    ) -> (Self::Prod<ColMut<'a>>, Self::Prod<ColMut<'a>>);
}

#[diagnostic::do_not_recommend]
impl Split for Empty {
    fn split_mut<'a>(
        _: Self::Prod<ColMut<'a>>,
        _: usize,
    ) -> (Self::Prod<ColMut<'a>>, Self::Prod<ColMut<'a>>) {
        (PNil, PNil)
    }
}

#[diagnostic::do_not_recommend]
impl<H: Elem, T: Split> Split for Cons<H, T> {
    fn split_mut<'a>(
        p: Self::Prod<ColMut<'a>>,
        mid: usize,
    ) -> (Self::Prod<ColMut<'a>>, Self::Prod<ColMut<'a>>) {
        let PCons {
            head,
            tail,
        } = p;
        let (a, b) = head.split_at_mut(mid);
        let (ta, tb) = T::split_mut(tail, mid);
        (
            PCons {
                head: a,
                tail: ta,
            },
            PCons {
                head: b,
                tail: tb,
            },
        )
    }
}
