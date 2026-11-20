//! Ad-hoc spike, not a bench: wall time of `cargo check` and `cargo build` of
//! the generated library, per query and per list length.
//!
//! `drive <check|build> <reps> <n> [<n> ...]`. Each run sets a fresh salt, so
//! the library is rebuilt every time and the dependency is not. One process at
//! a time. Prints the least and the middle of the repetitions in milliseconds,
//! and `fails` with the first line of the error where the compiler refused.

use std::env;
use std::process::Command;
use std::time::Instant;

const QUERIES: &[&str] =
    &["base", "length", "contains_last", "contains_all", "pick_last", "pick_all", "sub_rev", "project"];

fn run(mode: &str, q: &str, n: usize, salt: u64) -> Result<f64, String> {
    let t = Instant::now();
    let out = Command::new("cargo")
        .args([mode.trim_start_matches("cold-"), "--lib", "-q"])
        // `cold-check` and `cold-build` turn the incremental cache off, so a
        // run pays for the whole library. Without it a repetition that only
        // changes the salt is answered mostly from the cache.
        .env("CARGO_INCREMENTAL", if mode.starts_with("cold-") { "0" } else { "1" })
        .env("SPIKE_N", n.to_string())
        .env("SPIKE_Q", q)
        .env("SPIKE_SALT", salt.to_string())
        .output()
        .expect("cargo could not be run");
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    if out.status.success() {
        Ok(ms)
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| l.starts_with("error")).unwrap_or("error").to_string();
        Err(first.chars().take(140).collect())
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mode = args[0].as_str();
    if mode == "verify" {
        // `drive verify <n>`: shows that a run rechecks the library (cargo's
        // own verbose lines) and that the checker can say no (the control).
        let n: usize = args[1].parse().unwrap();
        let out = Command::new("cargo")
            .args(["check", "--lib", "-v"])
            .env("SPIKE_N", n.to_string())
            .env("SPIKE_Q", "sub_rev")
            .env("SPIKE_SALT", "verify-a")
            .output()
            .expect("cargo could not be run");
        for l in String::from_utf8_lossy(&out.stderr).lines() {
            if l.contains("Dirty") || l.contains("Fresh") || l.contains("Finished") {
                println!("sub_rev at {n}: {}", l.trim().chars().take(150).collect::<String>());
            }
        }
        match run("check", "absent", n, 1) {
            Ok(ms) => println!("control at {n}: BUILT in {ms:.0} ms, so the runs check nothing"),
            Err(e) => println!("control at {n}: refused, as it must be: {e}"),
        }
        return;
    }
    if mode == "one" {
        // `drive one <query> <n>`: a single check of one query.
        let n: usize = args[2].parse().unwrap();
        match run("check", &args[1], n, 7) {
            Ok(ms) => println!("{} at {n}: checks in {ms:.0} ms", args[1]),
            Err(e) => println!("{} at {n}: fails: {e}", args[1]),
        }
        return;
    }
    let reps: usize = args[1].parse().unwrap();
    let ns: Vec<usize> = args[2 ..].iter().map(|a| a.parse().unwrap()).collect();
    let mut salt = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("cargo {mode} --lib, wall milliseconds, least and middle of {reps}");
    println!();
    println!("| query | n | least | middle |");
    println!("|---|---|---|---|");
    for q in QUERIES {
        for &n in &ns {
            let mut times = Vec::new();
            let mut failure = None;
            for _ in 0 .. reps {
                salt += 1;
                match run(mode, q, n, salt) {
                    Ok(ms) => times.push(ms),
                    Err(e) => {
                        failure = Some(e);
                        break;
                    },
                }
            }
            match failure {
                Some(e) => println!("| {q} | {n} | fails | {e} |"),
                None => {
                    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    println!("| {q} | {n} | {:.0} | {:.0} |", times[0], times[times.len() / 2]);
                },
            }
        }
    }
}
