# Ad-hoc spike: a custom tier file is not a build input cargo knows about

This is an ad-hoc spike written for the stack audit of 14 November 2026
(`research/202611140845_stack-audit/`). It measures nothing. It is three
builds whose outcome does not depend on the host.

## What was asked

`#[profile(<Name>)]` with a custom name reads
`notko-optimisers/<Name>.rs` from the consuming crate while the macro expands
(`notko-macros-core/src/discover.rs:160-167`). Nothing in the expansion names
that file. Does cargo rebuild the crate when only that file changes?

## What was done

The crate has one function under the custom tier `Trace` and one caller that
matches on `Outcome::Ok` and `Outcome::Err`. With `Trace` based on `Cold` the
function returns an `Outcome` and the caller builds. With `Trace` based on
`Warm` it returns a `Maybe` and the caller cannot build.

| step | change | `cargo check -v` | file |
|---|---|---|---|
| 1 | tier file says `based_on = "Cold"` | checks, finishes | `out_step1_tier_is_cold.txt` |
| 2 | tier file changed to `based_on = "Warm"`, nothing else touched | `Fresh`, finishes in 0.01 s, no error | `out_step2_tier_now_warm_nothing_else_touched.txt` |
| 3 | one comment line added to `src/lib.rs`, tier file as in step 2 | `Dirty`, then E0308 twice, does not compile | `out_step3_source_touched.txt` |

Step 3 is the control: it shows the change made in step 2 does break the
crate, so the green build of step 2 was a build of the old tier.

The source is committed as it stood after step 3, so it does not compile.
That is the result, not an accident. Put `Cold` back in
`notko-optimisers/Trace.rs` to make it build.

## What it shows

A crate that keeps a tier file beside its manifest, the route the README
gives first, can change that tier and get a build that still carries the old
one, until some source file of the crate changes or the target directory is
cleaned. The tier decides a function's return type and whether its error
expression is dropped, so a stale tier is a stale signature.

## What it does not show

The route through `notko-build` was not tried. Its build script declares the
tier directory with `rerun-if-changed` (`notko-build/src/lib.rs:285-292`), so
the script runs again when a tier file changes. Whether the crate is then
recompiled depends on how cargo treats a build script that ran again and
printed the same lines, and that was not tested here.
