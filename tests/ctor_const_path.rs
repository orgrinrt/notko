//! Const-feature variant of HasTrivialCtor smoke tests. Loaded when
//! the `const` feature is enabled (default).

use notko::HasTrivialCtor;

struct UnitMarker;

impl const HasTrivialCtor for UnitMarker {
    fn new() -> Self { UnitMarker }
}

struct PhantomMarker<T>(core::marker::PhantomData<T>);

impl<T> const HasTrivialCtor for PhantomMarker<T> {
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

const _UNIT_CONST: UnitMarker = UnitMarker::new();
const _PHANTOM_CONST: PhantomMarker<u32> = PhantomMarker::<u32>::new();
