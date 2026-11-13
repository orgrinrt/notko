//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Elem` and `Carrier`: what a product of a list holds for each member.
//!
//! A list of types says nothing about values, and a consumer holding values for
//! one wants them in more than one shape: one value per member for a row, a
//! slice per member for a column, an array per member for owned storage. The
//! shape is the carrier, a type-level function from a member to what is held
//! for it, and [`Project`](crate::Project) builds the product for a whole list
//! through whichever carrier is asked for.
//!
//! A member is a marker that names its value type, [`Elem`], rather than the
//! value type itself. Two members may carry the same value type and still be two
//! members, which a list of bare value types could not say.
//!
//! # Module layout
//!
//! The capabilities [`ByRow`], [`ByRowMut`] and [`Dup`] come in a const twin
//! and a plain twin like `Cardinal` does, and here both are written once, in
//! `carrier_items!`, which takes the constness as its one argument. The
//! declarations and the bodies are the same on both paths, so a second file
//! holding the same text with `const` struck out would only be a place for the
//! two to drift apart.

/// A member of a list, naming the one value type it carries.
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
/// the carriers here wants that.
#[diagnostic::on_unimplemented(
    message = "`{Self}` names no value type",
    note = "A member of a product is a marker implementing `Elem`, whose `Value` is what the product carries for it. A bare value type is not a member: wrap it in a marker of its own, so that two members carrying the same type stay two members."
)]
pub trait Elem: 'static {
    /// The value carried for this member.
    type Value: Copy + 'static;
}

/// What a product holds per member: the value, a column, an array.
///
/// `Of<E>` is the type held for member `E`. The four carriers in this crate are
/// [`Val`], [`Col`], [`ColMut`] and [`Arr`], and a consumer may add its own,
/// since nothing in [`Project`](crate::Project) rests on which carrier it was
/// handed beyond the capabilities the carrier implements.
pub trait Carrier {
    /// What is held for member `E`.
    type Of<E: Elem>;
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
impl Carrier for Val {
    type Of<E: Elem> = E::Value;
}

#[diagnostic::do_not_recommend]
impl<'a> Carrier for Col<'a> {
    type Of<E: Elem> = &'a [E::Value];
}

#[diagnostic::do_not_recommend]
impl<'a> Carrier for ColMut<'a> {
    type Of<E: Elem> = &'a mut [E::Value];
}

#[diagnostic::do_not_recommend]
impl<const N: usize> Carrier for Arr<N> {
    type Of<E: Elem> = [E::Value; N];
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
        pub $($c)? trait ByRow: Carrier {
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

        /// A carrier whose per-member item can be duplicated.
        ///
        /// A value is copied and a shared column is borrowed again, so a
        /// product over either can be taken apart into a smaller one while the
        /// original stays usable. An exclusive column cannot be, and has no impl.
        #[diagnostic::on_unimplemented(
            message = "`{Self}` cannot be duplicated",
            note = "The carriers duplicated are `Val` and `Col`. A `ColMut` duplicated would be two exclusive borrows of one column, and an `Arr<N>` would be a copy of the whole storage, so a part of either is cut through `Project::cols` first."
        )]
        pub $($c)? trait Dup: Carrier {
            /// Another of what `c` is, for the same member.
            fn dup<E: Elem>(c: &Self::Of<E>) -> Self::Of<E>;
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

        #[diagnostic::do_not_recommend]
        $($c)? impl Dup for Val {
            fn dup<E: Elem>(c: &Self::Of<E>) -> Self::Of<E> {
                *c
            }
        }

        #[diagnostic::do_not_recommend]
        $($c)? impl<'a> Dup for Col<'a> {
            fn dup<E: Elem>(c: &Self::Of<E>) -> Self::Of<E> {
                c
            }
        }
    };
}

#[cfg(feature = "const")]
carrier_items!(const);

#[cfg(not(feature = "const"))]
carrier_items!();
