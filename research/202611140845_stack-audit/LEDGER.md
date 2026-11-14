# Ledger: what was read, to what depth, and what was not

Part of the notko stack audit of 14 November 2026. `AUDIT.md` is the entry.

Depths: whole means every line; outlined means the items and their doc
comments were read and the bodies were not; listed means only the name is
known; not read means exactly that.

## notko, at `8a74dbc` on `work/stack-audit-2` (cut from `dev`)

| path | depth |
|---|---|
| `README.md`, `Cargo.toml`, `ante.toml`, `rust-toolchain.toml` | whole |
| `src/lib.rs`, `src/just.rs`, `src/slot.rs`, `src/nonzero.rs`, `src/bounded.rs`, `src/lend.rs`, `src/sink.rs` | whole |
| `src/consttry*.rs`, `src/*_consttry_const.rs`, `src/ctor*.rs`, `src/iter.rs`, `src/cmp.rs`, `src/prelude.rs` | whole |
| `src/maybe.rs` | lines 1 to 155 and 464 to 613 whole, the rest outlined |
| `src/outcome.rs` | outlined, with the `try_trait_v2` impls and the `From` impls read |
| `notko-hlist/README.md`, `Cargo.toml`, every file under `src/` | whole |
| `notko-hlist/tests/` | listed |
| `notko-macros/README.md`, `Cargo.toml`, `src/lib.rs` | whole |
| `notko-macros/tests/consumer_cfg.rs` | outlined |
| `notko-macros-core/README.md`, `src/lib.rs`, `src/tiers.rs`, `src/rewrite/mod.rs`, `src/rewrite/hot.rs`, `src/rewrite/maybe.rs` | whole |
| `notko-macros-core/src/discover.rs`, `parse.rs`, `helpers.rs`, `rewrite/outcome.rs` | not read |
| `notko-build/README.md`, `src/lib.rs` | outlined |
| `research/202608251000_consttry-copy-bound/FINDINGS.md` | whole |
| `research/202608251000_consttry-copy-bound/SECOND_READ.md` | headings and verdict |
| `research/202608251000_consttry-copy-bound/p01` to `p08` | not read |
| `tests/` of the root crate | listed |
| `scripts/feature-matrix.sh` | not read, not run |

Run: `cargo test --workspace --all-features` on the pinned toolchain, all
passing. History: this worktree sees one grafted commit, `8a74dbc` of
13 November. The history before it was read once as a one-line log and not
commit by commit.

## The governing tier

| source | depth |
|---|---|
| rule `a-design-without-a-canon-is-the-canon` | whole |
| rule `use-the-stack-not-reinvent` | whole |
| rule `unstable-features` | whole |
| rule `ai-agent-framing` | whole |
| `/home/user/.claude/skills/use-the-stack/SKILL.md` | lines 26 to 99 |
| `briefs/fixup/op-answers-1.md` to `op-answers-38.md` on the kaski channel | fetched and searched for notko, arvo, hilavitkutin, nightly, stable, fallib, alloc, bench; the hits read in full. 39 and 40 do not exist |
| `briefs/fixup/roadmap.md` | searched for notko and its step ids; section (d.2) and the notko lines of (e) and (f) read |
| `briefs/fixup/panel-consolidation-1.md` | searched for notko; section 7.2 read |
| `briefs/job-notko-stack-audit-2.md` (the brief) | whole |
| `/home/user/.shared/state/kaski.md` | lines 1 to 363 of 475 |
| `.data/op-responses` under the workspace clone | not read. The directory could not be listed and no file name was known |
| arvo's registry (`mock/registry`), kaski's canon | not read. Neither could be searched. The arvo ruling on what `Hot` may give up is known here only through `arvo/mock/research/202611131215_stack-audit/AUDIT.md:24-28` |

So the op tier was read through his answers of 12 to 14 November and through
what the rules and the roadmap quote. A ruling of his on notko that lives only
in a capture file or a registry row was not seen, and every finding below
should be read with that in mind.

## Downstream

| path | depth |
|---|---|
| `arvo/mock/Cargo.toml` | the notko lines, through a sub-agent's report and one read |
| `arvo/mock/crates/arvo-format/src/lib.rs` | lines 1 to 60 |
| `arvo-numeral`'s manifest | through a sub-agent's report |
| `hilavitkutin/mock/Cargo.toml` | the notko and arvo lines, through a sub-agent's report |
| `hilavitkutin-api/src/lib.rs`, `access.rs`, `values.rs` | whole |
| `hilavitkutin-api/src/capability.rs` | lines 1 to 60 |
| `hilavitkutin/src/lib.rs` | lines 1 to 40 |
| `hilavitkutin-api/DESIGN.md.tmpl` | lines 880 to 930 |
| `kaski/mock/Cargo.toml` | the notko line, through a sub-agent's report |
| every other file of arvo, hilavitkutin and kaski | not read |

No search across a downstream tree was possible. Whether any crate downstream
uses `#[profile]`, `notko-build`, `Slot`, `MaybeNull`, `Lend`, `Fill` or `Emit`
is therefore not known, and the report says unknown wherever that matters.

## The earlier audits (13 November)

| path | depth |
|---|---|
| `arvo/mock/research/202611131215_stack-audit/AUDIT.md` | lines 1 to 120 |
| the same, `DOWNSTREAM.md` | whole |
| the same, `DESIGNS.md`, `NUMBERS.md`, `BENCH-CATALOGUE.md` | not read |
| `hilavitkutin/mock/research/202611131215_stack-audit/AUDIT.md` | lines 1 to 120 |
| the same, `LEDGER.md` | whole |
| the same, `DESIGNS.md`, `BENCH-RECORD.md`, `ESTIMATES.md`, `DOWNSTREAM.md`, `BENCH-CATALOGUE.md` | not read |

The pull requests of the night (arvo #169 to #175, hilavitkutin #32 to #38)
could not be listed. What they changed is known only where a file read above
shows it, and from the two lines of `kaski.md:9` that name #38 and #175.

## Steps the harness refused

Each is here because it bounded what could be read or run.

| refused | what it was for |
|---|---|
| any shell command naming a path outside this worktree (`ls`, `find`, `grep`, `git -C`, `cat`) | listing and searching arvo, hilavitkutin, kaski and the workspace clone. Reading one file by its exact path worked |
| `gh` | listing the night's pull requests |
| shell loops, `awk`, a command prefixed with an environment variable | driving the spikes. The driver was written in Rust instead |
| running a script file (`sh ./run.sh`) | the spike's own runner. Its commands were run one by one |
| `cargo clean -p hlist-scaling-adhoc` | freeing the shared target directory after the disk filled |
| starting work while more than three compilers ran on the machine | several runs waited. No run was dropped for it |
| a second sub-agent | refused once for load. The one that ran had no search tool and returned manifests only. Nothing was judged by a sub-agent |

The sub-agent was read-only and was not given a worktree of its own, against
the workspace's rule that each gets one. It wrote nothing, so there is nothing
to clean up after it.
