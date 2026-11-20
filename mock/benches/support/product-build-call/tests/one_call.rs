//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Real calls, end to end: a prepared and built fixture passes the verdict, with its registry and empty alike, and a
//! second build with no clean between, which compiles nothing, fails it. The control is what shows the verdict can
//! refuse at all.

use product_build_call::{Fixture, build, calls, prepare, record, reset_log, verdict};

#[test]
fn a_call_from_clean_passes_and_a_build_without_the_clean_fails() {
    reset_log();

    for fixture in [Fixture::Registry, Fixture::Empty] {
        let absent = prepare(8);
        let out = build(8, fixture);
        record(8, fixture, &out, absent);
    }
    let good = calls();
    assert_eq!(good.len(), 2);
    assert!(verdict(&good, 8).is_ok(), "{good:?}");
    assert!(verdict(&good, 16).is_err(), "{good:?}");

    let again = build(8, Fixture::Empty);
    record(8, Fixture::Empty, &again, true);
    let all = calls();
    assert_eq!(all.len(), 3);
    assert!(verdict(&all, 8).is_err(), "{all:?}");
    reset_log();
}
