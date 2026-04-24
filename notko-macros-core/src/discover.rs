//! Discovery of custom tier definitions from `notko-optimizers/<Name>.rs`.
//!
//! The optimiser file's module-level doc-comment carries metadata in a
//! `key = value` key/value format. Recognised keys:
//!
//! | Key | Type | Default | Meaning |
//! |-----|------|---------|---------|
//! | `based_on` | built-in tier name | required | Which built-in strategy this tier inherits. |
//! | `inline` | `bool` | built-in default | Emit `#[inline]` on the rewritten function. |
//! | `panic_fmt` | string | `"hot path invariant violated: {err:?}"` | Format for the Err to panic rewrite (hot-strategy only). |
//!
//! The `@notko-optimizer` marker on the first line of the doc comment is
//! required to guard against accidental parsing of unrelated .rs files.
//!
//! Example:
//!
//! ```text
//! //! @notko-optimizer
//! //! based_on = "Cold"
//! //! inline = false
//! //! panic_fmt = "trace: {err:?}"
//! ```
//!
//! Downstream crates that want richer extension than parameterised built-in
//! strategies should author their own proc-macro attribute reusing
//! [`crate::rewrite`] primitives and their own `Tier`-implementing ZSTs.

use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::{Error, Result};

use crate::tiers::{CustomTier, Strategy};

/// Resolve a tier name to a [`CustomTier`].
///
/// Order:
/// 1. Built-in `Hot | Warm | Cold` ZST markers (see [`crate::tiers`]).
/// 2. `$CARGO_MANIFEST_DIR/notko-optimizers/<Name>.rs` parses metadata.
/// 3. `$NOTKO_OPTIMISERS_PATH/<Name>.rs` (set by notko-build; see task #99).
/// 4. Error with a diagnostic pointing at where the file should live.
pub fn resolve_tier(name: &str, span: Span) -> Result<CustomTier> {
    if let Some(tier) = CustomTier::builtin(name) {
        return Ok(tier);
    }

    if let Some(custom) = try_load_crate_local(name, span)? {
        return Ok(custom);
    }

    if let Some(custom) = try_load_accumulated(name, span)? {
        return Ok(custom);
    }

    let crate_local = crate_local_optimiser_path(name).unwrap_or_else(|| {
        PathBuf::from(format!("notko-optimizers/{name}.rs"))
    });
    Err(Error::new(
        span,
        format!(
            "unknown profile tier `{name}`. \
             built-ins: Hot | Warm | Cold. \
             custom tier expected at `{}` (crate-local) or \
             $NOTKO_OPTIMISERS_PATH/{name}.rs (via notko-build). \
             see notko-macros README for the .rs file shape.",
            crate_local.display()
        ),
    ))
}

fn try_load_crate_local(name: &str, span: Span) -> Result<Option<CustomTier>> {
    let Some(path) = crate_local_optimiser_path(name) else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    Some(parse_optimiser_file(&path, span)).transpose()
}

fn try_load_accumulated(name: &str, span: Span) -> Result<Option<CustomTier>> {
    let Ok(root) = std::env::var("NOTKO_OPTIMISERS_PATH") else {
        return Ok(None);
    };
    let path = Path::new(&root).join(format!("{name}.rs"));
    if !path.is_file() {
        return Ok(None);
    }
    Some(parse_optimiser_file(&path, span)).transpose()
}

fn crate_local_optimiser_path(name: &str) -> Option<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    Some(
        Path::new(&manifest_dir)
            .join("notko-optimizers")
            .join(format!("{name}.rs")),
    )
}

fn parse_optimiser_file(path: &Path, span: Span) -> Result<CustomTier> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        Error::new(
            span,
            format!("failed to read optimiser file `{}`: {e}", path.display()),
        )
    })?;

    let meta = extract_module_doc(&source);
    if !meta.lines().any(|l| l.trim() == "@notko-optimizer") {
        return Err(Error::new(
            span,
            format!(
                "optimiser file `{}` lacks the `@notko-optimizer` marker in its \
                 module doc comment. add `//! @notko-optimizer` on the first \
                 doc line.",
                path.display()
            ),
        ));
    }

    let mut based_on: Option<Strategy> = None;
    let mut inline: Option<bool> = None;
    let mut panic_fmt: Option<String> = None;

    for line in meta.lines() {
        let line = line.trim();
        if line.is_empty() || line == "@notko-optimizer" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "based_on" => {
                let s = trim_quotes(value);
                based_on = Some(Strategy::from_name(s).ok_or_else(|| {
                    Error::new(
                        span,
                        format!(
                            "optimiser file `{}`: unknown `based_on` value `{s}`. \
                             expected one of the built-in tier names: \
                             Hot | Warm | Cold.",
                            path.display()
                        ),
                    )
                })?);
            },
            "inline" => {
                inline = Some(match value {
                    "true" => true,
                    "false" => false,
                    other => {
                        return Err(Error::new(
                            span,
                            format!(
                                "optimiser file `{}`: `inline` must be `true` or `false`, got `{other}`.",
                                path.display()
                            ),
                        ));
                    },
                });
            },
            "panic_fmt" => {
                panic_fmt = Some(trim_quotes(value).to_string());
            },
            _ => {
                // Unknown keys are tolerated to allow forward-compatibility
                // with future metadata extensions.
            },
        }
    }

    let Some(strategy) = based_on else {
        return Err(Error::new(
            span,
            format!(
                "optimiser file `{}` is missing required `based_on` metadata \
                 (expected `//! based_on = \"Hot\"|\"Warm\"|\"Cold\"`).",
                path.display()
            ),
        ));
    };

    let inline = inline.unwrap_or_else(|| strategy.default_inline());

    Ok(CustomTier {
        strategy,
        inline,
        panic_fmt,
        source_path: Some(path.to_path_buf()),
    })
}

/// Extract the text of module-level doc comments (`//! ...`) from the top of
/// a .rs source, stopping at the first non-doc-comment, non-blank line.
fn extract_module_doc(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            out.push_str(rest.trim_start());
            out.push('\n');
        } else if trimmed.is_empty() {
            continue;
        } else {
            break;
        }
    }
    out
}

fn trim_quotes(s: &str) -> &str {
    let s = s.trim();
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
}
