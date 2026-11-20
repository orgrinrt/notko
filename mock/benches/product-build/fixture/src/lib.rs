//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! A registry of columns and what a consumer asks of the product over it, generated for one point of
//! `product-build`. The source is `product-build-call`'s `source`, at the path the bench sets.

#![no_std]
#![recursion_limit = "1024"]

/// The registry for the point the bench set. Behind `registry`, so a build that sets no source, as
/// `cargo mock bench test` makes, compiles an empty crate.
#[cfg(feature = "registry")]
pub mod registry {
    use notko_hlist::{
        Arr,
        Arrays,
        Cons,
        ContainsAll,
        Elem,
        Empty,
        Has,
        List,
        PCons,
        PNil,
        Place,
        Project,
        Rows,
        Val,
    };

    include!(env!("NOTKO_PRODUCT_FIXTURE_SOURCE"));
}
