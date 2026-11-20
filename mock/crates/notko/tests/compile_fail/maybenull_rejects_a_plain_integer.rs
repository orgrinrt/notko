//! `u32` fills no niche, so `MaybeNull<u32>` would be a `Maybe` wearing a
//! smaller type's clothes. The seal is what refuses it.

fn main() {
    let _ = notko::MaybeNull::<u32>::null();
}
