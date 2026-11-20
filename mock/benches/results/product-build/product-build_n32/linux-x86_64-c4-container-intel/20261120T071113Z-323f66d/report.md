# product-build: the product over a registry of columns, built in the dev profile

2 variants, 3 samples per variant.
Baseline: **from_clean**

## Highlights

Baseline for all deltas below: **from_clean**. (Deltas are paired `variant - baseline` medians; `*` marks a CI that excludes zero.)

### Baseline (from_clean) is the SLOWEST variant; every rival beats it

The declared/defaulted baseline from_clean has the worst median (143.00 ms). Every delta is therefore measured against the worst performer, which flatters all rivals and compresses the differences that matter among them (e.g. fastest cargo_alone at 42.88 ms).

_Why it matters:_ A baseline picked by accident (often the first variant to run / sort) silently skews every comparison. Re-baseline via `[bench.<name>.normalise]` on a representative variant.

### cargo_alone dominates: 233% faster than the next best (from_clean)

cargo_alone (42.88 ms) leads from_clean (143.00 ms) by 233%, a clear separation rather than a photo finish. CV 10.6%.

_Why it matters:_ A dominant, well-separated winner is a safe default pick for this workload shape.

### cargo_alone beats baseline by 70% (significant)

cargo_alone is -100.12 ms (70%) faster than baseline from_clean, with a CI that excludes zero.

_Why it matters:_ A large, significant improvement over the current baseline is a concrete reason to switch.

### Wide spread: slowest is 3.3x the fastest

Fastest cargo_alone (42.88 ms) to slowest from_clean (143.00 ms): 3.3x. The strategy choice matters a lot for this workload.

_Why it matters:_ A wide field means the strategy is load-bearing here; getting it right (or wrong) has large consequences.

## Key findings

- **Fastest: cargo_alone** at 42883026.9 ns median (-70.0% vs baseline)
- 1 variant significantly faster than baseline
- Spread: 3.33x (fastest 42883026.9 ns, slowest 142999418.5 ns)

## Methodology

| Parameter | Value |
|---|---|
| Workload | product-build: the product over a registry of columns, built in the dev profile |
| Passes | 3 |
| Runs per pass | 1 |
| Batch size | 1 |
| Harness runs | 1 |
| Cooldown schedule | 0ms |
| Master seed | 0x6e6f746b6f706201 |
| Counter frequency | 2100368366 Hz |
| Drift correction | none |

## End-to-end (all cooldowns combined)

| Variant | mean | median | best 20% | mid 60% | worst 20% | Δ mean |
|---|---|---|---|---|---|---|
| cargo_alone | 46007166ns | 42889911ns | 42688905ns | -0ns | 46007166ns | -68.86% |
| from_clean | 147753688ns | 143006208ns | 119484576ns | -0ns | 147753688ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| cargo_alone | 46000298ns | 42682490ns | 46000298ns | -68.87% | 0.000 |
| from_clean | 147745892ns | 119477000ns | 147745892ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (cargo_alone; best 20% batches)
- Ops per call: 32

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| cargo_alone | 0.000 | 99.5% |
| from_clean | 0.000 | 29.8% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| cargo_alone | 46007166ns | 46007166ns | -68.86% |
| from_clean | 147753688ns | 147753688ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 142999418ns | base | --- | [119477000, 180761258] | --- | --- | --- | --- |
| cargo_alone | 42883027ns | -100116391.6ns (-70.0%) | [-128325880, -76794509]ns | [42682490, 52435377] | YES (adj: no) | 0.2500 | 0.2500 | 0 |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean | cargo_alone |
|---|---|---|---|
| 1 | 1 | 142999418ns | -70.0% |
| 1 | 2 | 119477000ns | -64.3% |
| 1 | 3 | 180761258ns | -71.0% |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| cargo_alone | -0.177 | ok |
| from_clean | -0.418 | moderate- |

**Consistency summary:**

- **cargo_alone**: won 3/3, lost 0/3

## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| cargo_alone | 44360630.2ns | 46000298.2ns | 96.4% | HIGH |
| from_clean | 46664933.3ns | 147745891.9ns | 31.6% | HIGH |

## Distribution (algo ns)

```
cargo_alone (n=3, range 42682490.2-46000298.2 ns)
  42682490.2 |########################################
  42848380.6 |########################################
  43014271.0 |
  43180161.4 |
  43346051.8 |
  43511942.2 |
  43677832.6 |
  43843723.0 |
  44009613.4 |
  44175503.8 |
  44341394.2 |
  44507284.6 |
  44673175.0 |
  44839065.4 |
  45004955.8 |
  45170846.2 |
  45336736.6 |
  45502627.0 |
  45668517.4 |
  45834407.8 |
  (0 below, 1 above range)

from_clean (n=3, range 119476999.5-147745891.9 ns)
  119476999.5 |########################################
  120890444.1 |
  122303888.7 |
  123717333.4 |
  125130778.0 |
  126544222.6 |
  127957667.2 |
  129371111.9 |
  130784556.5 |
  132198001.1 |
  133611445.7 |
  135024890.3 |
  136438335.0 |
  137851779.6 |
  139265224.2 |
  140678668.8 |
  142092113.4 |########################################
  143505558.1 |
  144919002.7 |
  146332447.3 |
  (0 below, 1 above range)

```

## Diagnostics

- **cargo_alone**: bridge=105.5% of algo (FFI overhead may distort results)
- **from_clean**: bridge=32.7% of algo (FFI overhead may distort results)
