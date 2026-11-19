//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Project`: a list of members as a product of what a carrier holds for each,
//! and the walks between two carriers' products.
//!
//! [`Cons`] holds nothing, so a consumer wanting one value per member, or one
//! column per member, had to build the matching nest of structs by hand and walk
//! it by hand. `Project<C>` is that nest, derived from the list: `Cons<H, T>`
//! becomes [`PCons`] holding what `C` holds for `H` in front of the tail's
//! product, and `Empty` becomes [`PNil`]. Every operation on it recurses on the
//! list's structure and nothing else.
//!
//! The carrier is a parameter of the trait, so a product exists for a list and
//! a carrier together, and the cell names both, `PCons<C, H, T>`: a product never
//! stands for another list or another carrier, even where the fields coincide.
//!
//! # The walks name their carriers
//!
//! A walk moves between the products of one list through two carriers, a row of
//! values out of a product of columns, columns over a product of arrays. Each is
//! a trait taking its carriers as parameters, [`Rows<C>`] and [`Arrays<N>`], with
//! both products as supertraits. That is what lets the impl for `Cons<H, T>`
//! see the tail's two products, which a method generic over the carrier could
//! not: the impl would have to know the tail has a product through a carrier
//! it cannot name.
//!
//! # Whole-product operations are in place
//!
//! A product over a long list is as deep as the list, and moving one by value
//! recurses with a frame per level holding the rest of it. Deep enough, in a
//! debug build, that overflows a thread's default stack. So the operations on a
//! whole product of arrays take it by reference and write field by field:
//! [`Arrays::copy_from`] copies into an existing product, and [`Arrays::made`]
//! builds one inside a `MaybeUninit` through [`Arrays::splat_in`].
//!
//! # Module layout
//!
//! The walks come in a const twin and a plain twin, and both are written once
//! in `project_items!`, which takes the constness as its argument. See
//! `carrier.rs` for why.

use crate::carrier::{Arr, ByRow, ByRowMut, Carrier, Col, ColMut, Elem, Uniform, Val};
use crate::list::{Cons, Empty, List};
use crate::seal;

/// A cell of a product: what carrier `C` holds for member `H`, then the rest.
pub struct PCons<C: Carrier<H>, H, T> {
    /// What `C` holds for the head member.
    pub head: <C as Carrier<H>>::Of,
    /// The product of the tail.
    pub tail: T,
}

/// The end of a product.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PNil;

impl<C: Carrier<H>, H, T: Default> Default for PCons<C, H, T>
where
    <C as Carrier<H>>::Of: Default,
{
    fn default() -> Self {
        PCons {
            head: Default::default(),
            tail: T::default(),
        }
    }
}

impl<C: Carrier<H>, H, T: Clone> Clone for PCons<C, H, T>
where
    <C as Carrier<H>>::Of: Clone,
{
    fn clone(&self) -> Self {
        PCons {
            head: self.head.clone(),
            tail: self.tail.clone(),
        }
    }
}

impl<C: Carrier<H>, H, T: Copy> Copy for PCons<C, H, T> where <C as Carrier<H>>::Of: Copy {}

impl<C: Carrier<H>, H, T: PartialEq> PartialEq for PCons<C, H, T>
where
    <C as Carrier<H>>::Of: PartialEq,
{
    fn eq(&self, o: &Self) -> bool {
        self.head == o.head && self.tail == o.tail
    }
}

impl<C: Carrier<H>, H, T: Eq> Eq for PCons<C, H, T> where <C as Carrier<H>>::Of: Eq {}

impl<C: Carrier<H>, H, T: core::fmt::Debug> core::fmt::Debug for PCons<C, H, T>
where
    <C as Carrier<H>>::Of: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PCons")
            .field("head", &self.head)
            .field("tail", &self.tail)
            .finish()
    }
}

