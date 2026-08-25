//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-trait HasTrivialCtor declaration. See `ctor.rs` for the
//! cfg-gated module layout rationale.

/// The type ships a no-arg const constructor.
///
/// A type implementing `HasTrivialCtor` is saying it has an associated
/// `const fn new() -> Self` you can call in a const context with no
/// arguments. Handy for markers and phantom wrappers, and for anything
/// else whose construction is a typestate signal and carries no runtime
/// data.
///
/// The point is that `Type::<T>::new()` means the same thing everywhere,
/// instead of every wrapper inventing its own spelling.
pub const trait HasTrivialCtor: Sized {
    /// Construct a value with no arguments.
    fn new() -> Self;
}
