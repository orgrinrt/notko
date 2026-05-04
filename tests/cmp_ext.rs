//! Smoke tests for the `PartialOrdExt::partial_cmp_maybe` adapter.

use core::cmp::Ordering;

use notko::Maybe;
use notko::cmp::PartialOrdExt;

#[test]
fn integer_comparisons() {
    assert!(matches!(1_i32.partial_cmp_maybe(&2), Maybe::Is(Ordering::Less)));
    assert!(matches!(2_i32.partial_cmp_maybe(&2), Maybe::Is(Ordering::Equal)));
    assert!(matches!(3_i32.partial_cmp_maybe(&2), Maybe::Is(Ordering::Greater)));
}

#[test]
fn nan_returns_isnt() {
    let nan = f64::NAN;
    let one = 1.0_f64;
    assert!(matches!(one.partial_cmp_maybe(&nan), Maybe::Isnt));
    assert!(matches!(nan.partial_cmp_maybe(&one), Maybe::Isnt));
    assert!(matches!(nan.partial_cmp_maybe(&nan), Maybe::Isnt));
}

#[test]
fn float_total_ordering_when_finite() {
    let a = 0.5_f64;
    let b = 1.5_f64;
    assert!(matches!(a.partial_cmp_maybe(&b), Maybe::Is(Ordering::Less)));
    assert!(matches!(b.partial_cmp_maybe(&a), Maybe::Is(Ordering::Greater)));
}

#[test]
fn equivalent_to_partial_cmp_into() {
    let pairs: [(f64, f64); 4] =
        [(1.0, 2.0), (2.0, 2.0), (3.0, 2.0), (f64::NAN, 1.0)];
    for (a, b) in pairs {
        let from_ext = a.partial_cmp_maybe(&b);
        let from_into: Maybe<Ordering> = a.partial_cmp(&b).into();
        match (from_ext, from_into) {
            (Maybe::Is(x), Maybe::Is(y)) => assert_eq!(x, y),
            (Maybe::Isnt, Maybe::Isnt) => {},
            _ => panic!("ext / into divergence on {a} vs {b}"),
        }
    }
}
