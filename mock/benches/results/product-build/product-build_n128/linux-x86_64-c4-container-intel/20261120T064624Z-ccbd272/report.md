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
| from_clean | 750541489ns | 743825601ns | 743020943ns | -0ns | 750541489ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 750525132ns | 743011089ns | 750525132ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 128

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 99.9% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 750541489ns | 750541489ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 743815672ns | base | --- | [743011089, 764748634] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 743815672ns |
| 1 | 2 | 743011089ns |
| 1 | 3 | 764748634ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.186 | ok |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 42208363.7ns | 750525131.6ns | 5.6% | HIGH |

## Distribution (algo ns)

```
from_clean (n=3, range 743011089.1-750525131.6 ns)
  743011089.1 |########################################
  743386791.2 |
  743762493.4 |########################################
  744138195.5 |
  744513897.6 |
  744889599.7 |
  745265301.9 |
  745641004.0 |
  746016706.1 |
  746392408.2 |
  746768110.4 |
  747143812.5 |
  747519514.6 |
  747895216.7 |
  748270918.9 |
  748646621.0 |
  749022323.1 |
  749398025.3 |
  749773727.4 |
  750149429.5 |
  (0 below, 1 above range)

```
