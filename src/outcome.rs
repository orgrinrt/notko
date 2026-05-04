//! [`Outcome<T, E>`]: fallibility (replaces `Result<T, E>`).

use core::fmt;

/// Fallible computation outcome.
///
/// Replaces `core::result::Result<T, E>` in the hilavitkutin stack's
/// public APIs. Layout is platform-standard Rust repr; no `repr(C)`
/// forcing. Two-payload `repr(C)` enums have implementation-defined
/// edge cases at the C ABI boundary, and the documented guidance
/// for FFI-critical result layouts is to wrap the payload in a
/// dedicated `#[repr(C)]` struct rather than rely on Outcome's
/// default. See `lib.rs` module-level doc.
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

    /// Convert `&Outcome<T, E>` to `Outcome<&T, &E>`.
    #[inline]
    pub const fn as_ref(&self) -> Outcome<&T, &E> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(value),
            Outcome::Err(err) => Outcome::Err(err),
        }
    }

    /// Convert `&mut Outcome<T, E>` to `Outcome<&mut T, &mut E>`.
    #[inline]
    pub fn as_mut(&mut self) -> Outcome<&mut T, &mut E> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(value),
            Outcome::Err(err) => Outcome::Err(err),
        }
    }

    /// Convert into a [`crate::Maybe`], discarding any error.
    #[inline]
    pub fn ok(self) -> crate::Maybe<T> {
        match self {
            Outcome::Ok(value) => crate::Maybe::Is(value),
            Outcome::Err(_) => crate::Maybe::Isnt,
        }
    }

    /// Convert into a [`crate::Maybe`] of the error, discarding any ok value.
    #[inline]
    pub fn err(self) -> crate::Maybe<E> {
        match self {
            Outcome::Ok(_) => crate::Maybe::Isnt,
            Outcome::Err(err) => crate::Maybe::Is(err),
        }
    }

    /// Extract the error; panic if [`Outcome::Ok`].
    #[inline]
    #[track_caller]
    pub fn unwrap_err(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Outcome::Ok(value) => {
                panic!("called `Outcome::unwrap_err` on an `Ok` value: {value:?}")
            },
            Outcome::Err(err) => err,
        }
    }

    /// Extract the ok value or compute the fallback from the error.
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T {
        match self {
            Outcome::Ok(value) => value,
            Outcome::Err(err) => f(err),
        }
    }

    /// Extract the ok value or `T::default()`.
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Outcome::Ok(value) => value,
            Outcome::Err(_) => T::default(),
        }
    }

    /// Unwrap the ok value with a custom panic message.
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Outcome::Ok(value) => value,
            Outcome::Err(err) => panic!("{msg}: {err:?}"),
        }
    }

    /// Unwrap the error with a custom panic message.
    #[inline]
    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Outcome::Ok(value) => panic!("{msg}: {value:?}"),
            Outcome::Err(err) => err,
        }
    }

    /// Map the ok value to `U`, or return `default` if [`Outcome::Err`].
    #[inline]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, default: U, f: F) -> U {
        match self {
            Outcome::Ok(value) => f(value),
            Outcome::Err(_) => default,
        }
    }

    /// Map the ok value to `U`, or compute the default from the error.
    #[inline]
    pub fn map_or_else<U, D: FnOnce(E) -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            Outcome::Ok(value) => f(value),
            Outcome::Err(err) => default(err),
        }
    }

    /// Return `other` if [`Outcome::Ok`], else propagate the error.
    #[inline]
    pub fn and<U>(self, other: Outcome<U, E>) -> Outcome<U, E> {
        match self {
            Outcome::Ok(_) => other,
            Outcome::Err(err) => Outcome::Err(err),
        }
    }

    /// Chain another fallible computation with the ok value.
    #[inline]
    pub fn and_then<U, F: FnOnce(T) -> Outcome<U, E>>(self, f: F) -> Outcome<U, E> {
        match self {
            Outcome::Ok(value) => f(value),
            Outcome::Err(err) => Outcome::Err(err),
        }
    }

    /// Return `self` if [`Outcome::Ok`], else return `other`.
    #[inline]
    pub fn or<F>(self, other: Outcome<T, F>) -> Outcome<T, F> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(value),
            Outcome::Err(_) => other,
        }
    }

    /// Return `self` if [`Outcome::Ok`], else compute a recovery from the error.
    #[inline]
    pub fn or_else<F, O: FnOnce(E) -> Outcome<T, F>>(self, f: O) -> Outcome<T, F> {
        match self {
            Outcome::Ok(value) => Outcome::Ok(value),
            Outcome::Err(err) => f(err),
        }
    }

    /// `true` if [`Outcome::Ok`] and `predicate(value)` returns true.
    #[inline]
    pub fn is_ok_and<P: FnOnce(T) -> bool>(self, predicate: P) -> bool {
        match self {
            Outcome::Ok(value) => predicate(value),
            Outcome::Err(_) => false,
        }
    }

    /// `true` if [`Outcome::Err`] and `predicate(err)` returns true.
    #[inline]
    pub fn is_err_and<P: FnOnce(E) -> bool>(self, predicate: P) -> bool {
        match self {
            Outcome::Ok(_) => false,
            Outcome::Err(err) => predicate(err),
        }
    }

    /// Run `f` on the ok value if present; pass through unchanged.
    #[inline]
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Outcome::Ok(ref value) = self {
            f(value);
        }
        self
    }

    /// Run `f` on the error value if present; pass through unchanged.
    #[inline]
    pub fn inspect_err<F: FnOnce(&E)>(self, f: F) -> Self {
        if let Outcome::Err(ref err) = self {
            f(err);
        }
        self
    }
}

impl<T, E> From<Result<T, E>> for Outcome<T, E> {
    #[inline]
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(value) => Outcome::Ok(value),
            Err(err) => Outcome::Err(err),
        }
    }
}

impl<T, E> From<Outcome<T, E>> for Result<T, E> {
    #[inline]
    fn from(o: Outcome<T, E>) -> Self {
        match o {
            Outcome::Ok(value) => Ok(value),
            Outcome::Err(err) => Err(err),
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
