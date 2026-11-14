# notko as part of the stack: quality review and audit, 14 November 2026

State judged: notko `dev` at `8a74dbc`, with arvo `dev` and hilavitkutin `dev`
as their clones stood on the morning of 14 November. Nothing in notko was
changed. The files written are this report and two ad-hoc spikes under
`research/sketches/`.

## Summary

notko is the right crate in the right place. A small, dependency-free,
allocation-free home for the stack's words for can-fail and can-be-absent, for
lent storage and for writers, sitting below the numeric crate, is a sound
foundation and matches what op has said of it. Its best parts are the plain
ones: the three carriers as types, `Lend` and `Fill`, the writer traits, and
the structural half of the type-level list. Those are good as they are.

Three things are not.

First, the crate's headline claim is not what it delivers. It says its
carriers "differ by what a branch costs". `Outcome` costs what `Result` costs
and `Maybe` what `Option` costs, which is correct for a vocabulary and is not
a difference in cost. There is no bench of any notko form on record, in notko
or, as far as could be found, in a consumer. Every cost sentence in the crate
is unmeasured.

Second, `#[profile(Hot)]` is going wrong. It gives one public function two
return types depending on a cargo feature. A caller that builds and passes
its tests against the debug arm can fail to compile, or panic where it used to
recover, against the release arm; an ad-hoc spike written for this audit shows
all three cases. The arm keeps its check and turns the failure into a panic,
so in the kernel tried it bought no speed. The one statement of op's intent
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
and four to fourteen seconds at 256, per use, in an ad-hoc spike. That is
affordable for small lists and is the thing most likely to hurt an engine.

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
- `../sketches/202611140945_profile-arms-adhoc/` and
  `../sketches/202611140950_hlist-scaling-adhoc/`: the two ad-hoc spikes, each
  with its `FINDINGS.md`.

## What each pass changed

Pass 1. Everything above, written from one reading of the whole crate, the
governing tier as far as it could be reached, and the consumers' manifests
and contract crate. During it one prediction was overturned by its own spike
(the product's bounds checks were expected to block vectorisation and did
not) and one spike table was found to measure cached re-checks and was
replaced by a cold run.
