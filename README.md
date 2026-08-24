# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Fallibility primitives whose branch cost is chosen at the call site rather than fixed by the type. `no_std`, no alloc, zero deps.

</div>

`Option<T>` decides for you that absence costs a discriminant and a branch. `notko` moves that decision to
the call site: three types covering proven-present, ordinary absence, and a data-carrying error path, so a
value whose presence an invariant already guarantees stops paying for a check nobody needs. A `#[profile]`
attribute is the function-scoped form, rewriting a body to the tier you tag it with.

`notko` is `#![no_std]`, no alloc, no platform dependencies. The optional proc-macro crate uses std at
compile time only, which is where a macro runs and not where its output lands.

## Status

Early, so the api hasn't settled and a `0.0.x` bump can move it. Every release is tagged, and the log
between two tags is what actually moved. I'd pin an exact version rather than a range for now.

The default feature set needs a nightly compiler, because the const-trait machinery it turns on isn't
stable yet. On stable, turn the defaults off and the crate builds and works, with the const paths absent.
That is the one thing worth knowing before adding it.

The three unstable features it sits on, `try_trait_v2`, `try_trait_v2_residual` and `const_trait_impl`,
are all still moving upstream. Anything with a known soundness hole is left alone rather than worked around, so the surface here
is smaller than what nightly would allow.

## Installation

```bash
cargo add notko
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
notko = "0.0.1"
```

On stable Rust, and anywhere the const paths aren't wanted:

```toml
[dependencies]
notko = { version = "0.0.1", default-features = false }
```

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

With the `try_trait_v2` feature, `?` works on all three. The feature needs a nightly compiler for `notko`
itself; your own crate needs no feature gate of its own:

```rust
use notko::{Maybe, Outcome};

fn compose() -> Outcome<u32, &'static str> {
    let a = parse(b"foo")?;
    let b = lookup(a).ok_or("missing")?;
    Outcome::Ok(a + b)
}
```

`notko::prelude` re-exports the common surface in one import.

## Cost per call site

`Just<T>` is the proven-present case. `#[repr(transparent)]`, no discriminant, no branch, and with
`try_trait_v2` a `?` on it compiles to nothing at all. Reach for it where an invariant proves the error
variant unreachable: post-validation paths, codegen-reduced hot loops, wrappers that reify a guarantee.

`Maybe<T>` is the ordinary-absence case, and for pointer-shaped `T` (`&T`, `NonNull<T>`, every `NonZero*`,
function pointers) Rust niche-fills the enum so the whole thing is the size of `T`. Absence costs no extra
storage in those cases. Compile-time size assertions in `maybe.rs` pin the layout per supported shape.

`Outcome<T, E>` is the case where the error path carries data. Its layout is ordinary Rust repr; an
FFI-critical result layout wraps in a dedicated `#[repr(C)]` struct rather than relying on the default.

`Just` and `Maybe` both iterate, through `JustIter` and `MaybeIter`. `Outcome` implements `Default` as
`Ok(T::default())`, which exists so a contract can declare a default without its owner having to invent an
error value.

## Strategy-driven rewrite

`#[profile]` tags a function with a strategy and rewrites the body to the matching tier. Without it you
pick the type at every call site; with it you write one ordinary surface and the strategy lowers it.

The authoring form is plain `Result` with `Ok` and `Err`. The macro rewrites the signature and the body:

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

Third-party strategies live in a crate-local `notko-optimizers/<name>.rs` with a
`based_on = "Hot" | "Warm" | "Cold"` header. The value is case-sensitive; lowercase does not match and
fails the build. A sibling proc-macro crate reusing `notko-macros-core` is the other route. See
[`notko-macros`](https://github.com/orgrinrt/notko/tree/main/notko-macros).

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
no invalid pattern. `MaybeNull<&T>` does, laid out exactly as `Option<&T>` would be, and a reader needs no
knowledge of niche-fill to see it.

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

A `const` layout assertion is forced for every shape `NicheFilled` admits, so the build fails if a future
rustc ever regresses niche-filling for one of them. `NicheFilled` is sealed and its pointer families are
covered at all three metadata kinds, which is what makes that a claim about the whole set rather than about
the members somebody happened to list.

### Value invariants

`Boundable` declares that a type is bounded to `[MIN, MAX]`. Its `try_new` constructor returns
`Outcome<Self, BoundError<I>>`, and `BoundError` names whether the rejected value was `Below { value, min }`
or `Above { value, max }`. The bound is checked at construction, so consumers rely on it rather than
re-checking at every read.

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
| `const` | on | Declares `ConstTry`, `ConstFromResidual` and `HasTrivialCtor` as `const trait`s. Requires nightly. Turn off with `default-features = false` to build on stable; the traits then exist in plain form. |
| `try_trait_v2` | off | Impl `core::ops::Try` for `Just` / `Maybe` / `Outcome`, enabling `?`. Requires nightly. |
| `macros` | off | Re-export `#[profile]` from `notko-macros` at the crate root. |
| `all` | off | Every pathway at once: `const`, `macros` and `try_trait_v2`. Worth enabling somewhere that actually compiles, so the gated `Try` impls are exercised rather than sitting dormant, since dormant gated code is how an upstream API change breaks a consumer unnoticed. |

Without `try_trait_v2` the types still work; only `?` is unavailable.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
