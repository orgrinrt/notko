// A cut duplicates every field it names. A per-member carrier says for each
// form whether it can be duplicated, through `DupForm`, and an exclusive
// borrow it states no duplication for is not cut out.

use notko_hlist::{Cons, Empty, Form, PCons, PNil, PerMember, Project, Shape, SubOf};

struct Count;

struct Write<'f>(core::marker::PhantomData<&'f ()>);
impl<'f> Shape for Write<'f> {
    type Is = PerMember;
}
impl<'f> Form<Count> for Write<'f> {
    type Of = &'f mut u32;
}

type Writes = Cons<Count, Empty>;

fn main() {
    let mut n = 0;
    let p: <Writes as Project<Write<'_>>>::Prod = PCons { head: &mut n, tail: PNil };
    let _ = <Writes as SubOf<Write<'_>, Writes, _>>::sub(&p);
}
