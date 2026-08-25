# `const_destruct` (#133214): independent second read

**Phase one.** Written and committed before reading `FINDINGS.md` or any probe under this
directory. Sources: the tracking issue and its linked issues via the GitHub API, the `rust-src`
component of the pinned toolchain, the governing rule
(`.claude/rules/unstable-features.md`), and eleven probes of my own under
`second_read_probes/`. Phase two is appended at the bottom, after this file was committed.

## Contamination, disclosed up front

I ran `git grep -n 'const_destruct'` across the notko working tree while establishing the
current gate inventory, and it returned matching lines from `FINDINGS.md`. I saw five
fragments: that the feature is "in none of the vetting tables", the two `core`/`alloc` gate
paths, the number `158` attached to something I could not see the subject of, a phrase about
Route 9 needing no `const_destruct`, and a line from `PROPOSED.diff` adding the gate under the
`const` cargo feature.

What that does and does not cost:

- The `core/src/lib.rs:104` and `alloc/src/lib.rs:106` gate sites were already in my context
  from my own `rust-src` grep, run before the `git grep`. That finding is independently mine.
- "In none of the vetting tables" I had already established by reading the rule's tables
  directly. Independently mine.
- The number `158` I had not seen. I subsequently measured `157` myself, under a pattern I
  state below. I did not go looking for a way to reach 158.
- I now know the earlier expert proposes adopting the gate and that some "Route 9" avoids it.
  I did not know its verdict, its reasoning, or its tier before forming mine.

I did not open `FINDINGS.md` or any probe. The verdict below is formed from the sources named
above. From that point on I excluded `research/` from every grep.

## Verdict

**WATCH** in the rule's own vocabulary: allowed, sound, on a plausible stabilisation path, but
carrying known incomplete-implementation rough edges. It belongs in the "Watch" table beside
`const_trait_impl`, which it cannot be adopted without.

It is emphatically **not** forbidden and **not** remove-stale. Every one of the rule's four
forbidding signals is absent, and I checked each rather than inferring from the absence of an
`I-unsound` label alone.

The choice between ALLOWED and WATCH is the only part I regard as genuinely arguable, and the
practical consequence is nil: WATCH is a sub-kind of allowed, so both permit the gate. What
differs is whether the rough edges get written down. Two are live, so I say they should be, and
that is what WATCH is for.

## The three facts the vetting procedure demands

### Stabilisation status: unstable, actively developed, no FCP

The tracking issue is open, opened 2024-11-19 by compiler-errors, labelled `T-lang`, `T-libs`,
`T-types`, `C-tracking-issue`, `F-const_trait_impl`, `PG-const-traits`. The FCP checkbox in the
opening post is unchecked and no `proposed-final-comment-period` or `final-comment-period` label
is present, so **FCP has not started**.

Development is current, not historical. Pull requests naming the feature run from #132329
(2024-10-29, the implementation) through #146187 (2025-09, constifying `ptr::drop_in_place`),
#147708 (2025-10, putting `mem::drop` behind this gate), #153874 (2026-03, `constify const Fn*:
Destruct`) and #155616 (2026-04). The issue's own timeline shows referenced events into
2026-06, and its most recent event is the **`T-libs` label added on 2026-08-12**, thirteen days
before this reading. Adding libs to a tracking issue is movement toward libs-api sign-off, which
is a stabilisation-direction signal rather than an abandonment one.

### Soundness: no known hole, and the marker is not forgeable

- **No `I-unsound` issue exists.** Searching `repo:rust-lang/rust label:I-unsound const_destruct`
  returns 0. Searching `label:I-unsound Destruct in:title` returns one closed 2015-era MIR
  optimisation bug about destructuring tuples, unrelated to this trait.
- **No sound subset was split off.** `min_const_destruct` does not exist; the search returns 0.
  The rule names a split-off subset as "a strong signal the full feature is unsound by design",
  and it is the discriminator that puts both `specialization` and `generic_const_exprs` in the
  forbidden table. Its absence here is the opposite signal, and it is the single most important
  structural difference between this feature and the two the rule forbids.
- **The trait cannot be implemented by a user.** `core/src/marker.rs:1056-1062` declares it
  `#[lang = "destruct"]`, `#[rustc_deny_explicit_impl]`, `#[rustc_dyn_incompatible_trait]`,
  `pub const trait Destruct: PointeeSized {}`. So the marker is compiler-synthesised and nobody
  can assert droppability for a type that is not droppable. Probe `f_` confirms this on the pin:
  an explicit impl is refused with `E0322: explicit impls for the Destruct trait are not
  permitted`. That closes the obvious route by which a marker trait becomes a soundness hole.
