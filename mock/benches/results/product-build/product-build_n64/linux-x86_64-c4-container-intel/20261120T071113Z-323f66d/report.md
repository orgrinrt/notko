# product-build: the product over a registry of columns, built in the dev profile

2 variants, 3 samples per variant.
Baseline: **from_clean**

## Highlights

Baseline for all deltas below: **from_clean**. (Deltas are paired `variant - baseline` medians; `*` marks a CI that excludes zero.)

### Baseline (from_clean) is the SLOWEST variant; every rival beats it

The declared/defaulted baseline from_clean has the worst median (246.54 ms). Every delta is therefore measured against the worst performer, which flatters all rivals and compresses the differences that matter among them (e.g. fastest cargo_alone at 54.15 ms).

_Why it matters:_ A baseline picked by accident (often the first variant to run / sort) silently skews every comparison. Re-baseline via `[bench.<name>.normalise]` on a representative variant.

### cargo_alone dominates: 355% faster than the next best (from_clean)

cargo_alone (54.15 ms) leads from_clean (246.54 ms) by 355%, a clear separation rather than a photo finish. CV 10.2%.

_Why it matters:_ A dominant, well-separated winner is a safe default pick for this workload shape.

### cargo_alone beats baseline by 77% (significant)

cargo_alone is -190.11 ms (77%) faster than baseline from_clean, with a CI that excludes zero.

_Why it matters:_ A large, significant improvement over the current baseline is a concrete reason to switch.

### cargo_alone is fastest but the noisiest (CV 10.2%)

cargo_alone wins on median (54.15 ms) yet has the highest variance (CV 10.2%), while from_clean is the steadiest (CV 4.2%, 246.54 ms).

_Why it matters:_ For latency-sensitive or tail-bound paths, the steadier variant can beat the faster-on-average one; weigh peak vs consistency.

### cargo_alone shows alternating (throttle bounce) (autocorr -0.65)

cargo_alone's per-pass series has lag-1 autocorrelation -0.65, indicating alternating (throttle bounce). Its timing may not be at steady state.

_Why it matters:_ Autocorrelated samples violate the independence the CIs assume; the interval is optimistic until the drift is warmed out or cooled down.

### Wide spread: slowest is 4.6x the fastest

Fastest cargo_alone (54.15 ms) to slowest from_clean (246.54 ms): 4.6x. The strategy choice matters a lot for this workload.

_Why it matters:_ A wide field means the strategy is load-bearing here; getting it right (or wrong) has large consequences.

## Key findings

- **Fastest: cargo_alone** at 54145359.1 ns median (-78.0% vs baseline)
- 1 variant significantly faster than baseline
- Spread: 4.55x (fastest 54145359.1 ns, slowest 246536829.6 ns)

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
| cargo_alone | 51426417ns | 54152095ns | 43689735ns | -0ns | 51426417ns | -78.80% |
| from_clean | 242607918ns | 246547842ns | 228271141ns | -0ns | 242607918ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| cargo_alone | 51420064ns | 43683693ns | 51420064ns | -78.80% | 0.000 |
| from_clean | 242598425ns | 228261950ns | 242598425ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (cargo_alone; best 20% batches)
- Ops per call: 64

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| cargo_alone | 0.000 | 80.7% |
| from_clean | 0.000 | 17.7% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| cargo_alone | 51426417ns | 51426417ns | -78.80% |
| from_clean | 242607918ns | 242607918ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 246536830ns | base | --- | [228261950, 252996495] | --- | --- | --- | --- |
| cargo_alone | 54145359ns | -190105688.4ns (-77.1%) | [-198851136, -184578257]ns | [43683693, 56431141] | YES (adj: no) | 0.2500 | 0.2500 | 0 |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean | cargo_alone |
|---|---|---|---|
| 1 | 1 | 252996495ns | -78.6% |
| 1 | 2 | 228261950ns | -80.9% |
| 1 | 3 | 246536830ns | -77.1% |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| cargo_alone | -0.648 | HIGH- (thermal bounce) |
| from_clean | -0.624 | HIGH- (thermal bounce) |

**Consistency summary:**

- **cargo_alone**: won 3/3, lost 0/3

## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| cargo_alone | 43476647.1ns | 51420064.4ns | 84.6% | HIGH |
| from_clean | 45676299.8ns | 242598425.1ns | 18.8% | HIGH |

## Distribution (algo ns)

```
cargo_alone (n=3, range 43683693.0-51420064.4 ns)
  43683693.0 |########################################
  44070511.6 |
  44457330.1 |
  44844148.7 |
  45230967.3 |
  45617785.9 |
  46004604.4 |
  46391423.0 |
  46778241.6 |
  47165060.1 |
  47551878.7 |
  47938697.3 |
  48325515.9 |
  48712334.4 |
  49099153.0 |
  49485971.6 |
  49872790.1 |
  50259608.7 |
  50646427.3 |
  51033245.9 |
  (0 below, 2 above range)

from_clean (n=3, range 228261950.4-242598425.1 ns)
  228261950.4 |########################################
  228978774.1 |
  229695597.9 |
  230412421.6 |
  231129245.3 |
  231846069.1 |
  232562892.8 |
  233279716.5 |
  233996540.3 |
  234713364.0 |
  235430187.7 |
  236147011.5 |
  236863835.2 |
  237580658.9 |
  238297482.7 |
  239014306.4 |
  239731130.1 |
  240447953.9 |
  241164777.6 |
  241881601.3 |
  (0 below, 2 above range)

```

## Diagnostics

- **cargo_alone**: bridge=83.6% of algo (FFI overhead may distort results)
- **from_clean**: bridge=17.0% of algo (FFI overhead may distort results)
