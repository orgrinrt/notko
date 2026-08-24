//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Storage a caller lends, and the filled prefix that comes back.
//!
//! Four places in this stack had grown their own version of one protocol:
//! hand me storage, fill part of it, give me back the part you filled, and if
//! it was too small tell me how much you wanted rather than only that you
//! failed. Each spelled it differently, each invented its own failure type,
//! and none of them could be handed storage built for another.
//!
//! It belongs here because it is the shape of a question with no domain in it.
//! A bit buffer, a row of edit distances, an argument vector and a line being
//! typed are unrelated things that all need to ask it, and the crate everything
//! already depends on is the only place they can share an answer.
//!
//! # What this buys, and what it does not
//!
//! It buys one named protocol instead of four, a failure that says how much was
//! wanted against how much was available, and a filled result that remembers
//! which lend it came from.
//!
//! It does not buy elided bounds checks. A prefix known to be no longer than
//! its capacity says nothing about whether a particular index is inside the
//! part that was filled, so indexing is checked like any other slice. Claiming
//! otherwise would be the sort of thing that reads well and is false.

use crate::outcome::Outcome;

/// Storage lent to something that will fill part of it.
///
/// Implemented for fixed arrays and for a mutable slice, which covers a stack
/// array, a slice out of an arena, and a region handed over by an allocator a
/// caller already holds. Anything else implements it in three lines.
///
/// Deliberately not generic over how the storage was obtained. That question
/// belongs to whoever obtained it, and a contract that answered it here would
/// be deciding on their behalf where memory comes from.
pub trait Lend<T> {
    /// The storage, in full.
    fn lend(&mut self) -> &mut [T];
}

impl<T, const N: usize> Lend<T> for [T; N] {
    fn lend(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Lend<T> for [T] {
    fn lend(&mut self) -> &mut [T] {
        self
    }
}

impl<T, L: Lend<T> + ?Sized> Lend<T> for &mut L {
    fn lend(&mut self) -> &mut [T] {
        (**self).lend()
    }
}

/// What a lend could not hold.
///
/// Carries both numbers because "it did not fit" leaves a caller guessing how
/// much larger to make it, and guessing is what turns one failed run into
/// several.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exhausted {
    /// How much was needed.
    ///
    /// The full requirement where it is known. A filler that discovers the
    /// shortfall partway through reports what it had reached, which is a lower
    /// bound and is documented as one, because a caller doubling from a lower
    /// bound converges and a caller told nothing does not.
    pub wanted: usize,
    /// How much the lend held.
    pub had: usize,
}

/// A borrowed cursor over lent storage, filled by pushing.
///
/// The append half of the protocol. Something that needs to read and rewrite
/// what it has already written takes the slice from [`Lend`] directly instead;
/// that is a different shape and pretending one covers both would make the
/// common case carry the general case's cost.
///
/// Bare `usize` here and in [`Exhausted`] is the definition-site exception this
/// crate exists to be. A typed width lives in a crate that depends on this one,
/// so naming one here would be a cycle.
#[derive(Debug)]
pub struct Fill<'a, T> {
    slots: &'a mut [T],
    used: usize,
}

impl<'a, T> Fill<'a, T> {
    /// Begin filling `storage`.
    ///
    /// `?Sized` because [`Lend`] is implemented for `[T]` as well as for an array, and
    /// without it that impl is unreachable here: `impl Lend<T>` implies `Sized`, so a
    /// `&mut [T]` was refused and reaching the slice impl meant lending a `&mut` to a
    /// `&mut [T]`.
    pub fn new(storage: &'a mut (impl Lend<T> + ?Sized)) -> Self {
        Self {
            slots: storage.lend(),
            used: 0,
        }
    }

