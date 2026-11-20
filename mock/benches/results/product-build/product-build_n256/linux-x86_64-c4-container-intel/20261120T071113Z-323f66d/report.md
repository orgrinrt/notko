# product-build: the product over a registry of columns, built in the dev profile

2 variants, 3 samples per variant.
Baseline: **from_clean**

## Highlights

Baseline for all deltas below: **from_clean**. (Deltas are paired `variant - baseline` medians; `*` marks a CI that excludes zero.)

### Baseline (from_clean) is the SLOWEST variant; every rival beats it

The declared/defaulted baseline from_clean has the worst median (3817.08 ms). Every delta is therefore measured against the worst performer, which flatters all rivals and compresses the differences that matter among them (e.g. fastest cargo_alone at 50.57 ms).

_Why it matters:_ A baseline picked by accident (often the first variant to run / sort) silently skews every comparison. Re-baseline via `[bench.<name>.normalise]` on a representative variant.

### cargo_alone dominates: 7449% faster than the next best (from_clean)

cargo_alone (50.57 ms) leads from_clean (3817.08 ms) by 7449%, a clear separation rather than a photo finish. CV 1.9%.

_Why it matters:_ A dominant, well-separated winner is a safe default pick for this workload shape.

### cargo_alone beats baseline by 99% (significant)

cargo_alone is -3766.51 ms (99%) faster than baseline from_clean, with a CI that excludes zero.

_Why it matters:_ A large, significant improvement over the current baseline is a concrete reason to switch.

### from_clean shows alternating (throttle bounce) (autocorr -0.65)

from_clean's per-pass series has lag-1 autocorrelation -0.65, indicating alternating (throttle bounce). Its timing may not be at steady state.

_Why it matters:_ Autocorrelated samples violate the independence the CIs assume; the interval is optimistic until the drift is warmed out or cooled down.

### Wide spread: slowest is 75.5x the fastest

Fastest cargo_alone (50.57 ms) to slowest from_clean (3817.08 ms): 75.5x. The strategy choice matters a lot for this workload.

_Why it matters:_ A wide field means the strategy is load-bearing here; getting it right (or wrong) has large consequences.

## Key findings

- **Fastest: cargo_alone** at 50565200.9 ns median (-98.7% vs baseline)
- 1 variant significantly faster than baseline
- Spread: 75.49x (fastest 50565200.9 ns, slowest 3817076259.8 ns)

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
| cargo_alone | 50239766ns | 50571988ns | 48923063ns | -0ns | 50239766ns | -98.67% |
| from_clean | 3791546698ns | 3817087788ns | 3721629534ns | -0ns | 3791546698ns | base |

## Function-under-test only (all cooldowns combined)

| Variant | mean | best 20% | worst 20% | Δ mean | throughput (Gops/s) |
|---|---|---|---|---|---|
| cargo_alone | 50232062ns | 48915047ns | 50232062ns | -98.68% | 0.000 |
| from_clean | 3791536987ns | 3721621287ns | 3791536987ns | base | 0.000 |

## Performance model

- Peak throughput: **0.000 Gops/s** (cargo_alone; best 20% batches)
- Ops per call: 256

| Variant | Gops/s (median) | % of peak |
|---|---|---|
| cargo_alone | 0.000 | 96.7% |
| from_clean | 0.000 | 1.3% |

## Per-cooldown breakdown (e2e mean)

| Variant | 0ms | avg | Δ avg |
|---|---|---|---|
| cargo_alone | 50239766ns | 50239766ns | -98.67% |
| from_clean | 3791546698ns | 3791546698ns | base |

## Statistical comparison (algo, 95% bootstrap CI)

| Variant | median | Δ median | Δ CI | 95% CI | sig? | adj. p | sign p | ties |
|---|---|---|---|---|---|---|---|---|
| from_clean | 3817076260ns | base | --- | [3721621287, 3835913415] | --- | --- | --- | --- |
| cargo_alone | 50565201ns | -3766511058.9ns (-98.7%) | [-3784697476, -3672706240]ns | [48915047, 51215939] | YES (adj: no) | 0.2500 | 0.2500 | 0 |

## Per-pass consistency (nonstop algo, Δ vs baseline)

| Run | Pass | from_clean | cargo_alone |
|---|---|---|---|
| 1 | 1 | 3817076260ns | -98.7% |
| 1 | 2 | 3721621287ns | -98.7% |
| 1 | 3 | 3835913415ns | -98.7% |

**Autocorrelation (lag-1) per-pass series:**

| Variant | r₁ | note |
|---|---|---|
| cargo_alone | -0.616 | HIGH- (thermal bounce) |
| from_clean | -0.651 | HIGH- (thermal bounce) |

**Consistency summary:**

- **cargo_alone**: won 3/3, lost 0/3

## Bridge overhead per variant

| Variant | mean bridge | algo mean | bridge % | flag |
|---|---|---|---|---|
| cargo_alone | 45869024.2ns | 50232062.5ns | 91.3% | HIGH |
| from_clean | 45984539.2ns | 3791536987.5ns | 1.2% |  |

## Distribution (algo ns)

```
cargo_alone (n=3, range 48915047.3-50232062.5 ns)
  48915047.3 |########################################
  48980898.1 |
  49046748.8 |
  49112599.6 |
  49178450.3 |
  49244301.1 |
  49310151.8 |
  49376002.6 |
  49441853.4 |
  49507704.1 |
  49573554.9 |
  49639405.6 |
  49705256.4 |
  49771107.2 |
  49836957.9 |
  49902808.7 |
  49968659.4 |
  50034510.2 |
  50100360.9 |
  50166211.7 |
  (0 below, 2 above range)

from_clean (n=3, range 3721621287.2-3791536987.5 ns)
  3721621287.2 |########################################
  3725117072.2 |
  3728612857.2 |
  3732108642.2 |
  3735604427.3 |
  3739100212.3 |
  3742595997.3 |
  3746091782.3 |
  3749587567.3 |
  3753083352.3 |
  3756579137.3 |
  3760074922.3 |
  3763570707.4 |
  3767066492.4 |
  3770562277.4 |
  3774058062.4 |
  3777553847.4 |
  3781049632.4 |
  3784545417.4 |
  3788041202.5 |
  (0 below, 2 above range)

```

## Diagnostics

- **cargo_alone**: bridge=94.9% of algo (FFI overhead may distort results)
