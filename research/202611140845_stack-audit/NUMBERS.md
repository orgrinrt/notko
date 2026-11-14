# Numbers: the record, why they are what they are, and estimates

Part of the notko stack audit of 14 November 2026. `AUDIT.md` is the entry.

Three kinds of statement are kept apart in this file and are never mixed.
A measurement on record names a committed result that existed before this
audit. An ad-hoc spike is something this audit wrote and ran, committed under
`research/sketches/` with a `FINDINGS.md`; its numbers come from one shared,
busy x86-64 virtual machine and are indications. An estimate is reasoning,
with its assumptions written out, and is not a number anybody measured.

## Part 1. Every bench and spike on record

### 1.1 Benches

There are none. notko has no `benches` directory, no bench target in any of
its five manifests, and no committed timing anywhere in the repository. The
roadmap's position is that "a bench of a notko form runs in the consumer's
harness" (channel `briefs/fixup/roadmap.md:382`). No such bench was found in
a consumer, and the consumers could not be searched, so the honest statement
is: none on record that this audit could find.

What follows from it. Every sentence in notko about cost is unmeasured. That
covers "differ by what a branch costs" (`README.md:7`), the cost column of
`src/lib.rs:44-54`, and the premise of `#[profile(Hot)]`.

### 1.2 `research/202608251000_consttry-copy-bound`

What it claims to measure: nothing about speed. It says so: "Not benchmarked,
and nothing here claims a performance number" (`FINDINGS.md`, closing
section). It is a set of compile probes that decide which bounds the const
forms of `?` can carry on the pinned toolchain.

Does it support what is drawn from it. Yes, for what it draws: the `Copy`
bound could be relaxed to a const `Destruct` bound, a const `From` is
available, and turning the default features off is not a dependable route to
stable inside a dependency graph (`FINDINGS.md:42`). The second read gives
`const_destruct` a verdict of WATCH, which matches the workspace rule's table.
The probes `p01` to `p08` were not opened for this audit, so the record is
taken at its word on what each printed.

Why the results are what they are. They are facts about the compiler at one
nightly, not about notko. They age with the toolchain pin and should be rerun
when it moves; nothing in the repository reruns them.

### 1.3 Ad-hoc spike `research/sketches/202611140945_profile-arms-adhoc`

What it asks: whether the `Just` arm of a hot function is faster than its
`Outcome` arm, whether notko's carriers cost more than core's, and whether a
caller written against one arm builds and behaves the same against the other.

Why the numbers are what they are.

- Every checked carrier, inlined, 0.60 to 0.69 ns per element against 0.14
  for the unchecked loop. The unchecked loop is vectorised. A loop with an
  early exit on a data-dependent condition is not, on this compiler, so each
  element pays a compare, a branch that is always predicted, and scalar
  arithmetic. The carrier's name does not enter into it: after inlining,
  `Result`, `Outcome`, `Option`, `Maybe` and `Just`-with-a-panic are the same
  machine code or within a few instructions of it. The assembly of these loops
  was not read; this is the mechanism the figures are consistent with, and the
  one thing that would contradict it, a carrier standing apart from the
  others, did not appear.
- `#[profile(Hot)]` in its two arms, 0.621 and 0.657. The rewrite keeps the
  check and panics on failure
  (`notko-macros-core/src/rewrite/hot.rs`), so the arms differ only in what
  the never-taken branch leads to.
- Behind a call, 1.2 to 1.8 ns for everything including the unchecked
  function. The call dominates. Identical code built twice differed by up to
  0.57 ns on these rows, so no ordering among them means anything.
- A four-word error behind a call: `Result` 2.24 to 2.35, `Outcome` 1.85 to
  2.06. Unknown. A wide enum is returned through memory, and two enums with
  the same variants in a different order can place the payload differently,
  which changes how many words are written on the success path. That is a
  guess. It was not checked in assembly, and it is one kernel.
- The probes are not numbers. They fail or pass by the type system and by a
  panic, and are independent of the host.

Does it support what is drawn from it. It supports "no gain was shown for the
`Just` arm in a small inlined kernel" and "the arms differ for callers". It
does not support "the `Just` arm can never pay"; its own `FINDINGS.md` says
where it might.

