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
| from_clean | 226176155ns | 225444868ns | 224038465ns | -0ns | 226176155ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 226167062ns | 224030471ns | 226167062ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 64

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 99.4% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 226176155ns | 226176155ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 225436235ns | base | --- | [224030471, 229034480] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 229034480ns |
| 1 | 2 | 225436235ns |
| 1 | 3 | 224030471ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.040 | ok |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 40991355.6ns | 226167062.0ns | 18.1% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 224030470.7-226167062.0 ns)
  224030470.7 |########################################
  224137300.3 |
  224244129.8 |
  224350959.4 |
  224457789.0 |
  224564618.5 |
  224671448.1 |
  224778277.6 |
  224885107.2 |
  224991936.8 |
  225098766.3 |
  225205595.9 |
  225312425.5 |
  225419255.0 |########################################
  225526084.6 |
  225632914.2 |
  225739743.7 |
  225846573.3 |
  225953402.8 |
  226060232.4 |
  (0 below, 1 above range)

```

## Diagnostics

- **from_clean**: bridge=18.4% of algo (FFI overhead may distort results)
