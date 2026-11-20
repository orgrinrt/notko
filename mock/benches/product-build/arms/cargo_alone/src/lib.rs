//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! `product-build` arm `cargo_alone`: the fixture built empty at `point`, the same cargo call as `from_clean` without the
//! registry, so what it takes is cargo's own work and the floor under `from_clean`.

use mockspace_bench_core::{FfiBenchCall, abi_hash, timed};

/// One call: the source written and the fixture cleaned untimed, the build timed, the record taken untimed.
/// Answers with the number of packages the compiler built.
///
/// # Safety
///
/// The harness passes an eight-byte output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bench_entry(
    _input_ptr: *const u8,
    output_ptr: *mut u8,
    point: usize,
) -> FfiBenchCall {
    timed! {
        setup {
            let absent_before = product_build_call::prepare(point);
        }
        run {
            let said = product_build_call::build(point, product_build_call::Fixture::Empty);
        }
        let built = product_build_call::record(point, product_build_call::Fixture::Empty, &said, absent_before);
        unsafe { output_ptr.cast::<u64>().write_unaligned(built) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_name() -> *const u8 {
    b"cargo_alone\0".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_abi_hash() -> u64 {
    abi_hash()
}

#[unsafe(no_mangle)]
pub extern "C" fn bench_output_size(_n: usize) -> usize {
    8
}
