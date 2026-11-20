// A row is read by naming the member, and a member the list lacks has no field
// to read.
//
// Two members rather than one on purpose. With one, the solver has a single
// candidate, unifies the member with it before it looks at the argument, and
// reports the argument's type as wrong, `E0308`: still a refusal, but not this
// crate's note.

use notko_hlist::{Cons, Elem, Empty, PCons, PNil, Vals};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}
struct Heat;
impl Elem for Heat {
    type Value = i32;
}
struct Absent;
impl Elem for Absent {
    type Value = i32;
}

fn main() {
    let row = Vals::<Cons<Pos, Cons<Heat, Empty>>>(PCons { head: 1, tail: PCons { head: 2, tail: PNil } });
    let _ = row.get(Absent);
}
