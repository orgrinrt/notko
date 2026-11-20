//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Plain-variant `Deref` and borrow conversions on `Just<T>`.
//! Loaded only when feature `const` is disabled.
//!
//! `Just<T>` is `#[repr(transparent)]` over a single `T` and documents itself
//! as a no-op extraction of that value, so the deref target is total: there is
//! no absent case for it to panic on. `Maybe` and `Outcome` deliberately get no
//! `Deref`, because theirs would have to invent a value for the empty variant.

use super::Just;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

impl<T> Deref for Just<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Just<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Borrow<T> for Just<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> BorrowMut<T> for Just<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
