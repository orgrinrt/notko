//! The twin of attempt 2 that must not build: the same product, kept past the
//! frame whose locals it borrows.

use core::marker::PhantomData;

use notko_hlist::{Carrier, Cons, Elem, Empty, PCons, PNil, Project};

pub struct Frame<'a>(PhantomData<&'a ()>);

impl<'a> Carrier for Frame<'a> {
    type Of<E: Elem> = &'a E::Value;
}

struct Pos;
impl Elem for Pos {
    type Value = [i32; 3];
}

struct Table;
impl Elem for Table {
    type Value = [u32; 4];
}

type Lanes = Cons<Pos, Cons<Table, Empty>>;

fn main() {
    let kept: <Lanes as Project>::Prod<Frame<'_>>;
    {
        let pos = [1, 2, 3];
        let table = [10, 20, 30, 40];
        kept = PCons {
            head: &pos,
            tail: PCons {
                head: &table,
                tail: PNil,
            },
        };
    }
    println!("{:?}", kept.tail.head);
}
