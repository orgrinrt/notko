# The concept and each design, one by one

Part of the notko stack audit of 14 November 2026. `AUDIT.md` is the entry and
carries the summary. Paths are relative to the notko root unless they begin
with another repository's name. "Ad-hoc spike" always means one of the two
directories under `research/sketches/` written for this audit; their numbers
are not measurements on record.

notko declares no canon and has no design document. Its READMEs, its module
documentation and its one research record stand in for one, under the rule
`a-design-without-a-canon-is-the-canon`. Above them sit op's words, as far as
they could be read (`LEDGER.md`).

Each part ends with one of four marks: good as it is; easy to get wrong but
can be right; going wrong; back to the drawing board.

## 1. The concept and the direction

What it says. Core's carriers differ by what they hold, notko's three "differ
by what a branch costs instead" (`README.md:7`). `src/lib.rs:44-54` gives the
table: `Just<T>` has no branch, `Maybe<T>` one, `Outcome<T, E>` one with a
payload. Around them the crate collects the small contracts every layer of the
stack needs before it has numbers: a writer (`src/sink.rs`), lent storage
(`src/lend.rs`), a niche-filled optional (`src/slot.rs`), boundedness and
non-zero as traits (`src/bounded.rs`, `src/nonzero.rs`), const forms of `?`
and of construction, and a type-level list in a crate of its own. It is
`no_std`, allocates nothing and depends on nothing.

What op has said. The stack is built bottom up, "arvo and notko first"
(`/home/user/.shared/state/kaski.md:240`). The layer rule gives notko
"foundational fallibility or boundedness" (rule `use-the-stack-not-reinvent`).
Outside crates are to "fit the notko fallibility and arvo numerics naturally",
and "notko has a shape for lending and accounting" (channel
`briefs/fixup/op-answers-30.md`). Against that, on codecs, "notko might become
too unfocused if we bundle in these too" (`op-answers-35.md`).

What holds. The direction is sound: one vocabulary for can-fail and
can-be-absent across four repositories, a home below the numeric crate for the
contracts that would otherwise be written three times, and nothing in it that
drags in an allocator or a runtime. The panel of 12 November found four faults
in notko, none in its concept, and named `lend.rs` "the standard the rest of
the stack should be held to" (channel `briefs/fixup/panel-consolidation-1.md`,
section 7.2).

What holds less well than it reads. The sentence the crate is built around is
that the carriers put "the hot, warm and cold split at the control-flow
level, so what a branch costs is picked where the call is written"
(`src/lib.rs:46-48`). As a classification that is true. As a choice among
three costs it is thinner than it sounds, because two of the three were
already in core at the same price: `Outcome` is `Result` and `Maybe` is
`Option` in cost. The ad-hoc spike
`research/sketches/202611140945_profile-arms-adhoc/` could not tell them
apart, inlined or behind a call (`out_tiers_release_debug_arm.txt`), which is
the right result for a vocabulary. The only new cost point is `Just`, and the
only means of moving a function between the points is the attribute of
section 3. So the whole of the cost promise rests on `Just` and on
`#[profile(Hot)]`, and what the rest of the carriers give is a shared
spelling. Both are worth having. The README and the table should say which is
which, and no cost in them is measured (`NUMBERS.md`, 1.1).

There is a second gap between the words and the tree. Op's intents for notko
(bit flags as something "notko could take naturally", byte casting it "just
re-exports or wraps", branch hints it should "re-export or integrate
optionally via a feature", accounting beside lending; `op-answers-30.md`,
`op-answers-34.md`) appear nowhere in the repository: no design, no note, no
ticket found. Accounting in particular is spoken of as present and is not:
`src/lend.rs` lends and refuses, and counts nothing across lends.

Mark: easy to get wrong but can be right. Right requires three things. The
claim restated as vocabulary and contract home, with every cost sentence
either measured or removed. A written scope for notko, in op's words, saying
what enters (the intents above) and what never does (codecs, numbers), so
that "unfocused" has a line to be measured against. And a decision on where
that scope is written, since the crate deliberately has no mockspace
(`briefs/fixup/roadmap.md:382`) and the README is the only standing design.

## 2. The three carriers as types

`Just<T>` is `#[repr(transparent)]` over its value (`src/just.rs:27`).
`Maybe<T>` is a plain two-variant enum left to the compiler's niche filling
(`src/maybe.rs:32-40`). `Outcome<T, E>` mirrors `Result`. Each converts to and
from its core twin (`src/outcome.rs:270`, `:280`; `src/maybe.rs:380`, `:390`),
and with `try_trait_v2` each takes `?` (`src/outcome.rs:344`,
`src/maybe.rs:441`, `src/just.rs:325`).

