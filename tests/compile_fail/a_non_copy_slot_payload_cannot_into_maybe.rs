//! `Slot<T>` admits `NonZeroable + NicheFilled`, and `&mut T` is in that set
//! and is not `Copy`. `into_maybe` takes `Copy` because a const fn has to drop
//! what it consumes, so this is the payload the bound genuinely costs.

pub struct Theirs(u32);

impl notko::NonZeroable for &mut Theirs {
    type Inner = u32;

    fn try_new(_raw: Self::Inner) -> notko::Maybe<Self> {
        notko::Maybe::Isnt
    }

    fn value(self) -> Self::Inner {
        self.0
    }
}

fn main() {
    let mut theirs = Theirs(7);
    let _ = notko::Slot::some(&mut theirs).into_maybe();
}
