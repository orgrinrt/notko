//! Attempt 2, against the shipped crate, expected to build and run.
//!
//! The borrow is put on the carrier, where the shipped `Col<'a>` already puts
//! one, and the member keeps a `'static` value type. `Frame<'a>` is a carrier
//! of this sketch's own: it holds `&'a Value` for every member. The product
//! through it is the shipped `Project::Prod`, read by member through the
//! shipped `Has` and cut through the shipped `SubOf`.

use core::marker::PhantomData;

use notko_hlist::{Carrier, Cons, Dup, Elem, Empty, Has, PCons, PNil, Place, Project, SubOf};

/// One borrowed value per member, for the frame `'a`.
pub struct Frame<'a>(PhantomData<&'a ()>);

impl<'a> Carrier for Frame<'a> {
    type Of<E: Elem> = &'a E::Value;
}

impl<'a> Dup for Frame<'a> {
    fn dup<E: Elem>(c: &Self::Of<E>) -> Self::Of<E> {
        c
    }
}

struct Pos;
impl Elem for Pos {
    type Value = [i32; 3];
}

struct Heat;
impl Elem for Heat {
    type Value = u8;
}

struct Table;
impl Elem for Table {
    type Value = [u32; 4];
}

type Lanes = Cons<Pos, Cons<Heat, Cons<Table, Empty>>>;
type Narrow = Cons<Table, Cons<Pos, Empty>>;

/// The read product of one frame: every field borrows for `'f`.
fn frame<'f>(
    pos: &'f [i32; 3],
    heat: &'f u8,
    table: &'f [u32; 4],
) -> <Lanes as Project>::Prod<Frame<'f>> {
    PCons {
        head: pos,
        tail: PCons {
            head: heat,
            tail: PCons {
                head: table,
                tail: PNil,
            },
        },
    }
}

/// The table of any list that holds one, read through the product.
fn table_of<'f, L: Has<Table, P>, P: Place>(p: &L::Prod<Frame<'f>>) -> &'f [u32; 4] {
    L::pick::<Frame<'f>>(p)
}

fn main() {
    // The frame. Everything the product borrows is a local of this block.
    {
        let pos = [1, 2, 3];
        let heat = 40u8;
        let table = [10, 20, 30, 40];

        let read = frame(&pos, &heat, &table);

        let t = table_of::<Lanes, _>(&read);
        assert!(
            core::ptr::eq(t, &table),
            "the field is the borrow, not a copy"
        );
        println!("table read by member through the borrowed product: {t:?}");
        println!("heat read by field: {}", read.tail.head);

        let part = <Narrow as SubOf<Lanes, _>>::sub::<Frame<'_>>(&read);
        assert!(core::ptr::eq(part.head, &table));
        assert!(core::ptr::eq(part.tail.head, &pos));
        println!(
            "a narrower product cut from it: table {:?}, pos {:?}",
            part.head, part.tail.head
        );
    }
    println!("the frame ended and the product with it");
}
