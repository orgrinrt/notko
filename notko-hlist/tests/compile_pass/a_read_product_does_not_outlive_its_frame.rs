// The twin: the product is read inside the frame.

use core::marker::PhantomData;

use notko_hlist::{Carrier, Cons, Empty, Has, PCons, PNil, Project};

struct Seq {
    len: usize,
}
struct Store;

struct Read<'f>(PhantomData<&'f ()>);
impl<'f> Carrier<Store> for Read<'f> {
    type Of = &'f Seq;
}

type Res = Cons<Store, Empty>;

fn main() {
    let seq = Seq {
        len: 4,
    };
    let kept: <Res as Project<Read<'_>>>::Prod = PCons {
        head: &seq,
        tail: PNil,
    };
    let s = <Res as Has<Read<'_>, Store, _>>::pick(&kept);
    let _ = s.len;
}
