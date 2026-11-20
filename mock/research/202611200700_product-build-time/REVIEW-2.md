# Review 2: `work/notko-build-bench` against `dev`

Canon gate: passed. I checked it myself before reading Review 1's verdict. notko declares no `canon_paths` (`mockspace.toml:16`), and the diff touches no design under `docs/design/`, no `mock/crates/`, no crate source, no root file and no `Cargo.toml`. This is the second read and it agrees with the first.

Test gate: not run. `cargo mock bench test product-build` and a direct `cargo test` in the support crate were both refused by the saturation guard, at load 9.28 and then 6.48 on 4 cores. Every test body in the surface was read: 5 unit tests in `record.rs` and 1 integration test in `tests/one_call.rs`. None is tautological. One assertion is redundant (finding 5). Green is unconfirmed by this review.

Public surface: nothing public is touched. All 18 commit messages were read one at a time. None has a byline, trailer or tool advert. None is over 72 characters.

## Verdict

MERGE AFTER FIXES.

## Review 1, finding by finding

1. Point and registry not checked: addressed. `verdict` takes the cell's point (`record.rs:66`, `src/lib.rs:227`). `with_registry` is read from cargo's `"features"` (`call.rs:175-177`). There is one spoil case per field, nine in all, including both registry directions and the point (`record.rs:142-152`). The point check still compares the arm's echo of `point` with `cell.config.n` and observes nothing about what was compiled. That is acceptable because `prepare` rewrites `registry_{n}.rs` on any difference. NOTES.md:11 words it correctly.
2. No floor: addressed in the data. Finding 2 below covers the claim made about it.
3. Which median: addressed (NOTES.md:19).
4. Extrapolation, `Holds for`: addressed (NOTES.md:28, :36).
5. Sample-bench sentence: addressed. The topic of round 202611200711 records the correction, and the locked changelist is left alone, which is the right call.
6. Hand-typed header: addressed.
7. `env!` in the test binary: addressed. `own_dir` asks `cargo metadata` (`call.rs:50-68`). The `env!("CARGO_MANIFEST_DIR")` left at `call.rs:30` is a compile-time constant in the library, not a rerun trigger.

## Findings

1. `mock/research/202611200700_product-build-time/NOTES.md:32` [correctness]: "its warm medians, 111, 225, 744 and 3596 ms, agree with this run within the ranges above" is false at every point. Each of them lies below this run's six-sample range:
   - 110.87 < 111.20
   - 225.44 < 228.27
   - 743.57 < 776.06
   - 3596.40 < 3695.05

   This run is higher at all four points, by 32, 21, 66 and 221 ms, 17 minutes later on the same machine. So the run-to-run shift is larger than the in-run range, and NOTES.md:40 ("six samples ... give the range above and no more") understates the uncertainty. The shape still survives: the earlier run, less a ~50 ms floor, gives 3.97x and 5.11x for the last two doublings, against 3.96x and 4.94x here. Fix: rewrite :32 to say the earlier run sits below this one at every point. State in :40 that the spread between runs exceeds the spread within a run, so absolute figures are good to about a tenth. The ratios and the exponent are the result.
2. `mock/benches/product-build/bench.toml:5,16` and `mock/design_rounds/202611200711/202611200711_changelist.src.lock.md` ("`bench.toml` names it as `floor`, so the harness subtracts it before every delta") [correctness]: this is false. The harness differences against `floor` only in `ratio` normalise mode (mockspace `bench-harness/src/report.rs:194-216`). This bench uses the default mode. The committed `report.md` for the run therefore quotes raw deltas: "cargo_alone is -762.61 ms (94%) faster than baseline". It also warns that the baseline is the worst performer. Every "above the floor" figure in NOTES.md is computed by hand from `summary.tsv`. Fix it one of two ways:
   - Set `baseline = "cargo_alone"`, drop `floor`, and re-run. The harness then reports `from_clean` minus the floor as a paired delta with a CI, which is exactly the number NOTES.md wants.
   - Correct the `bench.toml` comment and say in NOTES.md that the subtraction is done by hand.

   The locked changelist's claim is wrong either way. Record that in the next round, or in this round's topic if the round is reopened.
