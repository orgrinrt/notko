//! Expected to build and run. One read list, each member in its own form for
//! one frame, read back by member; the model's uniform carriers on a list of
//! `Elem` members beside it, with a row walk run at compile time and the
//! higher-ranked column walk.
#![feature(const_trait_impl)]

use consumer::{Near, Pos, Read, Reads, Seq, Store, Window};
use model::{Arr, Col, Cols, Cons, Elem, Empty, Has, PCons, PNil, Place, Project, Rows, Val};

struct Heat;
impl Elem for Heat {
    type Value = u8;
}
type Lanes = Cons<Pos, Cons<Heat, Empty>>;

fn store_of<'f, L: Has<Read<'f>, Store, P>, P: Place>(
    p: &<L as Project<Read<'f>>>::Prod,
) -> &'f Seq {
    *L::pick(p)
}

const ROW: <Lanes as Project<Val>>::Prod = {
    let arr: <Lanes as Project<Arr<3>>>::Prod = PCons {
        head: [1, 2, 3],
        tail: PCons {
            head: [7, 8, 9],
            tail: PNil,
        },
    };
    let row = <Lanes as Rows<Arr<3>>>::read_row(&arr, 1);
    core::mem::forget(arr);
    row
};

fn main() {
    {
        let column = [4, 5, 6, 7];
        let seq = Seq {
            items: [5, 6, 7, 0],
            len:   3,
        };
        let read: <Reads as Project<Read<'_>>>::Prod = PCons {
            head: 9,
            tail: PCons {
                head: Window {
                    column: &column,
                    centre: 2,
                },
                tail: PCons {
                    head: &seq,
                    tail: PNil,
                },
            },
        };
        let s = store_of::<Reads, _>(&read);
        assert!(
            core::ptr::eq(s, &seq),
            "the field is the borrow, not a copy"
        );
        println!(
            "resource read by member: {:?} of {}",
            &s.items[.. s.len],
            s.len
        );
        let w = <Reads as Has<Read<'_>, Near, _>>::pick(&read);
        println!(
            "window read by member: centre {} is {}",
            w.centre, w.column[w.centre]
        );
        let v = <Reads as Has<Read<'_>, Pos, _>>::pick(&read);
        println!("value read by member: {v}");
    }

    println!("row read at compile time: {} {}", ROW.head, ROW.tail.head);

    let arr: <Lanes as Project<Arr<2>>>::Prod = PCons {
        head: [10, 20],
        tail: PCons {
            head: [1, 2],
            tail: PNil,
        },
    };
    let cols: <Lanes as Project<Col<'_>>>::Prod = <Lanes as Cols<2>>::cols(&arr);
    let row = <Lanes as Rows<Col<'_>>>::read_row(&cols, 1);
    println!(
        "row through the column walk: {} {}",
        row.head, row.tail.head
    );
    println!("the frame ended and the read product with it");
}