- **Nothing in the thread raises soundness.** All eight comments are accounted for below.
  RalfJung, who leads wg-const-eval and is the person who would raise it, cc'd the working group
  in 2024 and his only subsequent comment (2026-02-18) is about whether the name `Destruct` is
  confusing. The thread's substantive content is one implementation gap and two naming or
  ergonomics questions.

### Staleness: not stale

Continuous activity across 2024, 2025 and 2026, with the most recent event thirteen days ago.
This is not the `unboxed_closures` shape the rule forbids for staleness (open since 2015, no
FCP, lang-team design concerns unresolved).

## The carve-out, and why it is only a fallback here

The rule's carve-out is for features "unlikely to ever stabilise". This one is on the pipeline,
so the carve-out is not the load-bearing argument. It nonetheless applies with unusual force,
and it is worth recording because it is the strongest single piece of evidence available:

**`core` and `alloc` both enable this exact gate**, at `core/src/lib.rs:104` and
`alloc/src/lib.rs:106` in the pinned toolchain's own `rust-src`. That is the carve-out's
condition met at its source rather than inferred.

The reliance is not token. Counting the string `[const] Destruct` under `core/src` and
`alloc/src`: **157 occurrences across 27 files**, 154 in core and 3 in alloc, at
`core/src/cell.rs`, `core/src/clone.rs`, `core/src/result.rs` and elsewhere. Two stable public
API items have their const-qualification gated on it:

- `core/src/ops/drop.rs:207-209`: `pub const trait Drop`, stable since 1.0, carrying
  `#[rustc_const_unstable(feature = "const_destruct", issue = "133214")]`.
- `core/src/mem/mod.rs:997-1003`: `pub const fn drop<T>(_x: T) where T: [const] Destruct`,
  stable since 1.0, same attribute.

That is precisely the shape the rule already accepted for `const_unsigned_bigint_helpers`: the
items are runtime-stable and only their const-qualification is gated, behind the const-traits
umbrella. The precedent is on the ALLOWED table and was reasoned exactly this way.

### No stable wrapper suffices

The carve-out's step 1 requires checking this, and the answer is no, for the capability in
general. Probe `c_` shows a bare generic `const fn consume<T>(_t: T) {}` refused with `E0493:
destructor of T cannot be evaluated at compile-time` on the pinned toolchain with no features.
Probe `i_` shows `const_trait_impl` alone does not fix it: same `E0493`. Probe `d_` shows the
`[const] Destruct` bound does. So the capability is carried by this feature specifically and is
not reachable from stable, nor from the gate notko already enables.

I scope that claim to the general capability. Whether notko's own particular need has a stable
route is a different question, and one I did not attempt in phase one because answering it means
reading the design under discussion.

## The rough edges, which are what make this WATCH rather than ALLOWED

Two are live, neither is a soundness matter, and both are the "incomplete implementation" band
the rule tolerates:

- **#148189, `Copy` should imply `const Destruct`** (open, `A-trait-system`, `T-compiler`,
  `needs-triage`, filed 2025-10-27 out of clarfonthey's question in the tracking thread). A
  `Copy` type cannot have a `Drop` impl, so the implication is sound and simply is not drawn.
  This one is directly on the topic this branch is named for, and anybody reasoning about a
  `Copy` bound interacting with const drop needs to know it is open.
- **#151502, `[const] Destruct` bounds aren't rendered** (open, `T-rustdoc`, `C-discussion`).
  Cosmetic, documentation-surface only.

