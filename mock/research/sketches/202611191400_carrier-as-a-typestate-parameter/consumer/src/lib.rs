//! A consumer of the model, standing where hilavitkutin's read lists would:
//! members of its own, and a read carrier of its own that states a form per
//! member for one frame.

use core::marker::PhantomData;

use model::{Carrier, Cons, Elem, Empty};

/// A resource that holds a collection, so it is not `Copy`.
pub struct Seq {
    pub items: [u32; 4],
    pub len:   usize,
}

/// A view of a column around one record.
#[derive(Clone, Copy)]
pub struct Window<'a> {
    pub column: &'a [i32],
    pub centre: usize,
}

/// A column read by value. Also an `Elem`, so the model's own carriers hold it.
pub struct Pos;
impl Elem for Pos {
    type Value = i32;
}

/// A column read as a window. Not an `Elem`.
pub struct Near;

/// A resource read by reference. Not an `Elem`, and its value is not `Copy`.
pub struct Store;

/// The read carrier for the frame `'f`. Every form it holds is one impl.
pub struct Read<'f>(PhantomData<&'f ()>);

impl<'f> Carrier<Pos> for Read<'f> {
    type Of = i32;
}
impl<'f> Carrier<Near> for Read<'f> {
    type Of = Window<'f>;
}
impl<'f> Carrier<Store> for Read<'f> {
    type Of = &'f Seq;
}

/// A read list. It names no lifetime: the frame is on the carrier.
pub type Reads = Cons<Pos, Cons<Near, Cons<Store, Empty>>>;
