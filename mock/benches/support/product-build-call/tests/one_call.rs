//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Real calls, end to end: a prepared and built fixture passes the verdict, with its registry and empty alike, and a
//! second build with no clean between, which compiles nothing, fails it, and so does the empty fixture recorded as
//! though it had asked for the registry, since cargo says it built without the feature. The controls are what show
//! the verdict can refuse at all.

use product_build_call::{Fixture, build, calls, prepare, record, reset_log, verdict};

#[test]
fn a_call_from_clean_passes_and_an_unclean_or_mislabelled_one_fails() {
    reset_log();

    for fixture in [Fixture::Registry, Fixture::Empty] {
        let absent = prepare(8);
        let out = build(8, fixture);
        record(8, fixture, &out, absent);
    }
    let good = calls();
    assert_eq!(good.len(), 2);
    assert!(verdict(&good, 8).is_ok(), "{good:?}");

    let again = build(8, Fixture::Empty);
    record(8, Fixture::Empty, &again, true);
    let unclean = calls();
    assert_eq!(unclean.len(), 3);
    assert!(verdict(&unclean[2 ..], 8).is_err(), "{unclean:?}");

    let absent = prepare(8);
    let empty = build(8, Fixture::Empty);
    record(8, Fixture::Registry, &empty, absent);
    let mislabelled = calls();
    assert_eq!(mislabelled.len(), 4);
    assert!(verdict(&mislabelled[3 ..], 8).is_err(), "{mislabelled:?}");
    reset_log();
}
