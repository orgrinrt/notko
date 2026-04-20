//! Build-script helper for [notko-macros].
//!
//! See the crate README for the full usage / mechanics / precedence story.
//! One entry point: [`collect_and_distribute`]. Call it from a consumer
//! crate's `build.rs`.
//!
//! [notko-macros]: https://github.com/orgrinrt/notko/tree/dev/notko-macros

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The name cargo will pass to dependents. Crates that wish to propagate
/// their own optimisers must declare `links = "notko-optimisers-<crate>"`
/// (the `notko-optimisers-` prefix being the convention this tool
/// recognises, with a unique per-crate suffix).
const META_KEY: &str = "notko-optimiser-path";

/// Env var the notko-macros proc-macro reads at expansion time.
const EXPANSION_ENV_VAR: &str = "NOTKO_OPTIMISERS_PATH";

/// Local-relative dir each crate uses to ship its own optimiser .rs files.
const LOCAL_DIR: &str = "notko-optimizers";

/// Sub-dir inside `$OUT_DIR` where accumulated optimiser files are written.
const OUT_SUBDIR: &str = "notko-optimisers";

/// Error type returned by [`collect_and_distribute`]. Wraps io + collision
/// reporting.
#[derive(Debug)]
pub enum Error {
    /// Mandatory cargo-supplied env var was missing.
    MissingEnv(&'static str),
    /// File system read/write failure.
    Io(io::Error),
    /// Two sources provided the same tier name. Paths to both are included.
    Collision {
        name: String,
        first: PathBuf,
        second: PathBuf,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingEnv(var) => write!(
                f,
                "notko-build: required cargo env var `{var}` was not set"
            ),
            Error::Io(e) => write!(f, "notko-build: io error: {e}"),
            Error::Collision { name, first, second } => write!(
                f,
                "notko-build: tier `{name}` provided by two sources: \
                 `{}` and `{}`. resolve by renaming one or dropping a \
                 local override in your crate's notko-optimizers/ dir.",
                first.display(),
                second.display()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Call from a consumer crate's `build.rs`. Scans crate-local
/// `notko-optimizers/` and dependency-propagated paths, accumulates them
/// into `$OUT_DIR/notko-optimisers/`, and emits cargo instructions to
/// expose the accumulated dir to the notko-macros proc-macro and to
/// downstream dependents.
///
/// Idempotent: safe to call every build.
pub fn collect_and_distribute() -> Result<(), Error> {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| Error::MissingEnv("CARGO_MANIFEST_DIR"))?;
    let out_dir = env::var("OUT_DIR").map_err(|_| Error::MissingEnv("OUT_DIR"))?;

    let local_dir = Path::new(&manifest_dir).join(LOCAL_DIR);
    let accumulated_dir = Path::new(&out_dir).join(OUT_SUBDIR);

    fs::create_dir_all(&accumulated_dir)?;

    // name -> path of the source that contributed this tier. Used for
    // collision detection.
    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();

    // Copy crate-local optimisers first. Local takes precedence over
    // DEP_*-propagated paths (the consumer's own files shadow deps).
    if local_dir.is_dir() {
        emit_rerun(&local_dir);
        copy_tree(&local_dir, &accumulated_dir, &mut seen, /* allow_shadow = */ true)?;
    }

    // Collect DEP_*_NOTKO_OPTIMISER_PATH env vars set by cargo from deps
    // that emitted the `notko-optimiser-path` meta key.
    for (key, value) in env::vars() {
        if let Some(rest) = key.strip_prefix("DEP_") {
            if rest.ends_with("_NOTKO_OPTIMISER_PATH") {
                let dep_dir = PathBuf::from(value);
                if dep_dir.is_dir() {
                    copy_tree(
                        &dep_dir,
                        &accumulated_dir,
                        &mut seen,
                        /* allow_shadow = */ false,
                    )?;
                }
            }
        }
    }

    // Expose the accumulated dir to the proc-macro via an env var set for
    // the build of THIS crate's rlib.
    println!(
        "cargo:rustc-env={}={}",
        EXPANSION_ENV_VAR,
        accumulated_dir.display()
    );
    // Propagate to downstream dependents via build-script metadata. Only
    // takes effect if the consumer's Cargo.toml declares a `links = ...`
    // value — otherwise cargo silently drops it.
    println!("cargo:{}={}", META_KEY, accumulated_dir.display());

    Ok(())
}

fn copy_tree(
    src: &Path,
    dst: &Path,
    seen: &mut BTreeMap<String, PathBuf>,
    allow_shadow: bool,
) -> Result<(), Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") {
            continue;
        }
        let tier_name = name.trim_end_matches(".rs").to_string();

        if let Some(existing) = seen.get(&tier_name) {
            if allow_shadow {
                // Local path takes precedence; overwrite the registration.
                // (Should not happen in normal flow because local is
                // processed first, but keep the semantics explicit.)
                seen.insert(tier_name.clone(), path.clone());
            } else {
                return Err(Error::Collision {
                    name: tier_name,
                    first: existing.clone(),
                    second: path,
                });
            }
        } else {
            seen.insert(tier_name.clone(), path.clone());
        }

        let dst_path = dst.join(name);
        fs::copy(&path, &dst_path)?;
    }
    Ok(())
}

