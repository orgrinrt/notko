//! Expected to fail: the twin of `w1`, whose read product is kept past the
//! frame its borrows come from.

use consumer::{Seq, Store, Read};
use model::{Cons, Empty, Has, PCons, PNil, Project};

type Res = Cons<Store, Empty>;

fn main() {
    let kept: <Res as Project<Read<'_>>>::Prod;
    {
        let seq = Seq {
            items: [1, 2, 3, 4],
            len:   4,
        };
        kept = PCons {
            head: &seq,
            tail: PNil,
        };
    }
    let s = <Res as Has<Read<'_>, Store, _>>::pick(&kept);
    println!("{}", s.len);
}
