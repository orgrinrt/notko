// The point of `L: Contains<X>` is that it proves `X` is in `L`. A marker
// trait carries no items, so an impl of it is one empty line, and if a
// downstream crate could write that line the bound would prove nothing: any
// type could claim to hold anything and every consumer bounding on membership
// would be relying on a claim rather than on a walk.
//
// The refusal comes through `List`, which `Contains` has as a supertrait and
// which is sealed. Same for `Length`, `ContainsAll` and `Concat`.

use notko_hlist::Contains;

struct Mine;
struct Absent;

impl Contains<Absent> for Mine {}

fn main() {}
