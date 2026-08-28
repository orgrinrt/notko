# `notko-build`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-build)](https://crates.io/crates/notko-build)
[![docs.rs](https://img.shields.io/docsrs/notko-build)](https://docs.rs/notko-build)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> The build-script half of `#[profile]`'s custom tiers. Gathers one directory for the proc-macro to read, from a crate and whichever of its direct dependencies opted in.

</div>

Custom tiers are ordinary files a crate keeps in its own `notko-optimisers/` directory, and by default
the [`notko-macros`](https://crates.io/crates/notko-macros) proc-macro only sees the ones belonging to
the crate it's expanding into. That's fine until you want a tier defined once and used elsewhere, and
cargo gives a proc-macro no way to reach a file in a dependency.

So this runs in a build script instead. It copies the crate's own optimiser files and the ones its
dependencies handed over into `$OUT_DIR/notko-optimisers/`, and points the proc-macro at the result
through `NOTKO_OPTIMISERS_PATH`, which is only worth doing when tiers are shared between crates in
the first place.

## Installation

```bash
cargo add --build notko-build
```

Or add to your `Cargo.toml`:

```toml
[build-dependencies]
notko-build = "0.0.1"
```

It has no dependencies of its own, and it belongs under `[build-dependencies]` rather than
`[dependencies]`, since nothing in it runs outside a build script.

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

It's idempotent, so calling it every build is the intended shape rather than something to guard. Called
anywhere that isn't a build script it returns an error rather than doing something surprising, since it
reads `CARGO_MANIFEST_DIR` and `OUT_DIR` and neither is there.

That's it, with one limit worth knowing before you count on it. `DEP_*` reaches your *direct*
dependencies only, and only the ones that declare `links` and run `collect_and_distribute` themselves. A
tier two hops away arrives if the crate in between forwards it, and does not if that crate does nothing.
So the reach is whatever the chain cooperates on, which is usually less than the whole graph.

### Provider crate (publishes optimisers to downstream dependents)

```toml
# Cargo.toml
[package]
name = "my-provider"
build = "build.rs"
links = "notko-optimisers-my-provider"  # any value, as long as it's unique in the graph

[build-dependencies]
notko-build = "0.0.1"
```

`links` is what makes cargo carry the metadata to your dependents at all, and cargo requires the value be
unique across the whole graph. The `notko-optimisers-` prefix above is only a habit that keeps two of
them from colliding; nothing here reads it.

Drop your optimiser files into `./notko-optimisers/*.rs`. Each needs the marker line and a `based_on`,
and the other two keys are optional:

```rust
//! @notko-optimiser
//! based_on = "Hot"
//! inline = true
//! panic_fmt = "trace invariant violated: {err:?}"
```

The file's own stem is the tier's name, casing and all, so `Trace.rs` is what `#[profile(Trace)]` looks
for. `panic_fmt` is read on the hot strategy alone. The full key list is in
[`notko-macros`](https://crates.io/crates/notko-macros)'s readme.

Anything that directly depends on `my-provider` and also runs `notko_build::collect_and_distribute()` in
its own build script will see these accumulated into its own `$OUT_DIR/notko-optimisers/` and usable by
`#[profile(Name)]`.

## How it works

1. Scans `$CARGO_MANIFEST_DIR/notko-optimisers/` for `*.rs`. One directory, not walked recursively, so a
   file in a subdirectory of it is not found.
2. Collects paths from `DEP_*_NOTKO_OPTIMISER_PATH` environment variables. Cargo sets these on the build
   scripts of crates that directly depend on an optimiser provider.
3. Copies the crate's own files first, then each dependency's, skipping any name the crate itself already
   claimed. Nothing checks a copied file for the `@notko-optimiser` marker, so a stray `.rs` sitting in
   that directory travels along and only fails later, at expansion.
4. Emits:
   - `cargo:rustc-env=NOTKO_OPTIMISERS_PATH=$OUT_DIR/notko-optimisers`, which the notko-macros
     proc-macro reads during expansion.
   - `cargo:notko-optimiser-path=$OUT_DIR/notko-optimisers`, which propagates this crate's accumulated
     optimisers to downstream dependents. It only reaches anyone if the crate declares `links`.
   - `cargo:rerun-if-changed=` for the local `notko-optimisers` directory and for each file in it, so an
     optimiser edited, added or removed invalidates the build. Only the local directory, and only when
     there is one: a consumer with no optimisers of its own emits none of these, and a dependency's
     directory is not watched either. Do note that emitting any of them at all opts the package out of
     cargo's default of rerunning when any file in it changed.

Two dependencies providing the same tier name is a build error, since nothing ranks one dependency
above another. The error names both source paths. Resolve it by renaming, or by putting a file of that
name in your own `notko-optimisers/`, which wins over every dependency's.

## Discovery precedence

The notko-macros proc-macro consults sources in this order:

1. Built-in ZST markers (`Hot`, `Warm`, `Cold`).
2. `$CARGO_MANIFEST_DIR/notko-optimisers/<Name>.rs` (crate-local;
   doesn't require notko-build).
3. `$NOTKO_OPTIMISERS_PATH/<Name>.rs` (accumulated; this crate is what usually
   sets that variable, though the lookup reads it wherever it came from).

The shadowing above is a separate thing from this list, and both point the same way. A dependency's file
losing to your own happens here, in the build script, where the loser is never copied and so never
reaches `$OUT_DIR` to be looked up at all.

## What isn't covered yet

The pieces have tests, the whole does not. Nothing in this repository declares `links` or runs a build
script, so the cargo handshake this crate exists for, provider emits, cargo forwards, consumer
accumulates, proc-macro resolves, has never been run end to end here. The copying, the collision, the
shadowing and the two emitted strings are all pinned; the hop between two crates is on paper. Worth
knowing which half you're relying on.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
