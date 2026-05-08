# Notko documentation audit round, 2026-05-05

**Date:** 2026-05-05.
**Phase:** Topic + plan, combined (notko has no mockspace flow; this file plays the role of topic + doc CL together).
**Scope:** notko (root README.md, three sub-crate README.md, notko-macros/tests/smoke.rs).
**Source:** `research/2026_05_05_doc_audit.md` (full audit findings).

## Why this file shape

Notko has no `mock/` directory; it is a flat Cargo workspace. The mockspace state machine does not apply. This file consolidates the workspace-wide-audit findings for notko into a topic-shaped plan that an executor can follow without opening a formal design round. The plan resolves to a regular feature-branch PR.

## Scope summary

The audit found 13 findings across 4 README files plus 1 test source file. Categories:

1. **Em-dashes in three sub-crate READMEs** (workspace-wide ban from `writing-style.md`). 11 occurrences total: 6 in `notko-build/README.md`, 4 in `notko-macros-core/README.md`, 1 in `notko-macros/tests/smoke.rs:7`.
2. **Doc-vs-source drift, load-bearing.** `notko-macros/README.md` describes a proc-macro called `optimize_for(hot|warm|cold)` (lowercase). The actual source-side macro is `profile(Hot|Warm|Cold)` (capitalised idents). Every example in the README is wrong against the current API. Repo-root README and the lib.rs module doc both use the correct `profile(Hot|Warm|Cold)` shape; only the sub-crate README is stale.
3. **Tier 1 task-id leakage** at `notko-macros/README.md:59-60`: `(task #99)`. Workspace-internal task ID; reader on crates.io has no resolution path.
4. **Spelling inconsistency in `notko-build/README.md`** between `notko-optimizers/` (US, the crate-local source dir) and `notko-optimisers/` (UK, the env var and out-dir). Both spellings are intentional per source (`LOCAL_DIR = "notko-optimizers"`, `OUT_SUBDIR = "notko-optimisers"`), but the README does not explain the deliberate split, leaving the reader to infer it. Add an explicit note.
5. **Glossary-style label-plus-em-dash bullets** in `notko-macros-core/README.md` "## Public API" section (lines 13-25). Per `writing-style.md`, label-plus-colon (and the close cousin label-plus-em-dash) bullets are the cheat-sheet form; convert to a two-column table.

## Decisions

### Decision 1: em-dash sweep across notko sub-crate READMEs and the test file

11 mechanical replacements per `writing-style.md`. Use period, comma, parens, or colon per the surrounding sentence shape.

### Decision 2: notko-macros README API resync

Rewrite every `optimize_for` reference in `notko-macros/README.md` to `profile`, every lowercase tier name (`hot`, `warm`, `cold`) to capitalised (`Hot`, `Warm`, `Cold`). Verify against `notko-macros/src/lib.rs:22` (`pub fn profile(...)`) and the root `README.md:152` (the canonical example).

The lock-time check: `cargo doc --open` on notko-macros and confirm the README example compiles when copied.

### Decision 3: drop `(task #99)` reference

`notko-macros/README.md:59-60`: replace "(task #99)" with no parenthetical, or with a public link to the GitHub repo issues page if useful (probably not; the reference is already to a "companion crate" that is independently named in the same paragraph, so the parenthetical is redundant).

### Decision 4: notko-build US/UK spelling note

`notko-build/README.md`: add a short note up front (after the tagline, before "## Usage") explaining the deliberate spelling split: source files live in `notko-optimizers/` (US) by historical convention; cargo metadata propagation and the env var use `notko-optimisers/` (UK) to match the British baseline. The split is intentional; readers should not assume a typo.

Optionally: rename one side to match the other to drop the split entirely. This is a substantive code-and-docs change that wants its own design round; the doc CL just adds the disambiguation note.

### Decision 5: notko-macros-core "## Public API" section

Convert the label-plus-em-dash bullet list (lines 13-25) to a two-column table:

```markdown
## Public API

| Item | Purpose |
|---|---|
| `tiers::{Tier, Strategy, CustomTier}` | Three built-in tiers and the resolved-tier struct handed to rewriters. |
| `parse::parse_tier_arg` | Parse the `tier` ident from an attribute-arg token stream. |
| `discover::resolve_tier` | Resolve a tier name to a `CustomTier`, looking up `notko-optimizers/<name>.rs` in the consumer's crate manifest dir and (optionally) in `$NOTKO_OPTIMISERS_PATH`. |
| `rewrite::{entry, rewrite_fn, HotRewriter, OutcomeRewriter}` | The rewrite engine. `entry` is the one-shot driver used by notko-macros itself; `rewrite_fn` takes an already-resolved `CustomTier`; the visitor structs are exposed for finer-grained use. |
| `rewrite::helpers::{is_ok_call, is_err_call, extract_result_inner_types, macro_last_ident_is, stmt_macro_last_ident_is}` | Shared AST utilities. |
```

The em-dash issue resolves automatically, the label-plus-period violation does not apply (table cells, not bullets), and the prose stays the same.

## Files touched

| Path | Action |
|------|--------|
| `notko-build/README.md` | 6 em-dash replacements (lines 71, 79, 81, 84, 92, 94); add US/UK spelling disambiguation note |
| `notko-macros-core/README.md` | 4 em-dash replacements (lines 13, 15, 17, 20, possibly 24); convert "## Public API" bullets to a two-column table |
| `notko-macros/README.md` | Rewrite every `optimize_for` reference to `profile`; lowercase tier names → capitalised; drop "(task #99)" reference |
| `notko-macros/tests/smoke.rs` | 1 em-dash replacement (line 7) |
| `README.md` (repo root) | No edit. Already clean per audit. |

## Verification

Lock-time:

1. `grep -nP '—' README.md notko-*/README.md notko-*/src/**.rs notko-*/tests/**.rs` returns zero hits.
2. `grep -nE 'optimize_for|task #' notko-macros/README.md` returns zero hits.
3. `grep -c 'notko-optimizers\|notko-optimisers' notko-build/README.md` confirms both spellings still appear (the disambiguation note is meant to coexist with both).
4. The notko-macros README's first usage example (`#[profile(Hot)] fn foo() -> Result<...>`) compiles when extracted into a doctest, or visually matches `src/lib.rs:22`.

## No design-round ceremony

Notko has no `mock/` and no design-round state machine. The PR shape is:

- One feature branch (e.g., `chore/docs-audit-round-2026-05-05`).
- One commit (or a few small commits) covering the per-file actions above.
- One PR to `dev` with the standard PR template (Summary + Test plan).
- Standard merge ceremony per the workspace branch-PR-flow rule.

The src CL concept does not apply (notko has no mockspace SRC phase). The em-dash sweep in `notko-macros/tests/smoke.rs:7` is one of the actions in the same PR; it is the only "source" change and is mechanical.

## Cross-references

- `research/2026_05_05_doc_audit.md`: full audit findings the topic acts on.
- `~/Dev/clause-dev/.claude/rules/documentation-writing.md`: surface taxonomy and the leakage rules that drove the audit.
- `~/Dev/clause-dev/.claude/rules/writing-style.md`: em-dash ban, ASCII-diagram ban, label-plus-colon ban.
- `~/Dev/clause-dev/.claude/skills/documentation-writing/SKILL.md`: the invokable checklist for any doc edit.