**One rough edge that the thread reports is fixed on the pinned toolchain**, and I checked
rather than assumed. gamozolabs (2025-04-29) reported that a `ManuallyDrop` wrapper carrying a
`const Drop` impl was wrongly refused; oli-obk replied the same day that it should work and that
he had a branch. Probe `e_` runs their snippet verbatim and it compiles clean. Probe `e2_`
strengthens it by constructing the value and forcing evaluation with `const _: () = ...`, since
`e_` alone only defines the function. Probe `e3_` is the negative control: the same shape with a
bare `T` field instead of `ManuallyDrop<T>` is still refused with `E0493`, the exact error
gamozolabs reported. So `e2_` passing is a real result about `ManuallyDrop` rather than the
compiler having stopped checking.

## Blast radius, which is the part worth flagging

**`const_destruct` cannot be adopted alone.** Probe `g_` enables only `const_destruct` and
writes a `[const] Destruct` bound; it is refused with `E0658: const trait impls are
experimental`, pointing at #143874. So every adoption of this feature also requires
`const_trait_impl`.

That is not new exposure for notko, which already gates `const_trait_impl` at `src/lib.rs:9`
behind the default-on `const` cargo feature, and the rule has it vetted at WATCH. But the
dependency should be recorded, because the honest statement of this feature's stabilisation
prospects is that it cannot stabilise before the umbrella does, and the umbrella (#143874,
RFC 3762) currently carries `S-tracking-needs-design-proposal` and `B-experimental`, last
updated 2026-04-20. `const_destruct` is not the long pole; the umbrella is.

The section of the rule on the forbidden list as verification infrastructure does not bite here.
That argument is about `specialization` and `TypeId` letting a type observe which instantiation
it is in, which would break model-width transfer. `Destruct` is a compiler-synthesised marker
that cannot be explicitly implemented (probe `f_`), so it offers no route to per-instantiation
behaviour. Adopting it does not widen that blast radius.

## The whole thread, so the reading is checkable

Eight comments, all of them:

1. RalfJung, 2024-11-19: cc wg-const-eval.
2. gamozolabs, 2025-04-29: the `ManuallyDrop` gap, with a playground link.
3. oli-obk, 2025-04-29: agrees it should work, has a branch.
4. gamozolabs, 2025-04-29: acknowledgement.
5. clarfonthey, 2025-09-26: why does `Copy` not imply `const Destruct`. Became #148189.
6. clarfonthey, 2025-10-16: #147708 merged, adding `mem::drop` under this flag.
7. FeldrinH, 2026-02-18: is `Destruct` a confusing name, would `Droppable` be better.
8. RalfJung, 2026-02-18: the term is used elsewhere, and internally as a synonym for drop glue.

No comment raises a soundness concern. The unresolved question in the opening post is whether
`~const` bounds should be allowed on `const Drop` impls, which compiler-errors argues for and
says is what nightly implements. Probes `d_` and `e2_` confirm that is still what the pinned
nightly implements.

## Probes

Eleven, at `second_read_probes/`, runnable in one command with `./run.sh`. Six are negative
controls that must fail to compile; the runner asserts each probe's expected outcome and exits
non-zero on any mismatch. It caught two defects in my own work: an exit-status capture that
reported every probe as passing including the ones that must fail, and my own misclassification
of `g_` as a positive when it is a control.

