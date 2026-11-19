// The frame is on the carrier, so a product of borrows through it ends when
// the borrowed values do, and one kept past them is refused.

use core::marker::PhantomData;

use notko_hlist::{Cons, Empty, Form, Has, PCons, PNil, PerMember, Project, Shape};

struct Seq {
    len: usize,
}
struct Store;

struct Read<'f>(PhantomData<&'f ()>);
impl<'f> Shape for Read<'f> {
    type Is = PerMember;
}
impl<'f> Form<Store> for Read<'f> {
    type Of = &'f Seq;
}

type Res = Cons<Store, Empty>;

fn main() {
    let kept: <Res as Project<Read<'_>>>::Prod;
    {
        let seq = Seq {
            len: 4,
        };
        kept = PCons {
            head: &seq,
            tail: PNil,
        };
    }
    let s = <Res as Has<Read<'_>, Store, _>>::pick(&kept);
    let _ = s.len;
}
