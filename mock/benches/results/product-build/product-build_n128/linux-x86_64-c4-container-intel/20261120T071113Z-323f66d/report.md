# product-build: the product over a registry of columns, built in the dev profile

2 variants, 3 samples per variant.
Baseline: **from_clean**

## Highlights

Baseline for all deltas below: **from_clean**. (Deltas are paired `variant - baseline` medians; `*` marks a CI that excludes zero.)

### Baseline (from_clean) is the SLOWEST variant; every rival beats it

The declared/defaulted baseline from_clean has the worst median (809.61 ms). Every delta is therefore measured against the worst performer, which flatters all rivals and compresses the differences that matter among them (e.g. fastest cargo_alone at 47.14 ms).

_Why it matters:_ A baseline picked by accident (often the first variant to run / sort) silently skews every comparison. Re-baseline via `[bench.<name>.normalise]` on a representative variant.

### cargo_alone dominates: 1617% faster than the next best (from_clean)

cargo_alone (47.14 ms) leads from_clean (809.61 ms) by 1617%, a clear separation rather than a photo finish. CV 5.2%.

_Why it matters:_ A dominant, well-separated winner is a safe default pick for this workload shape.

### cargo_alone beats baseline by 94% (significant)

cargo_alone is -762.61 ms (94%) faster than baseline from_clean, with a CI that excludes zero.

_Why it matters:_ A large, significant improvement over the current baseline is a concrete reason to switch.

### cargo_alone is fastest but the noisiest (CV 5.2%)

cargo_alone wins on median (47.14 ms) yet has the highest variance (CV 5.2%), while from_clean is the steadiest (CV 3.3%, 809.61 ms).

_Why it matters:_ For latency-sensitive or tail-bound paths, the steadier variant can beat the faster-on-average one; weigh peak vs consistency.

### cargo_alone shows alternating (throttle bounce) (autocorr -0.67)

cargo_alone's per-pass series has lag-1 autocorrelation -0.67, indicating alternating (throttle bounce). Its timing may not be at steady state.

_Why it matters:_ Autocorrelated samples violate the independence the CIs assume; the interval is optimistic until the drift is warmed out or cooled down.

### Wide spread: slowest is 17.2x the fastest

Fastest cargo_alone (47.14 ms) to slowest from_clean (809.61 ms): 17.2x. The strategy choice matters a lot for this workload.

_Why it matters:_ A wide field means the strategy is load-bearing here; getting it right (or wrong) has large consequences.

## Key findings

- **Fastest: cargo_alone** at 47139422.5 ns median (-94.2% vs baseline)
- 1 variant significantly faster than baseline
- Spread: 17.17x (fastest 47139422.5 ns, slowest 809613693.2 ns)

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
| cargo_alone | 48826301ns | 47146024ns | 47015338ns | -0ns | 48826301ns | -93.97% |
| from_clean | 809337651ns | 809624505ns | 776059948ns | -0ns | 809337651ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| cargo_alone | 48819147ns | 47008409ns | 48819147ns | -93.97% | 0.000 |
| from_clean | 809322286ns | 776051418ns | 809322286ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (cargo_alone; best 20% batches)
- Ops per call: 128

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| cargo_alone | 0.000 | 99.7% |
| from_clean | 0.000 | 5.8% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| cargo_alone | 48826301ns | 48826301ns | -93.97% |
| from_clean | 809337651ns | 809337651ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 809613693ns | base | --- | [776051418, 842301747] | --- | --- | --- | --- |
| cargo_alone | 47139422ns | -762605284.4ns (-94.2%) | [-795162325, -723741809]ns | [47008409, 52309610] | YES (adj: no) | 0.2500 | 0.2500 | 0 |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean | cargo_alone |
|---|---|---|---|
| 1 | 1 | 809613693ns | -94.2% |
| 1 | 2 | 776051418ns | -93.3% |
| 1 | 3 | 842301747ns | -94.4% |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| cargo_alone | -0.666 | HIGH- (thermal bounce) |
| from_clean | -0.504 | HIGH- (thermal bounce) |

**Consistency summary:**

- **cargo_alone**: won 3/3, lost 0/3

## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| cargo_alone | 44176009.7ns | 48819147.0ns | 90.5% | HIGH |
| from_clean | 42768153.8ns | 809322286.3ns | 5.3% | HIGH |

## Distribution (algo ns)

```
cargo_alone (n=3, range 47008408.8-48819147.0 ns)
  47008408.8 |########################################
  47098945.7 |########################################
  47189482.6 |
  47280019.5 |
  47370556.4 |
  47461093.3 |
  47551630.3 |
  47642167.2 |
  47732704.1 |
  47823241.0 |
  47913777.9 |
  48004314.8 |
  48094851.7 |
  48185388.6 |
  48275925.5 |
  48366462.5 |
  48456999.4 |
  48547536.3 |
  48638073.2 |
  48728610.1 |
  (0 below, 1 above range)

from_clean (n=3, range 776051418.5-809322286.3 ns)
  776051418.5 |########################################
  777714961.9 |
  779378505.3 |
  781042048.7 |
  782705592.1 |
  784369135.5 |
  786032678.9 |
  787696222.2 |
  789359765.6 |
  791023309.0 |
  792686852.4 |
  794350395.8 |
  796013939.2 |
  797677482.6 |
  799341026.0 |
  801004569.4 |
  802668112.8 |
  804331656.2 |
  805995199.6 |
  807658742.9 |
  (0 below, 2 above range)

```

## Diagnostics

- **cargo_alone**: bridge=86.3% of algo (FFI overhead may distort results)
