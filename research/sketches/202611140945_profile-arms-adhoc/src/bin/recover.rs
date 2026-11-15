//! Ad-hoc spike: what a caller that recovers from a hot function's error does
//! in each arm. Built with `--features probe_is_err`.

#[cfg(feature = "probe_is_err")]
fn main() {
    let arm = if cfg!(all(feature = "internal", not(debug_assertions))) {
        "release arm"
    } else {
        "debug arm"
    };
    println!("{arm}: calling a hot function with an input its check refuses, from a caller that tests is_err()");
    // 9 is over the limit of 5, so the source's `Err` path is taken.
    let v = profile_arms_adhoc::caller_recovers(std::hint::black_box(9), std::hint::black_box(5));
    println!("{arm}: the caller recovered and returned {v}");
}

#[cfg(not(feature = "probe_is_err"))]
fn main() {
    println!("build with --features probe_is_err");
}
