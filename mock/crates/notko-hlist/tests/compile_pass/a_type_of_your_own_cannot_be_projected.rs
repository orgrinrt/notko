// The twin: the same impl, of a trait of the consumer's own.

#[allow(unused_imports)]
use notko_hlist::Project;

struct Mine;

#[allow(dead_code)]
trait Projected {}

impl Projected for Mine {}

fn main() {}
