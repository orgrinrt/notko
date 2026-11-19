//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Discovery of custom tier definitions from `notko-optimisers/<Name>.rs`.
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
//! The `@notko-optimiser` marker on the first line of the doc comment is
//! required to guard against accidental parsing of unrelated .rs files.
//!
//! Example:
//!
//! ```text
//! //! @notko-optimiser
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
use syn::{Error, Path as SynPath, Result};

use crate::tiers::{CustomTier, Strategy};

/// Everything about a tier lookup that belongs to the macro doing it rather
/// than to this crate.
///
/// A third party authoring its own attribute macro on top of this needs five
/// things to be its own, and every one of them used to be a literal in the
/// function below: the crate its emitted code names, the cargo feature its
/// release arm is gated on, the directory a consumer keeps tier files in, the
/// environment variable its build script sets, and the marker line a tier file
/// carries. Fixing any of them here means a third party's macro silently
/// demanding `notko` be in scope in its own users' crates, under exactly that
/// spelling, with nothing to say so until one of them failed to compile.
///
/// [`Discovery::default`] is notko's own, so `resolve_tier` stays what it was.
#[derive(Debug, Clone)]
pub struct Discovery {
    /// The crate path emitted code names, as `::notko` does here.
    pub krate:        SynPath,
    /// The cargo feature the release arm is gated on.
    pub gate_feature: String,
    /// The directory under a consumer's manifest that holds tier files.
    pub dir:          String,
    /// The environment variable a build script sets to the accumulated
    /// directory.
    pub env_var:      String,
    /// The line a tier file's module doc must carry to be one.
    pub marker:       String,
    /// Where a reader is sent when a tier file's shape is wrong.
    pub docs:         String,
}

impl Default for Discovery {
    fn default() -> Self {
        Self {
            krate:        crate::tiers::default_krate(),
            gate_feature: crate::tiers::DEFAULT_GATE_FEATURE.to_string(),
            dir:          "notko-optimisers".to_string(),
            env_var:      "NOTKO_OPTIMISERS_PATH".to_string(),
            marker:       "@notko-optimiser".to_string(),
            docs:         "notko-macros README".to_string(),
        }
    }
}

impl Discovery {
    /// Resolve a tier name to a [`CustomTier`].
    ///
    /// Order:
    /// 1. Built-in `Hot | Warm | Cold` ZST markers (see [`crate::tiers`]).
    /// 2. `$CARGO_MANIFEST_DIR/<dir>/<Name>.rs` parses metadata.
    /// 3. `$<env_var>/<Name>.rs`, set by the build script.
    /// 4. Error with a diagnostic pointing at where the file should live.
    ///
    /// # Errors
    ///
    /// When the name is neither a built-in nor a tier file this can find, or
    /// when a file it finds does not parse as one.
    pub fn resolve(&self, name: &str, span: Span) -> Result<CustomTier> {
        if let Some(tier) = CustomTier::builtin(name) {
            return Ok(self.stamp(tier));
        }

        if let Some(custom) = self.try_load_crate_local(name, span)? {
            return Ok(self.stamp(custom));
        }

        if let Some(custom) = self.try_load_accumulated(name, span)? {
            return Ok(self.stamp(custom));
        }

        let crate_local = self
            .crate_local_optimiser_path(name)
            .unwrap_or_else(|| PathBuf::from(format!("{}/{name}.rs", self.dir)));
        Err(Error::new(
            span,
            format!(
                "unknown profile tier `{name}`. \
                 built-ins: Hot | Warm | Cold. \
                 custom tier expected at `{}` (crate-local) or \
                 ${}/{name}.rs (via the build script). \
                 see {} for the .rs file shape.",
                crate_local.display(),
                self.env_var,
                self.docs
            ),
        ))
    }

    /// Put this lookup's identity on a tier however the tier was found.
    ///
    /// Both `CustomTier::builtin` and the file parser hand back a tier
    /// carrying this crate's own defaults, because neither of them knows whose
    /// macro asked. Applying it in one place is what makes every route out of
    /// [`Discovery::resolve`] agree; applying it in three was how the file
    /// route came to be the only one that did not.
    fn stamp(&self, tier: CustomTier) -> CustomTier {
        tier.with_crate(self.krate.clone())
            .with_gate_feature(self.gate_feature.clone())
    }

    fn try_load_crate_local(&self, name: &str, span: Span) -> Result<Option<CustomTier>> {
        let Some(path) = self.crate_local_optimiser_path(name) else {
            return Ok(None);
        };
        if !path.is_file() {
            return Ok(None);
        }
        Some(parse_optimiser_file(&path, span, &self.marker)).transpose()
    }

    fn try_load_accumulated(&self, name: &str, span: Span) -> Result<Option<CustomTier>> {
        let Ok(root) = std::env::var(&self.env_var) else {
            return Ok(None);
        };
        let path = Path::new(&root).join(format!("{name}.rs"));
        if !path.is_file() {
            return Ok(None);
        }
        Some(parse_optimiser_file(&path, span, &self.marker)).transpose()
    }

