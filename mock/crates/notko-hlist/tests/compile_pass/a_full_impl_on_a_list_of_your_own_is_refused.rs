// The twin: the same impl, of a trait of the consumer's own with the same
// items.

use core::mem::MaybeUninit;

#[allow(unused_imports)]
use notko_hlist::Arrays;
use notko_hlist::{Arr, Col, ColMut, Cons, Elem, Empty, PCons, PNil, Val};

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

impl<const N: usize> Mirror<N> for Cons<Mine, Empty> {
    fn cols(p: &PCons<Arr<N>, Mine, PNil>) -> PCons<Col<'_>, Mine, PNil> {
        PCons {
            head: &p.head,
            tail: PNil,
        }
    }

    fn cols_mut(p: &mut PCons<Arr<N>, Mine, PNil>) -> PCons<ColMut<'_>, Mine, PNil> {
        PCons {
            head: &mut p.head,
            tail: PNil,
        }
    }

    fn copy_from(dst: &mut PCons<Arr<N>, Mine, PNil>, src: &PCons<Arr<N>, Mine, PNil>) {
        dst.head = src.head;
    }

    // Writes nothing, so `made` would read uninitialised memory were this
    // impl let in.
    fn splat_in(_: &mut MaybeUninit<PCons<Arr<N>, Mine, PNil>>, _: &PCons<Val, Mine, PNil>) {}
}

fn main() {}
