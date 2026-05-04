//! Smoke tests for the Outcome combinator surface (round-353).

use notko::Maybe;
use notko::Outcome;

#[test]
fn as_ref_and_as_mut() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    match o.as_ref() {
        Outcome::Ok(v) => assert_eq!(*v, 7),
        Outcome::Err(_) => panic!("expected Ok"),
    }

    let mut o: Outcome<i32, &'static str> = Outcome::Ok(1);
    if let Outcome::Ok(v) = o.as_mut() {
        *v = 99;
    }
    assert!(matches!(o, Outcome::Ok(99)));
}

#[test]
fn ok_and_err_projections() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert!(matches!(o.ok(), Maybe::Is(7)));

    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert!(matches!(o.err(), Maybe::Is("e")));

    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert!(matches!(o.err(), Maybe::Isnt));

    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert!(matches!(o.ok(), Maybe::Isnt));
}

#[test]
fn unwrap_err() {
    let o: Outcome<i32, &'static str> = Outcome::Err("boom");
    assert_eq!(o.unwrap_err(), "boom");
}

#[test]
fn unwrap_or_else() {
    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert_eq!(o.unwrap_or_else(|_| 99), 99);

    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert_eq!(o.unwrap_or_else(|_| 99), 7);
}

#[test]
fn unwrap_or_default() {
    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert_eq!(o.unwrap_or_default(), 0);
    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert_eq!(o.unwrap_or_default(), 7);
}

#[test]
fn expect_passes_on_ok() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert_eq!(o.expect("should not panic"), 7);
}

#[test]
fn expect_err_passes_on_err() {
    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert_eq!(o.expect_err("should not panic"), "e");
}

#[test]
fn map_or_and_map_or_else() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(3);
    assert_eq!(o.map_or(99, |x| x * 2), 6);

    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert_eq!(o.map_or(99, |x| x * 2), 99);

    let o: Outcome<i32, &'static str> = Outcome::Ok(3);
    assert_eq!(o.map_or_else(|_e| 99, |x| x * 2), 6);

    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert_eq!(o.map_or_else(|_e| 99, |x| x * 2), 99);
}

#[test]
fn and_then_and_or_else() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(5);
    let r: Outcome<i32, &'static str> = o.and_then(|x| Outcome::Ok(x * 2));
    assert!(matches!(r, Outcome::Ok(10)));

    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    let r: Outcome<i32, &'static str> = o.and_then(|x| Outcome::Ok(x * 2));
    assert!(matches!(r, Outcome::Err("e")));

    let o: Outcome<i32, &'static str> = Outcome::Err("a");
    let r: Outcome<i32, &'static str> = o.or_else(|_| Outcome::Ok(99));
    assert!(matches!(r, Outcome::Ok(99)));
}

#[test]
fn and_short_circuits_on_err() {
    let a: Outcome<i32, &'static str> = Outcome::Ok(1);
    let b: Outcome<&'static str, &'static str> = Outcome::Ok("two");
    let r: Outcome<&'static str, &'static str> = a.and(b);
    assert!(matches!(r, Outcome::Ok("two")));

    let a: Outcome<i32, &'static str> = Outcome::Err("e");
    let b: Outcome<&'static str, &'static str> = Outcome::Ok("two");
    let r: Outcome<&'static str, &'static str> = a.and(b);
    assert!(matches!(r, Outcome::Err("e")));
}

#[test]
fn or_returns_self_on_ok() {
    let a: Outcome<i32, &'static str> = Outcome::Ok(1);
    let b: Outcome<i32, &'static str> = Outcome::Ok(2);
    let r: Outcome<i32, &'static str> = a.or(b);
    assert!(matches!(r, Outcome::Ok(1)));

    let a: Outcome<i32, &'static str> = Outcome::Err("e");
    let b: Outcome<i32, &'static str> = Outcome::Ok(2);
    let r: Outcome<i32, &'static str> = a.or(b);
    assert!(matches!(r, Outcome::Ok(2)));
}

#[test]
fn is_ok_and_and_is_err_and() {
    let o: Outcome<i32, &'static str> = Outcome::Ok(5);
    assert!(o.is_ok_and(|x| x > 0));
    let o: Outcome<i32, &'static str> = Outcome::Ok(-1);
    assert!(!o.is_ok_and(|x| x > 0));
    let o: Outcome<i32, &'static str> = Outcome::Err("e");
    assert!(!o.is_ok_and(|x| x > 0));

    let o: Outcome<i32, &'static str> = Outcome::Err("boom");
    assert!(o.is_err_and(|e| e.starts_with("boom")));
    let o: Outcome<i32, &'static str> = Outcome::Err("ok");
    assert!(!o.is_err_and(|e| e.starts_with("boom")));
    let o: Outcome<i32, &'static str> = Outcome::Ok(7);
    assert!(!o.is_err_and(|e| e.starts_with("boom")));
}

#[test]
fn inspect_and_inspect_err_pass_through() {
    let mut sum = 0_i32;
    let o: Outcome<i32, &'static str> =
        Outcome::Ok(7).inspect(|x| sum += *x);
    assert_eq!(sum, 7);
    assert!(matches!(o, Outcome::Ok(7)));

    let mut last_err: Option<&'static str> = None;
    let o: Outcome<i32, &'static str> =
        Outcome::Err("boom").inspect_err(|e| last_err = Some(*e));
    assert_eq!(last_err, Some("boom"));
    assert!(matches!(o, Outcome::Err("boom")));
}

#[test]
fn from_result_and_back() {
    let r: Result<i32, &'static str> = Ok(7);
    let o: Outcome<i32, &'static str> = r.into();
    assert!(matches!(o, Outcome::Ok(7)));

    let r: Result<i32, &'static str> = Outcome::Err("e").into();
    assert_eq!(r, Err("e"));
}
