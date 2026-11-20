# benches

Canonical mockspace bench tree. Scaffolded by `mock bench init`; the
driver binary is generated from `bench.toml` on every run.

## Layout

- `bench.toml`: globals ([timing], [dispatch], [workload.*], [build]).
- `<bench>/bench.toml`: the bench. Its fields sit at the top level
  with no wrapper table, because the directory name is the bench's
  name. Optional [sweep.<name>] sections carry per-sweep points and
  overrides.
- `<bench>/arms/<arm>/src/lib.rs`: one measured cdylib per arm. The
  manifest is generated; writing a Cargo.toml in the arm directory
  takes it over.
- `<bench>/support/`, `support/`: ordinary library crates the arms
  and the optional hooks library link.
- `src/lib.rs` (optional): `pub fn hooks() -> Hooks` with any of
  `routine_for`, `after_cell`, `on_init`, `after_init`.
- `results/<bench>/<sweep>_n<point>/<machine class>/<run>/`: one
  directory per run, holding `samples.csv`, `meta.json`, `report.md`
  and `summary.tsv`. A run never replaces an earlier one, and runs on
  different machines sit side by side. Tracked; the history of a cell
  is its runs, which is what regression detection reads.

## Workflow

1. `mock bench add <bench> <arm>` scaffolds a bench or an arm.
2. Implement the arm's run block.
3. Point lists, arm lists, roles (baseline/floor), and timing live in
   the bench's own bench.toml.
4. `mock bench run [names...]` generates the driver, builds arms into
   tool-owned target directories with the pinned release profile, and
   measures. A bench name selects all its sweeps.
5. `mock bench report` renders a report for any run that has none.

The generated driver is the only runner. A driver of your own here is
refused by `mock bench run` and by the lint
`a-bench-runs-only-on-the-generated-driver`; anything it did belongs in
the hooks library. Results in an older layout are refused by
`bench-results-live-in-the-harness-layout`, and `mock bench migrate`
moves the ones it can translate.
