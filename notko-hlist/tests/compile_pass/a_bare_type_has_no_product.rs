// The twin: the empty list in place of the bare type.

use notko_hlist::{Empty, Project};

fn has_product<L: Project>() {}

fn main() {
    has_product::<Empty>();
    has_product::<Empty>();
}
