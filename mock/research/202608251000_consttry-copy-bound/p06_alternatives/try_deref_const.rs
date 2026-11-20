#![feature(const_trait_impl, deref_const)]
#![no_std]
#![crate_type = "lib"]
use core::mem::ManuallyDrop;
pub const fn f(m: ManuallyDrop<u32>) -> u32 { *m }
