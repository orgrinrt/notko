// `Project::made` reads its product as initialised on the strength of
// `splat_in` writing every field, which only this crate's two impls promise.
// That rests on no impl from outside existing, and this is the attempt: the
// trait takes no parameters, so the impl can only name its own type, and
// `List` under it is sealed.

use notko_hlist::Project;

struct Mine;

#[allow(dead_code)]
trait Projected {}

impl Project for Mine {}

fn main() {}