### 1.4 Ad-hoc spike `research/sketches/202611140950_hlist-scaling-adhoc`

What it asks: whether a row map through the product is as fast as slices, and
how compile time grows with list length.

Why the numbers are what they are.

- Run time, every arm equal at every size. `read_row` and `write_row` are
  small generic functions over a three-deep type; after monomorphisation and
  inlining they are three indexed loads and three indexed stores. The compiler
  turns the three per-row bounds checks into one trip count computed before
  the loop and vectorises the body. This was read in the emitted assembly,
  which was not committed. The rise from 0.12 to 0.43 ns per row at four
  million rows is memory bandwidth, equal in all arms.
- Compile time, linear queries. One trait obligation per cell.
- Compile time, all-against-all queries. `ContainsAll<R>` over a reversed
  list is n membership proofs each of depth up to n, and every obligation
  carries types of size proportional to n, so the work the solver does grows
  toward n cubed unless its cache collapses the repeats. The measured growth,
  four to eight times per doubling, sits between n squared and n cubed. How
  much the solver's cache saves was not examined.
- The compiler crash at 1024 members. A stack overrun in the compiler on a
  value or type nested 1024 deep. The list type alone checks at 2048. Which
  pass overran is unknown.

Does it support what is drawn from it. It supports "the product costs nothing
in a branch-free kernel on three columns" and "all-against-all bounds are
affordable to about 64 members and expensive from about 128". A first table
from this spike was misread as cold compile cost while it measured cached
re-checks; the `FINDINGS.md` records the mistake and which table replaced it.
That history is a reason to hold these figures loosely until a committed
bench repeats them.

### 1.5 The earlier audits' numbers

`arvo/mock/research/202611131215_stack-audit/NUMBERS.md` and
hilavitkutin's `BENCH-RECORD.md` and `ESTIMATES.md` were not read
(`LEDGER.md`). No number of theirs is used here.

## Part 2. Estimates: the designs built to intent, against four yardsticks

Every line of this part is an estimate by reasoning. The two places where an
ad-hoc spike bears on one are named. None is a measurement.

Assumptions throughout: release build, one codegen unit or LTO across the
boundary in question, x86-64 or aarch64, the kernel small enough to inline.
Where a notko form sits behind a call the compiler does not inline, the
estimate is the call, about one to two nanoseconds, and nothing about notko.

### 2.1 The carriers

Against hand-written assembly. A hand would return a small success value in a
register and signal failure in a flag or a second register. `Outcome<u32, E>`
with a small `E` does the second; `Maybe<T>` over a type with a niche does
the equivalent with no second register. Estimate: equal for small payloads.
For a wide error the compiler returns through memory where a hand would keep
the success path in registers; estimate one to three extra stores per call on
the success path, which matters only at call rates in the hundreds of
millions per second.

Against std Rust and no_std Rust. `Result` and `Option` are in `core`, so the
two yardsticks are one. Estimate: equal, by construction of the types. The
ad-hoc spike of 1.3 agrees for the cases it ran.

Against existing libraries. There is no library whose purpose is a renamed
`Result`; the comparable things are error-handling crates. `anyhow` boxes its
error, which allocates on the failure path and keeps the success path one
pointer wide; notko never allocates, so it is cheaper on failure and the same
or slightly wider on success. `thiserror` only derives impls on a plain enum
and is cost-neutral, the same as `Outcome` with that enum.

### 2.2 `#[profile(Hot)]` built to its intent

The intent is a tier where a failure proven unreachable costs nothing. Two
designs could meet it.

As built (check kept, failure panics). Estimate: no gain inside an inlined
loop, since the branch remains; a gain of the size of the wide-error return
(2.1 above) where the function is not inlined and the error is wide; zero
otherwise. The ad-hoc spike agrees on the first.

As a hand would write it (no check, the invariant established once outside
the loop). Estimate: this is the 0.14 against 0.6 ns per element gap of the
ad-hoc spike's raw arm, a factor of four in a kernel that vectorises once the
check is gone and nothing in a kernel that would not vectorise anyway. No
attribute on the function can deliver that; it comes from moving the check to
where the data enters, which is a type (`Boundable`, `NonZeroable`, a
validated slice) and not a tier.

