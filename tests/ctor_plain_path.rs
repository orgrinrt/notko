//! Plain-feature variant of HasTrivialCtor smoke tests. Loaded when
//! the `const` feature is disabled (consumers on stable Rust opt
//! into this via `default-features = false`).

use notko::HasTrivialCtor;

struct UnitMarker;

impl HasTrivialCtor for UnitMarker {
    fn new() -> Self { UnitMarker }
}

struct PhantomMarker<T>(core::marker::PhantomData<T>);

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
