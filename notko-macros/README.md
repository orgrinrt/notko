# `notko-macros`

[![Crates.io](https://img.shields.io/crates/v/notko-macros)](https://crates.io/crates/notko-macros) [![docs.rs](https://img.shields.io/docsrs/notko-macros)](https://docs.rs/notko-macros)

> An attribute macro that rewrites a `Result` function to one of `notko`'s three fallibility tiers. Ships `Hot`, `Warm` and `Cold`, and reads custom tiers from a file.

The `#[profile]` attribute takes an ordinary `Result` function and rewrites both its signature and its body into the tier named on the tag, so the choice sits in one place instead of in every type spelled out inside the function. The tiers themselves live in [`notko`](https://crates.io/crates/notko), which is what the rewritten body names, so that belongs in the dependencies as well.

## Usage

```bash
cargo add notko-macros
```

Or reach it through `notko`, which re-exports `#[profile]` under the `macros` feature:

```bash
cargo add notko --features macros
```

That leaves `notko`'s defaults on, which want a nightly compiler. On stable it's `cargo add notko --no-default-features --features macros`.

## Example

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

`Hot` becomes `Outcome<T, E>` in debug builds and `Just<T>` in a release build with the consumer's own `internal` feature on, where `Err` turns into a panic. `Cold` is always `Outcome<T, E>`, and `Warm` is always `Maybe<T>` with the whole `Err(..)` expression dropped, side effects included. The `internal` feature has to be declared in the consuming crate, with `internal = []` under `[features]`, since the `cfg` is read where the attribute expands. Without it everything still compiles on the `Outcome` arm, but each hot `#[profile]` warns.

The rewrite only fires on a return type spelled `Result<T, E>` or `Outcome<T, E>` with both arguments written out. Anything else, like a `Result<T>` alias, is emitted as written without an error, so writing the two arguments out is the fix when a tag seems to do nothing.

Custom tiers are a file at `notko-optimisers/<Name>.rs` in the consuming crate, with the same shape as above: `//! @notko-optimiser`, then `based_on` (one of `Hot`, `Warm` or `Cold`, case-sensitive), and optionally `inline` and `panic_fmt`, the second read on the hot strategy only. `#[profile(Trace)]` then looks for `Trace.rs`. [`notko-build`](https://crates.io/crates/notko-build) fills a third source for tiers shared between crates, and [`notko-macros-core`](https://crates.io/crates/notko-macros-core) has the pieces for writing a separate attribute of your own.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