3. `NOTES.md:25` [correctness]: the 128 row is computed from rounded medians. 809.61 - 47.14 = 762.47, so the value is 762 ms, not 763. The delta is 762.47 - 192.39 = 570.08, so +570, not +571. Every other cell of the table matches the committed `summary.tsv` and `samples.csv`, including all ranges, the per-column figures, 1.9x / 4.0x / 4.9x and the exponent of 2.3.
4. `NOTES.md:28` [correctness]: "The floor stays between 43 and 60 ms over every sample" is wrong at the low end. The n32 warm pass-2 sample is 42.69 ms. Change it to 42 to 60.
5. `mock/benches/support/product-build-call/tests/one_call.rs:24` [tests]: `verdict(&good, 16).is_err()` fails only because the recorded `columns` is the `8` the test itself passed. Nothing from the real build feeds it, so it duplicates `calls_all_at_another_point_fail` (`record.rs:165`). Replace it with the end-to-end negative Review 1 actually named: build `Fixture::Empty`, record it as `Fixture::Registry`, and assert the verdict fails. That is the only test that would show the feature parse refusing a lost `--features registry` on real cargo output.
6. Round 202611200711 [rules, `mockspace.md`: "topic, doc CL, lock, src CL, lock, close. No shortcuts"]: the round was opened after the work it governs.
   - The source change `323f66d` was committed at 07:10:31.
   - The run `20261120T071113Z-323f66d` started at 07:11:13.
   - The round was opened at 07:11:20 (`323bf3b`), and its doc changelist was locked 9 seconds later.

   The round is a retrospective record, not a gate. Round 202611200639 was done in the right order, so this was a shortcut, not a misunderstanding. Reordering the commits now would orphan the run directory's sha, so do not do that. Instead, state in the round that the source preceded it. `323f66d` also carries `REVIEW-1.md` inside a `fix:` commit. Leave it, but do not repeat it.
7. `d283b29` [rules, `pr-text-is-published-writing-in-ops-voice`]: the subject "record product-build run with the cargo_alone floor" leaves both identifiers unbackticked. Rewording it does not touch the run's sha.
8. `mock/benches/product-build/arms/{from_clean,cargo_alone}/src/lib.rs` [drift, `extract-the-shared-logic-never-mirror-it`]: the two arms differ only in the `Fixture` variant and the name string. Move the `setup` and `record` bodies behind one function in `product-build-call` that takes the `Fixture`, so each arm is the `timed!` shell alone.
9. `tests/one_call.rs:14,31` [tests, low]: `reset_log` and `prepare` act on the bench's real record and target (`own_dir`). Running the test during a `bench run` empties the run's call record and fails or falsely passes a cell. Give the test its own subdirectory under `own_dir`.

## Verified

- Every median, range, floor median, per-column figure, ratio, delta and the exponent in NOTES.md:23-28, against `samples.csv` and `summary.tsv` of `20261120T071113Z-323f66d` at all four points. The only mismatches are findings 3 and 4.
- 32 calls per point in each `calls.tsv` (16 processes x 2), 16 with the registry and 16 empty. All have one fixture artifact, zero others, absent before, present after, no lock and exit 0, so NOTES.md:11 holds.
- The src changelist of 202611200711 matches the code on `Fixture`, `record`, `verdict`, `own_dir` and the tests. The single mismatch is the `floor` claim (finding 2).
- No em-dashes, en-dashes, curly quotes or banned words in the added lines.

## Notes

Findings 1 to 4 are prose edits to NOTES.md and need no re-run. Finding 2's first option needs a re-run, and that re-run would also replace the hand arithmetic behind finding 3. The test suite still needs running once the machine's load allows. Nothing here was run.
