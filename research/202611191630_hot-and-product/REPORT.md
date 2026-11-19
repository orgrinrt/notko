# Job notko-hot-and-product: the hot tier said plainly, and the product carrier as typestate

The job's brief is the channel doc `briefs/job-notko-hot-and-product.md`, from stack-coord, carrying two of the maintainer's calls of 16 November, recorded in clause-dev `.shared/research/202611150830_stack-path/57-op-calls-on-the-night.md`. Call 1 keeps `#[profile(Hot)]` and asks that the documentation say plainly that it panics and aborts on an error and is for code that does not branch and does not fail. Call 2 admits the product's carrier as a parameter of the product trait only where the parameterisation is itself typestate, every member's form stated in the types. Later files 58 to 64 of that record do not touch notko.

notko declares no canon and has no mockspace, so its designs stand in for one, and `cargo mock canon contradicts` has nothing to run against here.

## The hot tier

Commit `b464afd`. The root README, the notko-macros README, the crate docs in `src/lib.rs` and `src/just.rs`, and the rewrite's own comments in `notko-macros-core` now say that Hot is for code that does not branch and does not fail, taken as an extra optimisation where that holds, and that on an error the shipping arm panics, and aborts under `panic = "abort"`. The line in `src/just.rs` claiming the two arms mean the same program is corrected: they agree on code that never fails, and on an error they differ by design. No behaviour changed. The removal proposal of file 56 was not taken.

## The product carrier

In order on the branch: the sketch `research/sketches/202611191400_carrier-as-a-typestate-parameter/` (`665c590`), the design `docs/design/notko-hlist-product-carrier.md` (`69915c8`), the tests (`e90887a`), the code (`3752651`), the target registration (`3eab720`), and a check that every test file is a declared target (`08ec816`).

The contract, posted to stack-coord before landing: `Carrier<M>` states one form per carrier and member; `Uniform` is the shipped carrier renamed, a `Carrier<E>` for every `Elem` member by a blanket impl; `Project<C>` takes the carrier as its parameter and `PCons<C, H, T>` names carrier and member; `Has<C, M, P>`, `SubOf<C, S, Ps>` with `Dup<M>`, the walks `Rows<C>` and `Arrays<N>`, and `PutRow` and `PutVals`. Every product trait is sealed by a crate-private token.

### Tests that failed first

At `e90887a`, the tests with the shipped product, `cargo test -p notko-hlist --test a_carrier_holds_a_form_per_member` did not build: 41 errors, among them `E0107` (the shipped `Carrier` and `Project` take no parameters), `E0119` (two `Carrier` impls for one carrier, since the shipped trait has no member parameter), and `E0277` (no product for a list of non-`Elem` members). The target was not yet named in the manifest at that commit; the run named it in a scratch worktree, and `3eab720` names it on the branch.

### Runs on the head

Toolchain `nightly-2026-05-28`.

- `cargo test -q -p notko-hlist`: every target green, the doctests included.
- `cargo test -q -p notko-hlist --no-default-features`: green.
- `cargo clippy -q -p notko-hlist --all-targets`: no warnings.
- `RUSTDOCFLAGS=-Dwarnings cargo doc -p notko-hlist --no-deps`: no warnings.
- The new check, `every_test_file_is_a_declared_target`, shown failing with the new target's path misspelled before it was committed.
- The notko-macros suites were not run to the end: `consumer_cfg.rs` ran past 30 minutes and was cut at the background limit (ticket `cfcb832cb235`). The hot-tier change is prose only.

## Review

Pass 1 of at most 3 is running; this section is written when the passes end.

## Tickets

- `cfcb832cb235`: the notko-macros `consumer_cfg` suite runs past 30 minutes in a cloud container.
- `30aa6117f340`, bumped once: the saturation guard counts the harness's own hung builds.
- `1b41bf7dd0cc`: `lazy-tool` calls homma from a cargo build script and deadlocks the stand-up's kaski build. Killing the tree on the maintainer's relayed yes was refused twice by the auto-mode classifier.
