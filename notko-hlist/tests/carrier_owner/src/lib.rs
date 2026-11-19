//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! A per-member carrier with no members of its own, for the test crate to
//! state forms on for members it owns.
#![no_std]

use core::marker::PhantomData;

use notko_hlist::{PerMember, Shape};

/// What a unit reads for the frame `'f`.
pub struct Read<'f>(PhantomData<&'f ()>);

impl<'f> Shape for Read<'f> {
    type Is = PerMember;
}
