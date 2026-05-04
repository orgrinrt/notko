//! [`Just<T>`] — infallible value wrapper.

use core::fmt;

/// Infallible value wrapper. The value is always present.
///
/// `#[repr(transparent)]` so it has zero cost over `T` at runtime.
///
/// Use as the hot-path type for positions that logically could be
/// [`crate::Outcome`] / [`crate::Maybe`] but whose error / absent variant
/// has been proven unreachable (codegen, reified invariants).
///
/// Implements [`core::ops::Try`] (with `Residual = core::convert::Infallible`)
/// when the `try_trait_v2` feature is enabled, so `?` on a `Just<T>` is a
/// no-op extraction of the inner value.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct Just<T>(T);

impl<T> Just<T> {
    /// Wrap a value. Always succeeds.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Unwrap to the inner value. Always succeeds.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Borrow the inner value.
    #[inline]
    pub const fn get(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Just<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Just<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Just").field(&self.0).finish()
    }
}

#[cfg(feature = "try_trait_v2")]
mod try_impl {
    use super::Just;
    use core::convert::Infallible;
    use core::ops::{ControlFlow, FromResidual, Try};

    impl<T> Try for Just<T> {
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

    impl<T> FromResidual<Infallible> for Just<T> {
        #[inline]
        fn from_residual(residual: Infallible) -> Self {
            match residual {}
        }
    }
}

mod consttry_impl {
    use super::Just;
    use crate::{ConstFromResidual, ConstTry};
    use core::convert::Infallible;
    use core::ops::ControlFlow;

    #[cfg(feature = "const")]
    impl<T: Copy> const ConstTry for Just<T> {
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

    #[cfg(not(feature = "const"))]
    impl<T> ConstTry for Just<T> {
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

    #[cfg(feature = "const")]
    impl<T: Copy> const ConstFromResidual<Infallible> for Just<T> {
        #[inline]
        fn from_residual(residual: Infallible) -> Self {
            match residual {}
        }
    }

    #[cfg(not(feature = "const"))]
    impl<T> ConstFromResidual<Infallible> for Just<T> {
        #[inline]
        fn from_residual(residual: Infallible) -> Self {
            match residual {}
        }
    }
}
