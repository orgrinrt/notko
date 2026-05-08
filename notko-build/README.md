# notko-build

Build-script helper for [notko-macros]: collects crate-local and
dependency-provided optimiser files into `$OUT_DIR/notko-optimisers/` and
exposes that path to the proc-macro via `NOTKO_OPTIMISERS_PATH`.

[notko-macros]: https://github.com/orgrinrt/notko/tree/dev/notko-macros

## A note on spelling

Source files live in `notko-optimizers/` (US "z"), by historical convention.
The env var, `links` key, and `$OUT_DIR` subdirectory use `notko-optimisers/`
(UK "s") to match the British baseline elsewhere in the project. Both forms
appear throughout this README. The split is intentional; readers should not
assume a typo.

## Usage

### Consumer-only crate (uses optimisers from deps)

```toml
# Cargo.toml
[package]
name = "my-crate"
build = "build.rs"

[build-dependencies]
notko-build = "0.1"

[dependencies]
notko-macros = "0.1"
# ... plus whichever crates in your dep tree provide the tiers you want to
# consume via `#[profile(X)]`
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    notko_build::collect_and_distribute()?;
    Ok(())
}
```

That's it. Optimisers contributed by any crate in the dep graph are
discoverable via `#[profile(Name)]`.

### Provider crate (publishes optimisers to downstream dependents)

```toml
# Cargo.toml
[package]
name = "my-provider"
build = "build.rs"
links = "notko-optimisers-my-provider"  # unique; required for cargo metadata propagation

[build-dependencies]
notko-build = "0.1"
```

Drop your optimiser files into `./notko-optimizers/*.rs`. Each must carry
the canonical header:

```rust
//! @notko-optimizer
//! based_on = "Cold"
//! inline = false
//! panic_fmt = "trace invariant violated: {err:?}"
```

Anything that directly depends on `my-provider` and also runs
`notko_build::collect_and_distribute()` in its own build script will see
these optimisers accumulated into its own `$OUT_DIR/notko-optimisers/`
and usable by `#[profile(Name)]`.

## How it works

1. Scans `$CARGO_MANIFEST_DIR/notko-optimizers/*.rs` (crate-local).
2. Collects paths from `DEP_NOTKO-OPTIMISERS-*_NOTKO_OPTIMISER_PATH`
   environment variables. Cargo sets these on build scripts of crates
   that depend on an optimiser provider.
3. Copies every `.rs` file into `$OUT_DIR/notko-optimisers/`.
4. Collisions: two sources providing the same tier name produce a build
   error unless the consumer's own crate-local file shadows both. The
   error lists both source paths; resolve by renaming or providing a
   local override.
5. Emits:
   - `cargo:rustc-env=NOTKO_OPTIMISERS_PATH=$OUT_DIR/notko-optimisers`,
     which the notko-macros proc-macro reads during expansion.
   - `cargo:notko-optimiser-path=$OUT_DIR/notko-optimisers`, which
     propagates this crate's accumulated optimisers to downstream
     dependents (only takes effect if the crate declares
     `links = "notko-optimisers-..."`).
   - `cargo:rerun-if-changed=notko-optimizers`, which invalidates the
     build when optimiser files change.

## Discovery precedence

The notko-macros proc-macro consults sources in this order:

1. Built-in ZST markers (`Hot`, `Warm`, `Cold`).
2. `$CARGO_MANIFEST_DIR/notko-optimizers/<Name>.rs` (crate-local;
   doesn't require notko-build).
3. `$NOTKO_OPTIMISERS_PATH/<Name>.rs` (accumulated; requires notko-build
   in the consumer's build.rs).

This means a consumer can always shadow a dep's optimiser by placing a
file of the same name in its own `notko-optimizers/` dir.

## License

MPL-2.0.
