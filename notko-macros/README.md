# `notko-macros`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-macros)](https://crates.io/crates/notko-macros)
[![docs.rs](https://img.shields.io/docsrs/notko-macros)](https://docs.rs/notko-macros)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> An attribute macro that rewrites a `Result` function to one of `notko`'s three fallibility tiers. Ships `Hot`, `Warm` and `Cold`, and reads custom tiers from a file.

</div>

The attribute takes an ordinary `Result` function and rewrites both its signature and its body into the
tier named on the tag, so the choice then sits in one place instead of in every type spelled out inside
the function.

The tiers themselves live in [`notko`](https://crates.io/crates/notko), which defines them and is what
the rewritten body names, so that one belongs in the dependencies as well.

## What gets rewritten, and what doesn't

The rewrite fires on a return type spelled `Result<T, E>` or `Outcome<T, E>`, with both arguments written
out at that signature. Anything else is emitted exactly as written, with no error and no warning: a
unit return, a bare type, and in particular the `type Result<T> = core::result::Result<T, MyError>` alias
that most crates of any size end up with.

That last one is worth knowing before you spend an afternoon on it. A tag that appears to have done
nothing has usually done nothing, and writing the two arguments out at that one signature is the fix.

## Built-in tiers

| Tier | Debug, or the feature off | Release with the feature on |
|------|---------------------------|-----------------------------|
| `Hot` | `Outcome<T, E>`; `Ok(x)` to `Outcome::Ok(x)`, `Err(e)` to `Outcome::Err(e)` | `Just<T>`; `Ok(x)` to `Just::new(x)`, `Err(e)` to `panic!(...)` |
| `Cold` | `Outcome<T, E>`, with the same two rewrites | Same |
| `Warm` | `Maybe<T>`; `Ok(x)` to `Maybe::Is(x)`, `Err(e)` to `Maybe::Isnt` | Same |

`Hot` is the only one that moves between builds. `Cold` and `Warm` emit one function with no `cfg` on it,
identical everywhere, and the split above is there for `Hot`'s sake.

Two things the table can't carry on its own. `Warm` drops the whole `Err(..)` expression rather than
evaluating it and throwing the value away, so a side effect written in there goes with it. And `Hot`'s
release arm also collapses a `match` on a `Result` into its success arm with an `unwrap`, which is the
same bet the tier is already making, but it is a second rewrite rather than a consequence of the first.

`Hot` gets `#[inline]` and `Cold` and `Warm` do not. That's a property of the built-in markers rather
than of the strategies, since a custom tier file sets `inline` itself and can turn it either way.

## The `internal` feature belongs to your crate

`#[profile(Hot)]` emits a `cfg(feature = "internal")`, and a `cfg` written by an attribute macro is read
against the features of the crate it expanded into. So the feature that decides which arm you get is
yours, not this crate's, and it has to be declared:

```toml
[features]
internal = []
```

Without that line the code still compiles and still behaves correctly, on the `Outcome<T, E>` arm, but
every hot-strategy `#[profile]` in the crate warns that `internal` is not a value `feature` can take.
Declaring it silences the warning and turns the switch on. `Cold` and `Warm` emit no `cfg` at all,
so they neither warn nor switch.

Leave it off and `Hot` stays `Outcome<T, E>`, which is the arm a published api wants: `Result`-family
signatures, errors that can be handled. Turn it on in a build with `debug_assertions` off and `Hot`
becomes `Just<T>` with the error arm panicking, which is the arm a binary wants when it has already
decided the error cannot happen. Both halves are load-bearing, so the feature on its own in a debug build
changes nothing.

`internal` is the name `#[profile]` uses and it is fixed there. Neither the attribute nor a tier file
changes it. What does change it is a different attribute of your own built on
[`notko-macros-core`](https://crates.io/crates/notko-macros-core), which takes a name of its own, and
that's what keeps two frameworks in one dependency graph from sharing a switch neither of them can turn
off separately.

Do note this crate declares an `internal` feature too, and it is not that switch. It's there for this
crate's own tests and `src/` never reads it, so `cargo add notko-macros --features internal` does
nothing at all.

## Installation

```bash
cargo add notko-macros
```

Or reach it through `notko` itself, which re-exports `#[profile]` at its root under the `macros` feature:

```bash
cargo add notko --features macros
```

That second line leaves `notko`'s own defaults on, and those want a nightly compiler. On stable it's

```bash
cargo add notko --no-default-features --features macros
```

instead, and `notko`'s readme carries the rest of that story.

## Usage

```rust
use notko_macros::profile;

#[derive(Debug)]
pub struct LookupErr;

#[profile(Hot)]
pub fn lookup(id: u32) -> Result<u32, LookupErr> {
    if id > 1000 {
        return Err(LookupErr);
    }
    Ok(id * 2)
}

#[derive(Debug)]
pub struct SetupErr;

#[profile(Cold)]
pub fn init(retries: u32) -> Result<u32, SetupErr> {
    if retries == 0 {
        return Err(SetupErr);
    }
    Ok(retries)
}
```

Both want `notko` in scope of the crate they land in, since that's what the rewritten signatures name.

## Custom tiers (crate-local)

Drop a file at `$CRATE_ROOT/notko-optimisers/<Name>.rs` with this shape:

```rust
//! @notko-optimiser
//! based_on = "Hot"
//! inline = true
//! panic_fmt = "trace invariant violated: {err:?}"
```

`based_on` is the only key that has to be there, and it takes `Hot`, `Warm` or `Cold`, case-sensitively,
so a lowercase spelling doesn't match and fails the build. `inline` takes `true` or `false` and defaults
to whatever the named strategy does. `panic_fmt` is read on the hot strategy alone, since it's the
message the release arm's panic carries; on a `Warm` or `Cold` tier it parses fine and then sits there
doing nothing.

The proc-macro reads the file at expansion time, through the consuming crate's `CARGO_MANIFEST_DIR`, so
the tier belongs to whichever crate the attribute is written in. The filename carries the tier's own
casing, so `#[profile(Trace)]` looks for `Trace.rs`, and on a case-sensitive filesystem that's the only
thing it finds. Custom tiers appear alongside the built-ins with no additional imports:

```rust
#[profile(Trace)]  // resolves via notko-optimisers/Trace.rs
pub fn some_work() -> Result<(), WorkErr> { /* ... */ }
```

There's a third source as well, an accumulated directory named by `$NOTKO_OPTIMISERS_PATH`, consulted
after the two above. That's what [`notko-build`](https://crates.io/crates/notko-build) fills, for the
case where a tier is defined in one crate and used in another. It reaches the direct dependencies that
opted into it rather than the whole graph, and that crate's readme has the conditions.

## Writing your own attribute macro

If the built-in strategies don't cover what you want and you need real AST-level control,
[`notko-macros-core`](https://crates.io/crates/notko-macros-core) has the pieces this crate is built
from, under `notko_macros_core::{tiers, parse, discover, rewrite}`. Nothing stops you writing your own
attribute on top of it. That crate's readme points at the docs, which is where the surface actually is.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
