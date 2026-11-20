// Phase two. FINDINGS section 7 claims both `[const] Destruct` and the older
// `~const Destruct` compile on the pin. Checking the `~const` half myself, since
// `d_` already covers `[const]`. core still carries exactly one `~const Destruct`
// site (core/src/intrinsics/mod.rs), so the spelling is expected to be live.
#![feature(const_trait_impl, const_destruct)]
use core::marker::Destruct;
const fn consume<T: ~const Destruct>(_t: T) {}
struct HasDrop;
impl const Drop for HasDrop { fn drop(&mut self) {} }
const _: () = consume(HasDrop);
fn main() {}
