# notko as part of the stack: quality review and audit, 14 November 2026

State judged: notko `dev` at `8a74dbc`, with arvo `dev` and hilavitkutin `dev`
as their clones stood on the morning of 14 November. Nothing in notko was
changed. The files written are this report and three ad-hoc spikes under
`research/sketches/`.

## Summary

notko is the right crate in the right place. A small, dependency-free,
allocation-free home for the stack's words for can-fail and can-be-absent, for
lent storage and for writers, sitting below the numeric crate, is a sound
foundation and matches what op has said of it. Its best parts are the plain
ones: the three carriers as types, `Lend` and `Fill`, the writer traits, and
the structural half of the type-level list. Those are good as they are.

Three things are not.

First, the crate's headline is about cost and nothing in it is measured. It
says its carriers "differ by what a branch costs". As a classification that
is true, and two of the three points were already in core at the same price:
`Outcome` costs what `Result` costs and `Maybe` what `Option` costs, which is
correct for a vocabulary. The whole of the cost promise therefore rests on
`Just` and on the attribute that switches a function to it. There is no bench
of any notko form on record, in notko or, as far as could be found, in a
consumer.

Second, `#[profile(Hot)]` is going wrong. It gives one public function two
return types depending on a cargo feature. A caller that builds and passes
its tests against the debug arm can fail to compile, or panic where it used to
recover, against the release arm; an ad-hoc spike written for this audit shows
all three cases, and the crate's own test holds it to the standard that a
construct builds in both arms or neither. The arm keeps its check and turns
the failure into a panic, so in the kernel tried it bought no speed, and the
most the spike leaves room for elsewhere is about half a nanosecond per call
that is not inlined. The one statement of op's intent
for a tier called Hot that this audit saw says it may give something up only
for a proven, meaningful gain. None is proven. The intent can be met, by a
type that cannot hold the bad value and by cold-path hints, with one return
type throughout.

Third, the newest design, the product of columns in notko-hlist, already
missed its first consumer. Its members must be `'static`, so hilavitkutin
wrote its own read product the day it landed, and it also keeps its own copies
of notko's writer traits. At run time the product is free in the one kernel
tried. At compile time a question asked of every member of a list grows
between quadratically and cubically: about a fifth of a second at 64 members
and four to fourteen seconds at 256, per use, in an ad-hoc spike. That is a
worst case, a whole list asked against itself. A kernel asking for a few
columns out of many asks far less, and whether an engine's real questions
come near the worst case is a bench nobody has written.

Around these: the stack is in practice nightly-only through notko, which is a
position op has not been seen to reject and which the README presents as
optional when it is not; `Slot` cannot hold the downstream non-zero types its
own documentation offers it to; and op's intents for notko (bit flags, byte
casting, branch hints, accounting beside lending) are written nowhere in the
repository.

For kaski the carriers and the write product fit, the hot tier should not be
used, and the read product and compile time are what will be felt first. For
a generic application the missing part is the edge to the rest of Rust and
any collection beyond append. For a daemon the contracts are the best fit of
all, and the hot tier and the nightly surface are what it would have to ban
or pin against.

What was not read matters and is in `LEDGER.md`: op's capture files and the
registries could not be opened, the downstream trees could not be searched,
and the night's pull requests could not be listed. Every number this audit
produced is an ad-hoc spike from one busy shared machine.

## The marks

| part | mark | where |
|---|---|---|
| the concept and direction | easy to get wrong but can be right | `DESIGNS.md` 1 |
| the three carriers as types | good as it is | `DESIGNS.md` 2 |
| `#[profile(Hot)]` | going wrong | `DESIGNS.md` 3 |
| `Warm`, `Cold`, the rewrite | easy to get wrong but can be right | `DESIGNS.md` 4 |
| `Maybe` layout, `MaybeNull`, `Slot`, `NicheFilled`, `NonZeroable`, `Boundable` | easy to get wrong but can be right | `DESIGNS.md` 5 |
| `ConstTry`, `ConstFromResidual`, `HasTrivialCtor` | good as it is | `DESIGNS.md` 6 |
| `Push`, `BulkPush`, `Emit` | good as it is | `DESIGNS.md` 7 |
| `Lend`, `Fill`, `Exhausted` | good as it is | `DESIGNS.md` 8 |
| notko-hlist: list, places, counting | good as it is | `DESIGNS.md` 9a |
| notko-hlist: membership | easy to get wrong but can be right | `DESIGNS.md` 9b |
| notko-hlist: the product | easy to get wrong but can be right | `DESIGNS.md` 9c |
| the toolchain position | easy to get wrong but can be right | `DESIGNS.md` 10 |
| how the crate is governed | going wrong | `DESIGNS.md` 11 |
| the benches | there are none; nothing to mark | `NUMBERS.md` 1.1 |
| the one research record on file | good as it is, for what it claims | `NUMBERS.md` 1.2 |
| fit to kaski | easy to get wrong but can be right | `DOWNSTREAM.md` 2.1 |
| fit to a generic application | easy to get wrong but can be right | `DOWNSTREAM.md` 2.2 |
| fit to a server daemon | easy to get wrong but can be right | `DOWNSTREAM.md` 2.3 |