fn emit_rerun(dir: &Path) {
    println!("cargo:rerun-if-changed={}", dir.display());
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let pid = std::process::id();
        let ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("notko-build-test-{name}-{pid}-{ns}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn copy_tree_collects_rs_files() {
        let src = tmp_dir("copy-src");
        let dst = tmp_dir("copy-dst");
        write(&src.join("trace.rs"), "//! @notko-optimizer\n");
        write(&src.join("audit.rs"), "//! @notko-optimizer\n");
        // A non-.rs file that should be ignored:
        write(&src.join("README.md"), "ignore me");

        let mut seen = BTreeMap::new();
        copy_tree(&src, &dst, &mut seen, false).unwrap();

        assert!(dst.join("trace.rs").is_file());
        assert!(dst.join("audit.rs").is_file());
        assert!(!dst.join("README.md").exists());
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn copy_tree_reports_collision_across_dep_sources() {
        let src_a = tmp_dir("coll-a");
        let src_b = tmp_dir("coll-b");
        let dst = tmp_dir("coll-dst");
        write(&src_a.join("trace.rs"), "//! @notko-optimizer\n// from a\n");
        write(&src_b.join("trace.rs"), "//! @notko-optimizer\n// from b\n");

        let mut seen = BTreeMap::new();
        copy_tree(&src_a, &dst, &mut seen, false).unwrap();

        let err = copy_tree(&src_b, &dst, &mut seen, false).unwrap_err();
        match err {
            Error::Collision { name, first, second } => {
                assert_eq!(name, "trace");
                assert_eq!(first.file_name().and_then(|s| s.to_str()), Some("trace.rs"));
                assert_eq!(second.file_name().and_then(|s| s.to_str()), Some("trace.rs"));
            },
            other => panic!("expected Collision, got {other:?}"),
        }
    }

    #[test]
    fn copy_tree_allows_local_to_shadow_existing() {
        let local = tmp_dir("shadow-local");
        let dst = tmp_dir("shadow-dst");
        write(&local.join("trace.rs"), "//! @notko-optimizer\n// local wins\n");

        let mut seen = BTreeMap::new();
        // Pretend a dep already registered it.
        seen.insert("trace".to_string(), PathBuf::from("/fake/dep/trace.rs"));

        copy_tree(&local, &dst, &mut seen, true).unwrap();

        assert!(dst.join("trace.rs").is_file());
        assert_eq!(
            seen.get("trace").and_then(|p| p.file_name()).and_then(|s| s.to_str()),
            Some("trace.rs")
        );
    }
}
