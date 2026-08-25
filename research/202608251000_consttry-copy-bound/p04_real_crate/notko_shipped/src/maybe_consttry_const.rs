//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-variant ConstTry / ConstFromResidual impls on `Maybe<T>`.
//! Loaded only when feature `const` is enabled.

use super::Maybe;
use crate::{ConstFromResidual, ConstTry};
use core::convert::Infallible;
use core::ops::ControlFlow;

const impl<T: Copy> ConstTry for Maybe<T> {
    type Output = T;
    type Residual = Maybe<Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Maybe::Is(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Maybe::Is(value) => ControlFlow::Continue(value),
            Maybe::Isnt => ControlFlow::Break(Maybe::Isnt),
        }
    }
}

const impl<T: Copy> ConstFromResidual<Maybe<Infallible>> for Maybe<T> {
    #[inline]
    fn from_residual(residual: Maybe<Infallible>) -> Self {
        match residual {
            Maybe::Isnt => Maybe::Isnt,
            Maybe::Is(never) => match never {},
        }
    }
}
