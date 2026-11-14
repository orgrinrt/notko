# Fit to downstream needs, and the earlier audits' findings tested

Part of the notko stack audit of 14 November 2026. `AUDIT.md` is the entry.
Design sections cited as "section n" are in `DESIGNS.md`.

## Part 1. What is actually used today

| consumer | how it names notko | evidence |
|---|---|---|
| arvo | branch `dev`, defaults off, features `const` and `try_trait_v2`; `notko-hlist` with defaults off | `arvo/mock/Cargo.toml:64`, `:67` |
| hilavitkutin | branch `dev`, `const` and `try_trait_v2`; `notko-hlist` with its defaults, so `marker_trait_attr` is on | `hilavitkutin/mock/Cargo.toml:37`, `:40` |
| kaski | branch `dev`, default features | `kaski/mock/Cargo.toml:535` |

All three track a moving branch of a crate whose README says not to put it
anywhere serious (`README.md:11`). The roadmap already noticed the locks
disagreeing and the two spellings of the source
(channel `briefs/fixup/roadmap.md:566-568`).

What hilavitkutin takes from notko-hlist is the list and membership
(`hilavitkutin/mock/crates/hilavitkutin-api/src/access.rs:13`) and the write
product (`values.rs:227`). What it does not take: the count (`access.rs:42-44`
adds one by hand to its own constant, where `Length` and `Cardinal` exist for
that), the read product (`values.rs:9-13`, its own, because of `'static`), and
the writer traits (`capability.rs:19-45`, its own copies). So of the four
things notko offers this consumer, two are used and two are duplicated beside
it.

Which other notko items are used downstream is unknown (`LEDGER.md`).

## Part 2. What each kind of consumer needs, and how the intent and design answer

### 2.1 kaski, a game engine

It needs, from the bottom crate:

- Carriers that cost nothing and say which functions can fail. Answered:
  section 2, good as it is.
- No release-only panic in a simulation step. A colony simulation that aborts
  in the shipped build on an input the debug build handled is the worst
  outcome a tier can produce. Not answered: section 3, going wrong. Whether
  kaski uses `#[profile]` at all is unknown; kaski takes notko with default
  features, and `macros` is not among them (`Cargo.toml:127`, `:131`), so
  unless its manifest adds the feature it does not.
- Columns of plain values read in place and written by mutation, which is
  op's ruling for the rewrite (`/home/user/.shared/state/kaski.md:45-51`).
  The product is the right shape for the write side. The read side needs
  borrowed values and the product cannot hold them: section 9c, easy to get
  wrong but can be right.
- Development builds that run. Unmeasured: `BENCH-CATALOGUE.md`, entry 3.
- Compile times that survive a registry of a hundred or more columns. At
  risk: section 9b and the ad-hoc spike's all-against-all figures. Whether a
  real system asks such questions is entry 6.
- A wasm target. Nothing in notko prevents it (no std, no allocator, no
  dependency); nothing measures it either.
- Fixed-point values through arvo, with optional and bounded forms. `Slot`
  cannot hold arvo's non-zero types (section 5).

Net for kaski: the vocabulary and the write product fit; the hot tier should
not be used; the read product and compile time are the two things that will
be felt first.

### 2.2 A generic application on the stack

It needs:

