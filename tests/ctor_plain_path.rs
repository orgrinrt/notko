//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Runtime-construction variant of HasTrivialCtor smoke tests.
//!
//! Its own test target with no `required-features`, so it compiles whether or
//! not the `const` feature is on: calling `new` at runtime is valid either way.
//! Under `--no-default-features` it is the only ctor coverage there is, which is
//! the point, since that is the configuration a consumer on stable gets.

use notko::HasTrivialCtor;

struct UnitMarker;

impl HasTrivialCtor for UnitMarker {
    fn new() -> Self {
        UnitMarker
    }
}

struct PhantomMarker<T>(core::marker::PhantomData<T>);

impl<T> HasTrivialCtor for PhantomMarker<T> {
    fn new() -> Self {
        PhantomMarker(core::marker::PhantomData)
    }
}

#[test]
fn the_trait_is_reachable_through_a_plain_generic_bound() {
    // Constructing at the call site proves only that the call site compiles.
    // What the trait is for is being a bound, so the check is that a generic
    // function holding nothing but `T: HasTrivialCtor` can build a `T`, which
    // is the only thing a consumer ever does with it.
    fn built<T: HasTrivialCtor>() -> T {
        T::new()
    }

    let _unit: UnitMarker = built();
    let _phantom: PhantomMarker<u32> = built();

    // Both markers carry no data, which is what makes a trivial ctor trivial.
    assert_eq!(core::mem::size_of::<UnitMarker>(), 0);
    assert_eq!(core::mem::size_of::<PhantomMarker<u32>>(), 0);

    // And the turbofish path, which is the other spelling a consumer reaches
    // for and resolves differently.
    assert_eq!(
        core::mem::size_of_val(&PhantomMarker::<u32>::new()),
        core::mem::size_of_val(&PhantomMarker::<u64>::new())
    );
}
