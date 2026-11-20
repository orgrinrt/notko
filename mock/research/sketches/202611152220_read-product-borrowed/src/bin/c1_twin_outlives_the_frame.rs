//! The twin of attempt 5 that must not build: the read product kept past the
//! frame whose column and resource it borrows.

use read_product_borrowed::form::{Cell, End, HasBy, ProjectBy};
use read_product_borrowed::{Read, Reads, Seq, Store, Window};

fn main() {
    let kept: <Reads as ProjectBy<Read<'_>>>::Prod;
    {
        let column = [4, 5, 6, 7];
        let seq = Seq {
            items: [5, 6, 7, 0],
            len:   3,
        };
        kept = Cell {
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
    }
    let s = <Reads as HasBy<Read<'_>, Store, _>>::pick(&kept);
    println!("{}", s.len);
}
