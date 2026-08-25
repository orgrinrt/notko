//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Build-script helper for [notko-macros].
//!
//! See the crate README for the full usage / mechanics / precedence story.
//! One entry point: [`collect_and_distribute`]. Call it from a consumer
//! crate's `build.rs`.
//!
//! [notko-macros]: https://crates.io/crates/notko-macros

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

/// The name cargo will pass to dependents. Crates that wish to propagate
/// their own optimisers must declare `links = "notko-optimisers-<crate>"`
/// (the `notko-optimisers-` prefix being the convention this tool
/// recognises, with a unique per-crate suffix).
const META_KEY: &str = "notko-optimiser-path";

/// Env var the notko-macros proc-macro reads at expansion time.
const EXPANSION_ENV_VAR: &str = "NOTKO_OPTIMISERS_PATH";

/// Local-relative dir each crate uses to ship its own optimiser .rs files.
const LOCAL_DIR: &str = "notko-optimisers";

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
        name:   String,
        first:  PathBuf,
        second: PathBuf,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingEnv(var) => {
                write!(f, "notko-build: required cargo env var `{var}` was not set")
            },
            Error::Io(e) => write!(f, "notko-build: io error: {e}"),
            Error::Collision {
                name,
                first,
                second,
            } => {
                write!(
                    f,
                    "notko-build: tier `{name}` provided by two sources: \
                 `{}` and `{}`. resolve by renaming one or dropping a \
                 local override in your crate's notko-optimisers/ dir.",
                    first.display(),
                    second.display()
                )
            },
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
/// `notko-optimisers/` and dependency-propagated paths, accumulates them
/// into `$OUT_DIR/notko-optimisers/`, and emits cargo instructions to
/// expose the accumulated dir to the notko-macros proc-macro and to
/// downstream dependents.
///
/// Idempotent: safe to call every build.
pub fn collect_and_distribute() -> Result<(), Error> {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| Error::MissingEnv("CARGO_MANIFEST_DIR"))?;
    let out_dir = env::var("OUT_DIR").map_err(|_| Error::MissingEnv("OUT_DIR"))?;

    let dep_dirs = dep_dirs_from(env::vars());

    let accumulated_dir = accumulate(
        &Path::new(&manifest_dir).join(LOCAL_DIR),
        &dep_dirs,
        &Path::new(&out_dir).join(OUT_SUBDIR),
    )?;

    emit_metadata(&accumulated_dir);
    Ok(())
}

/// Copy every optimiser this crate can see into `accumulated_dir` and return
/// it, with the crate's own files winning any name a dependency also claims.
///
/// Split out from [`collect_and_distribute`] because that one reads the
/// environment cargo hands a build script, and the environment is process-wide
/// while tests are not. This takes its three inputs as paths, so a test can
/// build a real tree and assert on what actually lands.
fn accumulate(
    local_dir: &Path,
    dep_dirs: &[PathBuf],
    accumulated_dir: &Path,
) -> Result<PathBuf, Error> {
    fs::create_dir_all(accumulated_dir)?;

    // Tier name to the source that contributed it, and whether that source was
    // this crate's own directory. Both halves are needed: the name to detect a
    // clash at all, and the origin to decide whether the clash is an error.
    let mut seen: BTreeMap<String, Origin> = BTreeMap::new();

    // This crate's own files first, so a name it claims is already registered
    // as local by the time any dependency is read.
    if local_dir.is_dir() {
        emit_rerun(local_dir);
        copy_tree(local_dir, accumulated_dir, &mut seen, Source::Local)?;
    }

    for dep_dir in dep_dirs {
        if dep_dir.is_dir() {
            copy_tree(dep_dir, accumulated_dir, &mut seen, Source::Dep)?;
        }
    }

    Ok(accumulated_dir.to_path_buf())
}

/// Where a tier came from, which is what decides whether a repeated name is a
/// shadow or a clash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    /// The consuming crate's own `notko-optimisers/` directory.
    Local,
    /// A directory a dependency propagated through build-script metadata.
    Dep,
}

/// The source that contributed a tier, kept so a later clash can be judged.
#[derive(Debug, Clone)]
struct Origin {
    path:   PathBuf,
    source: Source,
}

/// The dependency-contributed directories in a build script's environment.
///
/// Takes the variables rather than reading them, for the reason [`accumulate`]
/// is split out: the process environment is shared and tests are not, so a
/// filter that reads it directly can only be checked by hand.
fn dep_dirs_from(vars: impl Iterator<Item = (String, String)>) -> Vec<PathBuf> {
    vars.filter_map(|(key, value)| {
        let rest = key.strip_prefix("DEP_")?;
        rest.ends_with("_NOTKO_OPTIMISER_PATH")
            .then(|| PathBuf::from(value))
    })
    .collect()
}

