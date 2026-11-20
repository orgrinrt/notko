//! Attempt 1, against the shipped crate, expected not to build.
//!
//! The direct route: a member whose value is itself a borrow of a frame
//! lifetime, and a member whose value is not `Copy` (a resource holding a
//! collection, which is why it would be read by reference at all).

use core::marker::PhantomData;

use notko_hlist::Elem;

/// A resource value that holds a collection, so it is not `Copy`.
pub struct Seq {
    pub items: [u32; 4],
    pub len:   usize,
}

/// A member naming a borrow of the resource for the frame `'a`.
pub struct Res<'a>(PhantomData<&'a Seq>);

impl<'a> Elem for Res<'a> {
    type Value = &'a Seq;
}

/// A member naming the resource itself.
pub struct Store;

impl Elem for Store {
    type Value = Seq;
}

fn main() {}
