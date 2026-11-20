# How long the compiler takes over a registry of columns, on the product with its carrier

This is the first committed measurement of what `notko-hlist`'s product costs a consumer at compile time, asked for as N4 of the stack roadmap (clause-dev `.shared/research/202611191916_stack-next-tracks/README.md`, track 6). It is a run of the bench `product-build` in `mock/benches/`, over the product as it stands after the carrier landed (`Project<C>` with the carrier as its parameter, `Has<C, M, P>`, `Rows<C>`, `Arrays<N>`), at 32, 64, 128 and 256 columns. The bench follows the shape of mockspace's sketch `mock/research/sketches/202611160019_a-bench-arm-that-measures-a-build/` on mockspace `dev`: the timed block is one cargo build, and a hooks library refuses a cell where any call built other than what it was meant to.

The earlier figures for compile time are the ad-hoc spike `sketches/202611140950_hlist-scaling-adhoc/`, written against the product before the carrier, timed by hand and outside the harness. They are not comparable with this run and this run does not stand in for them. It measures a different question: a whole consumer-shaped crate built, not one query checked.

## What is built

The fixture crate `mock/benches/product-build/fixture/` has its source written per point by `mock/benches/support/product-build-call/` (`source.rs`). For `n` columns it holds `n` marker types, each an `Elem` over `u32`; the registry as one `Cons` list of them; the row as `<Registry as Project<Val>>::Prod` and one literal value of it; `n / 8` systems, each reading four columns through `Has::pick` and stating four reads and two writes as `ContainsAll` bounds on the registry; one function that makes the column arrays with `Arrays<4>::made` and writes and reads a row through `Rows<Arr<4>>`; and one function calling every system. The number of systems grows with the columns, so the number of lookups grows with them too, each a walk of up to `n` members. That is how a registry grows in a consumer, and it makes the work about quadratic by construction.

Each call, untimed, writes the source, brings `notko-hlist` up to date and cleans the fixture package alone. The timed block is `cargo build -p notko-product-fixture --features registry`, dev profile, offline and locked, with `CARGO_INCREMENTAL=0` and no compiler wrapper, into a target and build directory of the bench's own. After it, cargo's own messages are counted: the hooks refused any cell where a call compiled anything but the fixture, found the fixture's library before the build or not after it, waited on a lock, or failed. Every cell of this run passed that check, 14 calls each (`calls.tsv` in each run directory).

## Results

Run `20261120T065436Z-d4d4ad1`, machine class `linux-x86_64-c4-container-intel` (4 cpus, Xeon at 2.10 GHz, a shared container), `rustc 1.102.0-nightly (59cbdee2e 2026-11-17)`. The fixture is built in cargo's dev profile; the release profile in `meta.json` is the harness's, for the driver and the arm. Three passes, two samples each, so six samples per point. Directories: `mock/benches/results/product-build/product-build_n<point>/linux-x86_64-c4-container-intel/20261120T065436Z-d4d4ad1/`.

| columns | median | samples, ms | per column | against half the columns |
|---|---|---|---|---|
| 32 | 111 ms | 104 to 131 | 3.5 ms | |
| 64 | 225 ms | 209 to 240 | 3.5 ms | 2.0 times, +115 ms |
| 128 | 744 ms | 742 to 803 | 5.8 ms | 3.3 times, +518 ms |
| 256 | 3596 ms | 3410 to 3939 | 14.0 ms | 4.8 times, +2853 ms |

Up to 64 columns the cost is about linear and small. From 128 it grows faster than quadratic: the last doubling costs 4.8 times, an exponent of about 2.3. A registry of 256 columns costs a consumer three and a half seconds of a dev build for this shape of use, and the next doubling, on the same curve, would cost upwards of fifteen.

The refused first run, `20261120T064624Z-ccbd272`, is kept beside it. Its medians are close to these, but its check could not see the fixture's library, so it certifies nothing; why is in the commits `9252c11` and `d4d4ad1`.

## Holds for

columns 32, 64, 128, 256; systems `n / 8`, four picks and six bound members each; carrier `Val` and `Arr<4>`; dev profile, `CARGO_INCREMENTAL=0`, `notko-hlist` prebuilt; this machine class; this nightly.

## What it does not show

Nothing here is about a release build, about `cargo check` alone, about link time, or about a carrier other than the uniform one. Which of the pieces (the picks, the `ContainsAll` bounds, the row literal, the walks) carries the growth was not separated: the bench has one arm. A per-piece arm is the next question if the curve matters, and the ad-hoc spike's table names the likely ones (`ContainsAll` of a reordered list and a pick of every member grew between quadratic and cubic there). Six samples per point on a shared container give the range above and no more; a point whose range overlaps a neighbour's would need more passes before a difference between them is quoted.
