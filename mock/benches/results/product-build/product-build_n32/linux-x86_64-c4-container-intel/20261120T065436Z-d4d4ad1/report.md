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
| Counter frequency | 2100303536 Hz |
| Drift correction | none |

## End-to-end (all cooldowns combined)

| Variant | mean | median | best 20% | mid 60% | worst 20% | Δ mean |
|---|---|---|---|---|---|---|
| from_clean | 115158436ns | 110876792ns | 104069758ns | -0ns | 115158436ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 115148521ns | 104057844ns | 115148521ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 32

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 93.9% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 115158436ns | 115158436ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 110869355ns | base | --- | [104057844, 130518363] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 130518363ns |
| 1 | 2 | 110869355ns |
| 1 | 3 | 104057844ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.049 | ok |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 40637134.0ns | 115148520.8ns | 35.3% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 104057843.9-115148520.8 ns)
  104057843.9 |########################################
  104612377.7 |
  105166911.6 |
  105721445.4 |
  106275979.3 |
  106830513.1 |
  107385047.0 |
  107939580.8 |
  108494114.7 |
  109048648.5 |
  109603182.4 |
  110157716.2 |
  110712250.1 |########################################
  111266783.9 |
  111821317.8 |
  112375851.6 |
  112930385.4 |
  113484919.3 |
  114039453.1 |
  114593987.0 |
  (0 below, 1 above range)

```

## Diagnostics

- **from_clean**: bridge=36.2% of algo (FFI overhead may distort results)
