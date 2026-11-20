//! Ad-hoc spike, not a bench: one fallible step in a loop under each carrier.
//!
//! Two loop shapes. `sum` applies the step to each element and adds the
//! results, so the steps are independent and the loop is bound by throughput.
//! `chain` feeds each result into the next step, so the loop is bound by the
//! latency of one step. The check never fails on the data, which is the case
//! the hot tier is for.

use std::hint::black_box;
use std::time::Instant;

use notko::{Just, Maybe, Outcome};
use profile_arms_adhoc::*;

const N: usize = 4096;
const PASSES: usize = 2048;
const SAMPLES: usize = 15;
const LIM: u32 = 0x7fff_ffff;

macro_rules! loops {
    // carriers that support `?`
    (try $sum:ident, $chain:ident, $f:path, $ret:ty, $wrap:expr) => {
        #[inline(never)]
        fn $sum(data: &[u32], lim: u32) -> $ret {
            let mut acc = 0u32;
            for &x in data {
                acc = acc.wrapping_add($f(x, lim)?);
            }
            $wrap(acc)
        }
        #[inline(never)]
        fn $chain(data: &[u32], lim: u32) -> $ret {
            let mut acc = 0u32;
            for &x in data {
                acc = $f((acc ^ x) >> 1, lim)?;
            }
            $wrap(acc)
        }
    };
    // bare values
    (raw $sum:ident, $chain:ident, $f:path) => {
        #[inline(never)]
        fn $sum(data: &[u32], lim: u32) -> u32 {
            let mut acc = 0u32;
            for &x in data {
                acc = acc.wrapping_add($f(x, lim));
            }
            acc
        }
        #[inline(never)]
        fn $chain(data: &[u32], lim: u32) -> u32 {
            let mut acc = 0u32;
            for &x in data {
                acc = $f((acc ^ x) >> 1, lim);
            }
            acc
        }
    };
}

loops!(raw raw_inl_sum, raw_inl_chain, raw_inl);
loops!(raw raw_noinl_sum, raw_noinl_chain, raw_noinl);
loops!(try result_inl_sum, result_inl_chain, result_inl, Result<u32, Small>, Ok);
loops!(try result_noinl_sum, result_noinl_chain, result_noinl, Result<u32, Small>, Ok);
loops!(try outcome_inl_sum, outcome_inl_chain, outcome_inl, Outcome<u32, Small>, Outcome::Ok);
loops!(try outcome_noinl_sum, outcome_noinl_chain, outcome_noinl, Outcome<u32, Small>, Outcome::Ok);
loops!(try option_inl_sum, option_inl_chain, option_inl, Option<u32>, Some);
loops!(try option_noinl_sum, option_noinl_chain, option_noinl, Option<u32>, Some);
loops!(try maybe_inl_sum, maybe_inl_chain, maybe_inl, Maybe<u32>, Maybe::Is);
loops!(try maybe_noinl_sum, maybe_noinl_chain, maybe_noinl, Maybe<u32>, Maybe::Is);
loops!(try just_inl_sum, just_inl_chain, just_inl, Just<u32>, Just::new);
loops!(try just_noinl_sum, just_noinl_chain, just_noinl, Just<u32>, Just::new);
loops!(try result_wide_inl_sum, result_wide_inl_chain, result_wide_inl, Result<u32, Wide>, Ok);
loops!(try result_wide_noinl_sum, result_wide_noinl_chain, result_wide_noinl, Result<u32, Wide>, Ok);
loops!(try outcome_wide_inl_sum, outcome_wide_inl_chain, outcome_wide_inl, Outcome<u32, Wide>, Outcome::Ok);
loops!(try outcome_wide_noinl_sum, outcome_wide_noinl_chain, outcome_wide_noinl, Outcome<u32, Wide>, Outcome::Ok);

