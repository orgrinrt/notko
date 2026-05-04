//! Smoke tests for the `IteratorExt::next_maybe` adapter.

use notko::Maybe;
use notko::iter::IteratorExt;

#[test]
fn next_maybe_yields_is_then_isnt() {
    let mut it = [1, 2].into_iter();
    assert!(matches!(it.next_maybe(), Maybe::Is(1)));
    assert!(matches!(it.next_maybe(), Maybe::Is(2)));
    assert!(matches!(it.next_maybe(), Maybe::Isnt));
}

#[test]
fn empty_iterator_returns_isnt() {
    let mut it = core::iter::empty::<i32>();
    assert!(matches!(it.next_maybe(), Maybe::Isnt));
}

#[test]
fn works_with_custom_iterator() {
    struct Two(i32, i32, u8);
    impl Iterator for Two {
        type Item = i32;
        fn next(&mut self) -> Option<i32> {
            match self.2 {
                0 => { self.2 = 1; Some(self.0) },
                1 => { self.2 = 2; Some(self.1) },
                _ => None,
            }
        }
    }

    let mut it = Two(7, 8, 0);
    assert!(matches!(it.next_maybe(), Maybe::Is(7)));
    assert!(matches!(it.next_maybe(), Maybe::Is(8)));
    assert!(matches!(it.next_maybe(), Maybe::Isnt));
}

#[test]
fn equivalent_to_next_into() {
    let mut a = [1, 2, 3].into_iter();
    let mut b = [1, 2, 3].into_iter();
    for _ in 0..4 {
        let from_ext = a.next_maybe();
        let from_into: Maybe<i32> = b.next().into();
        match (from_ext, from_into) {
            (Maybe::Is(x), Maybe::Is(y)) => assert_eq!(x, y),
            (Maybe::Isnt, Maybe::Isnt) => {},
            _ => panic!("ext / into divergence"),
        }
    }
}
