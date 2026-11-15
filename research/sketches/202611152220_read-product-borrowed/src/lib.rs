//! Sketch N2: can a product over a notko-hlist list hold borrowed values of a
//! frame lifetime. See FINDINGS.md. Each attempt is a binary under `src/bin`;
//! this library holds the model attempt 5 is written against, and the members
//! and carriers its two binaries share.

pub mod form;

use core::marker::PhantomData;

use form::Form;
use notko_hlist::{Cons, Elem, Empty};

/// A resource value that holds a collection, so it is not `Copy`.
pub struct Seq {
    pub items: [u32; 4],
    pub len:   usize,
}

/// A view of a column around one record, borrowed for the frame.
#[derive(Clone, Copy)]
pub struct Window<'a> {
    pub column: &'a [i32],
    pub centre: usize,
}

/// What a member reads as, for a frame. The consumer's own trait.
pub trait Readable {
    type Val<'a>: Copy;
}

/// A column read by value. Also a member of the shipped crate's product, with
/// the same value type, which is what a write list would use.
pub struct Pos;
impl Readable for Pos {
    type Val<'a> = i32;
}
impl Elem for Pos {
    type Value = i32;
}

/// A column read as a window.
pub struct Near;
impl Readable for Near {
    type Val<'a> = Window<'a>;
}

/// A resource read by reference.
pub struct Store;
impl Readable for Store {
    type Val<'a> = &'a Seq;
}

/// The read carrier: for each member, what it reads as in the frame `'a`.
pub struct Read<'a>(PhantomData<&'a ()>);

impl<'a, M: Readable> Form<M> for Read<'a> {
    type Of = M::Val<'a>;
}

/// The shipped crate's one-value-per-member carrier, in the model's terms.
pub struct Value;

impl<M: Elem> Form<M> for Value {
    type Of = M::Value;
}

/// A read list. It names no lifetime: the frame is on the carrier.
pub type Reads = Cons<Pos, Cons<Near, Cons<Store, Empty>>>;

/// A list of the one member that is also written.
pub type Writes = Cons<Pos, Empty>;
