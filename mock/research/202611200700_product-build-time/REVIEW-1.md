# Review 1: `work/notko-build-bench` against `dev`

Canon gate: passed. notko declares no `canon_paths` (`mockspace.toml:16`), no design under `docs/design/` speaks to benches, and the round (`mock/design_rounds/202611200639/`) declares no design change. The bench is tier-2 tooling under `mock/`, so nothing governs it above the bench rules. This is the first read; a second is owed.

Test gate: support crate run from a `git archive` of the branch, `CARGO_INCREMENTAL=0 cargo test --offline`: 6 unit and 1 integration test, all green. Every body read. None is tautological; the seven spoil cases in `record.rs` and the clean-versus-no-clean control in `tests/one_call.rs` are real refusals. What is missing from the suite is listed under finding 1.

No commit carries a `Co-Authored-By` or `Claude-Session` trailer. Every subject is `type: lowercase imperative`, under 72 characters.

## Verdict

MERGE AFTER FIXES.

## Findings

1. `mock/benches/support/product-build-call/src/record.rs:576` and `mock/benches/src/lib.rs:253` [correctness]: the check can pass a wrong call, two ways.
   - The cell's point is never checked. `columns` is the `n` the arm passed to `record` (`call.rs:442`), and `verdict` only checks every call against the first. `AfterCell::config` carries the point (mockspace `bench-harness/src/driver/hooks.rs:112`) and the hook ignores it, so `bench.toml`'s "at that cell's point" is not checked by anything.
   - Whether the registry was compiled is never checked. `record` counts any non-fresh `compiler-artifact` named `notko_product_fixture` (`call.rs:446-451`). A build that loses `--features registry` compiles the empty crate (`fixture/src/lib.rs:210`), reports one fresh artifact, and passes. Cargo's artifact message carries `"features":[...]`; require `registry` in it.
   - Add a spoil case for each to `every_call_that_did_not_build_the_fixture_alone_fails_the_cell`, and a point argument to `verdict`.
2. `mock/benches/product-build/bench.toml:140` [correctness]: with only one arm, cargo's fixed cost is folded into every figure: process start, resolution, lock, the fingerprint walk of `notko-hlist`. That cost is unmeasured. NOTES.md's "per column" column, "about linear and small" up to 64, and the title "How long the compiler takes" all rest on it being negligible, and at 32 columns (111 ms) it may well not be. Add a baseline arm that runs the same cargo call with `registry` off (the empty crate) at every point. Then either subtract it, or drop the per-column and linearity claims. Re-run.
3. `mock/research/202611200700_product-build-time/NOTES.md:258-265` [correctness]: "Three passes, two samples each, so six samples per point" sits next to a "median" column, but each median is that of the three `warm` samples only (`summary.tsv`, `mode=warm`). The six-sample medians are 111.4, 227.2, 777.6 and 3567.7 ms. At 128 the gap is 744 against 778, and the cold samples there all sit above the warm ones. State which median is quoted, or quote both. The ranges, ratios, deltas and the 2.3 exponent do match the committed samples.
4. `NOTES.md:267` [rules, `every-finding-carries-its-predicate`]: "the next doubling ... would cost upwards of fifteen" is an extrapolation to 512 columns, outside the `Holds for` line. Cut it or mark it as unmeasured. Add to `Holds for` that the timed block is a whole `cargo build` invocation and that the median is over the warm mode.
5. `mock/design_rounds/202611200639/202611200639_changelist.src.lock.md:225` [drift]: "The init sample bench is removed." No sample bench exists in `dev` or in any commit on the branch (`git show --stat ccbd272`). Drop the sentence.
6. `mock/benches/support/product-build-call/tests/one_call.rs:1-4` [rules]: the header opens on a blank line and lacks the closing rule line every other file has. This is a hand-typed header; let ante write it.
7. `tests/one_call.rs:13`, `src/call.rs:326` [rules, `tests.md`]: `env!` in a test binary makes `cargo mock test` rerun it on every change. The fixture path is unavoidable here; say so where it is used, or derive it at run time.

## Verified

- Setup (source write, `notko-hlist` build, `clean -p`) and record are outside the timed `run` block; `e2e_ns` exceeds `algo_ns` by microseconds, not by a cargo invocation.
- All 56 recorded calls of the passing run: one fixture artifact, zero others, absent before, present after, no lock wait, exit 0.
- The run commit `d4d4ad1` is the last commit to touch bench code.
- Cited sketches and the stack-tracks research exist where named.
- No em-dashes, curly quotes or banned words in the added prose. The bold in the changelist headers is the round template's, and `dev`'s round already has it.
