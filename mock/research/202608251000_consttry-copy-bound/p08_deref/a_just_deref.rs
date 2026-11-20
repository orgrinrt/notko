// PROBE p08-a: Deref for Just, const and non-const, plus the neighbouring
// core conversions. Just is #[repr(transparent)] over a single private field
// and documents itself as "a no-op extraction of the inner value", which is
// exactly the shape Deref is for.
#![feature(const_trait_impl, const_convert, const_destruct)]
#![no_std]
#![crate_type = "lib"]
use core::ops::{Deref, DerefMut};
use core::borrow::{Borrow, BorrowMut};

#[repr(transparent)]
pub struct Just<T>(T);
impl<T> Just<T> { pub const fn new(v: T) -> Self { Just(v) } }

const impl<T> Deref for Just<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T { &self.0 }
}
const impl<T> DerefMut for Just<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}
const impl<T> AsRef<T> for Just<T> {
    #[inline]
    fn as_ref(&self) -> &T { &self.0 }
}
const impl<T> Borrow<T> for Just<T> {
    #[inline]
    fn borrow(&self) -> &T { &self.0 }
}
const impl<T> BorrowMut<T> for Just<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T { &mut self.0 }
}

// const deref, non-Copy payload
pub struct NotCopy(pub u32);
pub const fn const_deref() -> u32 {
    let j = Just::new(NotCopy(5));
    (*j).0
}
// runtime deref + method passthrough
pub fn runtime_deref(j: &Just<u32>) -> u32 { **j }
