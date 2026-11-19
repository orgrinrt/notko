// The twin: the same duplication on a per-member carrier of the crate's own.

use notko_hlist::{DupForm, Elem, Form, PerMember, Shape};

struct Pos;
impl Elem for Pos {
    type Value = i32;
}

struct Mine<const N: usize>;
impl<const N: usize> Shape for Mine<N> {
    type Is = PerMember;
}
impl<const N: usize> Form<Pos> for Mine<N> {
    type Of = [i32; N];
}
impl<const N: usize> DupForm<Pos> for Mine<N> {
    fn dup_form(c: &[i32; N]) -> [i32; N] {
        *c
    }
}

fn main() {}
