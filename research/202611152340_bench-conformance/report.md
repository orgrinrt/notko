# Bench conformance in notko

This is the report of the job that was to bring every bench and every
measurement of notko onto the current mockspace bench harness. In notko that
job has nothing to port, and the reason is not in the benches but in the
repository: notko has no mockspace, so it has no `mockspace.toml`, no mock
directory, no `cargo mock`, no bench harness and no lints of the harness to
pass. The meter the job was given, `cargo mock --lint-only --commit` green and
`cargo mock bench run` running every bench to the end, does not exist here.

Where notko's benches should live is a question op holds, ticket
`where_do_notko_s_benches_live`, with the options of its own mockspace with a
harness, hilavitkutin's harness, or no benches of its own. Choosing for him
would choose the harness, and with it the repository the benches land in, so
this job stops here and the ticket was bumped.

## What the tree holds

There is no bench in notko, in the legacy sections form or in the composed
form. `research/202611140845_stack-audit/BENCH-CATALOGUE.md` lists the benches
still to be written and says each of them would run in a consumer's harness.

The timers in the tree are all in committed sketches of the stack audit of 14
November 2026:

- `research/sketches/202611140945_profile-arms-adhoc/src/bin/tiers.rs`, the two
  arms of `#[profile(Hot)]` and the carriers beside `Result` and `Option`.
- `research/sketches/202611140950_hlist-scaling-adhoc/src/bin/drive.rs` and
  `src/bin/rows.rs`, the run time and compile time of a `notko-hlist` list.

`research/sketches/202611141230_tier-file-tracking-adhoc/` was also read. It
records which builds cargo reruns when a tier file changes, and it times
nothing.

Each of these already calls itself an ad-hoc spike in its `FINDINGS.md` and
says no number of it is to be quoted without those words. The catalogue says
they are first drafts of its entries 1, 2, 5 and 6, to be replaced by those
benches and not cited. They are history and stay as they are: no sketch was
deleted or edited. Once a harness exists, the bench that supersedes each one is
cited from its `FINDINGS.md`, which is part of that later work.

No test, support crate or script times anything. A search for `Instant` and
`elapsed` over `src/`, `tests/`, `scripts/` and every member crate finds only
the word Instantiating in doc and test comments.

## Gates and tests

Nothing in the build changed, so no gate or test was run for this job. The one
commit of the branch is this report, and it passed the repository's commit-msg
hook, the workspace byline guard, with no bypass.

## Branches

`origin/work/sketch-read-product-and-hot-tier` is not merged into `dev`. It
holds a sketch of a borrowed read product, committed about an hour and a half
before this report and timing nothing, so it is somebody else's live work and
not bench data. It was left alone. No `salvage/*` or `work/bench-*` branch
exists, local or remote.

## Left undone

Porting the timed sketches to harness benches, and every bench of the
catalogue, waits on `where_do_notko_s_benches_live`.
