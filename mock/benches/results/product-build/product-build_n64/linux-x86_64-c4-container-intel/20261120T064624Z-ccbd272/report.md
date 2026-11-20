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
| from_clean | 223881338ns | 215980276ns | 205245102ns | -0ns | 223881338ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 223871720ns | 205235247ns | 223871720ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 64

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 95.0% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 223881338ns | 223881338ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 215969934ns | base | --- | [205235247, 250409978] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 205235247ns |
| 1 | 2 | 250409978ns |
| 1 | 3 | 215969934ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.632 | HIGH- (thermal bounce) |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 38971238.9ns | 223871719.6ns | 17.4% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 205235246.9-223871719.6 ns)
  205235246.9 |########################################
  206167070.5 |
  207098894.2 |
  208030717.8 |
  208962541.4 |
  209894365.1 |
  210826188.7 |
  211758012.3 |
  212689836.0 |
  213621659.6 |
  214553483.2 |
  215485306.9 |########################################
  216417130.5 |
  217348954.1 |
  218280777.8 |
  219212601.4 |
  220144425.0 |
  221076248.7 |
  222008072.3 |
  222939895.9 |
  (0 below, 1 above range)

```

## Diagnostics

- **from_clean**: bridge=18.6% of algo (FFI overhead may distort results)