The surface is careful and the tests pass (`cargo test --workspace
--all-features`, run for this audit). The cost is the cost of the core types
(ad-hoc spike, as above). The conversions mean a generic application can sit
on notko at its own boundary and still speak `Result` to the ecosystem.

One thing is missing rather than wrong. `Outcome` and `Maybe` have no
`FromResidual<JustResidual>`, so `?` on a `Just` inside a function returning
either does not compile. `JustResidual` is uninhabited (`src/just.rs:302`), so
the two impls are one `match` on nothing each. Their absence is what turns
finding 3 below from an inconvenience into a build that fails only in release.

`Just` has no `Deref`. The research record built and tested one and said it
should land (`research/202608251000_consttry-copy-bound/FINDINGS.md:342`).
It has not landed and nothing records why.

Mark: good as it is, with the two residual impls owed.

## 3. `#[profile(Hot)]`: one function, two return types by build

What it says. `Hot` emits `Outcome<T, E>` in debug builds and `Just<T>` "in
release-internal builds, which the consumer opts into through its own
`internal` feature" (`README.md:73`). `src/lib.rs:60` says this is for builds
"where invariants are proven by construction". The rewrite emits the function
twice under complementary `cfg`s
(`notko-macros-core/src/rewrite/hot.rs:52`, `:55`), and in the `Just` arm an
`Err(e)` becomes a panic. The module that makes `?` work inside the narrowed
arm states the aim: "the whole point of the pair is that the two arms mean the
same program" (`src/just.rs:339-345`).

What op has said, as far as it was seen. The arvo audit quotes a ratified
arvo row, `ruling::hot_may_sacrifice_soundness_for_a_proven_meaningful_gain`,
with op's words that `Hot` "should not lose it for nothing, instead, provable
meaningful gains" (`arvo/mock/research/202611131215_stack-audit/AUDIT.md:24-28`).
That row is arvo's and is about arvo's strategy markers. It is the nearest
statement of his intent for a thing called `Hot`, and it is used here as such
and no further.

Findings.

3a. The two arms do not mean the same program for a caller. The module quoted
above closed the gap for `?` inside a hot body, and the crate's own test
covers exactly that direction: a hot function calling a cold one
(`notko-macros/tests/consumer_cfg.rs:161-172`). The other direction is open.
A function returning `Outcome` that calls a hot function with `?` builds in
the arm the tests run in and fails in the arm that ships, with E0277. A
`match` on `Outcome::Ok` and `Outcome::Err` builds in one and fails E0308 in
the other. A caller that asks `is_err()` and falls back gets its fallback in
one arm and a panic in the other. All three are in the ad-hoc spike
(`research/sketches/202611140945_profile-arms-adhoc/out_probe_*`,
`out_recover_*`). The sentence at `src/just.rs:342-345`, that the divergence
"surfaces on the consumer's release build and nowhere earlier", describes the
defect that file fixed and still describes these. The crate's own standard is
in the name of its test: a construct "builds in both arms or neither"
(`consumer_cfg.rs:161`). By that standard these are defects and not a matter
of taste. How far they bite depends on the consumer. One that builds both
configurations in its checks meets the two compile failures there, early. No
build shows the third, where the same call returns in one arm and panics in
the other.

3b. The return type of a function depends on a cargo feature. Features are
unified across a dependency graph, which the crate's own research record
establishes for notko's features
(`research/202608251000_consttry-copy-bound/FINDINGS.md:42`). The same holds
for a consumer's `internal`: one dependent turning it on changes the signature
of every public hot function in that crate for every other dependent. A
feature that changes a public type is not additive, and cargo assumes
features are. For a hot function private to its crate this reduces to 3a.

3c. Nothing on record shows a gain. notko has no bench. The ad-hoc spike found
the `Just` arm no faster than the `Outcome` arm (0.657 against 0.621 ns per
element, inside the noise of the host), and the mechanism is plain in the
rewrite: the check is kept and its failure is turned into a panic, so the loop
carries the same compare and branch. The table at `src/lib.rs:52` says "None:
no branch. `?` compiles away", which is true of `?` on a `Just` and not of
the hot function, whose body still branches. The one place the same spike
leaves room for a gain is a hot function that is not inlined and returns a
wide error, where the narrower return could be worth up to about half a
nanosecond per call or nothing; its figures cannot say which
(`FINDINGS.md` of the spike, last section). Against the one op statement
seen, the arm currently gives something up (a recoverable error becomes a
process abort) and has no proven gain to show for it.

