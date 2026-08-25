//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Const-variant ConstTry / ConstFromResidual impls on `Outcome<T, E>`.
//! Loaded only when feature `const` is enabled.

use super::Outcome;
use crate::{ConstFromResidual, ConstTry};
use core::convert::Infallible;
use core::ops::ControlFlow;

const impl<T: Copy, E: Copy> ConstTry for Outcome<T, E> {
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
const impl<T: Copy, E: Copy> ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, E> {
    #[inline]
    fn from_residual(residual: Outcome<Infallible, E>) -> Self {
        match residual {
            Outcome::Err(err) => Outcome::Err(err),
            Outcome::Ok(never) => match never {},
        }
    }
}
