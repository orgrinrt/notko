//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! A carrier of the consumer's own, stating a form per member for one frame.
//!
//! This is the read list a scheduler hands a unit: one member read by value, one
//! as a window on a column, one as a borrow of a resource that is not `Copy`.
//! The list names no lifetime; the frame is on the carrier, and the product
//! ends with it. The refusals that keep each member's form fixed are under
//! `tests/compile_fail/`.

use core::marker::PhantomData;

use notko_hlist::{
    Cons,
    Dup,
    Elem,
    Empty,
    Form,
    Has,
    PCons,
    PNil,
    PerMember,
    Place,
    Project,
    Shape,
    SubOf,
};

/// A resource holding a collection, so it is not `Copy`.
struct Seq {
    items: [u32; 4],
    len:   usize,
}

/// A view of a column around one record.
#[derive(Clone, Copy)]
struct Window<'a> {
    column: &'a [i32],
    centre: usize,
}

/// Read by value, and an `Elem`, so the crate's own carriers hold it too.
struct Pos;
impl Elem for Pos {
    type Value = i32;
}

/// Read as a window. Not an `Elem`.
struct Near;

/// Read by reference. Not an `Elem`, and its value is not `Copy`.
struct Store;

/// The read carrier for the frame `'f`.
struct Read<'f>(PhantomData<&'f ()>);
impl<'f> Shape for Read<'f> {
    type Is = PerMember;
}

impl<'f> Form<Pos> for Read<'f> {
    type Of = i32;
}
impl<'f> Form<Near> for Read<'f> {
    type Of = Window<'f>;
}
impl<'f> Form<Store> for Read<'f> {
    type Of = &'f Seq;
}

impl<'f> Dup<Pos> for Read<'f> {
    fn dup(c: &i32) -> i32 {
        *c
    }
}
impl<'f> Dup<Near> for Read<'f> {
    fn dup(c: &Window<'f>) -> Window<'f> {
        *c
    }
}
impl<'f> Dup<Store> for Read<'f> {
    fn dup(c: &&'f Seq) -> &'f Seq {
        c
    }
}

type Reads = Cons<Pos, Cons<Near, Cons<Store, Empty>>>;
type ResourceOnly = Cons<Store, Empty>;

fn store_of<'f, L: Has<Read<'f>, Store, P>, P: Place>(
    p: &<L as Project<Read<'f>>>::Prod,
) -> &'f Seq {
    L::pick(p)
}

fn read<'f>(column: &'f [i32; 4], seq: &'f Seq) -> <Reads as Project<Read<'f>>>::Prod {
    PCons {
        head: 9,
        tail: PCons {
            head: Window {
                column,
                centre: 2,
            },
            tail: PCons {
                head: seq,
                tail: PNil,
            },
        },
    }
}

#[test]
fn each_member_is_read_in_its_own_form() {
    let column = [4, 5, 6, 7];
    let seq = Seq {
        items: [5, 6, 7, 0],
        len:   3,
    };
    let p = read(&column, &seq);

    assert_eq!(*<Reads as Has<Read<'_>, Pos, _>>::pick(&p), 9);
    let w = <Reads as Has<Read<'_>, Near, _>>::pick(&p);
    assert_eq!(w.column[w.centre], 6);
    let s = store_of::<Reads, _>(&p);
    assert_eq!(&s.items[.. s.len], &[5, 6, 7]);
}

#[test]
fn a_borrowed_field_is_the_borrow_and_not_a_copy() {
    let column = [4, 5, 6, 7];
    let seq = Seq {
        items: [5, 6, 7, 0],
        len:   3,
    };
    let p = read(&column, &seq);

    assert!(core::ptr::eq(store_of::<Reads, _>(&p), &seq));
    let w = <Reads as Has<Read<'_>, Near, _>>::pick(&p);
    assert!(core::ptr::eq(w.column, &column[..]));
}

#[test]
fn a_narrower_read_list_is_cut_out_of_a_wider_one() {
    let column = [4, 5, 6, 7];
    let seq = Seq {
        items: [5, 6, 7, 0],
        len:   3,
    };
    let p = read(&column, &seq);

    let part = <ResourceOnly as SubOf<Read<'_>, Reads, _>>::sub(&p);
    assert!(core::ptr::eq(part.head, &seq));
    // The wider product is still there to read after the cut.
    assert_eq!(*<Reads as Has<Read<'_>, Pos, _>>::pick(&p), 9);
}
