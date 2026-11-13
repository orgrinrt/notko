//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The const path of `Project`, `Has`, `Vals` and `SubOf`: every call here
//! sits in a constant, so the build is the check that each one is
//! const-callable, and the assertions check that the compiler answered what the
//! runtime path answers in `a_product_follows_its_list`.

#![feature(const_trait_impl)]

use notko_hlist::{Arr, Cons, Elem, Empty, Has, PCons, PNil, Project, SubOf, Val, Vals};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Heat;
impl Elem for Heat {
    type Value = u8;
}

type Lanes = Cons<Pos, Cons<Heat, Empty>>;
type Back = Cons<Heat, Cons<Pos, Empty>>;

const ROW: <Lanes as Project>::Prod<Val> = PCons {
    head: 7,
    tail: PCons {
        head: 3,
        tail: PNil,
    },
};

/// Storage made, written at one row, copied and read back, all in a constant.
const COLS: <Lanes as Project>::Prod<Arr<3>> = {
    let mut cols = Lanes::made::<3>(&ROW);
    Lanes::write_row::<Arr<3>>(&mut cols, 1, &PCons {
        head: -1,
        tail: PCons {
            head: 9,
            tail: PNil,
        },
    });
    let mut copy = Lanes::made::<3>(&ROW);
    Lanes::copy_from::<3>(&mut copy, &cols);
    copy
};

const READ: <Lanes as Project>::Prod<Val> = Lanes::read_row::<Arr<3>>(&COLS, 1);

const HEAT: [u8; 3] = *<Lanes as Has<Heat, _>>::pick::<Arr<3>>(&COLS);

const BACK: <Back as Project>::Prod<Val> = <Back as SubOf<Lanes, _>>::sub::<Val>(&ROW);

const PUT: <Lanes as Project>::Prod<Val> = {
    let mut v = ROW;
    <Back as SubOf<Lanes, _>>::put_vals(&mut v, &PCons {
        head: 1,
        tail: PCons {
            head: 2,
            tail: PNil,
        },
    });
    v
};

const BY_MEMBER: (i32, u8) = {
    let mut v = Vals::<Lanes>(ROW);
    v.set(Heat, 5);
    (v.get(Pos), v.get(Heat))
};

#[test]
fn storage_is_made_and_written_in_a_constant() {
    assert_eq!(COLS.head, [7, -1, 7]);
    assert_eq!(COLS.tail.head, [3, 9, 3]);
}

#[test]
fn a_row_is_read_in_a_constant() {
    assert_eq!((READ.head, READ.tail.head), (-1, 9));
}

#[test]
fn a_member_s_field_is_picked_in_a_constant() {
    assert_eq!(HEAT, [3, 9, 3]);
}

#[test]
fn a_part_is_cut_and_written_back_in_a_constant() {
    assert_eq!((BACK.head, BACK.tail.head), (3, 7));
    assert_eq!((PUT.head, PUT.tail.head), (2, 1));
}

#[test]
fn values_are_read_and_written_by_member_in_a_constant() {
    assert_eq!(BY_MEMBER, (7, 5));
}
