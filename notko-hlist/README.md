# `notko-hlist`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-hlist)](https://crates.io/crates/notko-hlist)
[![docs.rs](https://img.shields.io/docsrs/notko-hlist)](https://docs.rs/notko-hlist)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> A heterogeneous type-level list, and the structural facts about one. Length in your own count type, membership, and append, all decided by the compiler.

</div>

A list here is `Empty`, or `Cons<H, T>` where the tail is itself a list, and neither of them exists at
run time: both are zero-sized markers and everything the crate says about a list it says through traits
the solver discharges. So `Cons<Db, Cons<Cache, Empty>>` is a set of things a function is allowed to
touch, or the axes of a shape, or the commands a shell knows, and a bound like `L: Contains<Db>` is the
compiler agreeing before anything runs.

Do note that this is deliberately small. Length, membership and append are the structural folds, the
ones needing no algebra, and a value-level fold that reduces with an identity and a combine is not
here, because that is numerics territory and belongs where the algebra lives.

## Installation

```bash
cargo add notko-hlist
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
notko-hlist = "0.0.1"
```

It has no dependencies, not even on `notko` itself, and the default set needs nightly. On stable, turn
the defaults off:

```toml
[dependencies]
notko-hlist = { version = "0.0.1", default-features = false }
```

## Usage

```rust
// The default set needs nightly, and this is where you say so.
#![feature(const_trait_impl)]

use notko_hlist::{Cardinal, Concat, Cons, Contains, ContainsAll, Empty, Length};

// The names are meant to appear at the definition and almost nowhere else, so
// alias them into whatever the thing actually is.
type NoStores = Empty;
type Store<H, T> = Cons<H, T>;

struct Db;
struct Cache;
struct Log;

type Reads = Store<Db, Store<Cache, NoStores>>;
type Writes = Store<Log, NoStores>;

// The count is your type, not one this crate picked. Implement the two Peano
// constructors on it and every list has a length in it.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Count(usize);

const impl Cardinal for Count {
    const ZERO: Self = Count(0);
    fn succ(self) -> Self {
        Count(self.0 + 1)
    }
}

// Resolved by the compiler, so this is a constant and not a walk.
const READS: Count = <Reads as Length<Count>>::LEN;

// Composition: a unit declaring both sets ends up with everything either had.
type Touched = <Reads as Concat<Writes>>::Out;

// And the bound a caller actually writes. `Touched` holding `Log` is checked
// here, and passing a set that does not is a compile error rather than a
// lookup that fails later.
fn runs_against<Set: Contains<Log> + ContainsAll<Reads>>() {}

fn main() {
    assert_eq!(READS, Count(2));
    runs_against::<Touched>();
}
```

The count being a parameter is the part worth explaining, since it looks like ceremony. This crate sits
under everything else, so it can't name a number type from a crate above it, and a counting crate above
it can't implement a trait and a type that are both foreign. Leaving the count to you is the one
arrangement the orphan rule allows, and it means the length comes back in the type you already count
with rather than in a `usize` you have to convert at every use.

## The traits are sealed

`List` is implemented for `Empty` and `Cons` and cannot be implemented for anything else, and
`Contains`, `ContainsAll`, `Length` and `Concat` all have it as a supertrait. So a type of your own
cannot claim to hold something, and `L: Contains<Db>` proves `Db` is in there instead of proving that
somebody wrote an empty impl saying so. A membership witness anybody can forge isn't a witness.

What it costs is bringing your own list type, which isn't really what the crate is for anyway: the
intended shape is aliasing the cell and the leaf into your own vocabulary, the way the example above
does, and that keeps them these two types.

## Features

Both are on by default and both need nightly. With the defaults off you get the list, `List`, `Concat`,
and `Length` through `len()`, which is what builds on stable back to 1.85.

| Feature | Adds | Unstable gate |
|---|---|---|
| `const` | `Cardinal` becomes a const trait, and `Length` gains `LEN`, resolved at compile time | `const_trait_impl` |
| `membership` | `Contains` and `ContainsAll` | `marker_trait_attr` |

Without `const` the count is still there and still right, just computed rather than named. Without
`membership` there is no way to ask whether a list holds a type at all, and that one is not a
simplification we chose: the head match and the recursive tail match overlap by construction, and
`#[marker]` is how coherence gets told the overlap is intended. There is a shape that works on stable,
carrying a position index through the bound, but it is a different surface rather than this one weaker,
so it isn't offered as a fallback.

## Status

Under active development and pre-1.0, so the api hasn't settled and breaking changes should be
expected, though the shape of it has been steady for a while and I'd be surprised if the cell and the
leaf moved at all. It's the trait names around them I'd caution against leaning on for anything serious
just yet.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
