// A crate owning a member that is not an `Elem` has a local type to put in a
// trait's member parameter, which the orphan rule would let it use to give
// `Val` a form for that member. `Val` is uniform, so it holds `Elem` members
// and nothing else: a form stated for it is refused at its shape, and the
// trait the product reads is sealed, so it cannot be stated there either.

use notko_hlist::{Cons, Empty, Form, Has, PCons, PNil, Project, Val};

struct Res;

impl Form<Res> for Val {
    type Of = &'static str;
}

fn main() {
    let p: <Cons<Res, Empty> as Project<Val>>::Prod = PCons { head: "a", tail: PNil };
    let _ = <Cons<Res, Empty> as Has<Val, Res, _>>::pick(&p);
}
