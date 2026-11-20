//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The three parts of one call: `prepare` and `record` untimed, `build` timed.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::record::log_path;

/// The fixture's package, and its library target as cargo names it in its messages and on disk.
pub(crate) const FIXTURE: &str = "notko-product-fixture";
const FIXTURE_LIB: &str = "notko_product_fixture";

/// The dependency that stays built across calls.
pub(crate) const PRODUCT: &str = "notko-hlist";

/// The variable the fixture's `include!` reads its source path from.
const SOURCE_VAR: &str = "NOTKO_PRODUCT_FIXTURE_SOURCE";

static CALLS: AtomicU64 = AtomicU64::new(0);

/// The fixture crate, in the `product-build` bench.
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../product-build/fixture")
}

/// What a call builds: the fixture with its registry, or the fixture with nothing in it, which leaves cargo's own
/// work alone (starting, resolving, fingerprinting, writing the empty library) as the floor under the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fixture {
    Registry,
    Empty,
}

impl Fixture {
    /// Whether the build asks for the `registry` feature.
    pub const fn wants_registry(self) -> bool {
        matches!(self, Fixture::Registry)
    }
}

/// The directory this bench owns under the build directory cargo resolves for the fixture, from the environment or
/// the workspace config alike, asked of `cargo metadata` once per process.
pub(crate) fn own_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let out = Command::new("cargo")
            .current_dir(fixture_dir())
            .args(["metadata", "--format-version", "1", "--no-deps", "--offline"])
            .output()
            .unwrap_or_else(|e| panic!("product-build-call: cargo metadata: {e}"));
        let said = String::from_utf8_lossy(&out.stdout);
        let key = "\"target_directory\":\"";
        let start = said
            .find(key)
            .unwrap_or_else(|| panic!("product-build-call: no target directory in cargo metadata"))
            + key.len();
        let end = start + said[start ..].find('"').expect("a closing quote");
        PathBuf::from(&said[start .. end]).join("notko-product-build")
    })
    .clone()
}

fn target_dir() -> PathBuf {
    own_dir().join("target")
}

fn source_path(n: usize) -> PathBuf {
    own_dir().join("source").join(format!("registry_{n}.rs"))
}

/// Whether the fixture's library is anywhere in the target directory. Cargo's build-dir layout keeps the rlib under
/// `debug/build/<package>/<hash>/out/` and copies nothing to `debug/`, so the file is looked for by name, not by a path.
fn library_present() -> bool {
    fn walk(dir: &Path, prefix: &str) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.flatten().any(|e| {
            let path = e.path();
            if path.is_dir() {
                walk(&path, prefix)
            } else {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix) && n.ends_with(".rlib"))
            }
        })
    }
    walk(&target_dir(), &format!("lib{FIXTURE_LIB}"))
}

/// Cargo for the fixture: dev profile, offline, against the committed lockfile, no incremental cache, and no
/// compiler wrapper, so a cache in front of rustc cannot answer for the compiler. The build directory is the bench's
/// own as well as the target directory: the workspace config sends every build's intermediates to the one shared
/// `build-dir`, where cleaning the fixture would reach past this bench and its library would not be found.
fn cargo(n: usize) -> Command {
    let mut c = Command::new("cargo");
    c.current_dir(fixture_dir())
        .env("CARGO_INCREMENTAL", "0")
        .env(SOURCE_VAR, source_path(n))
        .env("CARGO_BUILD_BUILD_DIR", target_dir())
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .arg("--offline")
        .arg("--locked");
    c
}

fn run(mut c: Command, what: &str) {
    let out = c
        .output()
        .unwrap_or_else(|e| panic!("product-build-call: {what}: cargo on the path: {e}"));
    assert!(
        out.status.success(),
        "product-build-call: {what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Untimed, before the build: the fixture's source for `n` written, `notko-hlist` built if it is not, and the
/// fixture package alone cleaned. Answers whether the fixture's library is then absent.
pub fn prepare(n: usize) -> bool {
    let path = source_path(n);
    let text = crate::source::source(n);
    if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        std::fs::create_dir_all(path.parent().unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    }
    let mut deps = cargo(n);
    deps.args(["build", "-p", PRODUCT, "--target-dir"])
        .arg(target_dir());
    run(deps, "building notko-hlist");
    let mut clean = cargo(n);
    clean
        .args(["clean", "-p", FIXTURE, "--target-dir"])
        .arg(target_dir());
    run(clean, "cleaning the fixture");
    !library_present()
}

/// Timed: the fixture built in cargo's dev profile, with its registry or empty.
pub fn build(n: usize, fixture: Fixture) -> Output {
    let mut c = cargo(n);
    c.args(["build", "-p", FIXTURE]);
    if fixture.wants_registry() {
        c.args(["--features", "registry"]);
    }
    c.args(["--message-format=json", "--target-dir"])
        .arg(target_dir());
    c.output().expect("cargo on the path")
}

/// Untimed, after the build: what the compiler built, from cargo's own messages, with one line appended to the
/// record. Whether the fixture was built with its registry is read from the features cargo says it built with,
/// not from what was asked. Answers with the number of packages the compiler built fresh.
pub fn record(n: usize, fixture: Fixture, output: &Output, absent_before: bool) -> u64 {
    let said = String::from_utf8_lossy(&output.stdout);
    let built: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("\"reason\":\"compiler-artifact\"") && l.contains("\"fresh\":false"))
        .collect();
    let own: Vec<&&str> = built
        .iter()
        .filter(|l| l.contains(&format!("\"name\":\"{FIXTURE_LIB}\"")))
        .collect();
    let fixture_built = own.len();
    let with_registry = own
        .iter()
        .any(|l| l.contains("\"features\":[\"registry\"]"));
    let others_built = built.len() - fixture_built;
    let present_after = library_present();
    let waited = String::from_utf8_lossy(&output.stderr).contains("Blocking waiting for file lock");
    let call = CALLS.fetch_add(1, Ordering::Relaxed) + 1;
    let exit = output
        .status
        .code()
        .map_or_else(|| "signal".to_string(), |c| c.to_string());
    let line = format!(
        "{}\t{call}\t{n}\t{}\t{fixture_built}\t{with_registry}\t{others_built}\t{absent_before}\t{present_after}\t{waited}\t{exit}\n",
        std::process::id(),
        fixture.wants_registry(),
    );
    std::fs::create_dir_all(own_dir()).unwrap_or_else(|e| panic!("{}: {e}", own_dir().display()));
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .and_then(|mut f| f.write_all(line.as_bytes()))
        .unwrap_or_else(|e| panic!("{}: {e}", log_path().display()));
    if !output.status.success() {
        eprintln!(
            "product-build-call: cargo failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    built.len() as u64
}