/// The product of the list `Self` through the carrier `C`: [`PCons`] per cell,
/// [`PNil`] at the end.
///
/// ```
/// use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Project, Val};
///
/// struct Pos;
/// impl Elem for Pos {
///     type Value = i32;
/// }
/// type Lanes = Cons<Pos, Empty>;
///
/// let row: <Lanes as Project<Val>>::Prod = PCons {
///     head: 7,
///     tail: PNil,
/// };
/// assert_eq!(row.head, 7);
/// ```
///
/// A product exists exactly where `C` holds a form for every member, through
/// [`Carrier`]. No impl outside this crate exists or can: the trait has a
/// required method taking a token whose type is unnameable outside this crate,
/// so an impl elsewhere cannot be written whatever its carrier.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no product through `{C}`",
    note = "A product is defined for lists built from `Empty` and `Cons<H, T>` where the carrier holds a form for every member, through `Carrier<H>`, so the usual cause is a member the carrier has no form for, or a tail that is not a list."
)]
pub trait Project<C>: List {
    /// The product through `C`.
    type Prod;

    /// The seal. Never needs calling.
    #[doc(hidden)]
    fn sealed(_: seal::Token);
}

#[diagnostic::do_not_recommend]
impl<C> Project<C> for Empty {
    type Prod = PNil;

    fn sealed(_: seal::Token) {}
}

#[diagnostic::do_not_recommend]
impl<C: Carrier<H>, H, T: Project<C>> Project<C> for Cons<H, T> {
    type Prod = PCons<C, H, T::Prod>;

    fn sealed(_: seal::Token) {}
}

