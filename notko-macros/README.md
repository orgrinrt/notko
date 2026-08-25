# `notko-macros`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-macros)](https://crates.io/crates/notko-macros)
[![docs.rs](https://img.shields.io/docsrs/notko-macros)](https://docs.rs/notko-macros)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> `#[profile(Hot)]` on a function, and the body gets rewritten to the fallibility tier you named. Compile time, no runtime cost.

</div>

You write an ordinary `Result` function and tag it. The macro rewrites the signature and the body to
whichever tier the profile names, so the same source compiles to a checked `Outcome<T, E>` in one build
and to an unchecked `Just<T>` in another, without you touching the function in between.

The tiers themselves come from [`notko`](https://crates.io/crates/notko), which is the crate that
defines them, so you'll want that as a dependency too.

## Built-in tiers

| Tier | Debug / standalone | Release + internal |
|------|--------------------|--------------------|
| `Hot` | `Outcome<T, E>` wrapping; `Ok(x)` → `Outcome::Ok(x)`, `Err(e)` → `Outcome::Err(e)` | `Just<T>`; `Ok(x)` → `Just::new(x)`, `Err(e)` → `panic!(...)` |
| `Cold` | `Outcome<T, E>` always | Same |
| `Warm` | `Maybe<T>` | `Maybe<T>` |

`Hot` gets `#[inline]`. `Cold` and `Warm` do not.

## The `internal` feature belongs to your crate

`#[profile(Hot)]` emits a `cfg(feature = "internal")`, and a `cfg` written by
an attribute macro is read against the features of the crate it expanded into.
So the feature that decides which arm you get is **yours**, not this crate's,
and it has to be declared:

```toml
[features]
internal = []
```

Without that line the code still compiles and still behaves correctly, on the
`Outcome<T, E>` arm, but every `#[profile]` in the crate warns that `internal`
is not a value `feature` can take. Declaring it makes the warning go away, and
gets you the switch.

`internal` is the name `#[profile]` uses, and it is the default rather than a
fixture. If you're building your own attribute on
[`notko-macros-core`](https://crates.io/crates/notko-macros-core) it takes a
name of its own, which is what keeps two frameworks in one dependency graph
from sharing a switch neither of them can turn off separately.

Leave it off and `Hot` stays `Outcome<T, E>`, which is the arm a published api
wants: `Result`-family signatures, errors that can be handled. Turn it on in a
build with `debug_assertions` off and `Hot` becomes `Just<T>` with the error
arm panicking, which is the arm a binary wants when it has already decided the
error cannot happen.

## Installation

```bash
cargo add notko-macros
```

Or reach it through `notko` itself, which re-exports `#[profile]` at its root under the `macros`
feature:

```bash
cargo add notko --features macros
```

## Usage

```rust
use notko_macros::profile;

#[profile(Hot)]
pub fn lookup(id: u32) -> Result<u32, MyErr> {
    if id > 1000 { return Err(MyErr); }
    Ok(id * 2)
}

#[profile(Cold)]
pub fn init(cfg: &Config) -> Result<State, SetupErr> { /* ... */ }
```

## Custom tiers (crate-local)

Drop a file at `$CRATE_ROOT/notko-optimizers/<Name>.rs` with this shape:

```rust
//! @notko-optimizer
//! based_on = "Cold"
//! inline = false
//! panic_fmt = "trace invariant violated: {err:?}"
```

The proc-macro reads the file at expansion time (via the consumer's
`CARGO_MANIFEST_DIR`) and applies the named built-in strategy with the
tier-specific parameters. The filename carries the tier's own casing, so
`#[profile(Trace)]` looks for `Trace.rs` and nothing else. Custom tiers appear
alongside built-ins with no additional imports:

```rust
#[profile(Trace)]  // resolves via notko-optimizers/Trace.rs
pub fn some_work() -> Result<(), Err> { /* ... */ }
```

If you want one crate to define `Trace` and others in the dep tree to use it, that's what the
[`notko-build`](https://crates.io/crates/notko-build) companion crate is for.

## Writing your own attribute macro

If the built-in strategies don't cover what you want and you need real AST-level control,
[`notko-macros-core`](https://crates.io/crates/notko-macros-core) has the pieces this crate is built
from, under `notko_macros_core::{tiers, parse, discover, rewrite}`. Nothing stops you writing your own
attribute on top of it. That crate's README has the map.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
