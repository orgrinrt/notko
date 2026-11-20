# How long a dev build takes over a registry of columns, on the product with its carrier

This is the first committed measurement of what `notko-hlist`'s product costs a consumer at compile time, asked for as N4 of the stack roadmap (clause-dev `.shared/research/202611191916_stack-next-tracks/README.md`, track 6). It is a run of the bench `product-build` in `mock/benches/`, over the product as it stands after the carrier landed (`Project<C>` with the carrier as its parameter, `Has<C, M, P>`, `Rows<C>`, `Arrays<N>`), at 32, 64, 128 and 256 columns. The bench follows the shape of mockspace's sketch `mock/research/sketches/202611160019_a-bench-arm-that-measures-a-build/` on mockspace `dev`: the timed block is one cargo build, and a hooks library refuses a cell where any call built other than what it was meant to.

The earlier figures for compile time are the ad-hoc spike `sketches/202611140950_hlist-scaling-adhoc/`, written against the product before the carrier, timed by hand and outside the harness. They are not comparable with this run and this run does not stand in for them. It measures a different question: a whole consumer-shaped crate built, not one query checked.

## What is built

The fixture crate `mock/benches/product-build/fixture/` has its source written per point by `mock/benches/support/product-build-call/` (`source.rs`). For `n` columns it holds `n` marker types, each an `Elem` over `u32`; the registry as one `Cons` list of them; the row as `<Registry as Project<Val>>::Prod` and one literal value of it; `n / 8` systems, each reading four columns through `Has::pick` and stating four reads and two writes as `ContainsAll` bounds on the registry; one function that makes the column arrays with `Arrays<4>::made` and writes and reads a row through `Rows<Arr<4>>`; and one function calling every system. The number of systems grows with the columns, so the number of lookups grows with them too, each a walk of up to `n` members. That is how a registry grows in a consumer, and it makes the work about quadratic by construction.

Each call, untimed, writes the source, brings `notko-hlist` up to date and cleans the fixture package alone. The timed block is `cargo build -p notko-product-fixture`, with `--features registry` for the arm that builds the product, dev profile, offline and locked, with `CARGO_INCREMENTAL=0` and no compiler wrapper, into a target and build directory of the bench's own. After it, cargo's own messages are counted: the hooks refused any cell where a call compiled anything but the fixture, found the fixture's library before the build or not after it, waited on a lock, ran at another point than its cell, built the fixture with other features than its arm asked for, or failed. Every cell of this run passed that check, 32 calls each over both arms (`calls.tsv` in each run directory).

## Results

Run `20261120T071113Z-323f66d`, machine class `linux-x86_64-c4-container-intel` (4 cpus, Xeon at 2.10 GHz, a shared container), `rustc 1.102.0-nightly (59cbdee2e 2026-11-17)`. The fixture is built in cargo's dev profile; the release profile in `meta.json` is the harness's, for the driver and the arms. Three passes, each taking one warm and one cold sample per arm, so six samples per arm per point. Directories: `mock/benches/results/product-build/product-build_n<point>/linux-x86_64-c4-container-intel/20261120T071113Z-323f66d/`.

Two arms. `from_clean` builds the fixture with its registry. `cargo_alone` builds the same fixture without the `registry` feature, so the crate is empty: cargo starts, resolves, fingerprints and writes a library, and the compiler has nothing of the product to do. It is the floor, and what is left above it is what the product costs the compiler.

The medians are the harness's, over the three warm samples; warm and cold mean nothing different here, since every call cleans the fixture before it builds. The range is over all six.

| columns | build, median | range, ms | floor, median | above the floor | per column | against half the columns |
|---|---|---|---|---|---|---|
| 32 | 143 ms | 111 to 181 | 43 ms | 100 ms | 3.1 ms | |
| 64 | 247 ms | 228 to 253 | 54 ms | 192 ms | 3.0 ms | 1.9 times, +92 ms |
| 128 | 810 ms | 776 to 1151 | 47 ms | 763 ms | 6.0 ms | 4.0 times, +571 ms |
| 256 | 3817 ms | 3695 to 3836 | 51 ms | 3767 ms | 14.7 ms | 4.9 times, +3004 ms |

The floor stays between 43 and 60 ms over every sample at every point, so cargo's own work is a fixed cost and none of the growth. Above it, the cost is about linear up to 64 columns. From 64 to 128 it about quadruples, and the last doubling costs 4.9 times, an exponent of about 2.3. A registry of 256 columns costs a consumer close to four seconds of compiler time in a dev build for this shape of use. Nothing past 256 was measured, and no figure past it is claimed.

The 32-column point is the noisiest: its range is wider than its floor-subtracted cost, and its median here is 32 ms above the one in the run before (below), which is within that range. Its figures say the cost there is around a tenth of a second and no more exactly than that.

Two earlier runs are kept beside this one. `20261120T064624Z-ccbd272` was refused: its check could not see the fixture's library, so it certifies nothing (the commits `9252c11` and `d4d4ad1`). `20261120T065436Z-d4d4ad1` passed with `from_clean` alone and no floor; its warm medians, 111, 225, 744 and 3596 ms, agree with this run within the ranges above, and it is superseded by this one only because it has nothing to subtract.

## Holds for

columns 32, 64, 128, 256; systems `n / 8`, four picks and six bound members each; carrier `Val` and `Arr<4>`; dev profile, `CARGO_INCREMENTAL=0`, no compiler wrapper, `notko-hlist` prebuilt; the timed block a whole `cargo build` of the fixture, the compiler's share read as that minus the `cargo_alone` floor at the same point; medians over the three warm samples; this machine class; this nightly.

## What it does not show

Nothing here is about a release build, about `cargo check` alone, about link time, or about a carrier other than the uniform one. Which of the pieces (the picks, the `ContainsAll` bounds, the row literal, the walks) carries the growth was not separated: the bench has one arm that builds the product, and its floor. A per-piece arm is the next question if the curve matters, and the ad-hoc spike's table names the likely ones (`ContainsAll` of a reordered list and a pick of every member grew between quadratic and cubic there). Six samples per arm per point on a shared container give the range above and no more; a point whose range overlaps a neighbour's would need more passes before a difference between them is quoted.
