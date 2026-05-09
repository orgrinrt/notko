//! HasTrivialCtor: const + non-const construction.

#![cfg_attr(feature = "const", feature(const_trait_impl))]

use notko::HasTrivialCtor;

struct UnitMarker;

#[cfg(feature = "const")]
impl const HasTrivialCtor for UnitMarker {
    fn new() -> Self { UnitMarker }
}

#[cfg(not(feature = "const"))]
impl HasTrivialCtor for UnitMarker {
    fn new() -> Self { UnitMarker }
}

struct PhantomMarker<T>(core::marker::PhantomData<T>);

#[cfg(feature = "const")]
impl<T> const HasTrivialCtor for PhantomMarker<T> {
    fn new() -> Self { PhantomMarker(core::marker::PhantomData) }
}

#[cfg(not(feature = "const"))]
impl<T> HasTrivialCtor for PhantomMarker<T> {
    fn new() -> Self { PhantomMarker(core::marker::PhantomData) }
}

#[test]
fn unit_marker_constructs() {
    let _m = UnitMarker::new();
}

#[test]
fn phantom_marker_constructs_with_turbofish() {
    let _m: PhantomMarker<u32> = PhantomMarker::<u32>::new();
}

#[cfg(feature = "const")]
const _UNIT_CONST: UnitMarker = UnitMarker::new();

#[cfg(feature = "const")]
const _PHANTOM_CONST: PhantomMarker<u32> = PhantomMarker::<u32>::new();
