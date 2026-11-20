//! Ad-hoc spike: is a custom tier file an input cargo knows about?
//!
//! `traced` is under the custom tier `Trace`, defined in
//! `notko-optimisers/Trace.rs`. `caller` is written against the `Outcome` that
//! a tier based on `Cold` gives, so it stops compiling the moment the tier is
//! based on `Warm`, which gives a `Maybe`. The steps are in `FINDINGS.md`.

// Step 3 of the spike adds this line and nothing else.
use notko::profile;

#[derive(Debug)]
pub struct Oops;

#[profile(Trace)]
pub fn traced(x: u32) -> Result<u32, Oops> {
    if x > 7 {
        return Err(Oops);
    }
    Ok(x + 1)
}

pub fn caller(x: u32) -> u32 {
    match traced(x) {
        notko::Outcome::Ok(v) => v,
        notko::Outcome::Err(_) => 0,
    }
}
