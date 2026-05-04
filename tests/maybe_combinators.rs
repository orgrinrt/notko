//! Smoke tests for the Maybe combinator surface (round-353).

use notko::Maybe;
use notko::Outcome;

use crate::heapless_min::CollectToSmallVec;

#[test]
fn unwrap_or_else() {
    assert_eq!(Maybe::Is(7).unwrap_or_else(|| 0), 7);
    assert_eq!(Maybe::<i32>::Isnt.unwrap_or_else(|| 99), 99);
}

#[test]
fn unwrap_or_default() {
    assert_eq!(Maybe::Is(5_u32).unwrap_or_default(), 5);
    assert_eq!(Maybe::<u32>::Isnt.unwrap_or_default(), 0);
}

#[test]
fn map_or() {
    assert_eq!(Maybe::Is(3).map_or(99, |x| x * 2), 6);
    assert_eq!(Maybe::<i32>::Isnt.map_or(99, |x| x * 2), 99);
}

#[test]
fn map_or_else() {
    assert_eq!(Maybe::Is(3).map_or_else(|| 99, |x| x * 2), 6);
    assert_eq!(Maybe::<i32>::Isnt.map_or_else(|| 99, |x| x * 2), 99);
}

#[test]
fn ok_or_else() {
    let m: Maybe<i32> = Maybe::Is(7);
    let o: Outcome<i32, &'static str> = m.ok_or_else(|| "absent");
    assert!(matches!(o, Outcome::Ok(7)));

    let m: Maybe<i32> = Maybe::Isnt;
    let o: Outcome<i32, &'static str> = m.ok_or_else(|| "absent");
    assert!(matches!(o, Outcome::Err("absent")));
}

#[test]
fn and_then() {
    let m: Maybe<i32> = Maybe::Is(5);
    let r = m.and_then(|x| if x > 0 { Maybe::Is(x * 2) } else { Maybe::Isnt });
    assert!(matches!(r, Maybe::Is(10)));

    let m: Maybe<i32> = Maybe::Isnt;
    let r = m.and_then(|x| Maybe::Is(x * 2));
    assert!(matches!(r, Maybe::Isnt));
}

#[test]
fn or_combinator() {
    assert!(matches!(Maybe::Is(1).or(Maybe::Is(2)), Maybe::Is(1)));
    assert!(matches!(Maybe::<i32>::Isnt.or(Maybe::Is(2)), Maybe::Is(2)));
    assert!(matches!(Maybe::<i32>::Isnt.or(Maybe::Isnt), Maybe::Isnt));
}

#[test]
fn or_else() {
    assert!(matches!(Maybe::Is(1).or_else(|| Maybe::Is(2)), Maybe::Is(1)));
    assert!(matches!(Maybe::<i32>::Isnt.or_else(|| Maybe::Is(2)), Maybe::Is(2)));
}

#[test]
fn filter() {
    assert!(matches!(Maybe::Is(5).filter(|x| *x > 0), Maybe::Is(5)));
    assert!(matches!(Maybe::Is(-1).filter(|x| *x > 0), Maybe::Isnt));
    assert!(matches!(Maybe::<i32>::Isnt.filter(|x| *x > 0), Maybe::Isnt));
}

#[test]
fn xor_combinator() {
    assert!(matches!(Maybe::Is(1).xor(Maybe::<i32>::Isnt), Maybe::Is(1)));
    assert!(matches!(Maybe::<i32>::Isnt.xor(Maybe::Is(2)), Maybe::Is(2)));
    assert!(matches!(Maybe::Is(1).xor(Maybe::Is(2)), Maybe::Isnt));
    assert!(matches!(Maybe::<i32>::Isnt.xor(Maybe::Isnt), Maybe::Isnt));
}

#[test]
fn zip() {
    assert!(matches!(Maybe::Is(1).zip(Maybe::Is("two")), Maybe::Is((1, "two"))));
    assert!(matches!(Maybe::<i32>::Isnt.zip(Maybe::Is("two")), Maybe::Isnt));
    assert!(matches!(Maybe::Is(1).zip(Maybe::<&str>::Isnt), Maybe::Isnt));
}