3d. The word. `Hot`, `Warm` and `Cold` here are fallibility tiers. arvo uses
`Hot`, `Warm`, `Cold` and `Precise` for arithmetic strategy. The two meet in
the same signatures in any crate that uses both.

3e. For a daemon the release arm is a liability by construction: an error that
depends on input, tagged hot by mistake, is a clean error in every test build
and a panic in production.

Mark: going wrong. The intent, a tier in which a proven-unreachable failure
costs nothing, is defensible and is op's. The mechanism is what fails it.
Right would require all of: one return type in both arms, or the two residual
impls of section 2 plus a sealed common surface so that no caller can write
code one arm rejects; a gain measured on a committed bench before the arm
exists at all; and the failure removed rather than moved, which means the
caller validates once and the hot function takes a type that cannot hold the
bad value (the shape `Boundable` and `NonZeroable` already point at). Where
only the cost of a rare branch is at stake, a cold-path hint keeps one type
and is the nearer fit to "what a branch costs"; op has said branch hints
belong in notko (`op-answers-34.md`). That last suggestion is this audit's
and is unproven: hilavitkutin's design bans the `likely` hint on a measurement
of its own (`NUMBERS.md`, 1.5), so it is a bench to run and not a conclusion.

## 4. `Warm`, `Cold` and the rewrite in general

`Cold` always emits `Outcome`, `Warm` always emits `Maybe` and "drops the
whole `Err(..)` expression, side effects included"; the rewrite fires only on
a return type spelled `Result<T, E>` or `Outcome<T, E>` and otherwise does
nothing "without a word" (`README.md:73`). Both hazards are documented and
neither is guarded. An attribute that deletes side effects, and one that
silently does nothing on a `Result<T>` alias, will each be met by someone who
did not read that paragraph. A hard error for an unrecognised return type
would cost one line of diagnostics.

The ad-hoc spike shows `Cold` and `Warm` cost what a call costs (they are not
inlined, `notko-macros-core/src/tiers.rs:69`, `:77`), which is the intent.

Custom tiers. A tier beyond the three is a file,
`notko-optimisers/<Name>.rs`, whose module comment carries `based_on`,
`inline` and `panic_fmt`; the macro reads it from the consumer's manifest
directory while expanding (`notko-macros-core/src/discover.rs:86-90`,
`:160-167`), and `notko-build` copies such files across crate boundaries from
a build script (`notko-build/src/lib.rs:6-22`). The code is careful about its
own past mistakes and says so in place (`discover.rs:284-288`,
`notko-build/src/lib.rs:142-147`). Two things about the design. A custom tier
can only re-parameterise one of the three built-in strategies, so the
mechanism is a file format, a build script, a `links` key and an environment
variable in service of two settings. And the file the macro reads is not
declared to cargo as an input: the path is kept on the tier
(`discover.rs:274`) and nothing emits it into the expansion, so by cargo's
rules an edit to a crate-local tier file does not rebuild the crate that uses
it. That second point is reasoning from how cargo tracks inputs and was not
run. No use of a custom tier was found anywhere, and the consumers could not
be searched.

Mark: easy to get wrong but can be right. Right requires an error where the
rewrite cannot apply, `Warm` refusing an error expression that is not plainly
free of effects, and for custom tiers either a consumer that needs them or
their removal until one does.

## 5. `Maybe`, `MaybeNull`, `Slot`, `NicheFilled`, `NonZeroable`

The layout story is that `Maybe<T>` is the same size as `T` when `T` has a
niche, that `MaybeNull<T>` is a transparent wrapper usable in a `#[repr(C)]`
struct (`src/maybe.rs:547-582`), and that a `const` assertion of equal size
stops the build when it is not (`src/maybe.rs:596-599`, forced for function
pointers at `:704-721`). The language guarantees this layout for `Option`.
For a user enum of the same shape the guarantee is weaker than that, as far as
this audit knows; the assertion is the right defence and makes a change in the
compiler a build failure rather than a wrong read across an FFI boundary.
Size is asserted. That the empty case is the all-zero pattern is checked in
tests only.

`Slot<T: NonZeroable + NicheFilled>` is where the design contradicts itself.
`src/slot.rs:19-22` offers it to downstream newtypes, "a nonzero flavour of a
fixed-point type say". `src/slot.rs:50-56` says `NicheFilled` is sealed and
that such a type therefore cannot be a `Slot` payload, adding "the seal has
not bound anything in practice". The consumer the first passage names is
arvo, and arvo is being rebuilt now. The seal will bind the first time arvo
wants an optional non-zero value the size of the value.

