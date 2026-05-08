# notko-macros-core

Non-proc-macro library crate housing the AST-rewrite primitives and
tier-discovery logic that power [notko-macros]'s
`#[profile(Tier)]` attribute. Exposed as a stable API so third-party
proc-macro crates can author their own fallibility-tier attributes without
reimplementing the rewrite engine.

[notko-macros]: https://github.com/orgrinrt/notko/tree/dev/notko-macros

## Public API

| Item | Purpose |
|---|---|
| `tiers::{Tier, Strategy, CustomTier}` | Three built-in tiers and the resolved-tier struct handed to rewriters. |
| `parse::parse_tier_arg` | Parse the tier ident from an attribute-arg token stream. |
| `discover::resolve_tier` | Resolve a tier name to a `CustomTier`, looking up `notko-optimizers/<Name>.rs` in the consumer's crate manifest dir and (optionally) in `$NOTKO_OPTIMISERS_PATH`. |
| `rewrite::{entry, rewrite_fn, HotRewriter, OutcomeRewriter}` | The rewrite engine. `entry` is the one-shot driver used by notko-macros itself; `rewrite_fn` takes an already-resolved `CustomTier`; the visitor structs are exposed for finer-grained use. |
| `rewrite::helpers::{is_ok_call, is_err_call, extract_result_inner_types, macro_last_ident_is, stmt_macro_last_ident_is}` | Shared AST utilities. |

## Authoring a third-party attribute macro

```rust
// my-macros/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn optimize_for_trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass your own bespoke CustomTier or call into the library directly.
    let input: syn::ItemFn = syn::parse(item).unwrap();
    let tier = notko_macros_core::tiers::CustomTier {
        strategy: notko_macros_core::tiers::Strategy::Cold,
        inline: false,
        panic_fmt: Some("trace invariant violated: {err:?}".into()),
        source_path: None,
    };
    notko_macros_core::rewrite::rewrite_fn(tier, input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
```

## License

MPL-2.0.
