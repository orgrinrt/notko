//! Where an item goes, as a contract rather than a decision.
//!
//! Two traits, because two shapes of receiver exist in this stack and each has a consumer.
//!
//! [`Push<T>`] takes an item through `&mut self` and cannot fail. That is a collector
//! somebody owns: a buffer being filled, a counter, a discard. The engine's work units pass
//! these around, and overflow is the implementor's problem rather than the caller's.
//!
//! [`Emit<T>`] takes an item through `&self` and can fail. That is a destination somebody
//! installed: a log, a serial port, a file, a channel shared between threads. Nobody holds it
//! exclusively, so `&mut self` is not available, and the write can fail for reasons the
//! caller did not cause and usually cannot fix.
//!
//! The fourth corner, shared and infallible, has no consumer and is not written down. The
//! third, exclusive and fallible, is `hilavitkutin_api::BoundedPush`, which lives there
//! because it needs a capacity to report and this crate has no numerics.
//!
//! # Why these are here rather than where they started
//!
//! `Push` and `BulkPush` began as `hilavitkutin_api::capability`'s, and nothing in them is
//! about pipelines: they name no numeric type, they carry no scheduling meaning, and a crate
//! with nothing to do with the engine wanting to accept items had to either depend on the
//! engine or write the same two traits again. The second is what happened, more than once.
//!
//! Until hilavitkutin re-exports these instead of declaring its own, both spellings exist and
//! a crate depending on the engine and on this one sees two same-named traits that do not
//! interconvert. That is a live duplicate, not a completed move, and it is what the engine
//! side of this change has to close.

use crate::outcome::Outcome;

/// Receive one item by value, through an exclusive reference.
///
/// Infallible: overflow is the implementor's problem. A receiver that refuses when full
/// implements `hilavitkutin_api::BoundedPush` alongside this, which reports the refusal and
/// the headroom that caused it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot accept items of type `{T}` via Push",
    note = "Implement `Push<T>` to declare item-acceptance. For a receiver reached through a shared reference, or one whose write can fail, implement `Emit<T>` instead."
)]
pub trait Push<T> {
    /// Accept `item`, for storage, forwarding, counting or discard at the implementor's
    /// discretion.
    fn push(&mut self, item: T);
}

/// Receive a slice of `Copy` items.
///
/// The default pushes per item, in order. Override where the target has a bulk path worth
/// taking: a byte sink backed by `copy_from_slice`, a vector write, a DMA descriptor.
///
/// `Push<T>` is a supertrait because a bulk push with no per-item meaning is not a thing a
/// caller can reason about.
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not implement BulkPush for `{T}`",
    note = "BulkPush extends `Push<T>` with slice-form acceptance. Implement it when the target has a bulk path worth taking; otherwise `Push` alone is the whole contract."
)]
pub trait BulkPush<T>: Push<T> {
    /// Accept `items` as a contiguous slice.
    fn push_bulk(&mut self, items: &[T])
    where
        T: Copy,
    {
        for item in items {
            self.push(*item);
        }
    }
}

/// Receive one item by value, through a shared reference, fallibly.
///
/// The shape of a destination rather than a collector: installed once, reached from anywhere,
/// held exclusively by nobody. A logger, a serial port, a file behind a lock, a channel.
///
/// Both differences from [`Push`] come from that. `&self` because a `&'static` install cannot
/// hand out `&mut`, and an implementor that needs interior mutability declares it where it
/// costs only itself. Fallible because the write can fail for reasons the caller neither
/// caused nor can act on, and a destination that pretends otherwise either panics or lies.
///
/// `Err` is the implementor's, so a sink writing to a file may report the io error and one
/// writing to a fixed buffer may report a unit. A caller that only needs to know whether the
/// item landed ignores it.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot emit items of type `{T}`",
    note = "Implement `Emit<T>` for a destination reached through a shared reference whose write can fail. For a collector somebody owns, implement `Push<T>` instead."
)]
pub trait Emit<T> {
    /// What a failed emit reports.
    type Err;

    /// Accept `item`, or report why it did not land.
    fn emit(&self, item: T) -> Outcome<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    use super::{BulkPush, Emit, Push};
    use crate::outcome::Outcome;

    /// Counts what it is given and stores nothing.
    #[derive(Default)]
    struct Counter {
        items: usize,
    }

    impl Push<u8> for Counter {
        fn push(&mut self, _item: u8) {
            self.items += 1;
        }
    }

    impl BulkPush<u8> for Counter {}

    /// Takes the bulk path instead of the default, so a test can tell which one ran.
    #[derive(Default)]
    struct BulkAware {
        items: usize,
        bulk_calls: usize,
    }

    impl Push<u8> for BulkAware {
        fn push(&mut self, _item: u8) {
            self.items += 1;
        }
    }

    impl BulkPush<u8> for BulkAware {
        fn push_bulk(&mut self, items: &[u8])
        where
            u8: Copy,
        {
            self.bulk_calls += 1;
            self.items += items.len();
        }
    }

    #[test]
    fn the_default_bulk_push_forwards_every_item_to_push() {
        let mut counter = Counter::default();
        counter.push_bulk(&[1, 2, 3, 4]);
        assert_eq!(counter.items, 4);
    }

    #[test]
    fn an_override_is_what_runs_when_one_exists() {
        // Without this, the test above passes whether or not overriding is possible at all:
        // a `push_bulk` that ignored the override and always looped would satisfy it.
        let mut aware = BulkAware::default();
        aware.push_bulk(&[1, 2, 3, 4]);
        assert_eq!(aware.items, 4);
        assert_eq!(aware.bulk_calls, 1);
    }

    /// Accepts nothing, through a shared reference, and says so.
    struct Closed;

    #[derive(Debug, PartialEq, Eq)]
    struct Shut;

    impl Emit<u8> for Closed {
        type Err = Shut;

        fn emit(&self, _item: u8) -> Outcome<(), Self::Err> {
            Outcome::Err(Shut)
        }
    }

    #[test]
    fn emit_reaches_a_destination_through_a_shared_reference() {
        // The point of the trait, rather than of this implementor: `sink` is not `mut`, and
        // the call still compiles. A `Push` bound here would not.
        let sink = Closed;
        let refuse = &sink;
        assert_eq!(refuse.emit(7).unwrap_err(), Shut);
    }

    /// Records what it was given, behind a `Cell`, which is what interior mutability costs an
    /// implementor that needs it.
    struct Recording {
        last: core::cell::Cell<u8>,
    }

    impl Emit<u8> for Recording {
        type Err = core::convert::Infallible;

        fn emit(&self, item: u8) -> Outcome<(), Self::Err> {
            self.last.set(item);
            Outcome::Ok(())
        }
    }

    #[test]
    fn an_implementor_that_needs_to_mutate_pays_for_it_alone() {
        let sink = Recording {
            last: core::cell::Cell::new(0),
        };
        assert!(sink.emit(9).is_ok());
        assert_eq!(sink.last.get(), 9);
    }
}