macro_rules! project_items {
    ($($c:ident)?) => {
        /// The rows of a product through the uniform carrier `C`, read and
        /// written as products of values.
        ///
        /// ```
        /// use notko_hlist::{Arr, Cons, Elem, Empty, PCons, PNil, Rows};
        ///
        /// struct Pos;
        /// impl Elem for Pos {
        ///     type Value = i32;
        /// }
        /// type Lanes = Cons<Pos, Empty>;
        ///
        /// let mut cols = PCons::<Arr<3>, Pos, PNil> { head: [1, 2, 3], tail: PNil };
        /// <Lanes as Rows<Arr<3>>>::write_row(&mut cols, 0, &PCons { head: 9, tail: PNil });
        /// assert_eq!(<Lanes as Rows<Arr<3>>>::read_row(&cols, 0).head, 9);
        /// ```
        ///
        /// Sealed the way [`Project`] is.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` has no rows through `{C}`",
            note = "Rows are read and written for lists whose every member implements `Elem`, through a uniform carrier: `Col`, `ColMut` or `Arr<N>` to read, `ColMut` or `Arr<N>` to write."
        )]
        pub $($c)? trait Rows<C: Uniform>: Project<C> + Project<Val> {
            /// The seal. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: seal::Token);

            /// The values at row `i`, one per member.
            fn read_row(p: &<Self as Project<C>>::Prod, i: usize) -> <Self as Project<Val>>::Prod
            where
                C: $([$c])? ByRow;

            /// Writes the values `v` at row `i`, one per member.
            fn write_row(p: &mut <Self as Project<C>>::Prod, i: usize, v: &<Self as Project<Val>>::Prod)
            where
                C: $([$c])? ByRowMut;
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C: Uniform> Rows<C> for Empty {
            fn sealed(_: seal::Token) {}

            fn read_row(_: &PNil, _: usize) -> PNil
            where
                C: $([$c])? ByRow,
            {
                PNil
            }

            fn write_row(_: &mut PNil, _: usize, _: &PNil)
            where
                C: $([$c])? ByRowMut,
            {
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<C: Uniform, H: Elem, T: $([$c])? Rows<C>> Rows<C> for Cons<H, T> {
            fn sealed(_: seal::Token) {}

            fn read_row(p: &<Self as Project<C>>::Prod, i: usize) -> <Self as Project<Val>>::Prod
            where
                C: $([$c])? ByRow,
            {
                PCons {
                    head: C::at::<H>(&p.head, i),
                    tail: T::read_row(&p.tail, i),
                }
            }

            fn write_row(p: &mut <Self as Project<C>>::Prod, i: usize, v: &<Self as Project<Val>>::Prod)
            where
                C: $([$c])? ByRowMut,
            {
                C::set::<H>(&mut p.head, i, v.head);
                T::write_row(&mut p.tail, i, &v.tail);
            }
        }

        /// A product of owned arrays of `N`: seen as columns, copied, and built
        /// from one row.
        ///
        /// ```
        /// use notko_hlist::{Arr, Arrays, Cons, Elem, Empty, PCons, PNil, Project, Rows, Val};
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
        /// let row: <Lanes as Project<Val>>::Prod = PCons { head: 7, tail: PCons { head: 3, tail: PNil } };
        /// let mut cols: <Lanes as Project<Arr<4>>>::Prod = <Lanes as Arrays<4>>::made(&row);
        ///
        /// <Lanes as Rows<Arr<4>>>::write_row(&mut cols, 2, &PCons { head: -1, tail: PCons { head: 9, tail: PNil } });
        /// assert_eq!(cols.head, [7, 7, -1, 7]);
        /// assert_eq!(<Lanes as Arrays<4>>::cols(&cols).tail.head[2], 9);
        /// ```
        ///
        /// Sealed the way [`Project`] is, and [`Arrays::made`] rests its `unsafe`
        /// on that: the two impls below are every impl there is.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` has no product of arrays of `{N}`",
            note = "A product of arrays is defined for lists whose every member implements `Elem`."
        )]
        pub $($c)? trait Arrays<const N: usize>:
            Project<Arr<N>> + Project<Val> + for<'a> Project<Col<'a>> + for<'a> Project<ColMut<'a>>
        {
            /// The seal. Never needs calling.
            #[doc(hidden)]
            fn sealed(_: seal::Token);

            /// A shared column over each member's array.
            fn cols(p: &<Self as Project<Arr<N>>>::Prod) -> <Self as Project<Col<'_>>>::Prod;

            /// An exclusive column over each member's array.
            fn cols_mut(p: &mut <Self as Project<Arr<N>>>::Prod) -> <Self as Project<ColMut<'_>>>::Prod;

            /// Copies `src` into `dst`, field by field.
            ///
            /// In place rather than by value: see the module docs on depth.
            fn copy_from(dst: &mut <Self as Project<Arr<N>>>::Prod, src: &<Self as Project<Arr<N>>>::Prod);

            /// Writes every field of `dst`, each member's array filled with that
            /// member's value from `row`.
            ///
            /// Every field, the head and then the tail's, down to `PNil`. That is
            /// the promise [`Arrays::made`] reads `dst` as initialised on, and
            /// the two impls in this crate are the only ones that make it.
            fn splat_in(
                dst: &mut core::mem::MaybeUninit<<Self as Project<Arr<N>>>::Prod>,
                row: &<Self as Project<Val>>::Prod,
            );

            /// A product of arrays with every row equal to `row`, built in place.
            fn made(row: &<Self as Project<Val>>::Prod) -> <Self as Project<Arr<N>>>::Prod {
                let mut p = core::mem::MaybeUninit::uninit();
                Self::splat_in(&mut p, row);
                // SAFETY: every impl of `Arrays` is one of the two below, since
                // an impl must supply `sealed`, whose `seal::Token` argument no
                // other crate can name. Both `splat_in`s write every field of the
                // product, the head and then the tail through the tail's own
                // `splat_in`, down to `PNil`, so `p` is initialised here.
                unsafe { p.assume_init() }
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<const N: usize> Arrays<N> for Empty {
            fn sealed(_: seal::Token) {}

            fn cols(_: &PNil) -> PNil {
                PNil
            }

            fn cols_mut(_: &mut PNil) -> PNil {
                PNil
            }

            fn copy_from(_: &mut PNil, _: &PNil) {}

            fn splat_in(dst: &mut core::mem::MaybeUninit<PNil>, _: &PNil) {
                dst.write(PNil);
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<const N: usize, H: Elem, T: $([$c])? Arrays<N>> Arrays<N> for Cons<H, T> {
            fn sealed(_: seal::Token) {}

            fn cols(p: &<Self as Project<Arr<N>>>::Prod) -> <Self as Project<Col<'_>>>::Prod {
                PCons {
                    head: &p.head,
                    tail: T::cols(&p.tail),
                }
            }

            fn cols_mut(p: &mut <Self as Project<Arr<N>>>::Prod) -> <Self as Project<ColMut<'_>>>::Prod {
                PCons {
                    head: &mut p.head,
                    tail: T::cols_mut(&mut p.tail),
                }
            }

            fn copy_from(dst: &mut <Self as Project<Arr<N>>>::Prod, src: &<Self as Project<Arr<N>>>::Prod) {
                dst.head = src.head;
                T::copy_from(&mut dst.tail, &src.tail);
            }

            fn splat_in(
                dst: &mut core::mem::MaybeUninit<<Self as Project<Arr<N>>>::Prod>,
                row: &<Self as Project<Val>>::Prod,
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
                        as *mut core::mem::MaybeUninit<<T as Project<Arr<N>>>::Prod>);
                    T::splat_in(tail, &row.tail);
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
/// Apart from the walks and plain on both paths, because it cannot be const:
/// it takes the product by value and hands its fields on, and the const checker
/// on this toolchain reads a by-value parameter of a generic type as dropped at
/// the end of the body whether or not every field was moved out. Lifting that
/// takes `const_precise_live_drops`, which is not a vetted feature. Cutting a
/// column for threads is a runtime act in any case.
///
/// ```
/// use notko_hlist::{
///     Arr,
///     Arrays,
///     Cons,
///     Elem,
///     Empty,
///     PCons,
///     PNil,
///     Project,
///     Split,
/// };
///
/// struct Pos;
/// impl Elem for Pos {
///     type Value = i32;
/// }
/// type Lanes = Cons<Pos, Empty>;
///
/// let mut cols: <Lanes as Project<Arr<4>>>::Prod = PCons {
///     head: [1, 2, 3, 4],
///     tail: PNil,
/// };
/// let (front, back) = <Lanes as Split>::split_mut(
///     <Lanes as Arrays<4>>::cols_mut(&mut cols),
///     1,
/// );
/// front.head[0] = 10;
/// back.head[0] = 20;
/// assert_eq!(cols.head, [10, 20, 3, 4]);
/// ```
///
/// Sealed the way [`Project`] is. No `unsafe` rests on it.
pub trait Split: for<'a> Project<ColMut<'a>> {
    /// The seal. Never needs calling.
    #[doc(hidden)]
    fn sealed(_: seal::Token);

    /// Every member's column cut at `mid`, the rows before it in the first
    /// product and the rest in the second.
    ///
    /// Panics when `mid` is past the end of a column, as `split_at_mut` does.
    fn split_mut<'a>(
        p: <Self as Project<ColMut<'a>>>::Prod,
        mid: usize,
    ) -> (
        <Self as Project<ColMut<'a>>>::Prod,
        <Self as Project<ColMut<'a>>>::Prod,
    );
}

#[diagnostic::do_not_recommend]
impl Split for Empty {
    fn sealed(_: seal::Token) {}

    fn split_mut<'a>(
        _: <Self as Project<ColMut<'a>>>::Prod,
        _: usize,
    ) -> (
        <Self as Project<ColMut<'a>>>::Prod,
        <Self as Project<ColMut<'a>>>::Prod,
    ) {
        (PNil, PNil)
    }
}

#[diagnostic::do_not_recommend]
impl<H: Elem, T: Split> Split for Cons<H, T> {
    fn sealed(_: seal::Token) {}

    fn split_mut<'a>(
        p: <Self as Project<ColMut<'a>>>::Prod,
        mid: usize,
    ) -> (
        <Self as Project<ColMut<'a>>>::Prod,
        <Self as Project<ColMut<'a>>>::Prod,
    ) {
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
