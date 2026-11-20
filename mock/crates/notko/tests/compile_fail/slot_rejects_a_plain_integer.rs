//! `Slot<T>` needs a niche to put its empty case in, and `u32` has none. The
//! seal on `NicheFilled` is what refuses it, one layer below `MaybeNull`.

fn main() {
    let _ = notko::Slot::<u32>::NONE;
}
