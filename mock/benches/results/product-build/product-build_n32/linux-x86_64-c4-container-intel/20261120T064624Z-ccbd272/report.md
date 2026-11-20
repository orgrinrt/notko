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
| from_clean | 115550038ns | 117098497ns | 112248158ns | -0ns | 115550038ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 115539370ns | 112239206ns | 115539370ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 32

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 95.9% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 115550038ns | 115550038ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 117086918ns | base | --- | [112239206, 117291987] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 112239206ns |
| 1 | 2 | 117291987ns |
| 1 | 3 | 117086918ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.188 | ok |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 42633027.2ns | 115539370.2ns | 36.9% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 112239205.9-115539370.2 ns)
  112239205.9 |########################################
  112404214.1 |
  112569222.3 |
  112734230.5 |
  112899238.8 |
  113064247.0 |
  113229255.2 |
  113394263.4 |
  113559271.6 |
  113724279.8 |
  113889288.0 |
  114054296.2 |
  114219304.5 |
  114384312.7 |
  114549320.9 |
  114714329.1 |
  114879337.3 |
  115044345.5 |
  115209353.7 |
  115374362.0 |
  (0 below, 2 above range)

```

## Diagnostics

- **from_clean**: bridge=37.7% of algo (FFI overhead may distort results)
