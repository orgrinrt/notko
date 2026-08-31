//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `Slot<T>` prints the words its own surface uses.
//!
//! It is a `#[repr(transparent)]` newtype over [`notko::Maybe`], and a derived
//! `Debug` delegates to the inner value and prints `Slot(Isnt)`. `Isnt` is not
//! a word this type says anywhere: the absent value is `NONE`, the predicate is
//! `is_none`, the constructor is `some`. A reader chasing an empty field is
//! told about the layer underneath, in a name they cannot grep for.

use core::num::NonZeroU8;

use notko::Slot;

fn one() -> NonZeroU8 {
    NonZeroU8::new(1).expect("1 is not zero")
}

#[test]
fn the_absent_value_says_none_and_not_isnt() {
    let s: Slot<NonZeroU8> = Slot::NONE;
    let printed = format!("{s:?}");
    assert!(
        printed.contains("none"),
        "the absent slot does not say `none`: {printed}"
    );
    assert!(
        !printed.contains("Isnt"),
        "the absent slot leaks the inner type's vocabulary: {printed}"
    );
}

#[test]
fn the_present_value_still_carries_what_it_holds() {
    // The other side. A `Debug` that printed a fixed string would pass the test
    // above for a type that never shows its contents at all.
    let s = Slot::some(one());
    let printed = format!("{s:?}");
    assert!(
        printed.contains("Slot"),
        "the present slot does not name its own type: {printed}"
    );
    assert!(
        printed.contains('1'),
        "the present slot does not print what it holds: {printed}"
    );
}

#[test]
fn both_arms_keep_their_shape_under_the_alternate_form() {
    // `{:#?}` is what a derived `Debug` on a struct holding one of these
    // reaches for, and it lays out a tuple across lines. An arm that writes its
    // whole rendering as one string has nothing for the formatter to lay out,
    // so the two arms disagree about their shape under one specifier and a
    // pretty-printed struct comes back looking like two different types.
    //
    // The absent arm is the one that can go wrong: naming a fixed word is the
    // obvious way to write it and `write_str("Slot(none)")` passes every other
    // assertion in this file.
    let present = format!("{:#?}", Slot::some(one()));
    let absent = format!("{:#?}", Slot::<NonZeroU8>::NONE);

    for (name, printed) in [("present", &present), ("absent", &absent)] {
        assert!(
            printed.contains('\n'),
            "the {name} arm did not lay out across lines: {printed:?}"
        );
        assert!(
            printed.starts_with("Slot(\n"),
            "the {name} arm is not a tuple under the alternate form: {printed:?}"
        );
        assert!(
            printed.ends_with(",\n)"),
            "the {name} arm does not close like a tuple: {printed:?}"
        );
    }
}

#[test]
fn a_struct_holding_one_can_still_derive_debug() {
    // The reason the impl is written out rather than dropped. Removing the
    // derive without replacing it takes the field's printability with it, and
    // this file is where that is asserted by building.
    #[derive(Debug)]
    struct Descriptor {
        // Read through the derived `Debug` rather than by name, which is what
        // the assertion below does and what the field is here for.
        handle: Slot<NonZeroU8>,
    }

    let d = Descriptor {
        handle: Slot::some(one()),
    };
    let printed = format!("{d:?}");
    assert!(printed.contains("handle"), "{printed}");
    assert!(printed.contains("Slot"), "{printed}");
    // And by name, because rustc does not count a derived `Debug` as a read
    // and the field would otherwise be reported dead in a file whose whole
    // subject is printing it.
    assert!(d.handle.is_some());
}
