# The migration of notko: what leaves the branch tree and what stays

Read on `origin/dev` 2a58570, 22 November 2026, from `git ls-tree -r origin/dev -- mock` (641 files). Prepared for `cargo mock migrate directory-to-ref`, not run: the migration waits for mockspace v2 on `dev`.

## What `mock/` holds

- research: 277 files (160 in `202608251000_consttry-copy-bound`, 105 in `sketches`)
- crates: 245 files (notko, notko-build, notko-hlist, notko-macros, notko-macros-core)
- benches: 72 files, 57 of them results under `benches/results`
- design_rounds: 36 files, all in closed round directories, no flat file of an open round
- test-history: 6 files
- `agent/config.toml`, and at the root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `DESIGN.md.tmpl`
- 4 `*.md.tmpl` files: `DESIGN.md.tmpl` at the root and one `DESIGN.md.tmpl` in each of notko, notko-build and notko-hlist
- 6 rendered `README.md` files (five crates and `benches/`)
- no `registry/` directory: notko declares no canon (`mockspace.toml` has no `canon_paths`), so there is no registry namespace to make, and the one data namespace is all the migration does

## The data namespace

One `[[data_ref]]` over the whole mock directory, in the shape muisti declares (`path = "."` takes files by `include` only, and each pattern names a directory in full):

```toml
[[data_ref]]
namespace = "mock"
path = "."
include = [
    "research/**",
    "benches/**",
    "design_rounds/*/**",
    "crates/**/*.md.tmpl",
    "*.md.tmpl",
]
```

The migration call, with `--form branch` from a container (custom refs do not pass the cloud proxy):

```
cargo mock migrate directory-to-ref --dry-run --namespace mock --path . --form branch \
  --include 'research/**' --include 'benches/**' --include 'design_rounds/*/**' \
  --include 'crates/**/*.md.tmpl' --include '*.md.tmpl'
```

What each pattern takes and the count the dry run must show: research 277, benches 72, design_rounds 36, crate templates 3, root template 1. Total 389. The dry run names every path, so a count that differs is a pattern to fix before the run.

- `design_rounds/*/**` reaches only into a round directory, so a flat file of an open round stays on its branch.
- `benches/**` takes the harness sources, `benches/README.md` and the results together, as muisti does.
- `research/**` takes `research/rustfmt.toml` too, which only formats the research tree.

## What stays on the branch tree

- `crates/**` sources, tests, `Cargo.toml`, the `LICENSE` of each crate, `notko-macros/notko-optimisers/Trace.rs` and each rendered `README.md`
- `mock/Cargo.toml`, `mock/Cargo.lock`, `mock/rust-toolchain.toml`, `mock/agent/config.toml`
- `test-history` (6 files) has a store of its own and is not a data namespace; a `[[data_ref]]` path under it is refused
- notko has no `lints/` or `tools/` directory under `mock/`

## Before the run

- notko `dev` is a trunk that takes only a pull request on the forge, so the migration commit reaches it as one
- the engine pin `mockspace_branch = "dev"` follows the `dev` that will hold v2
- every remote branch is landed, recorded or retired first: notko has `dev` and `main` only, nothing to classify
