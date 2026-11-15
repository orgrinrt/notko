# Ad-hoc spike: what a `notko-hlist` list costs at run time and at compile time

This is an ad-hoc spike written for the stack audit of 14 November 2026
(`research/202611140845_stack-audit/`). It is not a bench. No number here
should be quoted without the words ad-hoc spike beside it.

## What was asked

1. Does a map over rows written through the product (`read_row`, `write_row`,
   `SubOf::put_row`) run as fast as the same map over plain slices?
2. How does the compiler's time grow with the length of the list, per kind of
   query?

## How it was run

One x86-64 virtual machine shared with other sessions' builds, toolchain
`nightly-2026-05-28`, nothing pinned, not quiet. The disk filled during one
run (see below).

`src/bin/rows.rs` is the run-time half: `c = a * 3 + b` over three `u32`
columns at 1024, 65536 and 4194304 rows, 11 samples, nanoseconds per row.

`build.rs` writes a library with a list of `SPIKE_N` members and one query
named by `SPIKE_Q`. `src/bin/drive.rs` times `cargo check --lib` or
`cargo build --lib` of that library, one process at a time, a fresh salt in
the source for every run.

- `drive verify 256` (`out_compile_verify.txt`) shows cargo marks the library
  dirty on each run, and that the control query, which asks the list for a
  member it does not hold, is refused with the crate's own diagnostic. So the
  runs do check something.
- `drive cold-check` turns the incremental cache off. This is the table to
  read for the cost of a query.
- `drive check` and `drive build` leave the cache on. Their first run at a
  size is cold and every later one changes one constant, so their least is the
  cost of a re-check after an unrelated edit, and only their largest figure is
  near a cold run. The first table produced by this spike
  (`out_compile_check.txt`) was read as cold cost for a while. It is not, and
  is kept with this note.

## Results, run time

`out_rows.txt`, least of 11:

| arm | 1024 rows | 65536 rows | 4194304 rows |
|---|---|---|---|
| slices, zip | 0.117 | 0.139 | 0.425 |
| slices, indexed, lengths asserted | 0.120 | 0.136 | 0.427 |
| slices, unchecked | 0.118 | 0.140 | 0.411 |
| product, `read_row` and `write_row` | 0.126 | 0.136 | 0.412 |
| product, `read_row` and `put_row` of one member | 0.127 | 0.140 | 0.433 |
| product, destructured by hand, zip | 0.129 | 0.137 | 0.448 |
| product over `Arr<1024>` | 0.130 | | |

No arm can be told from another. The prediction this spike was written to
test was that the bounds check `read_row` and `write_row` make per column per
row would stop the loop being vectorised. It was wrong. The assembly of
`product_whole_rows` was read (emitted with `cargo rustc --release --bin rows
-- --emit asm`, not committed): the compiler takes the least of the row count
and the three slice lengths as a trip count it can prove safe, runs a
four-lane `paddd` loop over that many rows, leaves the checked scalar loop for
whatever remains, and drops the stores that write `a` and `b` back to
themselves. The growth from 0.12 to 0.43 ns per row with size is the same in
every arm and is memory: twelve bytes read and four written per row leave the
cache at the largest size.

This is one kernel with no branch in it, three columns of one width. A kernel
with a data-dependent write, or columns the compiler cannot prove distinct,
was not tried.

## Results, compile time

`out_compile_cold_check.txt`, `cargo check --lib` with the incremental cache
off, least of 2, milliseconds:

| query | n = 32 | 64 | 128 | 256 |
|---|---|---|---|---|
| base (the list, its product type, one row value) | 91 | 97 | 124 | 255 |
| `Length` | 86 | 89 | 165 | 288 |
| `Contains` of the last member | 86 | 94 | 146 | 278 |
| `Has::pick` of the last member | 82 | 103 | 162 | 390 |
| `Project` (`made`, `write_row`, `read_row`) | 90 | 105 | 221 | 544 |
| `ContainsAll` of the list reversed | 83 | 167 | 686 | 4482 |
| `Has::pick` of every member in turn | 114 | 301 | 1351 | 11162 |
| `SubOf` of the list reversed | 124 | 313 | 1933 | 13691 |

The first five rows are one walk of the list and grow about linearly. The
last three ask a question of every member against the whole list. Each
doubling from 64 costs four to eight times, which is between quadratic and
cubic: n lookups, each a walk of up to n steps, each step carrying a type
whose printed size is itself n. At 32 members nothing is felt. At 128 the
all-against-all queries cost one to two seconds each use. At 256 they cost
four to fourteen seconds each use.

With the cache on (`out_compile_check_large.txt`, least of 3), a re-check
after an unrelated edit is 64 to 85 ms at 64 members, 100 to 220 ms at 256 and
160 to 520 ms at 512.

`out_compile_build.txt` is a debug build with the cache on, 2 runs. Its larger
figure is the first, near-cold run and includes code generation: `ContainsAll`
4.5 s at 256 and 35 s at 512, `pick` of every member 24 s at 256, `SubOf`
reversed 27 s at 256, `Project` 25 s at 512. Two entries at 512 failed because
the shared target disk filled (`No space left on device`). Those two are not a
result about the crate. Whether this spike's own output filled the disk or
another session's did is not known; the harness refused `cargo clean`, so the
library was rebuilt at 8 members afterwards to shrink what it left.

At 1024 members every query, the base included, ended with the compiler
killed by SIGSEGV (`out_compile_check_large.txt`). `out_compile_bare.txt`
narrows it: the list type alone checks at 1024 and at 2048, and the base with
its row value checks at 768. So it is the 1024-deep row value or the
normalised 1024-deep product type, and not the list as a type, that overruns
the compiler's stack. Which of the two was not isolated. The spike sets
`recursion_limit = "16384"`, so the limit a consumer would meet first at the
default 128, or at the 1024 the crate's notes advise, was not what stopped it.

## What it does not show

Nothing here is a statement about release builds of a real consumer, about
link time, or about error-message length when a query fails. The members are
all `u32` columns. A consumer with fewer, larger queries per list will sit
well under these figures, and one that asks all-against-all questions of many
lists will sit over them.
