// `Project::made` reads its product as initialised on the strength of
// `splat_in` writing every field, which only this crate's two impls do. This
// is the plainest attempt at another: an impl for a type of the consumer's own.
// `List` refuses it before the seal is reached; the full attempt is
// `a_full_impl_on_a_list_of_your_own_is_refused`.

use notko_hlist::Project;

struct Mine;

#[allow(dead_code)]
trait Projected {}

impl Project for Mine {}

fn main() {}
