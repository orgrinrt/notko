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
fn unit_marker_constructs() {
    let _m = UnitMarker::new();
}

#[test]
fn phantom_marker_constructs_with_turbofish() {
    let _m: PhantomMarker<u32> = PhantomMarker::<u32>::new();
}

const _UNIT_CONST: UnitMarker = UnitMarker::new();
const _PHANTOM_CONST: PhantomMarker<u32> = PhantomMarker::<u32>::new();
