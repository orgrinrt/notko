// A uniform carrier holds every member over the value it names through
// `Elem`, so it holds nothing for a bare value type, which names none.

use notko_hlist::{Uniform, Val};

fn main() {
    let _: <Val as Uniform>::Of<u8> = 1;
}
