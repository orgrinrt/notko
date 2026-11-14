# Benches still to write

Part of the notko stack audit of 14 November 2026. `AUDIT.md` is the entry.

notko has no bench on record (`NUMBERS.md`, 1.1). The roadmap says a bench of
a notko form runs in the consumer's harness (channel
`briefs/fixup/roadmap.md:382`), so each entry names the harness it would live
in. arvo's and hilavitkutin's bench harnesses exist; which of them is the
right home is the consumer's call. Each entry gives the scenario, the arms,
the sizes and what the result could decide. They are ordered by how much a
decision waits on them.

The two ad-hoc spikes of this audit are first drafts of entries 1, 2, 5 and 6
and should be replaced by them, not cited.

## 1. Does a hot arm pay

Scenario: a validated transform over a column, the validation being a range
check per element, the function tagged `#[profile(Hot)]`.
Arms: `Outcome` arm; `Just` arm; one type with the error constructor marked
cold; the check hoisted to one validation pass and the kernel unchecked over a
validated slice type; hand-written unchecked loop as the floor.
Each arm inlined and not inlined, with a one-word and a four-word error.
Sizes: 64, 4096 and 1048576 elements; failure rate 0, one in a million, one in
a hundred.
Decides: whether `#[profile(Hot)]` keeps its `Just` arm at all (`DESIGNS.md`,
section 3). If the `Just` arm does not beat the cold-hint arm by a stated
margin, the ruling quoted there says it should not exist.

## 2. The carriers against core, as a standing guard

Scenario: the same small fallible kernel through `Result`, `Outcome`,
`Option`, `Maybe`.
Arms: each carrier, inlined and behind a call, payloads of one, two and four
words, error of zero, one and four words.
Sizes: 4096 elements.
Decides: nothing new today; it is the regression guard that keeps "Outcome
costs what Result costs" true as the methods grow, and it settles the
unexplained wide-error difference of `NUMBERS.md` 1.3 with assembly beside
the numbers.

## 3. Debug-build cost of generic access

Scenario: kaski's development build. A row map through the product, a chain
of `?` through three functions, a `Fill` of 4096 pushes.
Arms: `opt-level` 0, 1 and 3; the notko form against the plain-slice or core
form.
Sizes: 4096 rows.
Decides: whether notko-hlist and the carriers need `#[inline]` or
`#[inline(always)]` on their small accessors, as hilavitkutin already does on
its own read product
(`hilavitkutin/mock/crates/hilavitkutin-api/src/values.rs:167-270`), and
whether a development build of the engine is usable.

## 4. Row access where the compiler cannot hoist the checks

Scenario: a map over the product with a data-dependent store (write `c` only
where `a` exceeds a threshold), and one with a gather (read `b` at an index
taken from `a`).
Arms: `read_row` and `write_row`; the product destructured into slices and
zipped; slices with lengths asserted once; unchecked.
Sizes: 1024, 65536, 4194304 rows; three and twelve columns.
Decides: whether the product needs a row-count-in-the-type form or a
checked-once view for slices (`NUMBERS.md` 2.6). The ad-hoc spike covered
only the case the compiler handles by itself.

## 5. Compile time of the list, cold and warm

Scenario: a library holding one list and one query, generated at each size.
Arms: `Length`; `Contains` of the last member; `ContainsAll` of a permuted
list; `Has::pick` of each member; `SubOf` of a permuted sublist of a quarter,
half and all of the list; `Project` for each carrier; a query that fails, for
the size and time of the diagnostic.
Sizes: 8, 16, 32, 64, 128, 256 members; `cargo check` cold, `cargo check`
after an unrelated edit, release build.
Decides: the ceiling on list length the crate should state, whether
`recursion_limit = "1024"` is advice the crate should give
(`notko-hlist/src/membership.rs:42`), and hilavitkutin's unit ceiling. Also
whether a non-marker form of membership is affordable, which is the fallback
if `marker_trait_attr` is withdrawn.

## 6. Compile time of a realistic consumer

Scenario: hilavitkutin's contract crate with a system of k kernels, each
reading four and writing two columns out of a registry of n.
Arms: k of 8, 64, 256; n of 16, 64, 128.
Sizes: as the arms.
Decides: whether entry 5's all-against-all figures are what a consumer pays
(each kernel asks a small sublist question) or a worst case nobody reaches.
This is the bench that says whether the list design scales to an engine.

## 7. `Fill` against the containers it replaces

Scenario: append n items of 1, 8 and 64 bytes one at a time and in batches.
Arms: `Fill` over a lent array; `Fill` over a lent slice; `heapless::Vec`;
`arrayvec::ArrayVec`; `Vec::with_capacity`; raw pointer writes as the floor.
Sizes: 16, 1024, 65536 items; batch of 1, 16, 1024.
Decides: whether the all-or-nothing batch rule (`src/lend.rs:150-156`) costs
anything, and gives the emission window (roadmap step HV15) its floor.

## 8. `Emit` through a shared receiver

Scenario: many producers writing records to one sink.
Arms: `Emit` implemented over a lock; over a single-producer queue per
thread; `Push` through an exclusive reference as the floor.
Sizes: 1, 4, 16 threads; records of 16 and 256 bytes.
Decides: nothing about notko's trait, which is one method; it decides what
the stack's default shared sink should be, and is listed because a daemon's
logging path is the first place `Emit` will be hot.

## 9. `Slot` and `MaybeNull` in a table

Scenario: a dense table of optional ids, scanned and filtered.
Arms: `Slot<NonZeroU32>`; `Option<NonZeroU32>`; `Maybe<NonZeroU32>`; a `u32`
with a hand-checked sentinel; `nonmax`.
Sizes: 4096 and 4194304 entries; 1, 50 and 99 percent occupied.
Decides: whether `Slot` earns its place beside `Maybe` on speed, or only on
what it says. If equal, the documentation should say equal.

## 10. A wasm arm for all of the above

The panel's wording rule is that no wasm performance statement is admitted
until a wasm arm exists (channel `briefs/fixup/panel-consolidation-1.md`,
call 11). None of the entries above has one, and no estimate in `NUMBERS.md`
is made for wasm. Entries 1, 3 and 4 are the ones whose result is most likely
to differ there, since wasm has no flags register and its engines vectorise
less.
