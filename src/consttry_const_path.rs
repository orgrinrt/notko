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
/// # Divergence from `core::ops::FromResidual`
///
/// The non-const variant of `ConstFromResidual` mirrors core's
/// shape exactly, including the `F: From<E>` cross-error
/// conversion. The const variant on `Outcome<T, E>` (in
/// `outcome.rs`) does NOT include the `F: From<E>` conversion
/// case because const trait bounds with `From` are not yet
/// stable. Code authored against the const path cannot do the
/// implicit error conversion that `?`-on-`Result` consumers
/// expect; explicit `Outcome::Err(e.into())` is the workaround.
///
/// Tracking issue for `From` in const trait bounds:
/// rust-lang/rust#143874 (and adjacent const-trait stabilization
/// work). When `From` lifts, the const variant's bound matches
/// core's shape and this divergence note can be removed.
pub const trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    /// Construct Self from a residual value.
    fn from_residual(residual: R) -> Self;
}
