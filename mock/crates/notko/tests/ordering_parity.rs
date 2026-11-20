//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The derived orderings run the same way the `core` types they stand in for do.
//!
//! A derived `Ord` on an enum orders by variant declaration, so the order these
//! types are written in is a public contract the same way their names are. It
//! is also the one part of the contract that no signature states and no
//! consumer sees until something sorts.
//!
//! `Maybe` was declared the other way round and sorted the other way round from
//! `Option` for it. Sorting a mixed list put the absent values at the opposite
//! end from where anyone coming off `Option` would look for them, and nothing
//! said so.

use core::cmp::Ordering;
use core::num::NonZeroU8;

use notko::{Maybe, MaybeNull, Outcome};

#[test]
fn absent_sorts_below_present_the_way_none_sorts_below_some() {
    assert_eq!(Maybe::Isnt.cmp(&Maybe::Is(0u8)), Ordering::Less);
    assert_eq!(None::<u8>.cmp(&Some(0u8)), Ordering::Less);

    assert_eq!(Maybe::Is(0u8).cmp(&Maybe::Isnt), Ordering::Greater);
    assert_eq!(Some(0u8).cmp(&None::<u8>), Ordering::Greater);
}

#[test]
fn a_mixed_sort_lands_in_the_same_places_as_the_core_one() {
    // The comparison a consumer actually performs, rather than one pair of
    // variants. A payload big enough to sort among itself, so this fails if the
    // variant order is right and the payload order is not.
    let mut ours = [Maybe::Is(2u8), Maybe::Isnt, Maybe::Is(1), Maybe::Isnt];
    let mut theirs = [Some(2u8), None, Some(1), None];
    ours.sort();
    theirs.sort();

    let ours_as_core: Vec<Option<u8>> = ours
        .iter()
        .map(|m| {
            match m {
                Maybe::Is(v) => Some(*v),
                Maybe::Isnt => None,
            }
        })
        .collect();
    assert_eq!(ours_as_core, theirs);
}

#[test]
fn ok_sorts_below_err_the_way_it_does_on_result() {
    // The control on the two above. `Outcome` is declared in the order `Result`
    // uses and must stay there, so a later flip of one type does not quietly
    // take the other with it.
    assert_eq!(
        Outcome::<u8, u8>::Ok(0).cmp(&Outcome::Err(0)),
        Ordering::Less
    );
    assert_eq!(Ok::<u8, u8>(0).cmp(&Err(0)), Ordering::Less);
}

#[test]
fn the_payload_still_decides_within_a_variant() {
    // Variant order is the outer key and the payload the inner one, which is
    // what makes the mixed sort above a real test rather than a partition.
    assert_eq!(Maybe::Is(1u8).cmp(&Maybe::Is(2)), Ordering::Less);
    assert_eq!(Maybe::Isnt::<u8>.cmp(&Maybe::Isnt), Ordering::Equal);
}

#[test]
fn maybe_null_carries_the_same_order_it_wraps() {
    // `MaybeNull<T>` is a transparent newtype over `Maybe<T>` and derives its
    // ordering, so it moved when `Maybe` moved. It is public, it is the type
    // an FFI boundary reaches for, and nothing here said what its order was.
    let null = MaybeNull::<NonZeroU8>::null();
    let one = MaybeNull::new(NonZeroU8::new(1).unwrap());
    let two = MaybeNull::new(NonZeroU8::new(2).unwrap());

    assert_eq!(null.cmp(&one), Ordering::Less, "null sorts below a value");
    assert_eq!(one.cmp(&two), Ordering::Less, "the payload decides within");
    assert_eq!(null.cmp(&null), Ordering::Equal);

    // And against the `core` shape it stands in for, which is the whole claim.
    let ours: Vec<Option<u8>> = {
        let mut v = vec![two, null, one, null];
        v.sort();
        v.into_iter()
            .map(|m| m.into_maybe().map(NonZeroU8::get).into())
            .collect()
    };
    let theirs = {
        let mut v = vec![Some(2u8), None, Some(1), None];
        v.sort();
        v
    };
    assert_eq!(ours, theirs);
}
