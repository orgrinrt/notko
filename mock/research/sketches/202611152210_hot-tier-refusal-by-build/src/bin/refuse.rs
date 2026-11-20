//! Calls the hot function twice in whatever build this is: once with an input
//! it admits, which is the control, and once with an input it refuses.
//!
//! The caller is written twice under the attribute's own gate, because the
//! function's return type differs between the two arms and no one caller that
//! names the refusal builds in both. The earlier spike
//! `202611140945_profile-arms-adhoc` holds the compiler output for that.

use std::hint::black_box;

use hot_tier_refusal_by_build::admit;

/// The arm the attribute emitted in this build, as the gate decides it.
const ARM: &str = if cfg!(all(feature = "internal", not(debug_assertions))) {
    "Just"
} else {
    "Outcome"
};

#[cfg(any(not(feature = "internal"), debug_assertions))]
fn ask(value: u32, limit: u32) {
    match admit(black_box(value), black_box(limit)) {
        notko::Outcome::Ok(v) => println!("admit({value}, {limit}) returned the value {v}"),
        notko::Outcome::Err(e) => println!("admit({value}, {limit}) returned the refusal {e:?}"),
    }
}

#[cfg(all(feature = "internal", not(debug_assertions)))]
fn ask(value: u32, limit: u32) {
    let v: notko::Just<u32> = admit(black_box(value), black_box(limit));
    println!("admit({value}, {limit}) returned the value {}", v.unwrap());
}

fn main() {
    println!(
        "feature internal: {}, debug assertions: {}, arm emitted: {ARM}",
        cfg!(feature = "internal"),
        cfg!(debug_assertions),
    );
    // The control: an input every build admits.
    ask(3, 5);
    // The case: 9 is over the limit of 5.
    ask(9, 5);
    println!("the process reached its end");
}
