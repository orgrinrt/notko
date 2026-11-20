# Ad-hoc spike: the two arms of `#[profile(Hot)]`, and the carriers beside the core ones

This is an ad-hoc spike written for the stack audit of 14 November 2026
(`research/202611140845_stack-audit/`). It is not a bench. Nothing in it is a
committed measurement of notko, and no number here should be quoted without the
words ad-hoc spike beside it.

## What was asked

1. Does a function under `#[profile(Hot)]` run faster in the arm that returns
   `Just<T>` than in the arm that returns `Outcome<T, E>`?
2. Do `Outcome` and `Maybe` cost anything against `Result` and `Option`?
3. Does a caller written against one arm of a hot function build against the
   other, and does it behave the same?

## How it was run

One x86-64 virtual machine shared with other sessions' builds, toolchain
`nightly-2026-05-28` as pinned by the repository, release profile of this
crate (`opt-level = 3`, one codegen unit). No core was pinned and the machine
was not quiet. `run.sh` lists the commands; the harness this audit ran under
refused to execute a script file, so each command was run by hand in the
order the file gives.

The kernel is a check and an add: `x < LIMIT` gives `x + 1`, otherwise an
error. `sum` folds the result over 4096 elements, `chain` feeds each result
into the next call. Every arm exists twice, once `#[inline]` and once
`#[inline(never)]`. 2048 passes per sample, 15 samples, nanoseconds per
element. No element ever fails, so the error path is never taken.

## Results

Committed in `out_tiers_release_debug_arm.txt` (feature `internal` off, so the
macro emits the `Outcome` arm) and `out_tiers_release_internal_arm.txt`
(feature `internal` on, debug assertions off, so the macro emits the `Just`
arm). Least of 15 samples, sum loop:

| arm | Outcome arm build | Just arm build |
|---|---|---|
| raw, no check, inlined | 0.139 | 0.150 |
| core Result, inlined | 0.641 | 0.639 |
| notko Outcome, inlined | 0.659 | 0.659 |
| core Option, inlined | 0.597 | 0.682 |
| notko Maybe, inlined | 0.598 | 0.683 |
| notko Just with a panic on the error, inlined | 0.658 | 0.678 |
| `#[profile(Hot)]` | 0.621 | 0.657 |
| raw, no check, call | 1.360 | 1.309 |
| core Result, call | 1.205 | 1.345 |
| notko Outcome, call | 1.324 | 1.373 |
| notko Maybe, call | 1.642 | 1.729 |
| notko Just with a panic, call | 1.231 | 1.796 |
| core Result, wide error, call | 2.243 | 2.347 |
| notko Outcome, wide error, call | 1.854 | 2.062 |
| `#[profile(Cold)]` | 1.242 | 1.336 |
| `#[profile(Warm)]` | 1.650 | 1.244 |

Only the three `#[profile]` rows are different code between the two columns.
Every other row is the same source built twice, so the difference between its
two columns is the noise of this host. That noise reaches 0.57 ns on a call
row (`Just with a panic, call`, 1.231 against 1.796) and 0.09 ns on an
inlined row.

The probes, committed as `out_probe_*` and `out_recover_*`:

| caller | Outcome arm | Just arm |
|---|---|---|
| `?` on a hot function, from a function returning `Outcome` | builds | E0277, `FromResidual<JustResidual>` is not implemented for `Outcome<u32, Small>` |
| `match` on `Outcome::Ok` and `Outcome::Err` | builds | E0308, mismatched types |
| `is_err()` on an input that fails, then a fallback | returns the fallback | panics, `hot path invariant violated: TooBig(9)` |

## What it shows

1. The hot arm bought nothing here. `#[profile(Hot)]` measured 0.621 in the
   `Outcome` arm and 0.657 in the `Just` arm, inside the noise and on the wrong
   side of it. The mechanism is in the rewrite: the `Just` arm keeps the check
   and turns the error into a panic, so the loop still carries one compare and
   one branch per element, and that branch is what keeps the loop from being
   vectorised. The raw arm, which has no check, is the only one that reaches
   0.14 ns.
2. `Outcome` and `Maybe` cost the same as `Result` and `Option` once inlined.
   No difference above the noise of the host.
3. Behind a call that is not inlined, every small carrier costs what the call
   costs, 1.2 to 1.8 ns with the raw call at 1.31 to 1.36. No carrier can be
   told from another at this resolution.
4. A wide error (four `u64`) behind a call that is not inlined is the one
   place a carrier showed: `Result` 2.24 to 2.35, `Outcome` 1.85 to 2.06. The
   cause was not isolated. It is presumably a difference in how the two enums
   are laid out and returned through memory, and it is one kernel on one host.
5. A caller that builds and behaves one way in the arm the tests run in can
   fail to build, or panic, in the arm that ships. This is not a measurement
   and does not depend on the host.

## What it does not show

It does not show that a `Just` arm can never pay. Where the hot function is
not inlined and returns a wide error, the `Just` arm returns a narrower value
and the rows already here bracket what that could be worth: a `Just` with a
panic behind a call measured 1.23 and 1.80 in its two builds, an `Outcome`
with a wide error behind a call 1.85 and 2.06. So a gain of up to about half
a nanosecond per call is possible and a gain of nothing is equally consistent
with these figures. The case was not built through the macro (it makes a hot
function `#[inline]`, so the case arises only when the inliner declines). It says
nothing about aarch64, about wasm, or about a kernel heavier than one add.
