# notko-macros

Proc-macro attribute `#[profile(Tier)]` for the [notko](https://github.com/orgrinrt/notko)
foundation primitives. AST-rewrites function bodies between Hot / Warm / Cold
fallibility tiers at compile time.

## Built-in tiers

| Tier | Debug / standalone | Release + internal |
|------|--------------------|--------------------|
| `Hot` | `Outcome<T, E>` wrapping; `Ok(x)` → `Outcome::Ok(x)`, `Err(e)` → `Outcome::Err(e)` | `Just<T>`; `Ok(x)` → `Just::new(x)`, `Err(e)` → `panic!(...)` |
| `Cold` | `Outcome<T, E>` always. | Same. |
| `Warm` | Passthrough. | Passthrough. |

`Hot` gets `#[inline]`. `Cold` and `Warm` do not.

## The `internal` feature belongs to your crate

`#[profile(Hot)]` emits a `cfg(feature = "internal")`, and a `cfg` written by
an attribute macro is read against the features of the crate it expanded into.
So the feature that decides which arm you get is **yours**, not this crate's,
and it has to be declared:

```toml
[features]
internal = []
```

Without that line the code still compiles and still behaves correctly, on the
`Outcome<T, E>` arm, but every `#[profile]` in the crate warns that `internal`
is not a value `feature` can take. Declaring it is what makes the warning go
away and the switch reachable.

Leave it off and `Hot` stays `Outcome<T, E>`, which is the arm a published api
wants: `Result`-family signatures, errors that can be handled. Turn it on in a
build with `debug_assertions` off and `Hot` becomes `Just<T>` with the error
arm panicking, which is the arm a binary wants when it has already decided the
error cannot happen.

## Usage

```rust
use notko_macros::profile;

#[profile(Hot)]
pub fn lookup(id: u32) -> Result<u32, MyErr> {
    if id > 1000 { return Err(MyErr); }
    Ok(id * 2)
}

#[profile(Cold)]
pub fn init(cfg: &Config) -> Result<State, SetupErr> { /* ... */ }
```

## Custom tiers (crate-local)

Drop a file at `$CRATE_ROOT/notko-optimizers/<name>.rs` with this shape:

```rust
//! @notko-optimizer
//! based_on = "Cold"
//! inline = false
//! panic_fmt = "trace invariant violated: {err:?}"
```

The proc-macro reads the file at expansion time (via the consumer's
`CARGO_MANIFEST_DIR`) and applies the named built-in strategy with the
tier-specific parameters. Custom tiers appear alongside built-ins with no
additional imports:

```rust
#[profile(Trace)]  // resolves via notko-optimizers/Trace.rs
pub fn some_work() -> Result<(), Err> { /* ... */ }
```

For optimiser sharing across crates (e.g. one crate defines `Trace`, others
in the dep tree consume it), use the `notko-build` companion crate.

## Under the hood

Third-party authors who need full AST-level control beyond the built-in
strategies can depend on [notko-macros-core] and reuse the primitives under
`notko_macros_core::{tiers, parse, discover, rewrite}` to author their own
attribute macros. See that crate's README for the public API map.

[notko-macros-core]: https://github.com/orgrinrt/notko/tree/main/notko-macros-core

## License

MPL-2.0.
