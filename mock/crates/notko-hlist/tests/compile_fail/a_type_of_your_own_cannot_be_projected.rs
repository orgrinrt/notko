// A product is the nest a list says, through this crate's two impls only. This
// is the plainest attempt at another: an impl for a type of the consumer's own,
// through a carrier of its own. `List` refuses it, and the seal behind it; the
// attempt on a real list is `a_product_through_a_carrier_of_your_own_is_this_crate_s`,
// and the one `Arrays::made` rests on is `a_full_impl_on_a_list_of_your_own_is_refused`.

use notko_hlist::Project;

struct Mine;

#[allow(dead_code)]
trait Projected {}

impl Project<Mine> for Mine {}

fn main() {}
