//! `Just<T>` always carries a value, so `unwrap_or_default` never reaches the
//! default. The bound is still a real one: this payload is refused here and
//! accepted by every other method on `Just`.

pub struct NoDefault(u32);

fn main() {
    let _ = notko::Just::new(NoDefault(1)).unwrap_or_default();
}
