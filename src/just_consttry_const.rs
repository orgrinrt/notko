//! Const-variant ConstTry / ConstFromResidual impls on `Just<T>`.
//! Loaded only when feature `const` is enabled.

use super::Just;
use crate::{ConstFromResidual, ConstTry};
use core::convert::Infallible;
use core::ops::ControlFlow;

const impl<T: Copy> ConstTry for Just<T> {
    type Output = T;
    type Residual = Infallible;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Just(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}

const impl<T: Copy> ConstFromResidual<Infallible> for Just<T> {
    #[inline]
    fn from_residual(residual: Infallible) -> Self {
        match residual {}
    }
}
