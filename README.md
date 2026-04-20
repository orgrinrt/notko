# notko

Foundation primitives for the [hilavitkutin](https://github.com/orgrinrt/hilavitkutin)
stack. Finnish *notko*: hollow, trough — the ground every downstream
crate sits on.

`#![no_std]`, no alloc, no deps.

## Contents

| Type / trait | Replaces |
|--------------|----------|
| `Just<T>` | Infallible wrapper; `Try` with `Residual = Infallible`. Zero cost. |
| `Maybe<T>` | `core::option::Option<T>` in stack APIs. `repr(C)`. |
| `Outcome<T, E>` | `core::result::Result<T, E>` in stack APIs. `repr(C)`. |
| `Boundable` | Trait: `[MIN, MAX]` integer-like. Arvo impls on `UFixed`/`IFixed`. |
| `NonZeroable` | Trait: nonzero form over a scalar. Arvo impls on `UFixed`/`IFixed`. |

## Why three, not one

`Just` / `Maybe` / `Outcome` are the control-flow analog of arvo's
numeric Hot / Warm / Cold / Precise strategy markers. Each type
represents a distinct *tier of fallibility* with a different cold-path
cost:

| Tier | Type | Cold path | When to use |
|------|------|-----------|-------------|
| **Hot** | `Just<T>` | None — no branch. | Value proven present. `?` compiles away. |
| **Warm** | `Maybe<T>` | Absent-variant discriminant, no payload. | "Not there" is ordinary, not exceptional. |
| **Cold** | `Outcome<T, E>` | Full error payload + branch. | Caller needs to know *why* on failure. |

`Just<T>` is `#[repr(transparent)]` — no discriminant, no branch, no
runtime cost. `Try` with `Residual = Infallible` means `?` on a `Just`
compiles to literally nothing. Use it where an invariant proves the
error variant unreachable (post-validation, codegen-reduced hot paths,
reified-guarantee wrappers).

`Maybe<T>` carries a one-bit discriminant and no payload on the absent
side. Ideal for "lookup miss" / "optional field" / "end-of-iteration"
positions where absence is a valid state, not a failure worth
explaining.

`Outcome<T, E>` is the full Result shape — discriminant plus error
payload — for positions where the error is informational and the
caller needs to branch on it.

## Pairs with a compile-time rewrite

The ladder is most valuable when paired with an `#[optimize_for(…)]`
proc-macro attribute that rewrites a function's return type between
builds — `Outcome<T, E>` in debug / standalone consumers, `Just<T>`
in internal-release builds where invariants are proven by
construction. The developer writes standard `Result` / `Ok` / `Err` /
`?`; the macro rewrites Ok → `Just(…)` and Err → `panic!(…)` in hot
release, leaving cold paths with full error handling and diagnostics.

Loimu's `loimu-codepath-macros` crate is the working prototype of
this pattern; the hilavitkutin stack adopts the primitives via notko
and the macro via a sibling crate (TBD). The primitives are usable
without the macro — the macro is an optional accelerator that turns
plain Rust source into the temperature-specific emitted form.

This is the same philosophy as arvo's Hot/Warm/Cold/Precise Strategy
markers: state the tradeoff at the type level, let the compiler pick
the concrete shape per call site. Arvo does it for numeric precision
vs. throughput. Notko does it for fallibility vs. branchlessness.

## ABI stability

`Maybe` and `Outcome` are `#[repr(C)]`. Their layout is fixed and
portable across compilations, ABI boundaries, and `dlopen`.
`core::option::Option<T>` depends on niche optimization that varies
by optimization level and Rust version; its layout is not a stable
contract. The stack's ecosystem explicitly needs stable layout —
plugin dispatch, C FFI, WASM component-model — so bare `Option` /
`Result` cannot appear in public API positions.

## Positioning

```
notko (zero deps)
  ↑
  ├── arvo (L0+; impls Boundable / NonZeroable on UFixed / IFixed)
  │     ↑
  │     ├── hilavitkutin-api + hilavitkutin
  │     ↑
  │     └── clause-* (compiler + runtime)
```

Public APIs in the downstream crates use `Maybe` / `Outcome` in place
of `Option` / `Result`. Raw std primitives are used only in std trait
method impls (`fn next() -> Option<Self::Item>`, `fn partial_cmp() ->
Option<Ordering>`, `fn fmt() -> fmt::Result`) where the trait signature
is fixed by the language.

## Cargo features

- `try_trait_v2` (nightly) — impl `core::ops::Try` for `Just` / `Maybe`
  / `Outcome`, so `?` works on them directly. Without this feature the
  types still function; only the `?` operator is unavailable.

## License

MPL-2.0. See `LICENSE`.
