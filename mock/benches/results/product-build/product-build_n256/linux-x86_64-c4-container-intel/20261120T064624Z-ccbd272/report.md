# product-build: the product over a registry of columns, built in the dev profile

1 variants, 3 samples per variant.
Baseline: **from_clean**

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
| Counter frequency | 2100425270 Hz |
| Drift correction | none |

## End-to-end (all cooldowns combined)

| Variant | mean | median | best 20% | mid 60% | worst 20% | Δ mean |
|---|---|---|---|---|---|---|
| from_clean | 3627508033ns | 3615930322ns | 3610455470ns | -0ns | 3627508033ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 3627497935ns | 3610445070ns | 3627497935ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 256

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 99.8% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 3627508033ns | 3627508033ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 3615920673ns | base | --- | [3610445070, 3656128061] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 3656128061ns |
| 1 | 2 | 3610445070ns |
| 1 | 3 | 3615920673ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.234 | moderate- |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 49586692.7ns | 3627497934.6ns | 1.4% |  |

## Distribution (algo ns)

```
from_clean (n=3, range 3610445069.8-3627497934.6 ns)
  3610445069.8 |########################################
  3611297713.0 |
  3612150356.3 |
  3613002999.5 |
  3613855642.8 |
  3614708286.0 |
  3615560929.2 |########################################
  3616413572.5 |
  3617266215.7 |
  3618118858.9 |
  3618971502.2 |
  3619824145.4 |
  3620676788.7 |
  3621529431.9 |
  3622382075.1 |
  3623234718.4 |
  3624087361.6 |
  3624940004.9 |
  3625792648.1 |
  3626645291.3 |
  (0 below, 1 above range)

```
