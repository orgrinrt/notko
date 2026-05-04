//! [`Just<T>`]: infallible value wrapper.

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
    ///
    /// Consumes `self`; works for any `T` (no `Copy` bound required).
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Alias for [`Just::into_inner`]. Mirrors the `Maybe::unwrap` /
    /// `Outcome::unwrap` vocabulary for symmetry across the fallibility tiers.
    #[inline]
    pub fn unwrap(self) -> T {
        self.0
    }

    /// Borrow the inner value.
    #[inline]
    pub const fn get(&self) -> &T {
        &self.0
    }

    /// Borrow the inner value as a [`Just`] of a reference.
    #[inline]
    pub const fn as_ref(&self) -> Just<&T> {
        Just(&self.0)
    }

    /// Map the inner value through `f`.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Just<U> {
        Just(f(self.0))
    }

    /// Mutable borrow of the inner value as a [`Just`] of `&mut T`. Mirrors
    /// [`Just::as_ref`].
    #[inline]
    pub fn as_mut(&mut self) -> Just<&mut T> {
        Just(&mut self.0)
    }

    /// Always returns `true`. Mirrors [`crate::Outcome::is_ok`] for tier
    /// symmetry: a `Just<T>` never holds an error.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        true
    }

    /// Always returns `false`. Mirrors [`crate::Outcome::is_err`] for tier
    /// symmetry: a `Just<T>` never holds an error.
    #[inline]
    pub const fn is_err(&self) -> bool {
        false
    }

    /// Always returns `true`. Mirrors [`crate::Maybe::is`] (the
    /// presence-axis predicate) for tier symmetry: a `Just<T>` is always
    /// present.
    #[inline]
    pub const fn is_some(&self) -> bool {
        true
    }

    /// Always returns `false`. Mirrors [`crate::Maybe::isnt`] for tier
    /// symmetry: a `Just<T>` is never absent.
    #[inline]
    pub const fn is_none(&self) -> bool {
        false
    }

    /// Run `f` on the inner value by reference, then pass `self` through.
    /// Tier-symmetric mirror of [`crate::Maybe::inspect`] /
    /// [`crate::Outcome::inspect`].
    #[inline]
    pub fn inspect<F: FnOnce(&T)>(self, f: F) -> Self {
        f(&self.0);
        self
    }

    /// Apply `f` to the inner value, returning the new `Just<U>`. Tier
    /// degenerate of [`crate::Maybe::and_then`] /
    /// [`crate::Outcome::and_then`]: the closure always runs because the
    /// "no value" / "error" branch does not exist.
    #[inline]
    pub fn and_then<U, F: FnOnce(T) -> Just<U>>(self, f: F) -> Just<U> {
        f(self.0)
    }

    /// Return `self`. The fallback is unused because a `Just<T>` never
    /// hits the recovery path. Tier-symmetric mirror of
    /// [`crate::Maybe::or`] / [`crate::Outcome::or`].
    #[inline]
    pub fn or(self, _fallback: Self) -> Self {
        self
    }

    /// Return `self`. The closure is unused. Tier-symmetric mirror of
    /// [`crate::Maybe::or_else`] / [`crate::Outcome::or_else`].
    #[inline]
    pub fn or_else<F: FnOnce() -> Self>(self, _f: F) -> Self {
        self
    }

    /// Return the inner value. The fallback is unused. Tier-symmetric
    /// mirror of [`crate::Maybe::unwrap_or`] /
    /// [`crate::Outcome::unwrap_or`].
    #[inline]
    pub fn unwrap_or(self, _fallback: T) -> T {
        self.0
    }

    /// Return the inner value. The closure is unused. Tier-symmetric
    /// mirror of [`crate::Maybe::unwrap_or_else`] /
    /// [`crate::Outcome::unwrap_or_else`].
    #[inline]
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, _f: F) -> T {
        self.0
    }

    /// Return the inner value. The `Default` bound is documentation only;
    /// `Just<T>` always carries a value. Tier-symmetric mirror of
    /// [`crate::Maybe::unwrap_or_default`] /
    /// [`crate::Outcome::unwrap_or_default`].
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.0
    }

    /// Return the inner value. The message is unused (no panic path).
    /// Tier-symmetric mirror of [`crate::Maybe::expect`] /
    /// [`crate::Outcome::expect`].
    #[inline]
    pub fn expect(self, _msg: &str) -> T {
        self.0
    }

    /// Apply `f` to the inner value. The default is unused because the
    /// "no value" branch does not exist. Tier-symmetric mirror of
    /// [`crate::Maybe::map_or`] / [`crate::Outcome::map_or`].
    #[inline]
    pub fn map_or<U, F: FnOnce(T) -> U>(self, _default: U, f: F) -> U {
        f(self.0)
    }

    /// Apply `f` to the inner value. The default closure is unused.
    /// Tier-symmetric mirror of [`crate::Maybe::map_or_else`] /
    /// [`crate::Outcome::map_or_else`].
    #[inline]
    pub fn map_or_else<U, D: FnOnce() -> U, F: FnOnce(T) -> U>(
        self,
        _default: D,
        f: F,
    ) -> U {
        f(self.0)
    }

    /// Convert to [`crate::Maybe::Is`]. Tier-symmetric mirror of
    /// [`crate::Outcome::ok`]: a `Just<T>` always projects to a present
    /// `Maybe<T>`.
    #[inline]
    pub fn ok(self) -> crate::Maybe<T> {
        crate::Maybe::Is(self.0)
    }

    /// Convert to [`crate::Outcome::Ok`]. The error is unused.
    /// Tier-symmetric mirror of [`crate::Maybe::ok_or`].
    #[inline]
    pub fn ok_or<E>(self, _err: E) -> crate::Outcome<T, E> {
        crate::Outcome::Ok(self.0)
    }

    /// Convert to [`crate::Outcome::Ok`]. The error closure is unused.
    /// Tier-symmetric mirror of [`crate::Maybe::ok_or_else`].
    #[inline]
    pub fn ok_or_else<E, F: FnOnce() -> E>(self, _err: F) -> crate::Outcome<T, E> {
        crate::Outcome::Ok(self.0)
    }

    /// Iterate the single inner value by reference. Mirrors
    /// [`crate::Maybe::iter`].
    #[inline]
    pub fn iter(&self) -> JustIter<&T> {
        JustIter {
            inner: Some(&self.0),
        }
    }
}

/// Borrowing iterator yielding the single inner value.
///
/// Created by [`Just::iter`] or by [`<&Just<T> as IntoIterator>`](IntoIterator).
pub struct JustIter<T> {
    inner: Option<T>,
}

impl<T> Iterator for JustIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.inner.take()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.inner.is_some() as usize;
        (n, Some(n))
    }
}

impl<T> ExactSizeIterator for JustIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.is_some() as usize
    }
}

impl<T> IntoIterator for Just<T> {
    type Item = T;
    type IntoIter = JustIter<T>;

    #[inline]
    fn into_iter(self) -> JustIter<T> {
        JustIter {
            inner: Some(self.0),
        }
    }
}

impl<'a, T> IntoIterator for &'a Just<T> {
    type Item = &'a T;
    type IntoIter = JustIter<&'a T>;

    #[inline]
    fn into_iter(self) -> JustIter<&'a T> {
        self.iter()
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

#[cfg(feature = "const")]
#[path = "just_consttry_const.rs"]
mod consttry_const_impl;

#[cfg(not(feature = "const"))]
#[path = "just_consttry_plain.rs"]
mod consttry_plain_impl;
