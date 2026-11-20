//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital

//! One real call, end to end: the record of a prepared and built fixture passes the verdict, and a second build with
//! no clean between, which compiles nothing, fails it. The control is what shows the verdict can refuse at all.

use product_build_call::{build, calls, prepare, record, reset_log, verdict};

#[test]
fn a_call_from_clean_passes_and_a_build_without_the_clean_fails() {
    // SAFETY: the only test in this binary, so no other thread reads the environment.
    unsafe { std::env::set_var("CARGO_TARGET_DIR", env!("CARGO_TARGET_TMPDIR")) };
    reset_log();

    let absent = prepare(8);
    let out = build(8);
    record(8, &out, absent);
    let good = calls();
    assert_eq!(good.len(), 1);
    assert!(verdict(&good).is_ok(), "{:?}", good);

    let again = build(8);
    record(8, &again, absent);
    let both = calls();
    assert_eq!(both.len(), 2);
    assert!(verdict(&both).is_err(), "{:?}", both);
    reset_log();
}
