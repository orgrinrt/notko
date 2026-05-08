//! Const-variant ConstTry / ConstFromResidual impls on `Outcome<T, E>`.
//! Loaded only when feature `const` is enabled.

use super::Outcome;
use crate::{ConstFromResidual, ConstTry};
use core::convert::Infallible;
use core::ops::ControlFlow;

impl<T: Copy, E: Copy> const ConstTry for Outcome<T, E> {
    type Output = T;
    type Residual = Outcome<Infallible, E>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Outcome::Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Outcome::Ok(value) => ControlFlow::Continue(value),
            Outcome::Err(err) => ControlFlow::Break(Outcome::Err(err)),
        }
    }
}

// `From` in const trait bounds is not yet stable; ConstFromResidual on Outcome
// omits the `F: From<E>` conversion variant in the const path. Consumers
// needing `E -> F` conversion through ConstFromResidual reach for the
// non-const path via `default-features = false`. See the `# Divergence`
// section on `ConstFromResidual`'s declaration in `consttry_const_path.rs`.
impl<T: Copy, E: Copy> const ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, E> {
    #[inline]
    fn from_residual(residual: Outcome<Infallible, E>) -> Self {
        match residual {
            Outcome::Err(err) => Outcome::Err(err),
            Outcome::Ok(never) => match never {},
        }
    }
}
