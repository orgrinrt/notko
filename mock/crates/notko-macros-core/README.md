# `notko-macros-core`

[![Crates.io](https://img.shields.io/crates/v/notko-macros-core)](https://crates.io/crates/notko-macros-core) [![docs.rs](https://img.shields.io/docsrs/notko-macros-core)](https://docs.rs/notko-macros-core)

> The AST rewriting behind `#[profile(Tier)]`, as an ordinary library. Ships `tiers`, `parse`, `discover` and `rewrite`, which is what another crate's fallibility-tier attribute would build on.

A proc-macro crate can't export anything that isn't a macro, so the rewrite engine that [`notko-macros`](https://crates.io/crates/notko-macros) runs on lives here as an ordinary library, and an attribute of another crate's own can build on it instead of doing the rewriting again. It holds the rewrite engine, the tier vocabulary it rewrites against and the discovery that turns a tier name into something to rewrite with, and the full surface is on [docs.rs](https://docs.rs/notko-macros-core).

## Usage

```bash
cargo add notko-macros-core
```

A rewrite has to name some crate for the type it produces, and a gate feature for the release arm, and both belong to the calling crate rather than to this one. So the library takes them as fields and defaults to nothing you'd want:

```rust
// my-macros/src/lib.rs
use notko_macros_core::tiers::{CustomTier, Hot};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn profile_asserted(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // one tier, built by hand
    let input = match syn::parse::<syn::ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };
    // both the crate and the gate feature belong to the calling crate, not this one
    let mut tier = CustomTier::from_marker::<Hot>()
        .with_crate(syn::parse_quote!(::my_runtime))
        .with_gate_feature("my_release_arm");
    tier.panic_fmt = Some("asserted invariant violated: {err:?}".into());
    notko_macros_core::rewrite::rewrite_fn(tier, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
```

`Discovery` is the same lookup notko's own macro runs, with every name in it the calling crate's. Every field is named rather than spread over `Discovery::default()`, which is notko's own and would emit code naming `notko` in consumers that never depended on it:

```rust
use notko_macros_core::discover::Discovery;

fn mine() -> Discovery {
    Discovery {
        krate: syn::parse_quote!(::my_runtime),
        gate_feature: "my_release_arm".to_string(),
        dir: "my-tiers".to_string(),
        env_var: "MY_TIERS_PATH".to_string(),
        marker: "@my-tier".to_string(),
        docs: "the my-macros readme".to_string(),
    }
}

// then, in the attribute:
let tier = mine().resolve(&name, span)?;
```

The surface moves with the rest of the crates, and on a `0.0.x` version cargo resolves to exactly that one, so what `cargo add` writes is already the pin.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/dev/LICENSE)