/// The `cargo:` lines a build script emits for an accumulated directory.
///
/// Built rather than printed, so what goes to stdout can be asserted on. A
/// wrong key here is silent: cargo ignores an instruction it does not
/// recognise, and the consumer's proc-macro simply finds nothing.
fn metadata_lines(accumulated_dir: &Path) -> [String; 2] {
    [
        // The proc-macro reads this during expansion of this crate's own rlib.
        format!(
            "cargo:rustc-env={}={}",
            EXPANSION_ENV_VAR,
            accumulated_dir.display()
        ),
        // Propagate to downstream dependents via build-script metadata. Only
        // takes effect if the consumer's Cargo.toml declares a `links = ...`
        // value, otherwise cargo silently drops it.
        format!("cargo:{}={}", META_KEY, accumulated_dir.display()),
    ]
}

fn emit_metadata(accumulated_dir: &Path) {
    for line in metadata_lines(accumulated_dir) {
        println!("{line}");
    }
}

/// Copy every `.rs` file directly under `src` into `dst`, registering each
/// tier name in `seen`.
///
/// A name already registered by [`Source::Local`] is skipped outright, file
/// and all, because the local file is the one that wins and copying over it
/// would silently undo that. A name registered by one dependency and claimed
/// by another is [`Error::Collision`]: nothing ranks two dependencies against
/// each other, so picking either would be arbitrary.
fn copy_tree(
    src: &Path,
    dst: &Path,
    seen: &mut BTreeMap<String, Origin>,
    source: Source,
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

        match seen.get(&tier_name) {
            Some(existing) if existing.source == Source::Local => continue,
            Some(existing) => {
                return Err(Error::Collision {
                    name:   tier_name,
                    first:  existing.path.clone(),
                    second: path,
                });
            },
            None => {
                seen.insert(tier_name.clone(), Origin {
                    path: path.clone(),
                    source,
                });
            },
        }

        fs::copy(&path, dst.join(name))?;
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

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn copy_tree_collects_rs_files_and_ignores_the_rest() {
        let src = tmp_dir("copy-src");
        let dst = tmp_dir("copy-dst");
        write(&src.join("trace.rs"), "//! @notko-optimiser\n");
        write(&src.join("audit.rs"), "//! @notko-optimiser\n");
        write(&src.join("README.md"), "ignore me");
        fs::create_dir_all(src.join("nested.rs")).unwrap();

        let mut seen = BTreeMap::new();
        copy_tree(&src, &dst, &mut seen, Source::Dep).unwrap();

        assert!(dst.join("trace.rs").is_file());
        assert!(dst.join("audit.rs").is_file());
        assert!(!dst.join("README.md").exists());
        // A directory whose name ends in `.rs` is not a file and is skipped.
        assert!(!dst.join("nested.rs").exists());
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn two_dependencies_claiming_one_tier_is_a_collision() {
        let dep_a = tmp_dir("coll-a");
        let dep_b = tmp_dir("coll-b");
        let local = tmp_dir("coll-local");
        let out = tmp_dir("coll-out");
        write(&dep_a.join("trace.rs"), "// from a\n");
        write(&dep_b.join("trace.rs"), "// from b\n");

        let err = accumulate(&local, &[dep_a.clone(), dep_b.clone()], &out).unwrap_err();
        match err {
            Error::Collision {
                name,
                first,
                second,
            } => {
                assert_eq!(name, "trace");
                assert_eq!(first, dep_a.join("trace.rs"));
                assert_eq!(second, dep_b.join("trace.rs"));
            },
            other => panic!("expected Collision, got {other:?}"),
        }
    }

    #[test]
    fn a_local_tier_shadows_a_dependencys_rather_than_colliding_with_it() {
        let local = tmp_dir("shadow-local");
        let dep = tmp_dir("shadow-dep");
        let out = tmp_dir("shadow-out");
        write(&local.join("trace.rs"), "// local wins\n");
        write(&dep.join("trace.rs"), "// dep loses\n");
        write(&dep.join("audit.rs"), "// dep only\n");

        let dir = accumulate(&local, &[dep], &out).unwrap();

        // The readme promises the local file wins. That is two claims: the
        // build does not fail, and the bytes that land are the local ones.
        assert_eq!(read(&dir.join("trace.rs")), "// local wins\n");
        // A tier only the dependency has still arrives.
        assert_eq!(read(&dir.join("audit.rs")), "// dep only\n");
    }

    #[test]
    fn shadowing_holds_whichever_order_the_dependencies_arrive_in() {
        // Cargo hands `DEP_*` vars over in whatever order it likes, so the
        // local file has to win against a dependency read first and against
        // one read last.
        for extra_first in [true, false] {
            let local = tmp_dir("order-local");
            let dep_a = tmp_dir("order-a");
            let dep_b = tmp_dir("order-b");
            let out = tmp_dir("order-out");
            write(&local.join("trace.rs"), "// local wins\n");
            write(&dep_a.join("trace.rs"), "// a loses\n");
            write(&dep_b.join("audit.rs"), "// b only\n");

            let deps = if extra_first { vec![dep_b, dep_a] } else { vec![dep_a, dep_b] };
            let dir = accumulate(&local, &deps, &out).unwrap();

            assert_eq!(read(&dir.join("trace.rs")), "// local wins\n");
            assert_eq!(read(&dir.join("audit.rs")), "// b only\n");
        }
    }

    #[test]
    fn a_crate_with_no_local_directory_still_accumulates_its_dependencies() {
        let local = tmp_dir("absent-local").join("does-not-exist");
        let dep = tmp_dir("absent-dep");
        let out = tmp_dir("absent-out");
        write(&dep.join("trace.rs"), "// dep\n");

        let dir = accumulate(&local, &[dep], &out).unwrap();
        assert_eq!(read(&dir.join("trace.rs")), "// dep\n");
    }

    #[test]
    fn a_dependency_path_that_does_not_exist_is_skipped_rather_than_fatal() {
        let local = tmp_dir("gone-local");
        let out = tmp_dir("gone-out");
        write(&local.join("trace.rs"), "// local\n");

        let dir = accumulate(&local, &[PathBuf::from("/nonexistent/notko")], &out).unwrap();
        assert_eq!(read(&dir.join("trace.rs")), "// local\n");
    }

    fn v(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, x)| (k.to_string(), x.to_string()))
            .collect()
    }

    #[test]
    fn only_the_dep_vars_naming_an_optimiser_path_are_collected() {
        let got = dep_dirs_from(
            v(&[
                ("DEP_NOTKO_OPTIMISERS_FOO_NOTKO_OPTIMISER_PATH", "/a"),
                ("DEP_NOTKO_OPTIMISERS_BAR_NOTKO_OPTIMISER_PATH", "/b"),
                // Right prefix, wrong suffix: another crate's `links` metadata.
                ("DEP_OPENSSL_INCLUDE", "/openssl"),
                // Right suffix, no prefix: not something cargo set for a dep.
                ("MY_NOTKO_OPTIMISER_PATH", "/mine"),
                // Neither.
                ("PATH", "/usr/bin"),
                ("OUT_DIR", "/out"),
            ])
            .into_iter(),
        );
        assert_eq!(got, vec![PathBuf::from("/a"), PathBuf::from("/b")]);
    }

    #[test]
    fn an_environment_with_no_dependency_paths_yields_none() {
        let got = dep_dirs_from(v(&[("PATH", "/usr/bin"), ("OUT_DIR", "/out")]).into_iter());
        assert!(
            got.is_empty(),
            "picked up {got:?} from an environment with none"
        );
    }

    #[test]
    fn the_emitted_lines_carry_the_keys_cargo_and_the_macro_read() {
        let lines = metadata_lines(Path::new("/out/notko-optimisers"));

        // The exact spellings, because both are silent when wrong: cargo
        // ignores an instruction it does not recognise, and the proc-macro
        // reads an environment variable by name and finds nothing.
        assert_eq!(
            lines[0],
            "cargo:rustc-env=NOTKO_OPTIMISERS_PATH=/out/notko-optimisers"
        );
        assert_eq!(lines[1], "cargo:notko-optimiser-path=/out/notko-optimisers");
    }

    #[test]
    fn the_directory_a_crate_ships_its_own_files_in_is_the_documented_one() {
        // The fourth name, and the one the emitted lines above cannot reach:
        // it names a directory in the consumer's source tree rather than
        // anything this writes out. Wrong, and the scan simply finds nothing
        // and says nothing, which is the same silence as the other three.
        assert_eq!(LOCAL_DIR, "notko-optimisers");
        assert_eq!(
            LOCAL_DIR, OUT_SUBDIR,
            "the directory a crate ships and the one this accumulates into are \
             documented as the same name; a reader who learns one has learned both"
        );
    }
}
