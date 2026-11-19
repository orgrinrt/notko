# `notko-build`

[![Crates.io](https://img.shields.io/crates/v/notko-build)](https://crates.io/crates/notko-build) [![docs.rs](https://img.shields.io/docsrs/notko-build)](https://docs.rs/notko-build)

> The build-script half of `#[profile]`'s custom tiers. Gathers one directory for the proc-macro to read, from a crate and whichever of its direct dependencies opted in.

Custom tiers are ordinary files a crate keeps in its own `notko-optimisers/` directory, and by default the [`notko-macros`](https://crates.io/crates/notko-macros) proc-macro only sees the ones belonging to the crate it's expanding into. This build-script helper is for the case where a tier is defined in one crate and used in another. It copies the crate's own optimiser files and the ones its dependencies handed over into `$OUT_DIR/notko-optimisers/`, and points the proc-macro at the result through `NOTKO_OPTIMISERS_PATH`.

## Usage

```bash
cargo add --build notko-build
```

```toml
[build-dependencies]
notko-build = "0.0.1"
```

A crate that only uses tiers from its dependencies calls it from its `build.rs`, and calling it on every build is the intended shape. It returns an error anywhere that isn't a build script.

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    notko_build::collect_and_distribute()?;
    Ok(())
}
```

A crate that provides tiers declares `links` with any value that's unique in the graph, since that's what makes cargo carry the metadata to its dependents, and keeps its tiers in `./notko-optimisers/*.rs`. Each file needs the marker line and a `based_on`, and the other two keys are optional:

```rust
//! @notko-optimiser
//! based_on = "Hot"
//! inline = true
//! panic_fmt = "trace invariant violated: {err:?}"
```

The file's stem is the tier's name, so `Trace.rs` is what `#[profile(Trace)]` looks for, and the full key list is in the [`notko-macros`](https://crates.io/crates/notko-macros) readme. Two dependencies providing the same tier name is a build error naming both, and a file in the crate's own directory wins over every dependency's.

The `DEP_*` variables only reach a crate's direct dependencies, and only the ones that declare `links` and run this themselves, so a tier two hops away arrives only if the crate in between forwards it. The pieces have tests, but the whole cargo handshake hasn't been run end to end here.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
