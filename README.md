# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Pick branch cost per call site or per strategy tag. `no_std`, no alloc, repr-transparent FFI, zero deps.

</div>

`notko` puts the cost of an absent value at the call site rather than baking it into the type, like `Option<T>` does; a `#[profile]` attribute is the function-scoped form that rewrites the body to match the strategy tag.

Three tiers: `Just<T>` for proven-present (zero discriminant, repr-transparent, `?` compiles to nothing), `Maybe<T>` for ordinary absence (matches `Option<T>`'s niche-fill for pointer-shaped payloads, uses one word for presence), `Outcome<T, E>` for the full case where the error path carries data. `MaybeNull<T: NicheFilled>` carries them across FFI; the trait admits only types with a null bit pattern. `notko` is `#![no_std]`, no alloc, no platform deps; the optional proc-macro crate uses std at compile time only.

## Cost per call site

`Just<T>` is the proven-present case. `#[repr(transparent)]`, no discriminant, no branch. With the `try_trait_v2` feature, `?` on a `Just` compiles to literally nothing. Use it where an invariant proves the error variant unreachable: post-validation paths, codegen-reduced hot loops, wrappers that reify a guarantee.

`Maybe<T>` is the ordinary-absence case. One bit on the absent side; for pointer-shaped `T` (`&T`, `NonNull<T>`, every `NonZero*`, function pointers) Rust niche-fills the enum so the size matches `T`. Compile-time size assertions in `maybe.rs` pin the layout per supported shape.

`Outcome<T, E>` is the case where the error path carries data. Layout is platform-standard Rust repr; FFI-critical result layouts wrap in a dedicated `#[repr(C)]` struct, not the default.

## Strategy-driven rewrite

The `#[profile]` attribute lets you tag a function with a strategy and have the macro rewrite the body to the matching tier. Without it, you pick the type at every call site; with it, you write one consistent surface (`Maybe<T>` or `Outcome<T, E>`) and the strategy lowers it.

```rust
use notko::{profile, Outcome};

#[profile(Hot)]
fn compute(x: u32) -> Outcome<u32, Oops> {
    // Author writes plain Outcome / Ok / Err / ?. The macro rewrites
    // the body to match the chosen tier at expansion time.
    Outcome::Ok(x + 1)
}
```

Built-in strategies are `Hot`, `Warm`, `Cold` (ZST markers passed as idents). Debug builds get `Outcome<T, E>` regardless of tier so the error path stays observable; release-internal builds (the consumer opts in via its own `internal` feature) get `Just<T>` on `Hot` with `Err` lowered to `panic!`. `Warm` is passthrough. `Cold` always emits `Outcome`.

Third-party strategies live in a crate-local `notko-optimizers/<name>.rs` file with a `based_on = "hot" | "warm" | "cold"` header, or as a sibling proc-macro crate reusing `notko-macros-core`. See [`notko-macros`](https://github.com/orgrinrt/notko/tree/dev/notko-macros).

Enable the `macros` feature on `notko` to get `profile` re-exported at the crate root.

## Boundary types

Types that exist because something at the boundary forces a shape: layout invariants for FFI, value invariants for bounded scalars.

### Layout invariants

`MaybeNull<T: NicheFilled>` is a `#[repr(transparent)]` newtype with a guaranteed null bit pattern. The sealed `NicheFilled` trait restricts `T` to types where the all-zeros bit pattern is invalid: `&T`, `&mut T`, `NonNull<T>`, every `NonZero*`, and `extern` / `unsafe extern` / plain / `unsafe` `fn` pointers of arities zero through eight. A `MaybeNull<u32>` does not compile because `u32` has no invalid bit pattern; `MaybeNull<&T>` does, with the same layout `Option<&T>` would have.

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

Per-instantiation `const _LAYOUT_ASSERT` runs on every call site; the build fails if a future rustc ever regresses niche-filling for one of the supported shapes.

### Value invariants

`Boundable` declares "this type is bounded to `[MIN, MAX]`". A `Boundable::try_new` constructor returns `Outcome<Self, BoundError<I>>`; `BoundError` names whether the rejected value was `Below { value, min }` or `Above { value, max }`.

`NonZeroable` declares "this type has a zero sentinel and a nonzero guarantee form", the niche-fill marker. Combined with `Slot<T>`, a `T: NonZeroable + NicheFilled` becomes a pointer-niche-shaped wrapper whose `Slot::Isnt` matches `T`'s invalid bit pattern.

`IteratorExt` and `PartialOrdExt` bridge `core::Iterator::next` and `core::PartialOrd::partial_cmp` (which return `Option`) to `Maybe` at the call site; see rustdoc.

## Layout is the contract

At an `extern "C"` (or any platform ABI) boundary, the compiler can't help you: the bytes are the contract. `Option<T>`'s niche-fill is a documented stable layout for `Option<&T>`, `Option<NonNull<T>>`, `Option<NonZero*>`, and `Option<fn>`, but it relies on a reader knowing that niche-fill is what guarantees the layout.

`MaybeNull` is the same guarantee made syntactically explicit. The `NicheFilled` trait is sealed; the supported set of payload types is enumerated; the build fails if you try to instantiate `MaybeNull<u32>`. A reader does not need to know about niche-fill to know `MaybeNull<&T>` lays out as a single null-or-not pointer; the sealed trait makes the intent legible at the type signature.

The cost is small: the niche set is fixed at the language level, so consumers who want to extend it (a new sealed trait impl) need a `notko` release. The benefit is that an FFI descriptor full of `MaybeNull<fn>` slots tells you exactly what shipped, and the compiler refuses any `MaybeNull<usize>` mistake at the boundary.

`Boundable` and `NonZeroable` carry the same idea in a different domain. A value with a known range or a known sentinel can carry that fact in its type, and consumers can rely on the bound at construction rather than checking at every read.

## Installation

```bash
cargo add notko
```

Or in `Cargo.toml`:

```toml
[dependencies]
notko = "0.1"
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

// Post-validation: invariant proves the value is present.
fn post_validated(value: u32) -> Just<u32> {
    Just::new(value)
}
```

With the `try_trait_v2` feature on nightly, `?` works on all three:

```rust
#![feature(try_trait_v2)]
use notko::{Just, Maybe, Outcome};

fn compose() -> Outcome<u32, &'static str> {
    let a = parse(b"foo")?;
    let b = lookup(a).ok_or("missing")?;
    Outcome::Ok(a + b)
}
```

## Status & features

`notko` is on `0.1.x`; the API is stable enough for downstream use, but several pieces gate on unstable rustc features (`adt_const_params`, `try_trait_v2`, `const_trait_impl`). `notko` tracks them as they mature; features known to have soundness issues are intentionally skipped.

| Feature | Default | Effect |
|---|---|---|
| `const` | on | Enable const-trait machinery (`ConstTry`, `ConstFromResidual`, `Slot`'s const inherent methods). Requires nightly. Disable via `default-features = false` on stable. |
| `try_trait_v2` | off | Impl `core::ops::Try` for `Just` / `Maybe` / `Outcome`. Requires nightly. |
| `macros` | off | Re-export `#[profile]` from `notko-macros` at the crate root. |

Without `try_trait_v2` the types still work; only the `?` operator is unavailable.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
