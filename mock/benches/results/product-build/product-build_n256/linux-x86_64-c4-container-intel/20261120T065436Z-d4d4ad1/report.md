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
| from_clean | 3657627713ns | 3596409826ns | 3539091287ns | -0ns | 3657627713ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| from_clean | 3657619113ns | 3539084152ns | 3657619113ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (from_clean; best 20% batches)
- Ops per call: 256

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| from_clean | 0.000 | 98.4% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| from_clean | 3657627713ns | 3657627713ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 3596401674ns | base | --- | [3539084152, 3837371513] | --- | --- | --- | --- |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean |
|---|---|---|
| 1 | 1 | 3837371513ns |
| 1 | 2 | 3539084152ns |
| 1 | 3 | 3596401674ns |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| from_clean | -0.280 | moderate- |

**Consistency summary:**


## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| from_clean | 47153244.0ns | 3657619113.1ns | 1.3% |  |

## Distribution (algo ns)

```
from_clean (n=3, range 3539084151.8-3657619113.1 ns)
  3539084151.8 |########################################
  3545010899.9 |
  3550937647.9 |
  3556864396.0 |
  3562791144.1 |
  3568717892.1 |
  3574644640.2 |
  3580571388.3 |
  3586498136.3 |
  3592424884.4 |########################################
  3598351632.5 |
  3604278380.5 |
  3610205128.6 |
  3616131876.7 |
  3622058624.7 |
  3627985372.8 |
  3633912120.9 |
  3639838868.9 |
  3645765617.0 |
  3651692365.1 |
  (0 below, 1 above range)

```
