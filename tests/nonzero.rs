//! Smoke tests for the [`NonZeroable`] blanket impls over
//! `core::num::NonZero{U,I}*`.

use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

use notko::{Maybe, NonZeroable};

#[test]
fn try_new_zero_returns_isnt_unsigned() {
    assert!(matches!(<NonZeroU8 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroU16 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroU32 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroU64 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroU128 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroUsize as NonZeroable>::try_new(0), Maybe::Isnt));
}

#[test]
fn try_new_zero_returns_isnt_signed() {
    assert!(matches!(<NonZeroI8 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroI16 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroI32 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroI64 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroI128 as NonZeroable>::try_new(0), Maybe::Isnt));
    assert!(matches!(<NonZeroIsize as NonZeroable>::try_new(0), Maybe::Isnt));
}

#[test]
fn try_new_nonzero_round_trips_unsigned() {
    let cases: [(u32, u32); 4] = [(1, 1), (42, 42), (255, 255), (u32::MAX, u32::MAX)];
    for (input, expected) in cases {
        match <NonZeroU32 as NonZeroable>::try_new(input) {
            Maybe::Is(v) => assert_eq!(v.get(), expected),
            Maybe::Isnt => panic!("nonzero {input} must accept"),
        }
    }
}

#[test]
fn try_new_nonzero_round_trips_signed() {
    let cases: [i32; 5] = [1, -1, 42, i32::MIN, i32::MAX];
    for input in cases {
        match <NonZeroI32 as NonZeroable>::try_new(input) {
            Maybe::Is(v) => assert_eq!(v.get(), input),
            Maybe::Isnt => panic!("nonzero {input} must accept"),
        }
    }
}

#[test]
fn value_extracts_underlying() {
    let nz = NonZeroU64::new(123_456_789).expect("nonzero literal");
    assert_eq!(NonZeroable::value(nz), 123_456_789);

    let nzi = NonZeroI64::new(-987).expect("nonzero literal");
    assert_eq!(NonZeroable::value(nzi), -987);
}

/// Generic helper exercises `T: NonZeroable<Inner = u32>` so the
/// blanket bound shape used by downstream consumers is exercised
/// through the test surface.
#[test]
fn generic_constructor_via_trait_bound() {
    fn build<T: NonZeroable<Inner = u32>>(raw: u32) -> Maybe<T> {
        T::try_new(raw)
    }

    assert!(matches!(build::<NonZeroU32>(0), Maybe::Isnt));
    match build::<NonZeroU32>(42) {
        Maybe::Is(v) => assert_eq!(v.get(), 42),
        Maybe::Isnt => panic!("42 must accept"),
    }
}
