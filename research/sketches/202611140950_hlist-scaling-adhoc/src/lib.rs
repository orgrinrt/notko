//! Ad-hoc spike for the stack audit of 14 November 2026. See FINDINGS.md.
//!
//! The library is whatever `build.rs` generated: a list of `SPIKE_N` members
//! and one query over it.

#![feature(const_trait_impl)]
#![recursion_limit = "16384"]
#![allow(unused_imports, dead_code)]

use notko_hlist::{
    Arr,
    Cardinal,
    Cons,
    Contains,
    ContainsAll,
    Elem,
    Empty,
    Has,
    Length,
    PCons,
    PNil,
    Place,
    Project,
    SubOf,
    Val,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Count(pub usize);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);

    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

include!(concat!(env!("OUT_DIR"), "/gen.rs"));
