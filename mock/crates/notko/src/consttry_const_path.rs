//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-trait path. Loaded only when feature `const` is enabled.

use core::ops::ControlFlow;

/// Const-callable parallel of `core::ops::Try`.
///
/// Mirrors `core::ops::Try`'s shape exactly: associated `Output` and
/// `Residual` types, plus `from_output` and `branch` methods. Reuses
/// `core::ops::ControlFlow` directly (its enum constructors are
/// stable-const).
pub const trait ConstTry {
    /// The "successful" type emerging from the `?` operator.
    type Output;

    /// The "residual" type carrying the early-return information.
    type Residual;

    /// Construct the value back from a successful Output.
    fn from_output(output: Self::Output) -> Self;

    /// Decide whether to short-circuit (Break) or continue (Continue).
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// Const-callable parallel of `core::ops::FromResidual`.
///
/// Mirrors core's shape, the `F: From<E>` cross-error conversion included, in
/// both configurations. So an error converts into another on the way out of a
/// const context exactly as it does at runtime, and nothing here asks for an
/// explicit `Outcome::Err(e.into())` that the runtime path would not.
pub const trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    /// Construct Self from a residual value.
    fn from_residual(residual: R) -> Self;
}
