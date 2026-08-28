# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Fallibility primitives for `no_std` rust, in three tiers, so what a branch costs is something you pick per call site rather than something the type already decided. Ships `Just`, `Maybe` and `Outcome`, and a `#[profile]` attribute for tagging a whole function at once.

</div>

Three carriers instead of the one. `Just<T>` for when an invariant has already proved the value is
there, `Maybe<T>` for ordinary absence, and `Outcome<T, E>` for when the error carries something with
it. The idea is that you pick per call site, instead of taking whatever a single type settled on for
the whole program.

The case for splitting them is fairly narrow and I'd rather state it plainly than oversell it.
`Option<T>` fixes what absence costs at a discriminant and a branch, everywhere, and most of the time
that's completely fine and nobody notices. Where the value has been proved present already the check
still gets emitted, and there's nothing in the type that can say otherwise. `Just<T>` is what's missing
there: `#[repr(transparent)]`, and with `try_trait_v2` its `?` has no error arm to branch to.

There's also a `#[profile]` attribute further down, if you'd rather tag a function once than pick a
type at every site inside it.

It's `#![no_std]`, no alloc, no platform deps, and in the default build no dependencies at all. The
`macros` feature is the one exception, since it pulls the proc-macro crate in and that one uses std and
`syn`. Only at compile time though, which is the only place a macro runs, so none of it lands in your
binary.

## Status

Early days, so the api hasn't settled and the next release can move things out from under whatever you
wrote against this one. Every release is tagged and the log between two tags is what actually moved, and
we'll try to keep the migration notes worth reading. I'd caution against putting this anywhere serious
just yet.

The default feature set needs a nightly compiler, because the const-trait machinery it turns on isn't
stable yet. On stable, turn the defaults off and the crate builds and works from 1.85
onwards, with the const paths absent. That's what the `rust-version` in the manifest is about, so don't
read it as the floor for the default set, which has no floor on stable at all. Probably the main thing
to know before you add it.

It sits on five unstable features, across the two optional sets. `try_trait_v2` and
`try_trait_v2_residual` carry the `?` operator for the three carriers; `const_trait_impl`,
`const_destruct` and `const_convert` carry the const paths, and the middle one is what lets a value be
dropped in a const context, which is why the const carriers can take ownership at all. All five are
still moving upstream. We've stayed off the ones with known soundness holes rather than working around
them, so the surface here is a bit smaller than what nightly would let us do. Might grow later, might
not.

## Installation

```bash
cargo add notko
```

On stable Rust, and anywhere the const paths aren't wanted:

```bash
cargo add notko --no-default-features
```

`cargo add` writes `notko = "0.0.1"`, and on a `0.0.x` version that already means that one and nothing
else, so there's nothing further to pin. Getting the next one is you changing the number yourself, and
reading the log between the two tags before you do.

## Usage

```rust
use notko::{Just, Maybe, Outcome};

fn lookup(key: u32) -> Maybe<u32> {
    if key == 0 { Maybe::Isnt } else { Maybe::Is(key * 2) }
}

fn parse(bytes: &[u8]) -> Outcome<u32, &'static str> {
    if bytes.is_empty() { Outcome::Err("empty") } else { Outcome::Ok(42) }
}

// post-validation: an invariant already proved the value present
fn post_validated(value: u32) -> Just<u32> {
    Just::new(value)
}
```

With the `try_trait_v2` feature, `?` works on all three. It needs a nightly compiler, since that is what
`notko` itself compiles under, but your own crate needs no feature gate of its own:

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

There's a `notko::prelude` too, if you'd rather pull the common surface in one import.

## Cost per call site

`Just<T>` is the proven-present case. `#[repr(transparent)]`, so it's the layout of the `T` and nothing
more, and with `try_trait_v2` a `?` on it has no error arm to branch to. Reach for it where an invariant
proves the error variant unreachable: post-validation paths, codegen-reduced hot loops, wrappers that
make a guarantee concrete.

`Maybe<T>` is the ordinary-absence case, and for pointer-shaped `T` (`&T`, `&mut T`, `NonNull<T>`, every
`NonZero*`, function pointers) Rust niche-fills the enum so the whole thing is the size of `T`. Absence
costs no extra storage in those cases. Compile-time size assertions in `maybe.rs` pin the layout per
supported shape.

`Outcome<T, E>` is the case where the error path carries data. Its layout is ordinary Rust repr, so if
you need an exact result layout across an FFI boundary, wrap the payload in your own `#[repr(C)]` struct
instead of leaning on this one.

