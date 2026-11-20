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
fn unit_marker_constructs() {
    let _m = UnitMarker::new();
}

#[test]
fn phantom_marker_constructs_with_turbofish() {
    let _m: PhantomMarker<u32> = PhantomMarker::<u32>::new();
}