- To build on a toolchain it can justify. A general application usually
  wants stable. notko as consumed is nightly, pinned (section 10). An
  application that accepts the stack accepts that pin; the README should tell
  it so in one line. The intent seen (op chose arvo's breadth without "stable
  Rust first", `op-answers-28.md`) does not promise stable, so this is a
  cost stated, not a fault.
- To meet the ecosystem at its edges: `std::io`, `serde`, async runtimes,
  error reporting. notko gives `From` in both directions between its carriers
  and core's (`src/outcome.rs:270`, `:280`; `src/maybe.rs:380`, `:390`), which
  is enough for `?` at a boundary with one `.into()`. It gives nothing for
  `core::error::Error`, for error context, or for `core::fmt::Write` and
  `std::io::Write` beside `Push<u8>`. Op's line is that outside crates are
  integrated so they "fit the notko fallibility" (`op-answers-30.md`); for an
  application that integration is most of the work and none of it is
  designed.
- Collections. Op: "We don't want vectors or other collections from
  elsewhere. All we write are over contracts and traits" (`op-answers-30.md`).
  notko offers `Lend` and `Fill` and the writer traits. That is a contract for
  append and nothing for lookup, removal or iteration over a lent store. A
  generic application needs maps and queues on its first day. The intent is
  clear and defended; the design that answers it is a tenth written.
- Documentation that says what exists. The workspace catalogue is out of date
  about notko (section 11).

Net: usable for an application written wholly inside the stack by someone who
accepts nightly; the edges to the rest of Rust are the unanswered part.

### 2.3 A server daemon on the stack

It needs everything in 2.2 and:

- No path from input to abort. `#[profile(Hot)]` as designed is exactly such
  a path when misapplied, and nothing in the attribute can tell a proven
  invariant from a hopeful one (section 3e). `Fill` refusing instead of
  truncating or growing (`src/lend.rs:132`) is the opposite and is what a
  daemon wants. The `?` on an `Outcome` inside a `Just` function panics by
  design (`src/just.rs:22-26`); a daemon wants that to be a compile error it
  can opt out of, not a default.
- Accounted memory. Op: "every single allocation is accounted for and
  reasoned and statically typed" (`op-answers-30.md`). Lending makes every
  buffer's owner explicit, which is the precondition. The accounting itself,
  who holds how much against which budget, has no form in notko (section 1).
- A shared, fallible writer for logs and metrics. `Emit` is that
  (`src/sink.rs:81`). Good as it is.
- Long-lived stability of the toolchain and of the crate's surface. A daemon
  is upgraded for years. Three WATCH features in notko's default set and one
  in notko-hlist's are each a future forced migration. For this consumer the
  toolchain position is the largest cost of the stack.

Net: the contracts a daemon needs most (`Fill`, `Emit`, the carriers) are the
best parts of notko. The hot tier and the nightly surface are the parts a
daemon would have to ban or pin against.

## Part 3. The earlier audits' findings, tested against the state now

Only findings that touch notko, or that this audit could test from what it
read, are listed. The rest of both audits stands untested here.

| finding of 13 November | where | now | evidence |
|---|---|---|---|
| notko is "good as it is, with the feature choice the one thing to look at" | arvo `DOWNSTREAM.md:218-229` | holds in part. It holds for what arvo uses (the carriers, the const forms, the list). It fails as a statement about notko whole: the hot tier and the `'static` product are not good as they are | sections 3 and 9c |
| nightly reaches arvo's consumers through the notko features arvo turns on | arvo `DOWNSTREAM.md:154-165` | still holds, and is wider: no consumer read avoids it, and the escape cannot be relied on in a graph | section 10 |
| notko uses bare primitives in its own source | arvo `DOWNSTREAM.md`, notko section | still holds, and is deliberate and explained at each site | `src/lend.rs:88-91`, `notko-hlist/src/carrier.rs:106` |
| unstable features are a risk in hilavitkutin, `marker_trait_attr` among them | hilavitkutin `AUDIT.md`, first 120 lines | changed shape. `marker_trait_attr` left hilavitkutin-api and now sits in notko-hlist's default features. The risk is the same size and has one owner | `hilavitkutin-api/src/lib.rs:13-16`; `notko-hlist/src/membership.rs:39` |
| `generic_const_exprs` is enabled in hilavitkutin against the rule that forbids it | hilavitkutin `AUDIT.md` | still holds, acknowledged in the source with a note to remove it | `hilavitkutin/mock/crates/hilavitkutin/src/lib.rs:14-22` |
| the unit ceiling of 256 overflows the trait solver; compile time is part of the product | hilavitkutin `AUDIT.md` | still holds in kind. The ad-hoc spike puts figures on the list's share of it | `research/sketches/202611140950_hlist-scaling-adhoc/out_compile_cold_check.txt` |
| hilavitkutin-api's access list duplicates notko-hlist's membership (panel fault H15) | channel `briefs/fixup/panel-consolidation-1.md`, section 7 | fixed | `hilavitkutin-api/src/access.rs:13` |
| notko-hlist lacks a member-to-place selector and a value product (panel faults K1, K2; roadmap NK1) | channel `briefs/fixup/roadmap.md:384` | fixed in notko at `8a74dbc`; the consumer's side half done, see Part 1 | `notko-hlist/src/has.rs`, `project.rs`; `values.rs:9-13` |
| a sticky-refusal law beside `Fill` (panel fault N3; roadmap NK2) | `roadmap.md:386` | not done; it waits on a hilavitkutin step by the roadmap's own order | `src/lend.rs` has no such law |

Not in either earlier audit and found here: the writer traits duplicated in
hilavitkutin-api (Part 1), and everything in section 3.

The night's pull requests (arvo #169 to #175, hilavitkutin #32 to #38) could
not be listed. The rows above are judged from the files as they stand on the
two `dev` clones, which is what the brief asks, and a finding that one of
those pull requests fixed without touching a file read here would be missed.