`Just` and `Maybe` both iterate, through `JustIter` and `MaybeIter`. `Outcome` gets a `Default` of
`Ok(T::default())`, which is there so a trait can name a default without whoever writes it having to
make up an error value that never happens.

## Strategy-driven rewrite

`#[profile]` tags a function with a strategy and rewrites the body to the matching tier. Without it
you're picking the type at every call site yourself, which gets old. With it you write one ordinary
`Result` surface and the macro rewrites it for you.

The authoring form is plain `Result` with `Ok` and `Err`. The macro rewrites the signature and the body.
This one needs `features = ["macros"]`, which is off by default:

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

Enable the `macros` feature to get `profile` re-exported at `notko`'s root.

## The other three crates

[`notko-macros`](https://crates.io/crates/notko-macros) is where `#[profile]` lives, and the `macros`
feature above is that same attribute re-exported from here. Depend on it directly if you want the
attribute and none of the carriers.

[`notko-macros-core`](https://crates.io/crates/notko-macros-core) is the rewrite engine underneath it, as
an ordinary library. It's there because a proc-macro crate can't export anything but macros, and it's
public so a third-party attribute can build on it rather than writing the rewrite again.

[`notko-build`](https://crates.io/crates/notko-build) is a build-script helper for the one case where a
tier is defined in one crate and used in another. Nothing else needs it.

## Boundary types

Types that exist because something at a boundary forces a shape: layout invariants for FFI, value
invariants for bounded scalars.

### Layout invariants

At an `extern "C"` boundary the bytes are the contract and the compiler cannot help. `Option<T>`'s
niche-fill is a stable documented layout for the pointer-shaped payloads, but reading a signature and
knowing that only works if you already know niche-fill is what guarantees it.

`MaybeNull<T: NicheFilled>` is that guarantee made syntactic. A `#[repr(transparent)]` newtype with a
guaranteed null bit pattern, where the sealed `NicheFilled` trait admits only types whose all-zeros
pattern is invalid: `&T`, `&mut T`, `NonNull<T>`, every `NonZero*`, and `extern` / `unsafe extern` / plain
/ `unsafe` fn pointers of arities zero through eight. `MaybeNull<u32>` does not compile, because `u32` has
no invalid pattern. `MaybeNull<&T>` does, and it lays out exactly like `Option<&T>` would, except now you
can see that from the signature without knowing anything about niche-fill.

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
sealed and covers the pointer families at all three metadata kinds, that's the whole set, not just
whichever ones we happened to write down.

### Value invariants

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

## Handing data across a boundary

Two protocols that keep coming up in code that owns no allocator, and that every crate answers privately
and slightly differently until one of them is named.

`Push<T>` takes an item through an exclusive reference and cannot fail, with `BulkPush<T>` taking a whole
slice at once for the cases where that is cheaper. Between them they describe a collector somebody owns
and is filling. `Emit<T>` is the other direction: an item through a shared reference, fallibly, which is
what an installed destination looks like when it is a log, a port, a file or a channel.

`Lend<T>` covers storage a caller hands over to be filled. The lent slice becomes a `Fill`, which is the
cursor the filler writes through and which hands back the prefix that actually got written when it's
finished with. A write past the end fails with an `Exhausted` saying how much was wanted against how much
there was, and that last part is the reason any of it exists: a bit
buffer, a row of edit distances, an argument vector and a line being typed have nothing in common except
that all four want to ask exactly this, and a failure that only says "did not fit" leaves the caller
guessing at how much bigger to try.

Do note it buys you nothing on bounds checks. A prefix known to be no longer than its capacity says
nothing about whether some index is inside the part that got filled, so indexing is checked like any
other slice.

## Cargo features

| Feature | Default | Effect |
|---|---|---|
| `const` | on | `ConstTry`, `ConstFromResidual` and `HasTrivialCtor` become `const trait`s. Needs nightly. |
| `try_trait_v2` | off | `core::ops::Try` for `Just` / `Maybe` / `Outcome`, so `?` works. Needs nightly. |
| `macros` | off | Re-exports `#[profile]` from `notko-macros` at the crate root. |
| `all` | off | All three at once. |

Without `try_trait_v2` the types all still work, you just don't get `?`. And on stable,
`default-features = false` gets you everything except the const paths, which then exist in plain
non-const form.

`all` is worth turning on somewhere that actually compiles, a consumer or a CI check, because gated code
nobody builds is how an upstream change breaks you without anyone noticing until much later.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
