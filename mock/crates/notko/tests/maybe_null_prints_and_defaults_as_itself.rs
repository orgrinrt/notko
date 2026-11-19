//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `MaybeNull<T>` speaks about null, in its own vocabulary, and defaults to it.
//!
//! The type wraps [`notko::Maybe`] and had neither `Debug` nor `Default`, so a
//! struct holding one could not derive either and a value could not be printed
//! at all. Both are ordinary expectations of a type sitting in an FFI struct,
//! which is the only place this type is for.

use core::num::NonZeroU8;

use notko::{Maybe, MaybeNull};

#[test]
fn the_default_is_null_rather_than_the_inner_type_s_own_default() {
    // A derive bounds `T: Default`, which is what rules it out: `NonZeroU8`
    // has none, so `MaybeNull::<NonZeroU8>::default` would not resolve at all.
    // The line below is that, asserted by building.
    //
    // What a derive would *do* where the bound is met is correct, because the
    // inner `Maybe<T>` defaults to `Isnt` on its own. The bound is the whole of
    // the reason, and an earlier version of this comment said the value was.
    let d: MaybeNull<NonZeroU8> = MaybeNull::default();
    assert!(d.is_null(), "the default carried a value");
    assert_eq!(d, MaybeNull::null());

    // And the other side, because a `Default` that is always null would pass
    // the assertion above for a type that could never hold anything.
    let one = MaybeNull::new(NonZeroU8::new(1).unwrap());
    assert!(one.is_non_null());
    assert_ne!(one, MaybeNull::default());
}

#[test]
fn a_struct_holding_one_can_derive_both() {
    // The reason either impl exists. Without them this file does not compile,
    // which is the whole of the finding restated as a build.
    #[derive(Debug, Default, PartialEq)]
    struct Descriptor {
        init: MaybeNull<NonZeroU8>,
        name: u8,
    }

    let d = Descriptor::default();
    assert_eq!(d, Descriptor {
        init: MaybeNull::null(),
        name: 0,
    });
    assert!(
        format!("{d:?}").contains("MaybeNull"),
        "the derived Debug did not reach the field: {d:?}"
    );
}

#[test]
fn printing_says_null_and_not_isnt() {
    // Delegating to `Maybe` would print `Isnt` for a value whose subject is
    // that it is null, and a reader chasing a field that came back zero from a
    // foreign call would be told about the wrong layer.
    let null = MaybeNull::<NonZeroU8>::null();
    let rendered = format!("{null:?}");
    assert_eq!(rendered, "MaybeNull(null)");
    assert!(
        !rendered.contains("Isnt"),
        "the wrapper printed the wrapped type's vocabulary: {rendered}"
    );

    let one = MaybeNull::new(NonZeroU8::new(7).unwrap());
    let rendered = format!("{one:?}");
    assert!(
        rendered.starts_with("MaybeNull(") && rendered.contains('7'),
        "the value did not survive into the rendering: {rendered}"
    );
    assert!(
        !rendered.contains("Is("),
        "the wrapper printed the wrapped type's vocabulary: {rendered}"
    );

    // The control on the pair: `Maybe` still prints its own way, so the two
    // renderings are distinguishable and neither has quietly become the other.
    assert_eq!(format!("{:?}", Maybe::<NonZeroU8>::Isnt), "Isnt");
}

#[test]
fn both_arms_keep_their_shape_under_the_alternate_form() {
    // `{:#?}` is what a derived `Debug` on a struct holding one of these
    // reaches for, and it lays out a tuple across lines. An arm that writes its
    // whole rendering as one string has nothing for the formatter to lay out,
    // so the two arms disagree about their shape under one specifier and a
    // pretty-printed struct comes back looking like two different types.
    //
    // The null arm is the one that can go wrong, and it did: naming a fixed
    // word is the obvious way to write it and passes every other assertion in
    // this file.
    let present = format!("{:#?}", MaybeNull::new(NonZeroU8::new(7).unwrap()));
    let absent = format!("{:#?}", MaybeNull::<NonZeroU8>::null());

    for (name, printed) in [("present", &present), ("absent", &absent)] {
        assert!(
            printed.starts_with("MaybeNull(\n"),
            "the {name} arm is not a tuple under the alternate form: {printed:?}"
        );
        assert!(
            printed.ends_with(",\n)"),
            "the {name} arm does not close like a tuple: {printed:?}"
        );
    }
}