`NonZeroable::try_new` and `Boundable::try_new` are not `const`
(`src/nonzero.rs:42`, `src/bounded.rs:81`). arvo's direction, as its audit
reports it, is admission decided at compile time. A contract for boundedness
that cannot be asked in a constant will be bypassed by the crate it was
written for.

Mark: easy to get wrong but can be right. Right requires a way for a
downstream type to assert its own niche (an unsafe trait with the size
assertion attached, in place of the seal), const forms of the two `try_new`,
and the zero-pattern check moved from tests to a `const` assertion.

## 6. `ConstTry`, `ConstFromResidual`, `HasTrivialCtor`

These exist because `?` and construction are not const on the pinned
toolchain. The research record is thorough about why each bound is what it is
and ends plainly: "Not benchmarked, and nothing here claims a performance
number" (`research/202608251000_consttry-copy-bound/FINDINGS.md`, closing
section). The fixes it proposed for the `Copy` bound landed. Three unstable
features carry it (`src/lib.rs:15-17`), all vetted by the workspace rule
`unstable-features`, two at WATCH. Whether anything downstream calls
`ConstTry` could not be searched.

Mark: good as it is, as long as it is understood as scaffolding that the
language is expected to replace, and as long as the default feature set
question of section 10 is settled.

## 7. The writer contracts: `Push`, `BulkPush`, `Emit`

Three traits split by who holds the receiver and whether the write can fail
(`src/sink.rs:9`, `:35`, `:52`, `:81`). The fourth corner, exclusive and
fallible, is left by design to the crate that owns a capacity type
(`src/sink.rs:20-22`), and the roadmap places it upstream with arvo's capacity
(`briefs/fixup/roadmap.md:386`). The shapes are minimal and right.

The fault is beside the design, not in it. `hilavitkutin-api/src/capability.rs:19-45`
defines its own `Push<T>` and `BulkPush<T>` of the same shape. The workspace
rule names notko's as the one to use, and the workspace's own catalogue lists
the hilavitkutin copies as the home
(`/home/user/.claude/skills/use-the-stack/SKILL.md:64-65`). The panel removed
one such duplicate (the membership traits, fault H15, now fixed at
`hilavitkutin-api/src/access.rs:13`). This one was not on its list.

Mark: good as it is. The duplicate is a finding against the stack, recorded in
`DOWNSTREAM.md`.

## 8. Lending: `Lend`, `Fill`, `Exhausted`

Storage comes from the caller, a `Fill` appends into it and refuses rather
than truncates, and the refusal reports what was wanted and what was had
(`src/lend.rs:40`, `:76-78`, `:132`, `:150-156`). The module is honest about
its limits: it "does not buy elided bounds checks" (`src/lend.rs:24`) and its
bare `usize` is "deliberate", the alternative being a dependency cycle with
the numeric crate (`src/lend.rs:88-91`).

Two things are owed and known. The sticky-refusal law (roadmap step NK2,
`briefs/fixup/roadmap.md:386`) has not landed; nothing in `src/lend.rs` states
that a `Fill` which refused once keeps refusing. And accounting, which op
names beside lending, has no form here (section 1).

Mark: good as it is. The bare `usize` is a ratified tension, not a defect: the
rule against bare primitives in public interfaces and the layering that puts
notko under arvo cannot both be met at this one point, and the module says so.

## 9. `notko-hlist`

Three layers, judged apart.

9a. The list, places and counting (`list.rs`, `at.rs`, `position*.rs`,
`place_of.rs`, `length*.rs`, `cardinal*.rs`, `concat.rs`). Sealed, small, and
it "never picks the number type on anybody's behalf"
(`notko-hlist/src/lib.rs:34`). Linear queries cost about what the list costs
to state (ad-hoc spike `research/sketches/202611140950_hlist-scaling-adhoc/`,
`out_compile_cold_check.txt`). Mark: good as it is.

