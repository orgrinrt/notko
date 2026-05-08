//! Plain-variant ConstTry / ConstFromResidual impls on `Outcome<T, E>`.
//! Loaded only when feature `const` is disabled.

use super::Outcome;
use crate::{ConstFromResidual, ConstTry};
use core::convert::Infallible;
use core::ops::ControlFlow;

impl<T, E> ConstTry for Outcome<T, E> {
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

impl<T, E, F: From<E>> ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, F> {
    #[inline]
    fn from_residual(residual: Outcome<Infallible, E>) -> Self {
        match residual {
            Outcome::Err(err) => Outcome::Err(F::from(err)),
            Outcome::Ok(never) => match never {},
        }
    }
}
