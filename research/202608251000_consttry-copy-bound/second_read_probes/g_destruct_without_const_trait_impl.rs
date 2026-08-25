// Blast radius: does adopting const_destruct drag in const_trait_impl (WATCH tier),
// or does it stand alone? Only const_destruct is enabled here.
#![feature(const_destruct)]
use core::marker::Destruct;
const fn consume<T: [const] Destruct>(_t: T) {}
fn main() {}