| Probe | Question | Expected | Result |
|---|---|---|---|
| `a_gate_name_live` | Is `const_destruct` a live gate name on the pin, or renamed out from under us the way `const_convert` nearly was | pass | compiles, unused-feature warning only. No `E0635`. |
| `b_destruct_needs_gate` | Control. Naming `Destruct` ungated must be refused | fail | `E0658`, citing issue #133214 exactly |
| `c_generic_const_drop_no_feature` | Control. Does a stable route exist | fail | `E0493` destructor of `T` |
| `d_generic_const_drop_with_feature` | What the feature buys | pass | compiles, including a user `impl const Drop` |
| `e_manuallydrop_rough_edge` | Is the thread's reported gap live on the pin | pass | compiles. Gap is fixed. |
| `e2_manuallydrop_forced` | Same, with construction and forced const eval | pass | compiles |
| `e3_negative_control_real_drop` | Control for `e2_`. Real drop glue must still be refused | fail | `E0367` and `E0493` |
| `f_deny_explicit_impl` | Control. Can a user forge the marker | fail | `E0322` explicit impls not permitted |
| `g_destruct_without_const_trait_impl` | Control. Does it stand alone | fail | `E0658` pointing at umbrella #143874 |
| `h_const_mem_drop` | Is core's own `const fn drop` const-callable downstream | pass | compiles under forced const eval |
| `i_const_trait_impl_alone_insufficient` | Control. Is the gate notko already has enough | fail | `E0493`, so `const_destruct` is necessary |

Toolchain for all of them: `rustc 1.98.0-nightly (57d06900f 2026-05-27)`, edition 2024.

## The row I would land

Under "Watch (allowed, sound, but carries known incomplete-implementation rough edges)":

| Feature | Tracking | Rough edge to be aware of |
|---|---|---|
| `const_destruct` | #133214 | Names `core::marker::Destruct` so a `[const]` bound can discharge drop in const contexts. No `I-unsound` anywhere, no subset split off, and the marker is `#[rustc_deny_explicit_impl]` so it cannot be forged (`E0322`). `core` and `alloc` both enable the gate (`core/src/lib.rs:104`, `alloc/src/lib.rs:106`) with 157 `[const] Destruct` bounds across 27 files, and the const-qualification of stable `Drop` and `mem::drop` rides on it, so the std-internal carve-out applies at its source. No stable wrapper: a generic `const fn` dropping its `T` is `E0493` without it, and `const_trait_impl` alone does not lift that. Requires `const_trait_impl`, so it cannot stabilise before umbrella #143874, which carries `S-tracking-needs-design-proposal`. Live rough edges: #148189 (`Copy` does not imply `const Destruct`) and #151502 (rustdoc does not render the bounds). The `ManuallyDrop` gap from the 2025-04 thread is fixed on the pin, verified. No FCP yet; `T-libs` added 2026-08-12. |

## What I did not check

- I did not read `FINDINGS.md`, `PROPOSED.diff` or any probe in this directory during phase one,
  beyond the grep fragments disclosed at the top.
- I did not verify behaviour on any toolchain other than `nightly-2026-05-28`. The other three
  installed toolchains would answer a dating question and I had no dating question.
- I did not read the linked PR diffs (#132329, #147708, #153874), only their titles, states and
  dates. My soundness conclusion rests on the absence of `I-unsound` across the repository, the
  `rustc_deny_explicit_impl` attribute confirmed by probe, and the content of the tracking
  thread, not on having audited the implementation.
- I did not attempt to price notko's own need or judge whether it should adopt the gate. That is
  the design question, not the vetting question, and the rule asks the vetting question.
- The rule's step 4 says a verdict lands as a row in its tables. I have not edited
  `.claude/rules/unstable-features.md`. Landing the row is a separate act, and under the
  two-expert requirement no such call goes on one reading alone.

## Beyond the question

Two things I noticed that the rules bear on, neither of which changes the verdict.

**The question as posed is well formed and its premise checks out.** `const_destruct` is
genuinely absent from every table in the rule: forbidden, allowed, watch, remove-stale and
contested. I read all five. Under "Where the inventory lives", a gate with no row "is unvetted
and must not ship", so vetting it before adoption is the rule working rather than ceremony.

**`tests/ctor_const_path.rs:13` carries a bare `#![feature(const_trait_impl)]`**, not the
`cfg_attr(feature = "const", ...)` form that `src/lib.rs:9` and `tests/consttry_smoke.rs:13`
use. The rule's audit scope explicitly covers `tests/`. The gate itself is vetted so this is not
an unvetted-gate finding, but the unconditional form means that test cannot build on a
stable-toolchain configuration even with `default-features = false`, which is the configuration
`Cargo.toml` advertises for stable consumers. Worth a look by whoever owns that file; it is
outside my question and I have not touched it.
