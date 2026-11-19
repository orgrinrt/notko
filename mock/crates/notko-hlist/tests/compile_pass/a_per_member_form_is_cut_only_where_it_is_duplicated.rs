// The twin: a shared borrow, stated as duplicated.

use notko_hlist::{Cons, DupForm, Empty, Form, PCons, PNil, PerMember, Project, Shape, SubOf};

struct Count;

struct Read<'f>(core::marker::PhantomData<&'f ()>);
impl<'f> Shape for Read<'f> {
    type Is = PerMember;
}
impl<'f> Form<Count> for Read<'f> {
    type Of = &'f u32;
}
impl<'f> DupForm<Count> for Read<'f> {
    fn dup_form(c: &&'f u32) -> &'f u32 {
        c
    }
}

type Reads = Cons<Count, Empty>;

fn main() {
    let n = 0;
    let p: <Reads as Project<Read<'_>>>::Prod = PCons {
        head: &n,
        tail: PNil,
    };
    let _ = <Reads as SubOf<Read<'_>, Reads, _>>::sub(&p);
}
