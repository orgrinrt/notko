//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! A per-member carrier owned by another crate, with the members owned here.
//!
//! The carrier is the fixture crate's `Read<'f>` and every member is this
//! crate's, which is how a scheduler's read carrier meets a consumer's
//! members. Coherence sees the two from different crates, so an impl that only
//! builds when carrier and member share a crate fails here and nowhere in
//! `a_carrier_holds_a_form_per_member`.

use carrier_owner::Read;
use notko_hlist::{Cons, DupForm, Elem, Empty, Form, Has, PCons, PNil, Project, SubOf};

/// An `Elem` member, which is what a blanket over uniform carriers could
/// collide with.
struct Pos;
impl Elem for Pos {
    type Value = i32;
}

/// A member that is not an `Elem`.
struct Store;

struct Seq(u32);

impl<'f> Form<Pos> for Read<'f> {
    type Of = i32;
}
impl<'f> Form<Store> for Read<'f> {
    type Of = &'f Seq;
}
impl<'f> DupForm<Pos> for Read<'f> {
    fn dup_form(c: &i32) -> i32 {
        *c
    }
}
impl<'f> DupForm<Store> for Read<'f> {
    fn dup_form(c: &&'f Seq) -> &'f Seq {
        c
    }
}

type Reads = Cons<Pos, Cons<Store, Empty>>;

#[test]
fn each_member_is_read_and_cut_through_the_other_crate_s_carrier() {
    let seq = Seq(5);
    let p: <Reads as Project<Read<'_>>>::Prod = PCons {
        head: 3,
        tail: PCons {
            head: &seq,
            tail: PNil,
        },
    };
    assert_eq!(*<Reads as Has<Read<'_>, Pos, _>>::pick(&p), 3);

    let part = <Cons<Store, Cons<Pos, Empty>> as SubOf<Read<'_>, Reads, _>>::sub(&p);
    assert!(core::ptr::eq(part.head, &seq));
    assert_eq!(part.head.0, 5);
    assert_eq!(part.tail.head, 3);
}
