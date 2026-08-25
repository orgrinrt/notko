//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Deref` and the borrow conversions on `Just<T>`.
//!
//! Ungated on purpose: these impls exist in both feature configurations, so
//! this target must build and pass with `const` on and off. That symmetry is
//! the same contract `consttry_parity.rs` asserts for `ConstTry`.

#![cfg_attr(feature = "const", feature(const_trait_impl))]
// A const deref coercion needs the const-traits gates in THIS crate, because
// `Deref`'s const-ness is `rustc_const_unstable(feature = "const_convert")`.
// A consumer wanting `*just` in a const fn pays the same cost.
#![cfg_attr(feature = "const", feature(const_convert))]

use core::borrow::{Borrow, BorrowMut};
use notko::Just;

#[derive(PartialEq, Eq, Debug)]
struct NotCopy(u32);

#[test]
fn deref_reads_through() {
    let j = Just::new(NotCopy(7));
    assert_eq!(*j, NotCopy(7));
    assert_eq!(j.0, 7, "field access through auto-deref");
}

#[test]
fn deref_mut_writes_through() {
    let mut j = Just::new(NotCopy(1));
    (*j).0 = 9;
    assert_eq!(*j, NotCopy(9));
}

#[test]
fn method_resolution_reaches_the_payload() {
    // The point of Deref here: T's inherent surface is reachable without
    // unwrapping first.
    let j = Just::new(String::from("abc"));
    assert_eq!(j.len(), 3);
    assert!(j.starts_with('a'));
}

#[test]
fn borrow_agrees_with_deref() {
    let j = Just::new(NotCopy(4));
    let b: &NotCopy = j.borrow();
    assert_eq!(b, &*j);
    assert!(core::ptr::eq(b, &*j), "borrow and deref name the same storage");
}

#[test]
fn inherent_as_ref_still_wins_and_returns_the_functor_shape() {
    // Guards the reason AsRef/AsMut are NOT implemented: the inherent methods
    // return Just<&T>, and adding the traits would shadow-conflict here.
    let j = Just::new(NotCopy(4));
    let f: Just<&NotCopy> = j.as_ref();
    assert_eq!(**f.get(), NotCopy(4));
}

#[test]
fn borrow_mut_writes_through() {
    let mut j = Just::new(NotCopy(0));
    { let m: &mut NotCopy = j.borrow_mut(); m.0 = 6; }
    assert_eq!(*j, NotCopy(6));
}

#[test]
fn deref_is_zero_cost_over_t() {
    // Just is repr(transparent); deref must not be observing a copy.
    let j = Just::new(NotCopy(3));
    let inner: *const NotCopy = &*j;
    let whole: *const Just<NotCopy> = &j;
    assert_eq!(inner as usize, whole as usize, "transparent: same address");
}

// A const proof. Gated, because the plain path's Deref is not const-callable
// and asserting otherwise there would be asserting the feature narrows.
#[cfg(feature = "const")]
const _CONST_DEREF: () = {
    let j = Just::new(7u32);
    assert!(*j == 7);
};
