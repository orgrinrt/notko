// A product is of a list, and a bare type is not one.

use notko_hlist::{Empty, Project};

fn has_product<L: Project>() {}

fn main() {
    has_product::<u8>();
    has_product::<Empty>();
}
