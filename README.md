# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Fallibility primitives for `no_std` rust. Ships `Just`, `Maybe` and `Outcome`, and a `#[profile]` attribute for tagging whole functions.

</div>

Core's carriers differ by what they hold, `Option<T>` for absence and `Result<T, E>` for an error with
a payload, and `notko`'s three differ by what a branch costs instead. `Just<T>` has no error case at
all and is `#[repr(transparent)]` over the value, so with `try_trait_v2` on there is nothing for `?` to
branch to. `Maybe<T>` handles ordinary absence, and `Outcome<T, E>` takes an error carrying data. Each
keeps a matching api on purpose (same `?`, same combinators, largely identical method names), so moving
a function across is usually a type change with the body left alone, although the guarantees are not
equal.

Do note that `Result<T, Infallible>` already gets you a good part of the way to `Just<T>`, since the
uninhabited error niches away and the branch is dead by construction. What you don't get from it is one
api across all three tiers, or the `#[profile]` attribute picking a tier per function, and those are
what this is actually for.

The `#[profile]` attribute takes an ordinary `Result` function and rewrites the signature and the body
into one of the three, so the choice sits in one place per function and the types inside follow from
the tier. Custom tiers are ordinary rust files a crate keeps in its own `notko-optimisers/` directory,
which `notko-build` gathers (its own, and the ones its direct dependencies opted into sharing) into one
place the proc-macro can read.

It's `#![no_std]`, no alloc, no platform deps, and in the default build no dependencies at all. The
`macros` feature is the one exception, since it pulls the proc-macro crate in and that one uses std and
`syn`. Only at compile time though, which is the only place a macro runs, so none of it lands in the
binary.

## Usage

```bash
cargo add notko
```

On stable, and anywhere the const paths aren't wanted:

```bash
cargo add notko --no-default-features
```

`cargo add` writes `notko = "0.0.1"`, and on a `0.0.x` version that already means that one and nothing
else, so what it wrote is the pin. Getting the next one means changing the number by hand, and reading
the log between the two tags before doing so.

The three carriers read much like the core ones they stand beside, with the names swapped for
`notko`'s own:

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

With the `try_trait_v2` feature, `?` works on all three. It needs a nightly compiler, since that is what
`notko` itself compiles under, but the consuming crate needs no feature gate of its own:

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

A `notko::prelude` ships as well, carrying the common surface in one import for the cases where naming
each of them gets tedious.

`#[profile]` tags a function with a strategy and rewrites the body to the matching tier, so the source
stays one ordinary `Result` surface and the tier decides what it becomes, which gives less control but
is more ergonomic for the usual cases. The authoring form is plain `Result` with `Ok` and `Err`, and the
macro rewrites both the signature and the body from there, so nothing in the source names a carrier at
all. It needs `features = ["macros"]`, which is off by default since it pulls in the proc-macro crate:

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

Built-in strategies are `Hot`, `Warm` and `Cold`, passed as idents. `Hot` emits `Outcome<T, E>` in debug
builds, so the error path stays observable; in release-internal builds, which the consumer opts into
through its own `internal` feature, it emits `Just<T>` with `Err` lowered to a panic. `Cold` always emits
`Outcome`. `Warm` rewrites to `Maybe<T>` in every build and drops the error, which is what choosing that
tier decides. Do note it drops the whole `Err(..)` expression rather than evaluating it and throwing the
value away, so anything with a side effect in there goes with it.

The rewrite only fires on a return type spelled `Result<T, E>` or `Outcome<T, E>` with both arguments
written out. Anything else, a bare type, a unit return, or the very common `type Result<T> =
core::result::Result<T, MyError>` alias, is emitted untouched and says nothing about it. So a tag that
appears to do nothing is usually that, and spelling the two arguments out is the fix.