// The attribute's own output. Which type `macro_hot` returns depends on the
// arm this binary was built in, and `?` is the one spelling that reads both
// from inside a function returning the same carrier.
#[cfg(any(not(feature = "internal"), debug_assertions))]
loops!(try macro_hot_sum, macro_hot_chain, macro_hot, Outcome<u32, Small>, Outcome::Ok);
#[cfg(all(feature = "internal", not(debug_assertions)))]
loops!(try macro_hot_sum, macro_hot_chain, macro_hot, Just<u32>, Just::new);
loops!(try macro_cold_sum, macro_cold_chain, macro_cold, Outcome<u32, Small>, Outcome::Ok);
loops!(try macro_warm_sum, macro_warm_chain, macro_warm, Maybe<u32>, Maybe::Is);

fn measure<R>(name: &str, shape: &str, data: &[u32], f: impl Fn(&[u32], u32) -> R) {
    let mut samples = [0f64; SAMPLES];
    // one warm pass
    black_box(f(black_box(data), black_box(LIM)));
    for s in &mut samples {
        let t = Instant::now();
        for _ in 0 .. PASSES {
            black_box(f(black_box(data), black_box(LIM)));
        }
        *s = t.elapsed().as_nanos() as f64 / (PASSES * N) as f64;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("| {name} | {shape} | {:.3} | {:.3} | {:.3} |", samples[0], samples[SAMPLES / 2], samples[SAMPLES - 1]);
}

fn main() {
    // A fixed xorshift fill, every value at or under the limit.
    let mut data = [0u32; N];
    let mut s = 0x9e37_79b9u32;
    for d in &mut data {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        *d = s >> 1;
    }

    let arm = if cfg!(all(feature = "internal", not(debug_assertions))) {
        "release arm (feature internal on, debug assertions off)"
    } else {
        "debug arm (feature internal off or debug assertions on)"
    };
    println!("profile arm this binary was built in: {arm}");
    println!("elements {N}, passes per sample {PASSES}, samples {SAMPLES}; nanoseconds per element");
    println!();
    println!("| arm | loop | min | median | max |");
    println!("|---|---|---|---|---|");

    macro_rules! run {
        ($name:literal, $sum:ident, $chain:ident) => {
            measure($name, "sum", &data, $sum);
            measure($name, "chain", &data, $chain);
        };
    }

    run!("raw, no check, inlined", raw_inl_sum, raw_inl_chain);
    run!("raw, no check, call", raw_noinl_sum, raw_noinl_chain);
    run!("core Result, inlined", result_inl_sum, result_inl_chain);
    run!("core Result, call", result_noinl_sum, result_noinl_chain);
    run!("notko Outcome, inlined", outcome_inl_sum, outcome_inl_chain);
    run!("notko Outcome, call", outcome_noinl_sum, outcome_noinl_chain);
    run!("core Option, inlined", option_inl_sum, option_inl_chain);
    run!("core Option, call", option_noinl_sum, option_noinl_chain);
    run!("notko Maybe, inlined", maybe_inl_sum, maybe_inl_chain);
    run!("notko Maybe, call", maybe_noinl_sum, maybe_noinl_chain);
    run!("notko Just with panic, inlined", just_inl_sum, just_inl_chain);
    run!("notko Just with panic, call", just_noinl_sum, just_noinl_chain);
    run!("core Result, wide error, inlined", result_wide_inl_sum, result_wide_inl_chain);
    run!("core Result, wide error, call", result_wide_noinl_sum, result_wide_noinl_chain);
    run!("notko Outcome, wide error, inlined", outcome_wide_inl_sum, outcome_wide_inl_chain);
    run!("notko Outcome, wide error, call", outcome_wide_noinl_sum, outcome_wide_noinl_chain);
    run!("#[profile(Hot)]", macro_hot_sum, macro_hot_chain);
    run!("#[profile(Cold)]", macro_cold_sum, macro_cold_chain);
    run!("#[profile(Warm)]", macro_warm_sum, macro_warm_chain);
}
