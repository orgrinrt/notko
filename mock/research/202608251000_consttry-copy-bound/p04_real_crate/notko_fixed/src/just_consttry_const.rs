//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-variant ConstTry / ConstFromResidual impls on `Just<T>`.
//! Loaded only when feature `const` is enabled.

use core::convert::Infallible;
use core::marker::Destruct;
use core::ops::ControlFlow;

use super::Just;
use crate::{ConstFromResidual, ConstTry};

const impl<T: [const] Destruct> ConstTry for Just<T> {
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

const impl<T: [const] Destruct> ConstFromResidual<Infallible> for Just<T> {
    #[inline]
    fn from_residual(residual: Infallible) -> Self {
        match residual {}
    }
}