9b. Membership (`membership.rs`). `Contains` is a `#[marker]` trait because a
list may hold a member twice (`notko-hlist/src/membership.rs:8-15`, `:39`), so
the crate's default features turn on `marker_trait_attr`, which the workspace
rule holds at WATCH. The hilavitkutin audit raised this feature as a risk in
hilavitkutin; it has since moved here
(`hilavitkutin/mock/crates/hilavitkutin-api/src/lib.rs:13-16` no longer names
it), which puts it in the right crate and does not lessen it. The cost at
compile time is the part nobody has stated: a question asked of every member
against the whole list grows between quadratically and cubically. In the
ad-hoc spike `ContainsAll` of a reversed list checks in 0.17 s at 64 members,
0.69 s at 128 and 4.5 s at 256, and `SubOf` reversed in 0.31 s, 1.9 s and
13.7 s (`out_compile_cold_check.txt`). The crate's diagnostic advises
`#![recursion_limit = "1024"]` (`membership.rs:42`), which invites lists of a
size at which each such bound costs minutes. There is no distinctness
predicate (`notko-hlist/src/lib.rs:84-89`), so "this kernel writes each column
once" cannot be said in the type. Mark: easy to get wrong but can be right.
Right requires a stated ceiling on list length with its compile cost beside
it, a committed compile-time bench in the consumer's harness, and a plan for
the day `marker_trait_attr` changes (the non-marker form the file mentions at
`membership.rs:38`).

9c. The product (`carrier.rs`, `project.rs`, `has.rs`, `sub.rs`), landed
13 November. At run time it is free in the one kernel tried: a map over three
columns through `read_row` and `write_row` ran at the speed of zipped slices,
0.126 against 0.117 ns per row at 1024 rows and no difference at larger
sizes, and the compiler vectorised it (ad-hoc spike, `out_rows.txt`). That is
one branch-free kernel and no more. The design fault is in the member trait:
`Elem: 'static` with `Value: Copy + 'static`
(`notko-hlist/src/carrier.rs:50-52`). The bound is a decision and is argued:
"A value that borrows would tie every product holding it to that borrow, and
none of the carriers here wants that" (`carrier.rs:42-45`). Within a day the
first consumer wanted exactly that and wrote
its own read product because "a value read may borrow" and "the shared crate's
product holds `'static` values only", with a note to move it back "once its
product can hold a borrowed value"
(`hilavitkutin/mock/crates/hilavitkutin-api/src/values.rs:9-13`). The roadmap's
test for this step was that the contract crate holds no local list item
(`briefs/fixup/roadmap.md`, line N2 of section (e)). By that test the step is
half done. The row index is a bare `usize` with its reason given
(`carrier.rs:106`), the same ratified tension as in section 8. Mark: easy to
get wrong but can be right. Right requires the member's value to carry a
lifetime before a second consumer builds on the `'static` form, and the
consumer's local product deleted when it does.

## 10. The toolchain position

The default feature set needs nightly (`README.md:11`, `Cargo.toml:127`).
Stable from 1.85 works with defaults off. The research record shows that
escape cannot be relied on inside a graph, because any dependent that leaves
the default on turns it on for all
(`research/202608251000_consttry-copy-bound/FINDINGS.md:42`). Every consumer
read turns on `const` and `try_trait_v2`
(`arvo/mock/Cargo.toml:64`, `hilavitkutin/mock/Cargo.toml:37`), and kaski
takes the defaults (`kaski/mock/Cargo.toml:535`). So in practice notko is a
nightly crate pinned to `nightly-2026-05-28`, and so is everything above it.

Op's words on this were looked for and one was found: for arvo he chose
general numerics without the option that read "but stable Rust first"
(`op-answers-28.md`). Nothing seen says notko must build on stable. The stack
rule vets each feature and none here is forbidden.

Mark: easy to get wrong but can be right. Right requires it said in the
README as a position (this stack is nightly, pinned, and why) in place of an
escape that does not hold, and a default set that is empty so that a feature
is something a consumer asks for. For a generic application or a daemon
outside the stack this is the first thing that decides whether notko can be
used at all; see `DOWNSTREAM.md`.

## 11. How the crate is governed

There is no canon and no mockspace, by intent
(`briefs/fixup/roadmap.md:382`). The README says "I'd not put it anywhere
serious yet" (`README.md:11`). Three repositories depend on branch `dev`.
The workspace catalogue that tells every agent what notko offers still lists
the attribute under its old name and omits `Slot`, `MaybeNull`, `Lend`, `Fill`,
`Emit` and the list crate
(`/home/user/.claude/skills/use-the-stack/SKILL.md:35`). There is no bench of
any notko form anywhere in notko; the roadmap's answer is that one "runs in
the consumer's harness" (`roadmap.md:382`), and none was found there either,
though the consumers could not be searched.

The intent that notko needs no ceremony is op's and is defended here: a crate
this small should not carry a design-round process. The finding is narrower.
The stack's lowest crate has performance sentences with no measurement behind
them, intents with no written home, and a description in the workspace that
is out of date, and each of those is what lets a design like section 3 stand
unchallenged for months.

Mark: going wrong, in the narrow sense above. Right requires a short written
scope, a place for benches of notko's forms that is named and used, and the
catalogue brought up to date.