Nothing is marked back to the drawing board. The nearest is the mechanism of
`#[profile(Hot)]`, a return type switched by a feature; its intent survives
and so the part is marked going wrong, with what right requires stated.

## A ratified position that works against a stated goal

The brief asks that such a thing be said and not weighed away. One was found.
The roadmap, stamped by two outside experts and built on op's order, refuses
a mockspace for notko and sends its benches to "the consumer's harness"
(channel `briefs/fixup/roadmap.md:382`). The stated goal is "optimisations
first and foremost, always" (`briefs/fixup/op-answers-6.md`). With no bench
address of its own and no consumer bench found, the lowest crate of an
optimisation-first stack has no measurement at all. The position is defended
here as to ceremony, since notko needs no design rounds. It works against the
goal as to measurement, and the evidence is section 1.1 of `NUMBERS.md`.

## The files

- `DESIGNS.md`: the concept and each design, with evidence, mark, and what
  right requires.
- `NUMBERS.md`: the record of benches and spikes with why each number is what
  it is, then estimates against hand-written assembly, std, no_std and named
  libraries.
- `BENCH-CATALOGUE.md`: ten benches still to write and what each decides.
- `DOWNSTREAM.md`: what each kind of consumer needs, and the earlier audits'
  findings tested against the state now.
- `LEDGER.md`: what was read to what depth, what was not, and what the
  harness refused.
- `../sketches/202611140945_profile-arms-adhoc/`,
  `../sketches/202611140950_hlist-scaling-adhoc/` and
  `../sketches/202611141230_tier-file-tracking-adhoc/`: the three ad-hoc
  spikes, each with its `FINDINGS.md`.

## What each pass changed

Pass 1. Everything above, written from one reading of the whole crate, the
governing tier as far as it could be reached, and the consumers' manifests
and contract crate. During it one prediction was overturned by its own spike
(the product's bounds checks were expected to block vectorisation and did
not) and one spike table was found to measure cached re-checks and was
replaced by a cold run.

Pass 2. The citations into hilavitkutin's contract crate and kaski's manifest
were read again at their lines, and the passages the three main findings rest
on were re-read in the source. Four things changed.

- The first finding was overstated. Pass 1 said the crate's claim "is not
  what the code delivers". The source states a classification
  (`src/lib.rs:46-48`), which is true. The finding is now that the cost
  promise rests wholly on `Just` and the attribute, and that none of it is
  measured (`DESIGNS.md` 1).
- The hot tier finding was narrowed and sharpened. Narrowed: a consumer that
  builds both configurations meets the two compile failures early, a private
  hot function is not subject to feature unification, and the spike leaves
  room for a small gain behind a call with a wide error. Sharpened: the
  crate's own test names the standard the three probes fail
  (`DESIGNS.md` 3a to 3c). The mark did not move.
- The advice to use cold-path hints was demoted from a remedy to a bench to
  run, because hilavitkutin's design bans the `likely` hint on a measurement
  of its own while op has asked for branch hints in notko. That tension is
  now recorded (`NUMBERS.md` 1.5).
- The custom-tier mechanism, which pass 1 had not read, was read and judged:
  heavy for two settings, and its tier file is not a build input cargo knows
  about, at that point by reasoning (`DESIGNS.md` 4).

Smaller: the compile-time figures are now called a worst case in the summary;
the `'static` bound on the product is quoted with its own justification; the
absence of consumer benches is now supported for hilavitkutin by its audit's
bench record, read whole.

Pass 3. The remaining citations into arvo, hilavitkutin and the workspace
catalogue were read again at their lines; all held. One claim that pass 2
had made by reasoning was put to a test and held: a third ad-hoc spike shows
a custom tier file can change while cargo reports the crate fresh and builds
the old tier (`../sketches/202611141230_tier-file-tracking-adhoc/`). The
central sections of `DESIGNS.md` were read through once more and three
wordings were corrected: a panic had been called a process abort, the hot
tier's intent had been attributed to op where the evidence is the design's
own text, and the reason `Cold` and `Warm` were not inlined in the spike was
misstated. No mark moved and no finding was added or withdrawn, so the passes
stop here.

What a further pass could still change, were the means there: a search of the
consumers for uses of `#[profile]`, `Slot` and the lending types; op's capture
files and the registries; and the benches of `BENCH-CATALOGUE.md`, the first
of which would settle whether the hot tier's mark is too kind or too harsh.
