# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Fallibility primitives whose branch cost is chosen at the call site rather than fixed by the type. `no_std`, no alloc, zero deps.

</div>

`Option<T>` has already decided for you what absence costs: a discriminant and a branch, everywhere,
whether or not you need them. Usually that's fine and you'd never notice. But when an invariant has
already proved the value is there, you're still paying for a check nobody needs, and there's no way to
say so in the type. So we ship three of them instead, one for proven-present, one for ordinary absence,
one that carries an error, and you pick per call site. There's a `#[profile]` attribute further down
too, for when you'd rather tag a whole function than choose at every site.

It's `#![no_std]`, no alloc, no platform deps. The proc-macro crate is the exception and does use std,
but only at compile time, which is the only place a macro ever runs, so none of it reaches your binary.

## Status

Early days, so the api hasn't settled and a `0.0.x` bump can move it under you. Every release is tagged
and the log between two tags is what actually moved, and we'll try to keep the migration notes worth
reading. I'd pin an exact version, not a range, and I'd caution against putting this anywhere serious
just yet.

The default feature set needs a nightly compiler, because the const-trait machinery it turns on isn't
stable yet. On stable, turn the defaults off and the crate builds and works, with the const paths
absent. Probably the main thing to know before you add it.

It sits on three unstable features, `try_trait_v2`, `try_trait_v2_residual` and `const_trait_impl`, and
all three are still moving upstream. We've stayed off the ones with known soundness holes rather than
working around them, so the surface here is a bit smaller than what nightly would let us do. Might grow
later, might not.

## Installation

```bash
cargo add notko
```

On stable Rust, and anywhere the const paths aren't wanted:

```bash
cargo add notko --no-default-features
```

Worth knowing that `0.0.x` releases are all mutually incompatible by semver's own rules, so a caret
range here buys you a break, not a fix. Pin the exact one.

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

`Just<T>` is the proven-present case. `#[repr(transparent)]`, no discriminant, no branch, and with
`try_trait_v2` a `?` on it compiles to nothing at all. Reach for it where an invariant proves the error
variant unreachable: post-validation paths, codegen-reduced hot loops, wrappers that make a guarantee
concrete.

`Maybe<T>` is the ordinary-absence case, and for pointer-shaped `T` (`&T`, `NonNull<T>`, every `NonZero*`,
function pointers) Rust niche-fills the enum so the whole thing is the size of `T`. Absence costs no extra
storage in those cases. Compile-time size assertions in `maybe.rs` pin the layout per supported shape.

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
`Outcome`. `Warm` is passthrough in every build and preserves the source `Result<T, E>` signature.

Third-party strategies live in a crate-local `notko-optimizers/<Name>.rs` with a
`based_on = "Hot" | "Warm" | "Cold"` header. The `based_on` value is case-sensitive, so lowercase doesn't
match and fails the build. A sibling proc-macro crate reusing `notko-macros-core` is the other route. See
[`notko-macros`](https://crates.io/crates/notko-macros).

Enable the `macros` feature to get `profile` re-exported at `notko`'s root.

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
