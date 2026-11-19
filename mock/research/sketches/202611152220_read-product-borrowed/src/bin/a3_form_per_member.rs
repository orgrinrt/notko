//! Attempt 3, against the shipped crate, expected not to build.
//!
//! Attempt 2 holds one form for every member, a shared borrow of the value.
//! A read list wants a form per member: one member by value, one as a window
//! on a column, one by reference. That needs the carrier to ask the member
//! what it reads as. Three ways of writing that carrier, each its own type.

use core::marker::PhantomData;

use notko_hlist::{Carrier, Elem};

/// What a member reads as, for a frame.
pub trait Readable: Elem {
    type Rd<'a>: Copy;
}

/// What a value type reads as, for a frame.
pub trait Form: Copy + 'static {
    type At<'a>: Copy;
}

/// Route a: ask the member through a trait of the consumer's.
pub struct ByMember<'a>(PhantomData<&'a ()>);

impl<'a> Carrier for ByMember<'a> {
    type Of<E: Elem> = <E as Readable>::Rd<'a>;
}

/// Route b: the same, with the bound stated on the impl's own item.
pub struct ByMemberWhere<'a>(PhantomData<&'a ()>);

impl<'a> Carrier for ByMemberWhere<'a> {
    type Of<E: Elem>
        = <E as Readable>::Rd<'a>
    where
        E: Readable;
}

/// Route c: ask the member's value type instead of the member.
pub struct ByValue<'a>(PhantomData<&'a ()>);

impl<'a> Carrier for ByValue<'a> {
    type Of<E: Elem> = <E::Value as Form>::At<'a>;
}

fn main() {}
