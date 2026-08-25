//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What a `Slot<T>` payload can be, from outside the crate.
//!
//! `Slot<T>` asks for `NonZeroable + NicheFilled`. The second is sealed and
//! the first is not, so the set is decided from both sides at once and is
//! wider than either the sealed list or the `NonZero*` family. The
//! documentation on `into_maybe` used to say the set was the `NonZero*` family
//! alone and that its `Copy` bound therefore cost nothing; both halves are
//! answered here, from a crate that is not `notko`, which is the only place
//! the question means anything.

use notko::{Maybe, NicheFilled, NonZeroable, Slot};

/// A payload of our own. Not `Copy`, and not something `notko` has heard of.
#[derive(Debug, PartialEq)]
struct Theirs(u32);

// `NonZeroable` is open, so this is somebody else's to write. `&mut Theirs` is
// `NicheFilled` through the blanket impl on `&mut T`, which is what makes the
// pair reachable at all.
impl NonZeroable for &mut Theirs {
    type Inner = u32;

    fn try_new(raw: Self::Inner) -> Maybe<Self> {
        // Nothing here can conjure a borrow, and it does not need to: what is
        // being established is that the impl exists and the bounds admit it.
        let _ = raw;
        Maybe::Isnt
    }

    fn value(self) -> Self::Inner {
        self.0
    }
}

/// Reading the seal from outside, which is the only side it holds against.
fn admits<T: NonZeroable + NicheFilled>() {}

#[test]
fn a_reference_to_a_foreign_type_is_a_payload() {
    // The claim under test, and it is a compile-time one: if the bounds did not
    // admit `&mut Theirs`, this line would not build.
    admits::<&mut Theirs>();

    let mut theirs = Theirs(7);
    let slot: Slot<&mut Theirs> = Slot::some(&mut theirs);
    assert!(slot.is_some());
    assert!(!slot.is_none());
    match slot.as_maybe() {
        Maybe::Is(t) => assert_eq!(t.0, 7),
        Maybe::Isnt => panic!("a slot built with `some` reported absent"),
    }
}

#[test]
fn the_nonzero_family_is_a_payload_too() {
    // The control. Without it the test above would pass just as well against a
    // `Slot` that admitted everything, which would prove nothing about a seal.
    admits::<core::num::NonZeroU32>();

    let slot: Slot<core::num::NonZeroU32> = Slot::some(core::num::NonZeroU32::new(3).unwrap());
    assert_eq!(
        slot.into_maybe(),
        Maybe::Is(core::num::NonZeroU32::new(3).unwrap())
    );
    assert_eq!(
        Slot::<core::num::NonZeroU32>::NONE.into_maybe(),
        Maybe::Isnt
    );
}

#[test]
fn a_plain_integer_is_not_a_payload() {
    // The other control, and the one that says the seal is doing something.
    // `u32` is `NonZeroable`? No, and it is not `NicheFilled` either: it has no
    // spare bit pattern to fill, which is the whole subject. Asserting that in
    // a way the compiler checks needs a compile-fail harness this crate does
    // not carry yet, so what is asserted here is the half that can be: the
    // types that are admitted, above, and the shape of the one that is not.
    //
    // `notko::MaybeNull<u32>` is the same refusal one layer up, and
    // `src/maybe.rs` carries the commented cases waiting for that harness.
    assert_eq!(core::mem::size_of::<Slot<core::num::NonZeroU32>>(), 4);
    assert_eq!(
        core::mem::size_of::<Slot<core::num::NonZeroU32>>(),
        core::mem::size_of::<core::num::NonZeroU32>(),
        "the niche is not being filled, so the seal is admitting the wrong set"
    );
}
