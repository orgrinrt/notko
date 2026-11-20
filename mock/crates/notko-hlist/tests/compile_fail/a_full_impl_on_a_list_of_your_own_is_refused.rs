// `Arrays::made` reads its product as initialised on the strength of
// `splat_in` writing every field, which only this crate's two impls do. This
// is the attempt that matters: every item written, for a list holding a type
// of the consumer's own, with a `splat_in` that writes nothing. The orphan rule
// refuses it, since `Arrays` has no type parameter a local type could stand in
// and `Cons` is not fundamental. Behind that the seal, `Arrays::sealed` taking
// a token no other crate can name, refuses it whatever the orphan rule allows.

use core::mem::MaybeUninit;

use notko_hlist::{Arr, Arrays, Col, ColMut, Cons, Elem, Empty, PCons, PNil, Val};

struct Mine;
impl Elem for Mine {
    type Value = u8;
}

#[allow(dead_code)]
trait Mirror<const N: usize> {
    fn cols(p: &PCons<Arr<N>, Mine, PNil>) -> PCons<Col<'_>, Mine, PNil>;
    fn cols_mut(p: &mut PCons<Arr<N>, Mine, PNil>) -> PCons<ColMut<'_>, Mine, PNil>;
    fn copy_from(dst: &mut PCons<Arr<N>, Mine, PNil>, src: &PCons<Arr<N>, Mine, PNil>);
    fn splat_in(dst: &mut MaybeUninit<PCons<Arr<N>, Mine, PNil>>, row: &PCons<Val, Mine, PNil>);
}

impl<const N: usize> Arrays<N> for Cons<Mine, Empty> {
    fn cols(p: &PCons<Arr<N>, Mine, PNil>) -> PCons<Col<'_>, Mine, PNil> {
        PCons { head: &p.head, tail: PNil }
    }
    fn cols_mut(p: &mut PCons<Arr<N>, Mine, PNil>) -> PCons<ColMut<'_>, Mine, PNil> {
        PCons { head: &mut p.head, tail: PNil }
    }
    fn copy_from(dst: &mut PCons<Arr<N>, Mine, PNil>, src: &PCons<Arr<N>, Mine, PNil>) {
        dst.head = src.head;
    }
    // Writes nothing, so `made` would read uninitialised memory were this
    // impl let in.
    fn splat_in(_: &mut MaybeUninit<PCons<Arr<N>, Mine, PNil>>, _: &PCons<Val, Mine, PNil>) {}
}

fn main() {}