    fn crate_local_optimiser_path(&self, name: &str) -> Option<PathBuf> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
        Some(
            Path::new(&manifest_dir)
                .join(&self.dir)
                .join(format!("{name}.rs")),
        )
    }
}

/// Resolve a tier name to a [`CustomTier`], notko's way.
///
/// [`Discovery::default`] and nothing else. A macro of your own wants its own
/// [`Discovery`], or the code it emits will name this crate in a consumer that
/// never depended on it.
///
/// # Errors
///
/// As [`Discovery::resolve`].
pub fn resolve_tier(name: &str, span: Span) -> Result<CustomTier> {
    Discovery::default().resolve(name, span)
}

fn parse_optimiser_file(path: &Path, span: Span, marker: &str) -> Result<CustomTier> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        Error::new(
            span,
            format!("failed to read optimiser file `{}`: {e}", path.display()),
        )
    })?;

    let meta = extract_module_doc(&source);
    if !meta.lines().any(|l| l.trim() == marker) {
        return Err(Error::new(
            span,
            format!(
                "optimiser file `{}` lacks the `{marker}` marker in its module \
                 doc comment. add `//! {marker}` on a doc line.",
                path.display()
            ),
        ));
    }

    let mut based_on: Option<Strategy> = None;
    let mut inline: Option<bool> = None;
    let mut panic_fmt: Option<String> = None;

    for line in meta.lines() {
        let line = line.trim();
        if line.is_empty() || line == marker {
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
        krate: crate::tiers::default_krate(),
        gate_feature: crate::tiers::DEFAULT_GATE_FEATURE.to_string(),
    })
}

/// Extract the text of module-level doc comments (`//! ...`) from the top of
/// a .rs source, stopping at the first line that is neither a comment nor
/// blank.
///
/// Ordinary `//` comments are walked past rather than treated as the end. A
/// licence header is the shape almost every file in a real tree opens with,
/// and stopping at one meant the marker on the first doc line below it was
/// never seen: the refusal then told the author to add a line that was
/// already there, and deleting the header was the only thing that worked.
fn extract_module_doc(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            out.push_str(rest.trim_start());
            out.push('\n');
        } else if trimmed.is_empty() || trimmed.starts_with("//") {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The header `ante` writes at the top of every file in a tree that keeps
    /// copyright notices, which is the shape this reader has to walk past.
    const HEADER: &str = "\
//----------------------------------------------------------------------
// Copyright (c) 2026                   somebody              them@example
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0
//----------------------------------------------------------------------
";

    const BODY: &str = "\
//! @notko-optimiser
//! based_on = \"Cold\"
//! inline = true
";

    #[test]
    fn a_licence_header_does_not_hide_the_marker_below_it() {
        // The reader used to stop at the first plain `//` line, so every file
        // written by a tool that adds a header parsed as having no module doc
        // at all. The refusal then told the author to add a marker line that
        // was already there, and deleting the header was the only thing that
        // worked.
        let doc = extract_module_doc(&format!("{HEADER}\n{BODY}"));
        assert!(
            doc.contains("@notko-optimiser"),
            "the marker was not found: {doc:?}"
        );
        assert!(
            doc.contains("based_on"),
            "a key below the marker was lost: {doc:?}"
        );
    }

    #[test]
    fn the_header_itself_does_not_become_part_of_the_doc() {
        // Walking past a comment is not the same as reading it. A header whose
        // text ended up in the doc would have every key in it read as a
        // setting, and `Copyright (c) 2026` parses as one under a permissive
        // enough reader.
        let doc = extract_module_doc(&format!("{HEADER}\n{BODY}"));
        assert!(
            !doc.contains("Copyright"),
            "the header leaked into the doc: {doc:?}"
        );
        assert!(
            !doc.contains("SPDX"),
            "the header leaked into the doc: {doc:?}"
        );
    }

    #[test]
    fn a_file_with_no_header_still_reads() {
        // The control on both above. A reader that skipped everything, or one
        // that consumed the whole file, would pass one of them and break this.
        let doc = extract_module_doc(BODY);
        assert!(
            doc.contains("@notko-optimiser"),
            "the plain shape stopped working"
        );
        assert_eq!(doc.lines().count(), 3, "the plain shape read {doc:?}");
    }

    #[test]
    fn the_first_real_line_of_code_still_ends_it() {
        // And the reader has not become one that runs to the end of the file.
        // A doc comment further down belongs to an item, not to the module,
        // and reading it would make an ordinary source file look like a tier.
        let src = format!("{BODY}\npub const X: u8 = 1;\n\n//! @notko-optimiser\n");
        let doc = extract_module_doc(&src);
        assert_eq!(
            doc.lines().count(),
            3,
            "the reader ran past the first item: {doc:?}"
        );
    }

    #[test]
    fn a_blank_line_between_comments_does_not_end_it() {
        // The header and the module doc are separated by one, which is the
        // whole reason this is worth pinning.
        let doc = extract_module_doc("// a comment\n\n// another\n\n//! @notko-optimiser\n");
        assert!(
            doc.contains("@notko-optimiser"),
            "a blank line ended the read: {doc:?}"
        );
    }
}
