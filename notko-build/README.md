# `notko-build`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-build)](https://crates.io/crates/notko-build)
[![docs.rs](https://img.shields.io/docsrs/notko-build)](https://docs.rs/notko-build)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Lets one crate define a `#[profile]` tier and the crates that depend on it use that tier by name.

</div>

Custom tiers are ordinary files a crate keeps in its own `notko-optimisers/` directory, and by default
the [`notko-macros`](https://crates.io/crates/notko-macros) proc-macro only sees the ones belonging to
the crate it's expanding into. That's fine until you want a tier defined once and used everywhere, and
cargo gives a proc-macro no way to reach a file in a dependency.

So this runs in a build script instead. It gathers every optimiser file the crate can see, its own and
its dependencies', into `$OUT_DIR/notko-optimisers/`, and points the proc-macro at the result through
`NOTKO_OPTIMISERS_PATH`. You only need it if you're sharing tiers across crates.

## The two spellings

Source files live in `notko-optimisers/`, with a z. The env var, the `links`
key and the `$OUT_DIR` subdirectory are `notko-optimisers/`, with an s. Both
spellings turn up below and neither is a typo, so it's worth having the pair in
mind: a directory named with the wrong one is simply not found, and nothing
says so.

## Usage

### Consumer-only crate (uses optimisers from deps)

```toml
# Cargo.toml
[package]
name = "my-crate"
build = "build.rs"

[build-dependencies]
notko-build = "0.0.1"

[dependencies]
notko-macros = "0.0.1"
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

That's it, with one limit worth knowing before you count on it. `DEP_*` reaches
your *direct* dependencies only, and only the ones that declare `links` and run
`collect_and_distribute` themselves. A tier two hops away arrives if the crate
in between forwards it, and does not if that crate does nothing. So the reach is
whatever the chain actually cooperates on, not the whole graph.

### Provider crate (publishes optimisers to downstream dependents)

```toml
# Cargo.toml
[package]
name = "my-provider"
build = "build.rs"
links = "notko-optimisers-my-provider"  # unique; required for cargo metadata propagation

[build-dependencies]
notko-build = "0.0.1"
```

Drop your optimiser files into `./notko-optimisers/*.rs`. Each must carry
the canonical header:

```rust
//! @notko-optimiser
//! based_on = "Cold"
//! inline = false
//! panic_fmt = "trace invariant violated: {err:?}"
```

Anything that directly depends on `my-provider` and also runs
`notko_build::collect_and_distribute()` in its own build script will see
these optimisers accumulated into its own `$OUT_DIR/notko-optimisers/`
and usable by `#[profile(Name)]`.

## How it works

1. Scans `$CARGO_MANIFEST_DIR/notko-optimisers/*.rs` (crate-local).
2. Collects paths from `DEP_*_NOTKO_OPTIMISER_PATH`
   environment variables. Cargo sets these on build scripts of crates
   that depend on an optimiser provider.
3. Copies every `.rs` file into `$OUT_DIR/notko-optimisers/`.
4. Emits:
   - `cargo:rustc-env=NOTKO_OPTIMISERS_PATH=$OUT_DIR/notko-optimisers`,
     which the notko-macros proc-macro reads during expansion.
   - `cargo:notko-optimiser-path=$OUT_DIR/notko-optimisers`, which
     propagates this crate's accumulated optimisers to downstream
     dependents (only takes effect if the crate declares
     `links = "notko-optimisers-..."`).
   - `cargo:rerun-if-changed=` for the local `notko-optimisers` directory
     and again for each file in it, which invalidates the build when an
     optimiser is edited, added or removed.

Two dependencies providing the same tier name is a build error, since nothing ranks one dependency
above another. The error names both source paths. Resolve it by renaming, or by putting a file of that
name in your own `notko-optimisers/`, which wins over every dependency's.

## Discovery precedence

The notko-macros proc-macro consults sources in this order:

1. Built-in ZST markers (`Hot`, `Warm`, `Cold`).
2. `$CARGO_MANIFEST_DIR/notko-optimisers/<Name>.rs` (crate-local;
   doesn't require notko-build).
3. `$NOTKO_OPTIMISERS_PATH/<Name>.rs` (accumulated; requires notko-build
   in the consumer's build.rs).

Which is what makes the shadowing above work: a file of the same name in your own
`notko-optimisers/` is found before anything a dependency contributed.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