    /// How much the lend holds.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// How much has been filled.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.used
    }

    /// Whether nothing has been filled.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// Append one item.
    ///
    /// Refuses rather than truncating. A filler that silently dropped what did
    /// not fit would hand back a result that looks complete, which is the one
    /// outcome worse than failing.
    pub fn push(&mut self, item: T) -> Outcome<(), Exhausted> {
        if self.used >= self.slots.len() {
            return Outcome::Err(Exhausted {
                // A lower bound: one more than what fit is what is known here,
                // and inventing the true total would mean knowing how much the
                // caller still intends to push.
                wanted: self.used + 1,
                had: self.slots.len(),
            });
        }
        self.slots[self.used] = item;
        self.used += 1;
        Outcome::Ok(())
    }

    /// Append every item, or refuse without writing any of them.
    ///
    /// All or nothing, because a partial fill from a batch is a result nobody
    /// can use: the caller cannot tell which items landed without counting, and
    /// the count it would have to trust is the one that just failed.
    pub fn extend<I>(&mut self, items: I) -> Outcome<(), Exhausted>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        // The length comes from the iterator that is about to be walked, rather than from
        // a clone of it. Counting a clone and then walking the original ties the two
        // together only by convention: `Clone` promises nothing about sequence length, so a
        // hand-written one that under-reports made the loop below write past the end. It
        // also cost a second full traversal, and excluded every iterator that is not
        // `Clone`.
        //
        // `T: Copy` went with it. It was never needed: `push` performs the same assignment
        // without it, so the bound only served to stop `Fill<'_, String>` from using this.
        let items = items.into_iter();
        let wanted = self.used + items.len();
        if wanted > self.slots.len() {
            return Outcome::Err(Exhausted {
                wanted,
                had: self.slots.len(),
            });
        }
        for item in items {
            self.slots[self.used] = item;
            self.used += 1;
        }
        Outcome::Ok(())
    }

    /// The filled prefix, giving the lend back.
    #[must_use]
    pub fn finish(self) -> &'a [T] {
        &self.slots[..self.used]
    }

    /// The filled prefix, still writable.
    ///
    /// For a filler that appends and then adjusts what it appended, which is
    /// common enough to be worth having and rare enough not to be the default.
    #[must_use]
    pub fn finish_mut(self) -> &'a mut [T] {
        &mut self.slots[..self.used]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fill_hands_back_only_what_was_filled() {
        let mut storage = [0u8; 8];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.push(1).is_ok());
        assert!(fill.push(2).is_ok());
        assert_eq!(fill.len(), 2);
        assert_eq!(fill.capacity(), 8);
        assert_eq!(fill.finish(), &[1, 2]);
    }

    #[test]
    fn a_full_lend_refuses_rather_than_truncating() {
        // Truncating would hand back a result that looks complete, which is the
        // one outcome worse than failing.
        let mut storage = [0u8; 2];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.push(1).is_ok());
        assert!(fill.push(2).is_ok());

        let Outcome::Err(exhausted) = fill.push(3) else {
            panic!("a lend of two took a third item");
        };
        assert_eq!(exhausted.had, 2);
        assert_eq!(exhausted.wanted, 3);
        // The refusal left what was already there alone.
        assert_eq!(fill.finish(), &[1, 2]);
    }

    #[test]
    fn a_batch_that_does_not_fit_writes_nothing() {
        // The property `extend` exists for. A partial batch is unusable: the
        // caller cannot tell which items landed except by trusting the count
        // that just failed.
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.push(9).is_ok());

        let Outcome::Err(exhausted) = fill.extend([1, 2, 3, 4]) else {
            panic!("four items fit in the three slots left");
        };
        assert_eq!(exhausted.wanted, 5);
        assert_eq!(exhausted.had, 4);
        assert_eq!(fill.finish(), &[9], "a refused batch left something behind");
    }

    #[test]
    fn a_batch_that_fits_lands_whole() {
        // The control. Every assertion above would hold for an `extend` that
        // never wrote anything at all.
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.extend([1, 2, 3]).is_ok());
        assert_eq!(fill.finish(), &[1, 2, 3]);
    }

    #[test]
    fn an_empty_lend_holds_nothing_and_says_so() {
        let mut storage = [0u8; 0];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.is_empty());
        assert_eq!(fill.capacity(), 0);

        let Outcome::Err(exhausted) = fill.push(1) else {
            panic!("a lend of nothing took an item");
        };
        assert_eq!(exhausted.had, 0);
        assert_eq!(exhausted.wanted, 1);
        assert!(fill.finish().is_empty());
    }

    #[test]
    fn a_slice_lends_as_well_as_an_array() {
        // The point of the trait: storage from an arena and storage on the
        // stack are the same to a filler.
        let mut backing = [0u8; 16];
        let mut region: &mut [u8] = &mut backing[4..8];
        let mut fill = Fill::new(&mut region);
        assert_eq!(fill.capacity(), 4);
        assert!(fill.extend([7, 7, 7, 7]).is_ok());
        assert!(fill.push(7).is_err(), "the region took more than it has");
    }

    #[test]
    fn a_non_copy_item_can_be_extended_as_well_as_pushed() {
        // `extend` used to require `T: Copy`, which `push` does not, so a `Fill` of
        // anything owning could be filled one item at a time and not in a batch. Nothing
        // about the write needs the bound.
        #[derive(Debug, PartialEq)]
        struct NotCopy(u8);

        let mut storage = [NotCopy(0), NotCopy(0), NotCopy(0)];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.extend([NotCopy(1), NotCopy(2)]).is_ok());
        assert_eq!(fill.finish(), &[NotCopy(1), NotCopy(2)]);
    }

    #[test]
    fn a_bare_slice_is_lent_directly() {
        // `Fill::new` took `impl Lend<T>`, which implies `Sized`, so the `Lend for [T]`
        // impl could not be reached through it: lending a slice meant lending a `&mut` to
        // a `&mut [T]`. This is the shape the module documentation describes.
        let mut backing = [0u8; 8];
        let region: &mut [u8] = &mut backing[2..6];
        let mut fill = Fill::new(region);
        assert_eq!(fill.capacity(), 4);
        assert!(fill.extend([1, 2, 3, 4]).is_ok());
        assert!(fill.push(5).is_err());
    }

    #[test]
    fn a_fill_can_be_adjusted_after_appending() {
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.extend([1, 2, 3]).is_ok());
        let written = fill.finish_mut();
        written[0] = 9;
        assert_eq!(written, &[9, 2, 3]);
    }
}
