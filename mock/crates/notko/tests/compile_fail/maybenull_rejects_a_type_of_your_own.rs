//! The seal holds against a downstream type, which is the only side it holds
//! against at all. A struct wrapping a niche-filling field does not inherit
//! the niche.

pub struct Plain(u32);

fn main() {
    let _ = notko::MaybeNull::<Plain>::null();
}
