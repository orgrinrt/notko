//! [`Maybe<T>`] — presence (replaces `Option<T>`).

use core::fmt;

/// Presence. Either carries a value ([`Maybe::Is`]) or doesn't ([`Maybe::Isnt`]).
///
/// Replaces `core::option::Option<T>` in the hilavitkutin stack's public APIs.
/// `repr(C)` so layout is stable across ABI boundaries; no dependency on
/// `core::option`.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Maybe<T> {
    Is(T),
    Isnt,
}

impl<T> Maybe<T> {
    /// `true` if this is [`Maybe::Is`].
    #[inline]
    pub const fn is(&self) -> bool {
        matches!(self, Maybe::Is(_))
    }

    /// `true` if this is [`Maybe::Isnt`].
    #[inline]
    pub const fn isnt(&self) -> bool {
        matches!(self, Maybe::Isnt)
    }

    /// Convert by value into a `&Maybe<&T>` (zero-cost).
    #[inline]
    pub const fn as_ref(&self) -> Maybe<&T> {
        match self {
            Maybe::Is(value) => Maybe::Is(value),
            Maybe::Isnt => Maybe::Isnt,
        }
    }

    /// Extract the inner value; panic if absent.
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Maybe::Is(value) => value,
            Maybe::Isnt => panic!("called `Maybe::unwrap` on an `Isnt` value"),
        }
    }

    /// Extract the inner value or return `fallback`.
    #[inline]
    pub fn unwrap_or(self, fallback: T) -> T {
        match self {
            Maybe::Is(value) => value,
            Maybe::Isnt => fallback,
        }
    }

    /// Map the inner value if present.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Maybe<U> {
        match self {
            Maybe::Is(value) => Maybe::Is(f(value)),
            Maybe::Isnt => Maybe::Isnt,
        }
    }
}

impl<T> Default for Maybe<T> {
    #[inline]
    fn default() -> Self {
        Maybe::Isnt
    }
}

impl<T: fmt::Debug> fmt::Debug for Maybe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Maybe::Is(value) => f.debug_tuple("Is").field(value).finish(),
            Maybe::Isnt => f.write_str("Isnt"),
        }
    }
}

#[cfg(feature = "try_trait_v2")]
mod try_impl {
    use super::Maybe;
    use core::convert::Infallible;
    use core::ops::{ControlFlow, FromResidual, Try};

    impl<T> Try for Maybe<T> {
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

    impl<T> FromResidual<Maybe<Infallible>> for Maybe<T> {
        #[inline]
        fn from_residual(residual: Maybe<Infallible>) -> Self {
            match residual {
                Maybe::Isnt => Maybe::Isnt,
                Maybe::Is(never) => match never {},
            }
        }
    }
}
