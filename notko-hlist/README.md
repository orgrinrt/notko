# `notko-hlist`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-hlist)](https://crates.io/crates/notko-hlist)
[![docs.rs](https://img.shields.io/docsrs/notko-hlist)](https://docs.rs/notko-hlist)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> A heterogeneous type-level list, and the structural facts about one. Length in your own count type, membership, position, and append, all decided by the compiler.

</div>

A list here is `Empty`, or `Cons<H, T>` where the tail is itself a list, and neither of them exists at
run time: both are zero-sized markers and everything the crate says about a list it says through traits
the solver discharges. So `Cons<Db, Cons<Cache, Empty>>` is a set of things a function is allowed to
touch, or the axes of a shape, or the commands a shell knows, and a bound like `L: Contains<Db>` is the
compiler agreeing before anything runs.

Membership and position are two ways of asking after a member and they answer different questions.
`L: Contains<Db>` says the type is in there and keeps the depth out of the bound, so a list reordered
later breaks nobody. `At<P>` says which type is at a place, with the place written as `Here` or
`There<P>` and read back as a number through `Position`, which is what a consumer indexing a flat run
of values beside the list actually needs.

Do note that this is deliberately small. Length, membership, position and append are the structural
folds, the ones needing no algebra, and a value-level fold that reduces with an identity and a combine
is not here, because that is numerics territory and belongs where the algebra lives.

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

use notko_hlist::{At, Cardinal, Concat, Cons, Contains, ContainsAll, Empty, Here, Length, Position, There};

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

// Position, where the depth is the question rather than something to hide.
// `Cache` is the second store read, and the index is what you hand a run of
// values sitting beside the list.
type Second = There<Here>;
const SECOND: Count = <Second as Position<Count>>::INDEX;

// And the bound a caller actually writes. `Touched` holding `Log` is checked
// here, and passing a set that does not is a compile error rather than a
// lookup that fails later.
fn runs_against<Set: Contains<Log> + ContainsAll<Reads>>() {}

// The member at a position, named in the signature. Handed the wrong position
// this does not compile, which is the whole reason to index this way.
fn second_store<L: At<Second, Member = M>, M>(value: M) -> M {
    let _ = core::marker::PhantomData::<L>;
    value
}

fn main() {
    assert_eq!(READS, Count(2));
    assert_eq!(SECOND, Count(1));
    runs_against::<Touched>();
    let _: Cache = second_store::<Reads, Cache>(Cache);
}
```

The count being a parameter is the part worth explaining, since it looks like ceremony. This crate sits
under everything else, so it can't name a number type from a crate above it, and a counting crate above
it can't implement a trait and a type that are both foreign. Leaving the count to you is the one
arrangement the orphan rule allows, and it means the length comes back in the type you already count
with rather than in a `usize` you have to convert at every use.

## The traits are sealed

`List` is implemented for `Empty` and `Cons` and cannot be implemented for anything else, and
`Contains`, `ContainsAll`, `Length`, `At` and `Concat` all have it as a supertrait. So a type of your
own cannot claim to hold something, and `L: Contains<Db>` proves `Db` is in there instead of proving
that somebody wrote an empty impl saying so, which wouldn't be worth much as a guarantee.

A place is sealed the same way, through `Here` and `There`, and `Place` is the public name of that seal
because a consumer writing a signature generic over the position has to be able to spell the bound. Both
`At` and `Position` take it, and for `Position` the reason is the sharper version of the one above: a
position of your own could carry whatever index it liked, and a bound reading the count alone would index
wherever it was told. `At` needs it for a reason that is not obvious and cost a review to find. Its
parameter comes after `Self` and would otherwise be free, so a crate downstream of this one could fill it
with a type of its own and write `impl<H, T: List> At<Mine> for Cons<H, T>`, which the orphan rule allows
and which answers for every list in the graph. The bound is what refuses that, and
`tests/compile_fail/` holds the construction.

What it costs is bringing your own list type, which isn't really what the crate is for anyway: the
intended shape is aliasing the cell and the leaf into your own vocabulary, the way the example above
does, and that keeps them these two types.

## Features

Both are on by default and both need nightly. With the defaults off you get the list, `List`, `Concat`,
`At`, and `Length` and `Position` through their calls, which is what builds on stable back to 1.85.

| Feature | Adds | Unstable gate |
|---|---|---|
| `const` | `Cardinal` becomes a const trait, `Length` gains `LEN` and `Position` gains `INDEX`, both resolved at compile time | `const_trait_impl` |
| `membership` | `Contains` and `ContainsAll` | `marker_trait_attr` |

Without `const` the count is still there and still right, just computed rather than named. Without
`membership` there is no way to ask whether a list holds a type at all, and that one is not a
simplification we chose: the head match and the recursive tail match overlap by construction, and
`#[marker]` is how coherence gets told the overlap is intended. There is a membership shape that works
on stable, carrying an index witness through the bound, but it is a different surface rather than this
one weaker, so it isn't offered as a fallback. `At` is not that shape and is not a fallback for it: it
answers which member is at a place you name, where membership answers whether a type is in there at
all, and it needs no feature because its two impls do not overlap.

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
