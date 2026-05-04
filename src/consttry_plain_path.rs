//! Plain-trait path. Loaded only when feature `const` is disabled.

use core::ops::ControlFlow;

/// Const-callable parallel of `core::ops::Try`. Stable-Rust shape.
pub trait ConstTry {
    /// The "successful" type emerging from the `?` operator.
    type Output;

    /// The "residual" type carrying the early-return information.
    type Residual;

    /// Construct the value back from a successful Output.
    fn from_output(output: Self::Output) -> Self;

    /// Decide whether to short-circuit (Break) or continue (Continue).
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

/// Const-callable parallel of `core::ops::FromResidual`. Stable-Rust shape.
pub trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    /// Construct Self from a residual value.
    fn from_residual(residual: R) -> Self;
}
