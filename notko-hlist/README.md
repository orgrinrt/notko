# `notko-hlist`

[![Crates.io](https://img.shields.io/crates/v/notko-hlist)](https://crates.io/crates/notko-hlist) [![docs.rs](https://img.shields.io/docsrs/notko-hlist)](https://docs.rs/notko-hlist)

> A heterogeneous type-level list, and the structural facts about one. Length in a count type of the consumer's own, membership, position, and append, all decided by the compiler.

A list here is `Empty`, or `Cons<H, T>` where the tail is itself a list, and neither exists at run time: both are zero-sized markers, and everything the crate says about a list it says through traits the compiler discharges. So `Cons<Db, Cons<Cache, Empty>>` can be a set of things a function is allowed to touch, and a bound like `L: Contains<Db>` is the compiler agreeing before anything runs. `At<P>` goes the other way and says which type sits at a place, written as `Here` or `There<P>` and read back as a number through `Position`.

It's small on purpose, with length, membership, position and append and nothing that needs algebra. The count is a parameter of the traits, so the length comes back in whatever type the consumer already counts with. The `List` trait is sealed, so a type of your own can't claim to hold something, and `L: Contains<Db>` really proves `Db` is in there.

It has no dependencies, and it's pre-1.0 so the trait names may still move. The two features, `const` and `membership`, are both on by default and both need nightly. With them off it builds on stable from 1.85, minus `Contains`, `ContainsAll` and the const lengths.

## Usage

```bash
cargo add notko-hlist
```

```toml
[dependencies]
notko-hlist = "0.0.1"
```

On stable, turn the defaults off:

```toml
[dependencies]
notko-hlist = { version = "0.0.1", default-features = false }
```

## Example


```rust
// The default set needs nightly, and this is where that gets said.
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

// The count is the consumer's type, not one this crate picked. Implement the two Peano
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
// `Cache` is the second store read, and the index is what gets handed to a run of
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

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
