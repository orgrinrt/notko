//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Plain-trait HasTrivialCtor declaration. See `ctor.rs` for the
//! cfg-gated module layout rationale.

/// The type ships a no-arg constructor.
///
/// Plain-feature variant: identical surface to the const-feature
/// variant minus the `const` keyword. Consumers on stable Rust opt
/// into this form via `default-features = false`.
pub trait HasTrivialCtor: Sized {
    /// Construct a value with no arguments.
    fn new() -> Self;
}
