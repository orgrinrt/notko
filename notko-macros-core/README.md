# `notko-macros-core`

<div align="center" style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/stargazers)
[![Crates.io](https://img.shields.io/crates/v/notko-macros-core)](https://crates.io/crates/notko-macros-core)
[![docs.rs](https://img.shields.io/docsrs/notko-macros-core)](https://docs.rs/notko-macros-core)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/notko.svg)](https://github.com/orgrinrt/notko/issues)
![License](https://img.shields.io/github/license/orgrinrt/notko?color=%23009689)

> The AST rewriting behind `#[profile(Tier)]`, as an ordinary library you can build your own attribute macro on.

</div>

It's here because a proc-macro crate can't export anything that isn't a macro, so there was nowhere
else to put the rewrite engine that [`notko-macros`](https://crates.io/crates/notko-macros) runs on.
And once it had to exist separately it may as well be public: if you want to write your own
fallibility-tier attribute, you can build on this instead of doing the rewrite engine again.

Do note the surface still moves with the rest of the crates, so pin an exact version.

## Installation

```bash
cargo add notko-macros-core
```

## What's in it

The rewrite engine, the tier vocabulary it rewrites against, and the discovery
that turns a tier name into something to rewrite with. The full surface with
working links is on [docs.rs](https://docs.rs/notko-macros-core), which is a
better place for it than a table here that drifts the first time a type moves.

## Authoring a third-party attribute macro

```rust
// my-macros/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn profile_asserted(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // build the tier yourself, or load one with discover::resolve_tier
    let input = match syn::parse::<syn::ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };
    let tier = notko_macros_core::tiers::CustomTier {
        strategy: notko_macros_core::tiers::Strategy::Hot,
        inline: true,
        panic_fmt: Some("asserted invariant violated: {err:?}".into()),
        source_path: None,
        krate: syn::parse_quote!(::my_runtime),  // both of these are yours
        gate_feature: "my_release_arm".to_string(),
    };
    notko_macros_core::rewrite::rewrite_fn(tier, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
```

The last two fields are the ones worth a second look. A rewrite has to name some
crate for the type it produces, and whatever it names becomes a requirement on
your users' crates rather than on yours. Same for the feature the release arm is
gated on: that `cfg` is read where the attribute expanded, so leaving it at the
default means anyone who flips it flips yours too.

`krate` is read whichever strategy you pick, since every one of them names a
crate for the type it writes. `gate_feature` is the hot strategy's alone: on a
cold or warm tier it sits there unused, and the day the strategy changes is the
day it starts mattering.

Do note the parse is matched, not unwrapped. In a proc-macro an `unwrap` gives whoever used your
attribute a panic with no span on it, where `to_compile_error` points at their own code.

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/notko/blob/main/LICENSE)
