//! Smoke tests for [`Boundable`] + [`BoundError`].

use notko::{BoundError, Boundable, Outcome};

/// Toy newtype implementing `Boundable` over a `u8` range `[10, 100]`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Pct(u8);

impl Boundable for Pct {
    type Inner = u8;
    const MIN: u8 = 10;
    const MAX: u8 = 100;

    fn try_new(value: u8) -> Outcome<Self, BoundError<u8>> {
        if value < Self::MIN {
            return Outcome::Err(BoundError::Below {
                value,
                min: Self::MIN,
            });
        }
        if value > Self::MAX {
            return Outcome::Err(BoundError::Above {
                value,
                max: Self::MAX,
            });
        }
        Outcome::Ok(Pct(value))
    }

    fn value(self) -> u8 {
        self.0
    }
}

#[test]
fn in_range_constructs() {
    assert!(matches!(Pct::try_new(10), Outcome::Ok(Pct(10))));
    assert!(matches!(Pct::try_new(50), Outcome::Ok(Pct(50))));
    assert!(matches!(Pct::try_new(100), Outcome::Ok(Pct(100))));
}

#[test]
fn below_min_rejects_with_below_variant() {
    let err = match Pct::try_new(0) {
        Outcome::Ok(_) => panic!("0 must reject"),
        Outcome::Err(e) => e,
    };
    assert_eq!(
        err,
        BoundError::Below {
            value: 0,
            min: 10,
        },
    );
}

#[test]
fn above_max_rejects_with_above_variant() {
    let err = match Pct::try_new(200) {
        Outcome::Ok(_) => panic!("200 must reject"),
        Outcome::Err(e) => e,
    };
    assert_eq!(
        err,
        BoundError::Above {
            value: 200,
            max: 100,
        },
    );
}

#[test]
fn value_round_trips() {
    let p = match Pct::try_new(42) {
        Outcome::Ok(p) => p,
        Outcome::Err(_) => panic!("42 must accept"),
    };
    assert_eq!(p.value(), 42);
}

#[test]
fn bound_error_clone_and_eq() {
    let err: BoundError<u8> = BoundError::Below {
        value: 5,
        min: 10,
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);

    let other: BoundError<u8> = BoundError::Above {
        value: 200,
        max: 100,
    };
    assert!(err != other);
}

#[test]
fn bound_error_debug_renders_field_names() {
    let err: BoundError<u8> = BoundError::Below {
        value: 5,
        min: 10,
    };
    let rendered = format!("{err:?}");
    assert!(rendered.contains("Below"));
    assert!(rendered.contains("value"));
    assert!(rendered.contains("min"));
}

/// Confirms `BoundError<I>` does NOT require `I: Copy`. A non-Copy
/// inner (like a heapless wrapper) still satisfies the bound.
#[test]
fn bound_error_works_with_clone_only_inner() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct CloneOnly(u32);

    let err: BoundError<CloneOnly> = BoundError::Above {
        value: CloneOnly(99),
        max: CloneOnly(50),
    };
    let cloned = err.clone();
    assert_eq!(err, cloned);
}