Against existing means. `core::hint::cold_path`, `#[cold]` on the error
constructor, and `likely`-style hints keep one return type. Estimate: they
recover the layout benefit (the failure path out of line) and none of the
vectorisation benefit, the same as the tier as built, at no cost to callers.

### 2.3 `Slot`, `MaybeNull`

Against a hand: equal, they are the value with one pattern reserved, which is
what a hand does with a null pointer or a zero id. Against `core`:
`Option<NonZero<T>>` and `Option<&T>` are the same layout; estimate equal.
Against `nonmax` (the reserved pattern is the maximum, by an xor on every
access): `Slot` reserves zero and needs no xor, so estimate one instruction
cheaper per access than `nonmax`, and equal to `Option<NonZero>`.

### 2.4 `Fill` over lent storage

Against a hand: a hand writes through a pointer and a count with one capacity
compare per append; `Fill::push` is that. Estimate: equal for single pushes.
For a bulk append the all-or-nothing rule needs the length up front
(`src/lend.rs:150-156`), one compare for the batch, then a copy; estimate
equal to `copy_from_slice` when the source is a slice and equal to a hand loop
otherwise. Against std: `Vec::push` carries a capacity compare and a cold
growth path; `Fill` has the compare and a refusal in place of growth, so
estimate equal on the hot path and strictly bounded where `Vec` may
reallocate. Against `heapless::Vec` and `arrayvec::ArrayVec` (inline fixed
storage, the same refusal on overflow): estimate equal; the difference is
that `Fill` borrows its storage from whoever lends it, which those two cannot
express.

### 2.5 `Push`, `BulkPush`, `Emit`

Statically dispatched traits with one method each. Estimate: zero cost over
calling the implementor directly, against every yardstick. Comparable:
`core::fmt::Write` (bytes of text, fallible), `embedded-io::Write` (bytes,
fallible, short writes), `bytes::BufMut` (bytes, infallible, panics when
full). notko's are generic in the item, which none of those is, and the
infallible ones are infallible in the type and not by panic.

### 2.6 `notko-hlist`

Run time against a hand: a type-level list has no run-time form, and the
product's row access is indexed loads and stores. Estimate: equal to
hand-written struct-of-arrays code wherever the compiler can prove the trip
count, as in the ad-hoc spike of 1.4. Estimate with less confidence: in a
kernel with a data-dependent store, or where columns come through a pointer
the compiler cannot prove distinct, the per-row bounds checks remain and cost
one compare and branch per column per row, a few tens of percent in a tight
loop. A hand would hoist them. The crate offers no unchecked row access and
no way to state the row count in the type for slices, so that case has no
remedy today except taking the product apart by hand.

Run time against existing frameworks. `frunk` holds values in its list and is
also zero cost at run time; it has no column products. `soa_derive` generates
a concrete struct of vectors per type and allocates; its iteration is zipped
slices, equal to the product here. The query tuples of `bevy_ecs` and `hecs`
resolve columns at run time per archetype and then iterate slices; estimate
the product is equal inside a chunk and cheaper at the chunk boundary, since
which columns exist is settled at compile time.

Compile time against the same. Estimate: comparable to `frunk` for the same
list lengths, since both recurse through the trait solver cell by cell, and
far above a derive that emits flat code. The ad-hoc spike's figures are the
only evidence. This is the yardstick on which the design is weakest, and for
hilavitkutin, whose audit already calls compile time part of the product
(`hilavitkutin/mock/research/202611131215_stack-audit/AUDIT.md`, the
unstable-feature and solver-overflow findings in its first 120 lines), it is
the one that will be felt.

### 2.7 What the estimates rest on, and what would overturn them

They rest on inlining. notko-hlist carries no `#[inline]` attribute; its
functions are generic, so they are available to inline across crates, and the
ad-hoc spike saw them inlined. A debug build inlines nothing, and there every
carrier method, every `read_row` and every `pick` is a call. For kaski, which
is played in development builds too, the debug cost of deep generic access is
a real number nobody has. That is the first bench in `BENCH-CATALOGUE.md`.
