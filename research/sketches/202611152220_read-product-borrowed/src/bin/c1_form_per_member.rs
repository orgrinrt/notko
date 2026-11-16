//! Attempt 5, the model in `src/form.rs` over the shipped crate's list,
//! expected to build and run.
//!
//! One `'static` read list of three members, each read in its own form for one
//! frame: a value, a window on a column, a reference to a resource that is not
//! `Copy`. Read back by member.

use notko_hlist::Place;
use read_product_borrowed::form::{Cell, End, HasBy, ProjectBy};
use read_product_borrowed::{Near, Pos, Read, Reads, Seq, Store, Value, Window, Writes};

/// The resource of any read list that holds one, for the frame `'f`.
fn store_of<'f, L: HasBy<Read<'f>, Store, P>, P: Place>(
    p: &<L as ProjectBy<Read<'f>>>::Prod,
) -> &'f Seq {
    L::pick(p)
}

fn window_of<'f, L: HasBy<Read<'f>, Near, P>, P: Place>(
    p: &<L as ProjectBy<Read<'f>>>::Prod,
) -> Window<'f> {
    L::pick(p)
}

fn main() {
    {
        let column = [4, 5, 6, 7];
        let seq = Seq {
            items: [5, 6, 7, 0],
            len:   3,
        };

        let read: <Reads as ProjectBy<Read<'_>>>::Prod = Cell {
            head: 9,
            tail: Cell {
                head: Window {
                    column: &column,
                    centre: 2,
                },
                tail: Cell {
                    head: &seq,
                    tail: End,
                },
            },
        };

        let s = store_of::<Reads, _>(&read);
        assert!(core::ptr::eq(s, &seq), "the field is the borrow, not a copy");
        println!("resource read by member: {:?} of {}", &s.items[..s.len], s.len);

        let w = window_of::<Reads, _>(&read);
        assert!(core::ptr::eq(w.column, &column[..]));
        println!("window read by member: centre {} is {}", w.centre, w.column[w.centre]);

        let pos: i32 = <Reads as HasBy<Read<'_>, Pos, _>>::pick(&read);
        println!("value read by member: {pos}");
    }

    // The same model holds the shipped crate's values for a member that names
    // one through `Elem`, so one trait carries both products.
    let written: <Writes as ProjectBy<Value>>::Prod = Cell {
        head: 11,
        tail: End,
    };
    println!("the write side through the same model: {}", written.head);
    println!("the frame ended and the read product with it");
}
