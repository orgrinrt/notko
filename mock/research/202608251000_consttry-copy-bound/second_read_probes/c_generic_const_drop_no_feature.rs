// Q: does a stable wrapper suffice? A generic const fn that must drop its T, with
// no feature at all. If this compiles, `const_destruct` buys nothing here.
// Expected: refused ("destructor cannot be evaluated at compile-time").
const fn consume<T>(_t: T) {}
fn main() {}
