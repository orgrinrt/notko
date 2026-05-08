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

The `internal` feature on this crate (or on a crate that re-enables it
transitively) controls the hot-tier release codegen. External published
consumers leave `internal` off → Hot stays as `Outcome<T, E>`, i.e. stable
`Result`-family signatures for the public API.

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
strategies can depend on this crate and reuse the primitives under
`notko_macros::internal::*` to author their own attribute macros.
See `src/rewrite/` for the building blocks.

## License

MPL-2.0.
