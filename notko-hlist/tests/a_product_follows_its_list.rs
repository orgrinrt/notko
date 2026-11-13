//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `PlaceOf`, `Project`, `Split`, `Has`, `Vals` and `SubOf`, on both paths.
//!
//! Ungated, since every item here is in every configuration: the const path
//! changes whether the calls may run in a constant, which
//! `a_product_is_built_at_compile_time` pins, and not what they answer. The
//! refusals are in `tests/compile_fail/`, each beside its twin in
//! `tests/compile_pass/`.

use core::marker::PhantomData;

use notko_hlist::{
    Arr,
    At,
    Col,
    ColMut,
    Cons,
    Elem,
    Empty,
    Has,
    Here,
    PCons,
    PNil,
    Place,
    PlaceOf,
    Project,
    Split,
    SubOf,
    There,
    Val,
    Vals,
};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Heat;
impl Elem for Heat {
    type Value = u8;
}

struct Lag;
impl Elem for Lag {
    type Value = i64;
}

type Lanes = Cons<Pos, Cons<Heat, Cons<Lag, Empty>>>;

fn row(pos: i32, heat: u8, lag: i64) -> <Lanes as Project>::Prod<Val> {
    PCons {
        head: pos,
        tail: PCons {
            head: heat,
            tail: PCons {
                head: lag,
                tail: PNil,
            },
        },
    }
}

/// Answers the place `PlaceOf` infers for `M` in `L`, as a type a caller compares
/// by annotation: a wrong place is a build that does not succeed.
fn place_of<L: PlaceOf<M, P>, M, P: Place>() -> PhantomData<P> {
    // The list is named in the body as well as in the bound, since a parameter
    // appearing only in a bound reads to a linter as one nobody uses, and here
    // the bound is what is under test.
    let _ = PhantomData::<L>;
    PhantomData
}

// The place read back as a count is pinned beside the other counts, in
// `a_count_counts` and `a_count_counts_without_const`, since a `Cardinal` impl
// is spelled differently on the two paths and one file cannot hold both.

#[test]
fn a_member_is_found_at_its_place() {
    let _: PhantomData<Here> = place_of::<Lanes, Pos, _>();
    let _: PhantomData<There<Here>> = place_of::<Lanes, Heat, _>();
    let _: PhantomData<There<There<Here>>> = place_of::<Lanes, Lag, _>();
}

/// `PlaceOf` carries `At` at the same place, so a bound on the one reads the other.
fn member_at_the_place_is<L: PlaceOf<M, P>, M, P: Place>() -> PhantomData<<L as At<P>>::Member> {
    PhantomData
}

#[test]
fn the_place_found_names_the_member_back() {
    let _: PhantomData<Heat> = member_at_the_place_is::<Lanes, Heat, _>();
}

#[test]
fn a_member_is_found_whatever_else_the_list_holds() {
    // Not a member of interest at the head, twice over, and the place still
    // comes out where the member is.
    type Padded = Cons<Pos, Cons<Pos, Cons<Heat, Empty>>>;
    let _: PhantomData<There<There<Here>>> = place_of::<Padded, Heat, _>();
}

#[test]
fn a_row_written_reads_back_and_leaves_the_others() {
    let mut cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row(1, 2, 3));
    Lanes::write_row::<Arr<4>>(&mut cols, 2, &row(-5, 9, 40));
    assert_eq!(Lanes::read_row::<Arr<4>>(&cols, 2), row(-5, 9, 40));
    for i in [0, 1, 3] {
        assert_eq!(Lanes::read_row::<Arr<4>>(&cols, i), row(1, 2, 3));
    }
}

#[test]
fn made_fills_every_row_of_every_member() {
    let cols: <Lanes as Project>::Prod<Arr<3>> = Lanes::made::<3>(&row(7, 8, 9));
    assert_eq!(cols.head, [7; 3]);
    assert_eq!(cols.tail.head, [8; 3]);
    assert_eq!(cols.tail.tail.head, [9; 3]);
}

