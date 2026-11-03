# `notko`

[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko) [![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)

> Fallibility primitives for `no_std` rust. Ships `Just`, `Maybe` and `Outcome`, and a `#[profile]` attribute for tagging whole functions.

Core's carriers differ by what they hold, `Option<T>` for absence and `Result<T, E>` for an error with a payload, and `notko`'s three differ by what a branch costs instead. `Just<T>` has no error case at all and is `#[repr(transparent)]` over the value, `Maybe<T>` handles ordinary absence, and `Outcome<T, E>` takes an error carrying data. Each keeps a matching api (same `?`, same combinators, largely the same method names), so moving a function across is usually a type change with the body left alone.

The `#[profile]` attribute takes an ordinary `Result` function and rewrites the signature and the body into one of the three, so the choice sits in one place per function. It's `#![no_std]`, no alloc, and in the default build no dependencies at all. The `macros` feature is the one exception, since it pulls in the proc-macro crate, which uses std and `syn` at compile time only.

It's early days and the api hasn't settled, so I'd not put it anywhere serious yet. The default feature set needs a nightly compiler, because the const-trait machinery it turns on isn't stable. On stable, turn the defaults off and it builds from 1.85 onwards, with the const paths absent.

## Usage

```bash
cargo add notko
```

On stable, and anywhere the const paths aren't wanted:

```bash
cargo add notko --no-default-features
```

The three carriers read much like the core ones they stand beside, with the names swapped for `notko`'s own:

```rust
use notko::{Just, Maybe, Outcome};

fn lookup(key: u32) -> Maybe<u32> {
    if key == 0 { Maybe::Isnt } else { Maybe::Is(key * 2) }
}

fn parse(bytes: &[u8]) -> Outcome<u32, &'static str> {
    if bytes.is_empty() { Outcome::Err("empty") } else { Outcome::Ok(42) }
}

// post-validation, so the error case cannot arise here
fn post_validated(value: u32) -> Just<u32> {
    Just::new(value)
}
```

With the `try_trait_v2` feature, `?` works on all three. It needs a nightly compiler, since that is what `notko` itself compiles under, but the consuming crate needs no feature gate of its own:

```rust
use notko::{Maybe, Outcome};

# fn parse(_: &[u8]) -> Outcome<u32, &'static str> { Outcome::Ok(1) }
# fn lookup(_: u32) -> Maybe<u32> { Maybe::Is(2) }
fn compose() -> Outcome<u32, &'static str> {
    let a = parse(b"foo")?;
    let b = lookup(a).ok_or("missing")?;
    Outcome::Ok(a + b)
}
```

`#[profile]` tags a function with a strategy and rewrites the body to the matching tier, so the source stays plain `Result` with `Ok` and `Err`. It needs `features = ["macros"]`:

```rust
use notko::profile;

#[derive(Debug)]
struct Oops;

// returns Outcome<u32, Oops> after expansion
#[profile(Hot)]
fn compute(x: u32) -> Result<u32, Oops> {
    Ok(x + 1)
}
```

The built-in strategies are `Hot`, `Warm` and `Cold`. `Hot` emits `Outcome<T, E>` in debug builds and `Just<T>` in release-internal builds, which the consumer opts into through its own `internal` feature. `Cold` always emits `Outcome`, and `Warm` always emits `Maybe<T>` and drops the whole `Err(..)` expression, side effects included. The rewrite only fires on a return type spelled `Result<T, E>` or `Outcome<T, E>` with both arguments written out, and anything else, like a `Result<T>` alias, is left alone without a word, so spelling the two arguments out is the fix when a tag seems to do nothing.

Third-party strategies live in a crate-local `notko-optimisers/<Name>.rs` with a `based_on = "Hot" | "Warm" | "Cold"` header, which is case-sensitive. A sibling proc-macro crate reusing `notko-macros-core` is the other route, and [`notko-macros`](https://crates.io/crates/notko-macros) has the details.

## Example

A small row parser of the kind that keeps turning up where there is no allocator. A single byte is a digit or it isn't, so that one is a `Maybe`, a whole field can fail for a reason worth reporting, so that one is an `Outcome`, and the lent storage answers with `Exhausted` when it runs out.

```rust
use notko::prelude::*;

#[derive(Debug, PartialEq)]
enum RowError {
    Empty,
    NotADigit(u8),
    TooMany(Exhausted),
}

// absence carries no reason here, so Maybe is the shape
fn digit(byte: u8) -> Maybe<u32> {
    if byte.is_ascii_digit() { Maybe::Is(u32::from(byte - b'0')) } else { Maybe::Isnt }
}

// the reason matters for a whole field, so this one is an Outcome
fn number(field: &[u8]) -> Outcome<u32, RowError> {
    if field.is_empty() {
        return Outcome::Err(RowError::Empty);
    }
    let mut acc = 0u32;
    for &byte in field {
        let d = digit(byte).ok_or(RowError::NotADigit(byte))?;  // Maybe lifts to Outcome
        acc = acc * 10 + d;
    }
    Outcome::Ok(acc)
}

// storage comes from the caller, and the part that got filled goes back
fn parse_row<'a>(line: &[u8], into: &'a mut [u32]) -> Outcome<&'a [u32], RowError> {
    let mut fill = Fill::new(into);
    for field in line.split(|&b| b == b',') {
        fill.push(number(field)?).map_err(RowError::TooMany)?;  // refuses, never truncates
    }
    Outcome::Ok(fill.finish())
}

let mut room = [0u32; 4];
assert_eq!(parse_row(b"12,7,300", &mut room), Outcome::Ok(&[12, 7, 300][..]));
assert_eq!(parse_row(b"1,x", &mut room), Outcome::Err(RowError::NotADigit(b'x')));
assert_eq!(
    parse_row(b"1,2,3,4,5", &mut room),
    Outcome::Err(RowError::TooMany(Exhausted { wanted: 5, had: 4 })),
);
```

## Features

There are four cargo features. `const` is on by default and turns `ConstTry`, `ConstFromResidual` and `HasTrivialCtor` into `const trait`s, which needs nightly. `try_trait_v2` makes `?` work on the three carriers, also nightly, and `macros` re-exports `#[profile]` from `notko-macros`. `all` turns on the three at once, and it's worth turning on somewhere that compiles it, like a consumer or a CI check, so gated code doesn't rot unnoticed.

Besides the three carriers there are boundary types, `MaybeNull` for FFI pointers with a guaranteed null bit pattern and `Boundable` for values bounded to a range, and the `Push`, `Emit` and `Lend` protocols for code that owns no allocator. The rustdoc covers them.

The other crates in this repository are [`notko-hlist`](https://crates.io/crates/notko-hlist), a type-level list, [`notko-macros`](https://crates.io/crates/notko-macros), where `#[profile]` lives, [`notko-macros-core`](https://crates.io/crates/notko-macros-core), the rewrite engine under it, and [`notko-build`](https://crates.io/crates/notko-build), a build-script helper for tiers shared between crates.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