#[test]
fn take() {
    let mut m = Maybe::Is(42);
    let taken = m.take();
    assert!(matches!(taken, Maybe::Is(42)));
    assert!(matches!(m, Maybe::Isnt));
}

#[test]
fn replace() {
    let mut m = Maybe::Is(1);
    let prev = m.replace(2);
    assert!(matches!(prev, Maybe::Is(1)));
    assert!(matches!(m, Maybe::Is(2)));
}

#[test]
fn is_some_and() {
    assert!(Maybe::Is(5).is_some_and(|x| x > 0));
    assert!(!Maybe::Is(-1).is_some_and(|x| x > 0));
    assert!(!Maybe::<i32>::Isnt.is_some_and(|x| x > 0));
}

#[test]
fn is_none_or() {
    assert!(Maybe::<i32>::Isnt.is_none_or(|_| false));
    assert!(Maybe::Is(5).is_none_or(|x| x > 0));
    assert!(!Maybe::Is(-1).is_none_or(|x| x > 0));
}

#[test]
fn flatten() {
    let m: Maybe<Maybe<i32>> = Maybe::Is(Maybe::Is(7));
    assert!(matches!(m.flatten(), Maybe::Is(7)));

    let m: Maybe<Maybe<i32>> = Maybe::Is(Maybe::Isnt);
    assert!(matches!(m.flatten(), Maybe::Isnt));

    let m: Maybe<Maybe<i32>> = Maybe::Isnt;
    assert!(matches!(m.flatten(), Maybe::Isnt));
}

#[test]
fn copied_and_cloned() {
    let v = 42_i32;
    let r: Maybe<&i32> = Maybe::Is(&v);
    assert!(matches!(r.copied(), Maybe::Is(42)));
    let r: Maybe<&i32> = Maybe::Is(&v);
    assert!(matches!(r.cloned(), Maybe::Is(42)));
}

#[test]
fn iter_yields_once_or_zero() {
    let m: Maybe<i32> = Maybe::Is(7);
    let collected: heapless_min::SmallVec<[i32; 2]> = m.into_iter().collect_to_smallvec();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected.values[0], 7);

    let m: Maybe<i32> = Maybe::Isnt;
    let collected: heapless_min::SmallVec<[i32; 2]> = m.into_iter().collect_to_smallvec();
    assert_eq!(collected.len(), 0);
}

#[test]
fn iter_borrowed() {
    let m: Maybe<i32> = Maybe::Is(7);
    let mut count = 0_i32;
    for x in m.iter() {
        assert_eq!(*x, 7);
        count += 1;
    }
    assert_eq!(count, 1);

    let m: Maybe<i32> = Maybe::Isnt;
    let mut count = 0_i32;
    for _ in m.iter() {
        count += 1;
    }
    assert_eq!(count, 0);
}

#[test]
fn from_option_and_back() {
    let m: Maybe<i32> = Some(42).into();
    assert!(matches!(m, Maybe::Is(42)));
    let opt: Option<i32> = Maybe::Is(42).into();
    assert_eq!(opt, Some(42));

    let m: Maybe<i32> = None.into();
    assert!(matches!(m, Maybe::Isnt));
    let opt: Option<i32> = Maybe::<i32>::Isnt.into();
    assert_eq!(opt, None);
}

#[test]
fn as_mut() {
    let mut m: Maybe<i32> = Maybe::Is(1);
    if let Maybe::Is(r) = m.as_mut() {
        *r = 99;
    }
    assert!(matches!(m, Maybe::Is(99)));
}

// Tiny inline alternative to a dependency — keeps the tests self-contained.
mod heapless_min {
    pub struct SmallVec<A> {
        pub values: A,
        len: usize,
    }

    pub trait CollectToSmallVec: Iterator + Sized {
        fn collect_to_smallvec(self) -> SmallVec<[Self::Item; 2]>
        where
            Self::Item: Copy + Default,
        {
            let mut out = SmallVec { values: [Default::default(); 2], len: 0 };
            for x in self {
                if out.len < 2 {
                    out.values[out.len] = x;
                    out.len += 1;
                }
            }
            out
        }
    }

    impl<I: Iterator + Sized> CollectToSmallVec for I {}

    impl<A> SmallVec<A> {
        pub fn len(&self) -> usize {
            self.len
        }
    }
}
