// The twin: a member the row holds.

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
    let _ = row.get(Heat);
    let _ = Absent;
}
