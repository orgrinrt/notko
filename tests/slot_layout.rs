//! Layout assertions for `Slot<T>`. Pins `size_of::<Slot<T>> ==
//! size_of::<T>` at runtime so a layout regression surfaces as a
//! test failure even if the per-instantiation `_LAYOUT_ASSERT`
//! consts in `slot.rs` are bypassed (e.g. by inlining changes that
//! avoid evaluating them).

use core::mem::{align_of, size_of};
use core::num::{NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroIsize, NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize};
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
fn slot_nonzero_u8_to_u64_have_integer_layout() {
    assert_eq!(size_of::<Slot<NonZeroU8>>(), size_of::<u8>());
    assert_eq!(size_of::<Slot<NonZeroU16>>(), size_of::<u16>());
    assert_eq!(size_of::<Slot<NonZeroU32>>(), size_of::<u32>());
    assert_eq!(size_of::<Slot<NonZeroU64>>(), size_of::<u64>());
}

#[test]
fn slot_nonzero_i8_to_i64_have_integer_layout() {
    assert_eq!(size_of::<Slot<NonZeroI8>>(), size_of::<i8>());
    assert_eq!(size_of::<Slot<NonZeroI16>>(), size_of::<i16>());
    assert_eq!(size_of::<Slot<NonZeroI32>>(), size_of::<i32>());
    assert_eq!(size_of::<Slot<NonZeroI64>>(), size_of::<i64>());
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
