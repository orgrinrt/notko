//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Storage a caller lends, and the filled prefix that comes back.
//!
//! Hand me storage, fill part of it, give me back the part you filled, and
//! if it was too small tell me how much you actually wanted, not just that
//! you failed.
//!
//! It's a question with no domain in it. A bit buffer, a row of edit
//! distances, an argument vector and a line being typed have nothing in
//! common except that all four need to ask exactly this, and each one that
//! answers it privately answers it slightly differently and can't be handed
//! storage built for another.
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
    pub had:    usize,
}

/// A borrowed cursor over lent storage, filled by pushing.
///
/// The append half of the protocol. Something that needs to read and rewrite
/// what it has already written takes the slice from [`Lend`] directly instead;
/// that is a different shape and pretending one covers both would make the
/// common case carry the general case's cost.
///
/// Bare `usize` here and in [`Exhausted`] is deliberate, and it is the one
/// place in this crate where it is. A width with a type of its own lives in a
/// crate that depends on this one, so naming that type here would be a
/// dependency cycle.
#[derive(Debug)]
pub struct Fill<'a, T> {
    slots: &'a mut [T],
    used:  usize,
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
            used:  0,
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
                had:    self.slots.len(),
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
    ///
    /// The refusal is decided from [`ExactSizeIterator::len`], so all-or-nothing
    /// holds exactly as far as that length is honest. `len` is a safe method and
    /// an implementation is free to return the wrong number, which no bound here
    /// can prevent. What is guaranteed regardless is the part that matters: the
    /// write stays inside the lend. An iterator yielding more items than it
    /// reported fills the space that was checked and then stops, with
    /// [`Exhausted`] naming where it stopped, and those items have landed.
    pub fn extend<I>(&mut self, items: I) -> Outcome<(), Exhausted>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        // The length is asked of the iterator that is about to be walked, not
        // of a copy of it, so the number checked and the sequence written are
        // the same object rather than two things related by convention.
        let items = items.into_iter();
        let had = self.slots.len();
        let wanted = self.used.saturating_add(items.len());
        if wanted > had {
            return Outcome::Err(Exhausted {
                wanted,
                had,
            });
        }
        for item in items {
            // `wanted` came from `len()`, which is safe to implement and safe
            // to get wrong, so it decides the refusal and does not decide the
            // bound. The bound is the lend's own length, checked per item.
            if self.used == had {
                return Outcome::Err(Exhausted {
                    wanted: had.saturating_add(1),
                    had,
                });
            }
            self.slots[self.used] = item;
            self.used += 1;
        }
        Outcome::Ok(())
    }

    /// The filled prefix, giving the lend back.
    #[must_use]
    pub fn finish(self) -> &'a [T] {
        &self.slots[.. self.used]
    }

    /// The filled prefix, still writable.
    ///
    /// For a filler that appends and then adjusts what it appended, which is
    /// common enough to be worth having and rare enough not to be the default.
    #[must_use]
    pub fn finish_mut(self) -> &'a mut [T] {
        &mut self.slots[.. self.used]
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
        let mut region: &mut [u8] = &mut backing[4 .. 8];
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
        let region: &mut [u8] = &mut backing[2 .. 6];
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

    /// An `ExactSizeIterator` whose `len` is whatever it was told to say.
    ///
    /// `len` is a safe method with a documented contract and no way to enforce
    /// it, so this is a legal safe implementation and a consumer can write one
    /// by accident. It exists here because the guarantee `extend` makes is
    /// quantified over what `len` reports, and a test that only ever sees an
    /// honest one cannot tell the difference.
    struct Liar<I> {
        inner:   I,
        claimed: usize,
    }

    impl<I: Iterator> Iterator for Liar<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.claimed, Some(self.claimed))
        }
    }

    impl<I: Iterator> ExactSizeIterator for Liar<I> {
        fn len(&self) -> usize {
            self.claimed
        }
    }

    #[test]
    fn an_iterator_claiming_fewer_items_than_it_yields_cannot_write_past_the_lend() {
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);

        // Six items, reported as two. The capacity check passes on the
        // reported number, so the loop is what has to hold the line.
        let liar = Liar {
            inner:   [1u8, 2, 3, 4, 5, 6].into_iter(),
            claimed: 2,
        };

        let Outcome::Err(exhausted) = fill.extend(liar) else {
            panic!("six items cannot fit in a lend of four");
        };
        assert_eq!(exhausted.had, 4);
        assert_eq!(exhausted.wanted, 5);
        // Four landed and nothing beyond the lend was touched, which is the
        // whole claim: a wrong `len` costs the all-or-nothing property and
        // never costs memory outside the slice.
        assert_eq!(fill.finish(), &[1, 2, 3, 4]);
    }

    #[test]
    fn an_iterator_claiming_more_items_than_it_yields_is_refused_without_writing() {
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);

        // Two items, reported as nine. Over-reporting can only cost a refusal
        // of a batch that would have fit, and it must not cost a write.
        let liar = Liar {
            inner:   [1u8, 2].into_iter(),
            claimed: 9,
        };

        let Outcome::Err(exhausted) = fill.extend(liar) else {
            panic!("a claim of nine against a lend of four has to be refused");
        };
        assert_eq!(exhausted.wanted, 9);
        assert_eq!(exhausted.had, 4);
        assert_eq!(fill.finish(), &[]);
    }

    #[test]
    fn an_honest_iterator_that_exactly_fills_the_lend_is_accepted() {
        // The boundary the two liars sit either side of. Off by one here and
        // a full batch would be refused for fitting.
        let mut storage = [0u8; 4];
        let mut fill = Fill::new(&mut storage);
        assert!(fill.extend([1u8, 2, 3, 4]).is_ok());
        assert_eq!(fill.finish(), &[1, 2, 3, 4]);
    }
}
