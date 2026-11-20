//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-feature variant of HasTrivialCtor smoke tests.
//!
//! Its own test target, declared in `Cargo.toml` with
//! `required-features = ["const"]`, so cargo skips it entirely when the feature
//! is off. That is what lets the gate below be unconditional: the target only
//! exists in configurations where the trait is a `const trait`.

#![feature(const_trait_impl)]

use notko::HasTrivialCtor;

struct UnitMarker;

const impl HasTrivialCtor for UnitMarker {
    fn new() -> Self {
        UnitMarker
    }
}

struct PhantomMarker<T>(core::marker::PhantomData<T>);

const impl<T> HasTrivialCtor for PhantomMarker<T> {
    fn new() -> Self {
        PhantomMarker(core::marker::PhantomData)
    }
}

#[test]
fn a_const_impl_still_satisfies_the_plain_bound() {
    // The `const _` bindings at the bottom are the real assertion here: they
    // evaluate `new()` at compile time, which is the whole claim of a `const
    // impl`, and a regression makes the file stop building rather than stop
    // passing.
    //
    // What runtime can add is the half a compile-time binding cannot reach. A
    // `const impl` is also an ordinary impl, so a generic function taking the
    // plain bound has to accept it, and a value built through that path has to
    // be the same thing the const binding holds.
    fn built<T: HasTrivialCtor>() -> T {
        T::new()
    }

    let through_the_bound: UnitMarker = built();
    assert_eq!(
        core::mem::size_of_val(&through_the_bound),
        core::mem::size_of_val(&_UNIT_CONST),
        "the const path and the generic path disagree about the type"
    );

    let phantom: PhantomMarker<u32> = built();
    assert_eq!(
        core::mem::size_of_val(&phantom),
        core::mem::size_of_val(&_PHANTOM_CONST)
    );

    // A marker carrying no data is the premise both paths rest on, so it is
    // asserted rather than assumed.
    assert_eq!(core::mem::size_of::<UnitMarker>(), 0);
    assert_eq!(core::mem::size_of::<PhantomMarker<u32>>(), 0);
}

const _UNIT_CONST: UnitMarker = UnitMarker::new();
const _PHANTOM_CONST: PhantomMarker<u32> = PhantomMarker::<u32>::new();