#[test]
fn columns_read_and_write_the_arrays_under_them() {
    let mut cols: <Lanes as Project>::Prod<Arr<3>> = Lanes::made::<3>(&row(0, 0, 0));
    {
        let mut excl = Lanes::cols_mut::<3>(&mut cols);
        Lanes::write_row::<ColMut<'_>>(&mut excl, 1, &row(4, 5, 6));
        assert_eq!(Lanes::read_row::<ColMut<'_>>(&excl, 1), row(4, 5, 6));
    }
    let shared = Lanes::cols::<3>(&cols);
    assert_eq!(Lanes::read_row::<Col<'_>>(&shared, 1), row(4, 5, 6));
    assert_eq!(Lanes::read_row::<Col<'_>>(&shared, 0), row(0, 0, 0));
}

#[test]
fn copy_from_copies_every_member() {
    let src: <Lanes as Project>::Prod<Arr<2>> = Lanes::made::<2>(&row(3, 4, 5));
    let mut dst: <Lanes as Project>::Prod<Arr<2>> = Lanes::made::<2>(&row(0, 0, 0));
    Lanes::copy_from::<2>(&mut dst, &src);
    assert_eq!(dst, src);
}

#[test]
fn a_split_cuts_every_column_at_the_same_row() {
    let mut cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row(0, 0, 0));
    let (mut front, mut back) = Lanes::split_mut(Lanes::cols_mut::<4>(&mut cols), 1);
    assert_eq!((front.head.len(), back.tail.tail.head.len()), (1, 3));
    Lanes::write_row::<ColMut<'_>>(&mut front, 0, &row(1, 1, 1));
    Lanes::write_row::<ColMut<'_>>(&mut back, 0, &row(2, 2, 2));
    assert_eq!(Lanes::read_row::<Arr<4>>(&cols, 0), row(1, 1, 1));
    assert_eq!(Lanes::read_row::<Arr<4>>(&cols, 1), row(2, 2, 2));
    assert_eq!(Lanes::read_row::<Arr<4>>(&cols, 2), row(0, 0, 0));
}

// The contract on a row past the end is the slice's: a panic, the same as
// indexing or `split_at_mut` past the end, and never a read of another row.
#[test]
#[should_panic]
fn a_row_past_the_end_panics_rather_than_reading() {
    let cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row(0, 0, 0));
    let _ = Lanes::read_row::<Arr<4>>(&cols, 4);
}

#[test]
#[should_panic]
fn a_split_past_the_end_panics_rather_than_cutting() {
    let mut cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row(0, 0, 0));
    let _ = Lanes::split_mut(Lanes::cols_mut::<4>(&mut cols), 5);
}

#[test]
fn a_split_at_the_end_leaves_the_back_empty() {
    let mut cols: <Lanes as Project>::Prod<Arr<4>> = Lanes::made::<4>(&row(0, 0, 0));
    let (front, back) = Lanes::split_mut(Lanes::cols_mut::<4>(&mut cols), 4);
    assert_eq!((front.head.len(), back.tail.tail.head.len()), (4, 0));
}

fn heat<L: Has<Heat, P>, P: Place>(cols: &mut L::Prod<Arr<2>>) -> &mut [u8; 2] {
    L::pick_mut::<Arr<2>>(cols)
}

#[test]
fn pick_reaches_the_member_s_field_through_any_carrier() {
    let mut cols: <Lanes as Project>::Prod<Arr<2>> = Lanes::made::<2>(&row(1, 2, 3));
    heat::<Lanes, _>(&mut cols)[1] = 77;
    assert_eq!(cols.tail.head, [2, 77]);
    let shared = Lanes::cols::<2>(&cols);
    assert_eq!(
        *<Lanes as Has<Lag, _>>::pick::<Col<'_>>(&shared),
        &[3, 3][..]
    );
    assert_eq!(*<Lanes as Has<Pos, _>>::pick::<Val>(&row(9, 0, 0)), 9);
}

