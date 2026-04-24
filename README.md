# `notko`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko)](https://crates.io/crates/notko)
[![docs.rs](https://img.shields.io/docsrs/notko)](https://docs.rs/notko)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> Foundation primitives for the hilavitkutin stack. Three tiers of fallibility, two traits for bounded scalars, zero dependencies.

</div>

## What it is

`notko` ships the scalar-level vocabulary the [hilavitkutin](https://github.com/orgrinrt/hilavitkutin) stack uses in place of `core::option::Option`, `core::result::Result`, and bare integer primitives. `#![no_std]`, no alloc, no platform deps. Every downstream crate in the stack (`arvo`, `hilavitkutin`, `clause`) depends on this one.

The core idea: control flow has tiers. A value that is proven present should not pay branch cost. A value whose absence is ordinary should carry one bit. A value whose absence needs explanation should carry the full error payload. `notko` names those three tiers as distinct types so the compiler can pick the right shape per call site.

Function-pointer slots in FFI descriptors get their own layer: `MaybeNull<T: NicheFilled>` is a `#[repr(transparent)]` newtype that guarantees pointer-niche layout, replacing `Option<fn>` at `extern "C"` boundaries without giving up the null-as-absent bit pattern.

## Contents

| Type / trait | Purpose |
|---|---|
| `Just<T>` | Infallible value wrapper. `Try` with `Residual = Infallible`; `?` compiles to nothing. |
| `Maybe<T>` | Presence primitive. Replaces `Option<T>` in stack APIs. Niche-filled for pointer-shaped `T`. |
| `MaybeNull<T: NicheFilled>` | `#[repr(transparent)]` wrapper for FFI function-pointer and non-zero payloads. Sealed trait closes the set. |
| `NicheFilled` | Sealed marker for payloads with an all-zeros invalid bit pattern (references, `NonNull`, `NonZero*`, `fn` pointers of arity 0..=8). |
| `Outcome<T, E>` | Fallible result. Replaces `Result<T, E>` in stack APIs. |
| `Boundable` | Trait: "this type is bounded to `[MIN, MAX]`". Arvo impls it on `UFixed` / `IFixed`. |
| `NonZeroable` | Trait: "this type has a zero sentinel and a nonzero guarantee form". Arvo impls it on `UFixed` / `IFixed`. |

## Three tiers of fallibility

`Just` / `Maybe` / `Outcome` are the control-flow analog of arvo's numeric Hot / Warm / Cold / Precise strategy markers. Each tier has a distinct cold-path cost.

| Tier | Type | Cold path | When to use |
|---|---|---|---|
| **Hot** | `Just<T>` | No branch. | Value proven present. `?` compiles away. |
| **Warm** | `Maybe<T>` | Absent-variant discriminant, no payload. | Absence is ordinary, not exceptional. |
| **Cold** | `Outcome<T, E>` | Full error payload plus branch. | Caller needs to know *why* on failure. |

`Just<T>` is `#[repr(transparent)]`. No discriminant, no branch, no runtime cost. With the `try_trait_v2` feature, `?` on a `Just` compiles to literally nothing. Use it where an invariant proves the error variant unreachable: post-validation, codegen-reduced hot paths, reified-guarantee wrappers.

`Maybe<T>` carries a one-bit discriminant and no payload on the absent side. For pointer-shaped `T` (references, `NonNull`, `NonZero*`, function pointers) the Rust compiler niche-fills the enum, so `Maybe<&T>`, `Maybe<NonZeroU32>`, `Maybe<fn()>`, and similar shapes are the same size as `T` itself, with `Maybe::Isnt` represented by `T`'s invalid bit pattern. Compile-time size assertions in `maybe.rs` pin the layout for each supported shape.

`Outcome<T, E>` is the full two-payload tagged union. Layout is platform-standard Rust repr; consumers that need a specific C ABI result layout wrap the payload in a dedicated `#[repr(C)]` struct rather than relying on the default.

## `MaybeNull<T: NicheFilled>` for FFI positions

FFI descriptors need nullable function pointers with a guaranteed null-bit-pattern layout. `Option<fn>` carries that guarantee in `core`, but it is the wrong vocabulary for the rest of the stack. `MaybeNull<T: NicheFilled>` is the `#[repr(transparent)]` newtype that carries the same guarantee behind the stack's own type name.

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

The sealed `NicheFilled` trait admits only payload types whose invalid bit pattern is all zeros: `&T`, `&mut T`, `NonNull<T>`, every `NonZero*`, and `extern` / `unsafe extern` / plain / `unsafe` `fn` pointers of arities 0..=8. A `MaybeNull<u32>` does not compile, because `u32` has no invalid bit pattern. Per-instantiation `const _LAYOUT_ASSERT` runs on every call site, breaking the build if a future rustc ever regresses niche-filling for one of the supported shapes.

## Installation

```bash
cargo add notko
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
notko = "0.1"
```

As part of the hilavitkutin stack, `notko` is depended on transitively through `arvo` and does not usually need an explicit entry; add it directly when you want `Just` / `Maybe` / `Outcome` / `MaybeNull` in a crate that does not already pull in `arvo`.

## Usage

```rust
use notko::{Just, Maybe, Outcome};

fn lookup(key: u32) -> Maybe<u32> {
    if key == 0 { Maybe::Isnt } else { Maybe::Is(key * 2) }
}

fn parse(bytes: &[u8]) -> Outcome<u32, &'static str> {
    if bytes.is_empty() { Outcome::Err("empty") } else { Outcome::Ok(42) }
}

fn post_validated(value: u32) -> Just<u32> {
    Just(value)
}
```

With the `try_trait_v2` feature enabled on nightly, the `?` operator works on all three:

```rust
#![feature(try_trait_v2)]
use notko::{Just, Maybe, Outcome};

fn compose() -> Outcome<u32, &'static str> {
    let a = parse(b"foo")?;
    let b = lookup(a).ok_or("missing")?;
    Outcome::Ok(a + b)
}
```

See the [companion proc-macro crate `notko-macros`](https://github.com/orgrinrt/notko/tree/dev/notko-macros) for the `#[optimize_for(hot | warm | cold)]` attribute, which rewrites function return types between builds: `Outcome<T, E>` in debug and standalone consumers, `Just<T>` in internal-release builds where invariants are proven by construction.

## Cargo features

| Feature | Default | Effect |
|---|---|---|
| `try_trait_v2` | off | Impl `core::ops::Try` for `Just` / `Maybe` / `Outcome`. Requires nightly. |

Without `try_trait_v2` the types still work; only the `?` operator is unavailable.

## Positioning

```
notko  (zero deps)
  ↑
  ├── arvo  (L0+; impls Boundable / NonZeroable on UFixed / IFixed)
  │     ↑
  │     ├── hilavitkutin-api + hilavitkutin
  │     ↑
  │     └── clause-*  (compiler + runtime)
```

Public APIs in the downstream crates use `Maybe` / `Outcome` in place of `Option` / `Result`. Bare `core` primitives appear only where a trait method signature is fixed by the language (`fn next() -> Option<Self::Item>`, `fn partial_cmp() -> Option<Ordering>`, `fn fmt() -> fmt::Result`).

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
