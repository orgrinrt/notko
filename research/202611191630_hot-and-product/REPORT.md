# Job notko-hot-and-product: the hot tier said plainly, and the product carrier as typestate

The job's brief is the channel doc `briefs/job-notko-hot-and-product.md`, from stack-coord, carrying two of the maintainer's calls of 16 November, recorded in clause-dev `.shared/research/202611150830_stack-path/57-op-calls-on-the-night.md`. Call 1 keeps `#[profile(Hot)]` and asks that the documentation say plainly that it panics and aborts on an error and is for code that does not branch and does not fail. Call 2 admits the product's carrier as a parameter of the product trait only where the parameterisation is itself typestate, every member's form stated in the types. Later files 58 to 64 of that record do not touch notko.

notko declares no canon and has no mockspace, so its designs stand in for one, and `cargo mock canon contradicts` has nothing to run against here.

## The hot tier

Commit `b464afd`, with three corrections from review in `d37240e`. The root README, the notko-macros README, the crate docs in `src/lib.rs` and `src/just.rs`, and the rewrite's comments and tests in `notko-macros-core` now say that Hot is for code that does not branch and does not fail, taken as an extra optimisation where that holds, and that on an error the internal-release build panics, and aborts under `panic = "abort"`. The line in `src/just.rs` claiming the two arms mean the same program is corrected: they agree on code that never fails, and on an error they differ by design. No behaviour changed. The removal proposal of file 56 was not taken.

## The product carrier

In order on the branch: the sketch `research/sketches/202611191400_carrier-as-a-typestate-parameter/` (`665c590`), the design `docs/design/notko-hlist-product-carrier.md` (`69915c8`), the tests (`e90887a`), the code (`3752651`), the target registration (`3eab720`), a check that every test file is a declared target (`08ec816`), and the two review fixes (`d37240e`, `8d2b644`).

The contract as it lands, posted to stack-coord before landing and again with each change:

- A carrier declares `Shape` with `Is = Uniformly` or `Is = PerMember`.
- `Uniform`, the shipped carrier renamed, holds every `Elem` member over its value, and nothing else.
- A per-member carrier states `Form<M>` for each member it holds and `DupForm<M>` for each form it can duplicate.
- `Carrier<M>` and `Dup<M>`, what the product and the cut read, are sealed and routed by the shape; `DupAll` is uniform duplication.
- `Project<C>` takes the carrier as its parameter and `PCons<C, H, T>` names carrier and member; `Has<C, M, P>`, `SubOf<C, S, Ps>`, the walks `Rows<C>` and `Arrays<N>`, `PutRow` and `PutVals`. Every product trait is sealed.

### Tests that failed first

At `e90887a`, the tests with the shipped product, `cargo test -p notko-hlist --test a_carrier_holds_a_form_per_member` did not build: 41 errors, among them `E0107` (the shipped `Carrier` and `Project` take no parameters), `E0119` (two `Carrier` impls for one carrier, since the shipped trait has no member parameter), and `E0277` (no product for a list of non-`Elem` members). The target was not yet named in the manifest at that commit; the run named it in a scratch worktree, and `3eab720` names it on the branch.

### Runs on the head

Toolchain `nightly-2026-05-28`, at `8d2b644`.

- `CARGO_INCREMENTAL=0 cargo test -q --no-fail-fast --workspace --exclude notko-macros`: green.
- `cargo test -q -p notko-hlist --no-default-features`: green.
- `cargo clippy -q -p notko-hlist --all-targets`: no warnings.
- `RUSTDOCFLAGS=-Dwarnings cargo doc -p notko-hlist --no-deps`: no warnings.
- `cargo fmt -p notko-hlist --check`: clean.
- `every_test_file_is_a_declared_target` shown failing with a target's path misspelled before it was committed.
- The notko-macros suites were not run to the end: `consumer_cfg.rs` ran past 30 minutes and was cut at the background limit (ticket `cfcb832cb235`). The hot-tier change is prose only.

## Review

Three passes of a fresh `pr-reviewer-senior`, the limit.

- Pass 1, at `08ec816`: do not merge. A sketch manifest without `publish = false`, red in the root suite, which the branch had not run. A crate owning a member that is not an `Elem` could give `Val` a form for it, and a crate owning an `Elem` member could state `Dup` for it on `Arr` or `ColMut`, both through the member parameter. Fixed in `d37240e` with a declared `Shape`, `Form<M>`, a sealed `Carrier<M>` and `DupAll`, each case a refusal with its twin; the three prose findings fixed with it.
- Pass 2, at `d37240e`: merge after fixes. A per-member carrier in one crate could not state `Dup` for a member owned by another, the arrangement hilavitkutin's read carrier has, and a downstream uniform carrier could duplicate one member and not the rest. Fixed in `8d2b644` by routing `Dup` through the shape (`DupForm<M>`), with a fixture crate owning a carrier for the two-crate test. The empty list's product now asks its carrier for a shape.
- Pass 3, at `8d2b644`: running when this was committed; its outcome follows in the next commit of this file.

## Questions for the maintainer

Both were handed back by review as not the job's to settle. Filing them with `ask` hung behind the deadlock below (its cargo waited on the stand-up's lock for 20 minutes and was stopped), so they are here for the coordinator to file as question tickets, `--holder op`.

1. Who may state the forms of a per-member carrier. A crate that owns member `M` may write `impl Form<M> for Read<'f>` on a carrier it does not own, and that is the only way a consumer's members get into a scheduler's read list; the orphan rule lets the member's owner write it and nothing lets the carrier's owner write it for members it cannot name. The types still fix the form, one impl per pair, but what `Read` means for a member is set by the member's crate, `&'f mut T` included. Options: a, the member's crate states them and the carrier's owner documents what its forms mean, as built; b, a carrier names the members it accepts through a trait of its own and refuses the rest, which needs that trait on every member; c, something else.
2. Whether Hot panics in debug builds too. The code panics, and aborts under `panic = "abort"`, only in the internal-release build; every other build, debug included, returns the error as an `Outcome`, so tests see it, and the docs now say exactly that. The maintainer's words were "hot is intended to panic and abort when there's an error". Options: a, keep the debug arm returning the error, as documented; b, panic in every build, which is the option "Keep, panics everywhere" he was shown on 16 November and did not pick.

## Tickets

- `cfcb832cb235`: the notko-macros `consumer_cfg` suite runs past 30 minutes in a cloud container.
- `30aa6117f340`, bumped once: the saturation guard counts the harness's own hung builds.
- `1b41bf7dd0cc`, bumped once: `lazy-tool` calls homma from a cargo build script and deadlocks the stand-up's kaski build. Killing the tree on the maintainer's relayed yes was refused twice by the auto-mode classifier, and `ask` hung behind it.