Third-party strategies live in a crate-local `notko-optimisers/<Name>.rs` with a
`based_on = "Hot" | "Warm" | "Cold"` header. The `based_on` value is case-sensitive, so lowercase doesn't
match and fails the build. A sibling proc-macro crate reusing `notko-macros-core` is the other route. See
[`notko-macros`](https://crates.io/crates/notko-macros).

## Example

Here's a small row parser of the kind that keeps turning up in code with no allocator: a line of comma
separated numbers goes in, the numbers land in storage the caller lent, and the filled prefix comes back.
A single byte is a digit or it isn't, with no reason attached either way, so that one is a `Maybe`. A
whole field can fail for a reason worth reporting, so the tier goes up to `Outcome` there, and the
lent storage answers with `Exhausted` when it runs out, which says how much was wanted against how much
there was.

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

The `?` on the `Maybe` goes through `ok_or`, since a `Maybe` has no error to hand over and the conversion
has to be spelled, and the `?` on the `Outcome` goes straight through, same as it would on a `Result`.
The `Exhausted` at the end carries both numbers on purpose, because a failure that only says "did not
fit" leaves the caller guessing at how much bigger to try.

## Motivation

`Option<T>` is fine for most code and the discriminant it carries is rarely worth thinking about, but a
function handing one back says the value might be missing whether or not the caller already knew
better, so the check gets emitted and the optimiser only sometimes manages to remove it again (in an
inner loop, sometimes is not often enough). Whether any of it shows up in your own measurements is a
separate question, and depends much on what the surrounding code looks like.

So the cost gets picked per tier, where the call is written, and the type follows from that. `Just<T>`
carries no error variant, so it is the layout of the `T` and nothing more, and a `?` on it has no arm to
take once `try_trait_v2` is on. It belongs where the error case cannot arise, so post-validation paths,
codegen-reduced hot loops, and wrappers making a guarantee concrete.

`Maybe<T>` is the ordinary-absence case, and for a `T` shaped like a pointer (`&T`, `&mut T`,
`NonNull<T>`, every `NonZero*`, function pointers) rustc niche-fills the enum so the whole thing is the
size of `T`, meaning absence takes no extra storage in those cases, and compile-time size assertions in
`maybe.rs` pin that layout per supported shape so it cannot quietly regress.

`Outcome<T, E>` is the case where the error path carries data, and its layout is whatever `repr(Rust)`
decides, so an exact result layout across an FFI boundary wants the payload wrapped in a `#[repr(C)]`
struct of its own instead of leaning on this one.

`Just` and `Maybe` both iterate, through `JustIter` and `MaybeIter`. `Outcome` gets a `Default` of
`Ok(T::default())`, which is there so a trait can name a default without whoever writes it having to
make up an error value that never happens.

## Extras

### Status

Early days, so the api hasn't settled and the next release can move things out from under whatever was
written against this one. Every release is tagged and the log between two tags is what actually moved,
and we'll try to keep the migration notes worth reading. I'd caution against putting this anywhere
serious just yet.

The default feature set needs a nightly compiler, because the const-trait machinery it turns on isn't
stable yet. On stable, turn the defaults off and the crate builds and works from 1.85 onwards, with the
const paths absent. That's what the `rust-version` in the manifest is about, so it isn't the floor for
the default set, which has no floor on stable at all. Probably the main thing to know before adding it.

It sits on five unstable features, across the two optional sets. `try_trait_v2` and
`try_trait_v2_residual` carry the `?` operator for the three carriers; `const_trait_impl`,
`const_destruct` and `const_convert` carry the const paths, and the middle one is what lets a value be
dropped in a const context, which is why the const carriers can take ownership at all. All five are
still moving upstream. We've stayed off the ones with known soundness holes rather than working around
them, so the surface here is a bit smaller than what nightly would let us do. Might grow later, might
not.

### Cargo features

| Feature | Default | Effect |
|---|---|---|
| `const` | on | `ConstTry`, `ConstFromResidual` and `HasTrivialCtor` become `const trait`s. Needs nightly. |
| `try_trait_v2` | off | `core::ops::Try` for `Just` / `Maybe` / `Outcome`, so `?` works. Needs nightly. |
| `macros` | off | Re-exports `#[profile]` from `notko-macros` at the crate root. |
| `all` | off | All three at once. |

Without `try_trait_v2` the types still work and `?` is what goes missing, and on stable,
`default-features = false` leaves everything except the const paths, which then exist in plain
non-const form.

`all` is worth turning on somewhere that compiles it, a consumer or a CI check, because gated code
nobody builds is how an upstream change breaks a consumer without anyone noticing until much later.

### The other crates

[`notko-hlist`](https://crates.io/crates/notko-hlist) is a heterogeneous type-level list, `Empty` and
`Cons<H, T>`, with length, membership and append written as traits the compiler resolves rather than as
anything that runs. A cell holds nothing and can't be constructed at all, its one field being a phantom,
so what comes out of the list is a bound: `L: Contains<Db>` says the list holds a `Db`, and a function
asking for that can't be called with a list that doesn't. The `List` trait is sealed, because the
membership ones are `#[marker]` traits where an impl is a single empty line, so without the sealing a
bound like that would only prove somebody wrote that line. It doesn't come through `notko`, so depend on
it directly.

[`notko-macros`](https://crates.io/crates/notko-macros) is where `#[profile]` lives, and the `macros`
feature above is that same attribute re-exported from here. Depend on it directly for the attribute
without any of the carriers.

[`notko-macros-core`](https://crates.io/crates/notko-macros-core) is the rewrite engine underneath it, as
an ordinary library. It's there because a proc-macro crate can't export anything but macros, and it's
public so a third-party attribute can build on it rather than writing the rewrite again.

[`notko-build`](https://crates.io/crates/notko-build) is a build-script helper for the one case where a
tier is defined in one crate and used in another. Nothing else needs it.

### Boundary types

Around the three carriers there's a smaller set for the boundaries, where either the bytes or the value
are the contract: layout invariants for FFI, value invariants for bounded scalars.

At an `extern "C"` boundary the bytes are the contract and the compiler cannot help. `Option<T>`'s
niche-fill is a stable documented layout for the pointer-shaped payloads, but reading a signature and
knowing that only works if the reader already knows niche-fill is what guarantees it.

`MaybeNull<T: NicheFilled>` is that guarantee made syntactic. A `#[repr(transparent)]` newtype with a
guaranteed null bit pattern, where the sealed `NicheFilled` trait admits only types whose all-zeros
pattern is invalid: `&T`, `&mut T`, `NonNull<T>`, every `NonZero*`, and `extern` / `unsafe extern` / plain
/ `unsafe` fn pointers of arities zero through eight. `MaybeNull<u32>` does not compile, because `u32` has
no invalid pattern. `MaybeNull<&T>` does, and it lays out exactly like `Option<&T>` would, except now the
signature says so on its own, without any knowledge of niche-fill on the reader's side.

The cost is that the niche set is fixed at the language level, so extending it takes a `notko` release
rather than a downstream impl.

```rust
use notko::MaybeNull;

#[repr(C)]
pub struct ExtensionDescriptor {
    pub abi_version: u32,
    pub init_fn: MaybeNull<unsafe extern "C" fn(*mut core::ffi::c_void) -> u32>,
    pub shutdown_fn: MaybeNull<unsafe extern "C" fn(*mut core::ffi::c_void) -> u32>,
}

impl ExtensionDescriptor {
    pub const fn minimal(version: u32) -> Self {
        Self {
            abi_version: version,
            init_fn: MaybeNull::null(),
            shutdown_fn: MaybeNull::null(),
        }
    }
}
```

There's a `const` layout assertion forced for every shape `NicheFilled` admits, so if some future rustc
regresses niche-filling for one of them, the build breaks instead of the ABI. And since `NicheFilled` is
sealed and covers the pointer families at all three metadata kinds, that's the whole set, and not only
whichever ones we happened to write down.

`Boundable` declares that a type is bounded to `[MIN, MAX]`. Its `try_new` constructor returns
`Outcome<Self, BoundError<I>>`, and `BoundError` names whether the rejected value was `Below { value, min }`
or `Above { value, max }`. The bound is checked once at construction, so nothing downstream has to check
it again on every read.

`NonZeroable` declares that a type has a zero sentinel and a nonzero guarantee form. Combined with
`Slot<T>`, a `T: NonZeroable + NicheFilled` becomes a pointer-niche-shaped wrapper whose `Slot::NONE`
matches `T`'s invalid bit pattern.

`HasTrivialCtor` covers types constructible with no arguments, which is what lets a contract name a
default without knowing the concrete type.

`IteratorExt` and `PartialOrdExt` bridge `core::iter::Iterator::next` and
`core::cmp::PartialOrd::partial_cmp`, which return `Option`, to `Maybe` at the call site. See rustdoc.

### Lending and sinks

Two protocols that keep coming up in code that owns no allocator, and that every crate answers privately
and slightly differently until one of them is named.

`Push<T>` takes an item through an exclusive reference and cannot fail, with `BulkPush<T>` taking a whole
slice at once for the cases where that is cheaper. Between them they describe a collector somebody owns
and is filling. `Emit<T>` is the other direction: an item through a shared reference, fallibly, which is
what an installed destination looks like when it is a log, a port, a file or a channel.

`Lend<T>` covers storage a caller hands over to be filled. The lent slice becomes a `Fill`, which is the
cursor the filler writes through and which hands back the prefix that actually got written when it's
finished with. A write past the end fails with an `Exhausted` saying how much was wanted against how much
there was, and that last part is the reason any of it exists: a bit buffer, a row of edit distances, an
argument vector and a line being typed have nothing in common except that all four want to ask exactly
this, and a failure that only says "did not fit" leaves the caller guessing at how much bigger to try.

Do note that it does nothing at all for bounds checks. A prefix known to be no longer than its capacity
says nothing about whether some index is inside the part that actually got filled, so indexing stays
checked like on any other slice.

## Support

Feel free to contribute! If unsure about wasting work, the best practice is to throw in an issue describing what you'd do, and only then commit to writing a big PR, because chances are, it might not be something that belongs here. However, forks are always a valid choice and we'd encourage everyone to experiment and have their own takes on this. When doing this, do mind the license(s) though!

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