#[test]
fn vals_are_read_and_written_by_member() {
    let mut v = Vals::<Lanes>(row(1, 2, 3));
    v.set(Lag, -8);
    assert_eq!((v.get(Pos), v.get(Heat), v.get(Lag)), (1, 2, -8));
}

type Narrow = Cons<Lag, Cons<Pos, Empty>>;

#[test]
fn a_part_comes_out_in_its_own_order() {
    let part = <Narrow as SubOf<Lanes, _>>::sub::<Val>(&row(1, 2, 3));
    assert_eq!((part.head, part.tail.head), (3, 1));

    let cols: <Lanes as Project>::Prod<Arr<2>> = Lanes::made::<2>(&row(4, 5, 6));
    let shared = Lanes::cols::<2>(&cols);
    let part = <Narrow as SubOf<Lanes, _>>::sub::<Col<'_>>(&shared);
    assert_eq!((part.head, part.tail.head), (&[6, 6][..], &[4, 4][..]));
}

#[test]
fn a_part_is_written_back_at_its_members() {
    let mut cols: <Lanes as Project>::Prod<Arr<2>> = Lanes::made::<2>(&row(0, 0, 0));
    let part: <Narrow as Project>::Prod<Val> = PCons {
        head: 30,
        tail: PCons {
            head: 10,
            tail: PNil,
        },
    };
    <Narrow as SubOf<Lanes, _>>::put_row::<Arr<2>>(&mut cols, 1, &part);
    assert_eq!(Lanes::read_row::<Arr<2>>(&cols, 1), row(10, 0, 30));
    assert_eq!(Lanes::read_row::<Arr<2>>(&cols, 0), row(0, 0, 0));

    let mut vals = row(0, 7, 0);
    <Narrow as SubOf<Lanes, _>>::put_vals(&mut vals, &part);
    assert_eq!(vals, row(10, 7, 30));
}

#[test]
fn the_empty_list_is_a_part_of_every_list() {
    let _: PNil = <Empty as SubOf<Lanes, _>>::sub::<Val>(&row(1, 2, 3));
}

/// A member per index, so a long list is written as a count rather than as
/// sixty-four structs.
struct Lane<const K: usize>;
impl<const K: usize> Elem for Lane<K> {
    type Value = u32;
}

macro_rules! lanes {
    () => { Empty };
    ($k:literal $($rest:literal)*) => { Cons<Lane<$k>, lanes!($($rest)*)> };
}

type Deep = lanes!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

#[test]
fn a_deep_product_is_built_and_copied_on_a_default_stack() {
    // Sixty-four members of 256 rows is 64 KiB of product. Built or copied by
    // value through the recursion, each level holds the rest of it in its
    // frame, which is what the in-place operations are for. The thread is given
    // the default stack explicitly, so the run does not lean on whatever the
    // harness was started with.
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let zero: <Deep as Project>::Prod<Val> = Default::default();
            let mut row = zero;
            *<Deep as Has<Lane<63>, _>>::pick_mut::<Val>(&mut row) = 1;
            *<Deep as Has<Lane<40>, _>>::pick_mut::<Val>(&mut row) = 2;

            let src: <Deep as Project>::Prod<Arr<256>> = Deep::made::<256>(&row);
            let mut dst: <Deep as Project>::Prod<Arr<256>> = Deep::made::<256>(&zero);
            Deep::copy_from::<256>(&mut dst, &src);

            assert_eq!(
                *<Deep as Has<Lane<63>, _>>::pick::<Arr<256>>(&dst),
                [1; 256]
            );
            assert_eq!(
                *<Deep as Has<Lane<40>, _>>::pick::<Arr<256>>(&dst),
                [2; 256]
            );
            assert_eq!(*<Deep as Has<Lane<0>, _>>::pick::<Arr<256>>(&dst), [0; 256]);
            assert_eq!(Deep::read_row::<Arr<256>>(&dst, 255), row);
        })
        .expect("a thread")
        .join()
        .expect("the deep product fit the default stack");
}
