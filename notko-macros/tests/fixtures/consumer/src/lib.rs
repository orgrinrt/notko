//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! One `#[profile(Hot)]` function, which is all that is needed to make the
//! macro emit its `cfg` into a crate that is not the one defining it.

use notko::profile;

#[derive(Debug)]
pub struct Oops;

#[profile(Hot)]
pub fn double(x: u32) -> Result<u32, Oops> {
    if x == 0 {
        return Err(Oops);
    }
    Ok(x * 2)
}
