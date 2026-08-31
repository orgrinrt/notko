//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Layout assertions for `Slot<T>`, from outside the crate.
//!
//! The const assertions inside `slot.rs` are generated from one list, so they
//! cannot disagree with the trait impls. What they cannot do is notice the
//! list getting shorter: drop a type from it and every remaining assertion
//! still passes, over eleven twelfths of the surface.
//!
//! This file names all twelve by hand, which is the point. It is the external
//! copy the generated ones are checked against, and a type leaving the list
//! takes this file's build with it rather than going quiet.

use core::mem::{align_of, size_of};
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use notko::Slot;

#[test]
fn slot_nonzero_usize_has_pointer_layout() {
    assert_eq!(size_of::<Slot<NonZeroUsize>>(), size_of::<usize>());
    assert_eq!(align_of::<Slot<NonZeroUsize>>(), align_of::<usize>());
}

#[test]
fn slot_nonzero_isize_has_pointer_layout() {
    assert_eq!(size_of::<Slot<NonZeroIsize>>(), size_of::<isize>());
    assert_eq!(align_of::<Slot<NonZeroIsize>>(), align_of::<isize>());
}

#[test]
fn slot_unsigned_nonzero_have_integer_layout() {
    assert_eq!(size_of::<Slot<NonZeroU8>>(), size_of::<u8>());
    assert_eq!(size_of::<Slot<NonZeroU16>>(), size_of::<u16>());
    assert_eq!(size_of::<Slot<NonZeroU32>>(), size_of::<u32>());
    assert_eq!(size_of::<Slot<NonZeroU64>>(), size_of::<u64>());
    assert_eq!(size_of::<Slot<NonZeroU128>>(), size_of::<u128>());
    assert_eq!(align_of::<Slot<NonZeroU128>>(), align_of::<u128>());
}

#[test]
fn slot_signed_nonzero_have_integer_layout() {
    assert_eq!(size_of::<Slot<NonZeroI8>>(), size_of::<i8>());
    assert_eq!(size_of::<Slot<NonZeroI16>>(), size_of::<i16>());
    assert_eq!(size_of::<Slot<NonZeroI32>>(), size_of::<i32>());
    assert_eq!(size_of::<Slot<NonZeroI64>>(), size_of::<i64>());
    assert_eq!(size_of::<Slot<NonZeroI128>>(), size_of::<i128>());
    assert_eq!(align_of::<Slot<NonZeroI128>>(), align_of::<i128>());
}

#[test]
fn slot_none_round_trips_through_some_and_back() {
    let nz = NonZeroU32::new(42).unwrap();
    let s: Slot<NonZeroU32> = Slot::some(nz);
    assert!(s.is_some());
    assert!(!s.is_none());

    let none: Slot<NonZeroU32> = Slot::NONE;
    assert!(!none.is_some());
    assert!(none.is_none());
}

#[test]
fn slot_as_maybe_borrow_projects() {
    let nz = NonZeroU32::new(7).unwrap();
    let s: Slot<NonZeroU32> = Slot::some(nz);
    match s.as_maybe() {
        notko::Maybe::Is(v) => assert_eq!(v.get(), 7),
        notko::Maybe::Isnt => panic!("expected Is variant"),
    }
}
