# Sketch N3: does a hot function return the same refusal in a release build

Hypothesis: a function under `#[profile(Hot)]`, given an input its debug build refuses with an ordinary error, returns the same refusal in a release build and does not abort.

**Outcome:** FAILS WITH a panic, `hot path invariant violated: OverLimit { value: 9, limit: 5 }`, exit status 101, in a release build with the consumer's `internal` feature on; with `panic = "abort"` the same build aborts, exit status 134. A release build without the feature returns the same refusal as the debug build.

This is a feasibility sketch, written unattended on 15 November 2026 by a dispatched seat; the dispatcher opened the output files before writing this. Nothing in it is timed, and nothing in it is authority.

## What the earlier spike settled, and what it left

`research/sketches/202611140945_profile-arms-adhoc/` already shows, for a caller that asks `is_err()` and falls back: release without `internal` recovers (`out_recover_debug_arm.txt`), release with `internal` panics (`out_recover_release_arm.txt`). It also holds the two compile failures of a caller written against one arm (`out_probe_*`). None of that is repeated here.

It left: any dev build; the two conditions of the gate taken apart; the refusal itself printed, not a fallback; a literal abort; exit statuses; and its commands were issued by hand. This sketch's `run.sh` ran as one command.

## The function and the caller

`src/lib.rs`: `admit(value, limit) -> Result<u32, OverLimit>` under `#[profile(Hot)]`, refusing a value over the limit. `src/bin/refuse.rs` calls it with (3, 5), the control every build admits, then with (9, 5). The caller is written twice under the attribute's own gate, because no one caller that names the refusal builds in both arms.

## Attempts, one per build

Toolchain `nightly-2026-05-28`, notko with `macros` and `try_trait_v2`.

| build | feature `internal` | debug assertions | arm | admit(9, 5) | exit | output |
|---|---|---|---|---|---|---|
| dev | off | on | Outcome | returned the refusal | 0 | `out_dev_feature_off.txt` |
| dev | on | on | Outcome | returned the refusal | 0 | `out_dev_feature_on.txt` |
| release | off | off | Outcome | returned the refusal | 0 | `out_release_feature_off.txt` |
| release, debug assertions kept | on | on | Outcome | returned the refusal | 0 | `out_release_checked_feature_on.txt` |
| release | on | off | Just | panicked | 101 | `out_release_feature_on.txt` |
| release, `panic = "abort"` | on | off | Just | aborted | 134 | `out_release_abort_feature_on.txt` |

The four cells that return, each:

```
admit(3, 5) returned the value 3
admit(9, 5) returned the refusal OverLimit { value: 9, limit: 5 }
the process reached its end
exit status: 0
```

`out_release_feature_on.txt`, without its short backtrace:

```
feature internal: true, debug assertions: false, arm emitted: Just
admit(3, 5) returned the value 3

thread 'main' panicked at src/lib.rs:18:1:
hot path invariant violated: OverLimit { value: 9, limit: 5 }
exit status: 101
```

`out_release_abort_feature_on.txt` ends:

```
hot path invariant violated: OverLimit { value: 9, limit: 5 }
Aborted
exit status: 134
```

The control line `admit(3, 5) returned the value 3` is in all six, so the panic belongs to the refused input and not to the build.

## What it shows

The divergence is not between debug and release. It is between the one cell where the feature is on and debug assertions are off, and every other cell, which is the gate as the code has it (`notko-macros-core/src/rewrite/hot.rs:52`, `:55`). An earlier version of this file said no test pins this behaviour. That was wrong, found by a second read on 16 November: `notko-macros/tests/smoke.rs:73-79` (`hot_panics_on_the_error_arm`, under `should_panic`) pins the panic, and `notko-macros/README.md` documents it. `notko-macros/tests/consumer_cfg.rs` builds a consumer in both arms and runs neither. The code's own comment describes it as intended (`src/just.rs:22-26`); that is a statement about code, and notko declares no canon to check it against.

## Predicate

Holds for: toolchain `nightly-2026-05-28`; notko at `f5d8f56`, features `macros` and `try_trait_v2`; x86-64 Linux; a hot function whose source returns `Err` on an input-dependent check, written with `return Err(..)`.

- Same refusal as the debug build: holds where the consumer's `internal` feature is off, in the dev and the release profile; and where it is on with debug assertions on.
- Panic in place of the refusal: holds where `internal` is on and debug assertions are off. It unwinds under the default panic strategy and aborts under `panic = "abort"`.

Not checked: `?` inside the hot body, a `match` on a callee's result, other targets, other error types, and whether any consumer turns on `macros` or declares `internal`.
