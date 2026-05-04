//! [`Outcome<T, E>`] — fallibility (replaces `Result<T, E>`).

use core::fmt;

/// Fallible computation outcome.
///
/// Replaces `core::result::Result<T, E>` in the hilavitkutin stack's public
/// APIs. `repr(C)` so layout is stable across ABI boundaries; no dependency
/// on `core::result`.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Outcome<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Outcome<T, E> {
    /// `true` if this is [`Outcome::Ok`].
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Outcome::Ok(_))
    }

    /// `true` if this is [`Outcome::Err`].
    #[inline]
    pub const fn is_err(&self) -> bool {
        matches!(self, Outcome::Err(_))
    }

    /// Extract the ok value; panic if this is [`Outcome::Err`].
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Outcome::Ok(value) => value,
            Outcome::Err(err) => {
                panic!("called `Outcome::unwrap` on an `Err` value: {err:?}")
            },
        }
    }

    /// Extract the ok value or return `fallback`.
    #[inline]
    pub fn unwrap_or(self, fallback: T) -> T {
        match self {
            Outcome::Ok(value) => value,
            Outcome::Err(_) => fallback,
        }
    }

    /// Map the ok value.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Outcome<U, E> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(f(value)),
            Outcome::Err(err) => Outcome::Err(err),
        }
    }

    /// Map the error value.
    #[inline]
    pub fn map_err<U, F: FnOnce(E) -> U>(self, f: F) -> Outcome<T, U> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(value),
            Outcome::Err(err) => Outcome::Err(f(err)),
        }
    }
}

impl<T, E: fmt::Debug> fmt::Debug for Outcome<T, E>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Ok(value) => f.debug_tuple("Ok").field(value).finish(),
            Outcome::Err(err) => f.debug_tuple("Err").field(err).finish(),
        }
    }
}

#[cfg(feature = "try_trait_v2")]
mod try_impl {
    use super::Outcome;
    use core::convert::Infallible;
    use core::ops::{ControlFlow, FromResidual, Try};

    impl<T, E> Try for Outcome<T, E> {
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

    impl<T, E, F: From<E>> FromResidual<Outcome<Infallible, E>> for Outcome<T, F> {
        #[inline]
        fn from_residual(residual: Outcome<Infallible, E>) -> Self {
            match residual {
                Outcome::Err(err) => Outcome::Err(F::from(err)),
                Outcome::Ok(never) => match never {},
            }
        }
    }
}

#[cfg(feature = "const")]
#[path = "outcome_consttry_const.rs"]
mod consttry_const_impl;

#[cfg(not(feature = "const"))]
#[path = "outcome_consttry_plain.rs"]
mod consttry_plain_impl;
