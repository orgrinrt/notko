// The twin: the same impl, of a trait of the consumer's own with the same
// items.

use core::mem::MaybeUninit;

use notko_hlist::{Arr, ByRow, ByRowMut, Carrier, Col, ColMut, Cons, Elem, Empty, PCons, PNil, Project, Val};

struct Mine;
impl Elem for Mine {
    type Value = u8;
}

#[allow(dead_code)]
trait Mirror {
    type Prod<C: Carrier>;
    fn read_row<C: ByRow>(p: &Self::Prod<C>, i: usize) -> Self::Prod<Val>;
    fn write_row<C: ByRowMut>(p: &mut Self::Prod<C>, i: usize, v: &Self::Prod<Val>);
    fn cols<const N: usize>(p: &Self::Prod<Arr<N>>) -> Self::Prod<Col<'_>>;
    fn cols_mut<const N: usize>(p: &mut Self::Prod<Arr<N>>) -> Self::Prod<ColMut<'_>>;
    fn copy_from<const N: usize>(dst: &mut Self::Prod<Arr<N>>, src: &Self::Prod<Arr<N>>);
    fn splat_in<const N: usize>(dst: &mut MaybeUninit<Self::Prod<Arr<N>>>, row: &Self::Prod<Val>);
}

#[allow(unused_imports)]
use Project as _;

impl Mirror for Cons<Mine, Empty> {
    type Prod<C: Carrier> = PCons<C, Mine, PNil>;
    fn read_row<C: ByRow>(p: &PCons<C, Mine, PNil>, i: usize) -> PCons<Val, Mine, PNil> {
        PCons { head: C::at::<Mine>(&p.head, i), tail: PNil }
    }
    fn write_row<C: ByRowMut>(p: &mut PCons<C, Mine, PNil>, i: usize, v: &PCons<Val, Mine, PNil>) {
        C::set::<Mine>(&mut p.head, i, v.head);
    }
    fn cols<const N: usize>(p: &PCons<Arr<N>, Mine, PNil>) -> PCons<Col<'_>, Mine, PNil> {
        PCons { head: &p.head, tail: PNil }
    }
    fn cols_mut<const N: usize>(p: &mut PCons<Arr<N>, Mine, PNil>) -> PCons<ColMut<'_>, Mine, PNil> {
        PCons { head: &mut p.head, tail: PNil }
    }
    fn copy_from<const N: usize>(dst: &mut PCons<Arr<N>, Mine, PNil>, src: &PCons<Arr<N>, Mine, PNil>) {
        dst.head = src.head;
    }
    // Writes nothing, so `made` would read uninitialised memory were this
    // impl let in.
    fn splat_in<const N: usize>(_: &mut MaybeUninit<PCons<Arr<N>, Mine, PNil>>, _: &PCons<Val, Mine, PNil>) {}
}

fn main() {}
