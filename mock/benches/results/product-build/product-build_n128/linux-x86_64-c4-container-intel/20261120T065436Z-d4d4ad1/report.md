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
| from_clean | 753390927ns | 743581850ns | 741742326ns | -0ns | 753390927ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 753382373ns | 741735648ns | 753382373ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 128

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 99.8% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 753390927ns | 753390927ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 743571668ns | base | --- | [741735648, 774839804] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 774839804ns |
| 1 | 2 | 741735648ns |
| 1 | 3 | 743571668ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.196 | ok |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 38496449.9ns | 753382373.4ns | 5.1% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 741735648.4-753382373.4 ns)
  741735648.4 |########################################
  742317984.6 |
  742900320.9 |
  743482657.1 |########################################
  744064993.4 |
  744647329.6 |
  745229665.9 |
  745812002.1 |
  746394338.4 |
  746976674.6 |
  747559010.9 |
  748141347.1 |
  748723683.4 |
  749306019.6 |
  749888355.9 |
  750470692.1 |
  751053028.4 |
  751635364.6 |
  752217700.9 |
  752800037.1 |
  (0 below, 1 above range)

```
