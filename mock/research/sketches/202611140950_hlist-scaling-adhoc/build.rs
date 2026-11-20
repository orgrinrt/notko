//! Writes `gen.rs`: a list of `SPIKE_N` distinct members and one query over
//! it, named by `SPIKE_Q`. The driver in `src/bin/drive.rs` sets both and
//! times `cargo check` and `cargo build` of the library, so the cost of each
//! query at each length can be read apart from the cost of the list itself.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn list(names: &[String]) -> String {
    let mut s = String::new();
    for n in names {
        write!(s, "Cons<{n}, ").unwrap();
    }
    s.push_str("Empty");
    for _ in names {
        s.push('>');
    }
    s
}

fn row(n: usize) -> String {
    // PCons { head: 1, tail: PCons { ... PNil } }
    let mut s = String::new();
    for i in 0 .. n {
        write!(s, "PCons {{ head: {i}u32, tail: ").unwrap();
    }
    s.push_str("PNil");
    for _ in 0 .. n {
        s.push_str(" }");
    }
    s
}

fn main() {
    println!("cargo:rerun-if-env-changed=SPIKE_N");
    println!("cargo:rerun-if-env-changed=SPIKE_Q");
    println!("cargo:rerun-if-env-changed=SPIKE_SALT");

    let n: usize = env::var("SPIKE_N").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let q = env::var("SPIKE_Q").unwrap_or_else(|_| "base".to_string());
    let salt = env::var("SPIKE_SALT").unwrap_or_default();

    let names: Vec<String> = (0 .. n).map(|i| format!("M{i}")).collect();
    let rev: Vec<String> = names.iter().rev().cloned().collect();
    let last = names.last().cloned().unwrap();

    let mut g = String::new();
    writeln!(g, "// generated: n = {n}, q = {q}, salt = {salt}").unwrap();
    writeln!(g, "pub const SALT: &str = \"{salt}\";").unwrap();
    for name in &names {
        writeln!(g, "pub struct {name}; impl Elem for {name} {{ type Value = u32; }}").unwrap();
    }
    writeln!(g, "pub type L = {};", list(&names)).unwrap();
    writeln!(g, "pub type R = {};", list(&rev)).unwrap();
    writeln!(g, "pub type Row = <L as Project>::Prod<Val>;").unwrap();
    // `bare` leaves the row value out, so the list type stands alone.
    if q != "bare" {
        writeln!(g, "pub fn the_row() -> Row {{ {} }}", row(n)).unwrap();
    }

    match q.as_str() {
        "base" | "bare" => {},
        "length" => {
            writeln!(g, "pub const LEN: Count = <L as Length<Count>>::LEN;").unwrap();
        },
        "contains_last" => {
            writeln!(g, "fn needs<X: Contains<{last}>>() {{}}").unwrap();
            writeln!(g, "pub fn q() {{ needs::<L>() }}").unwrap();
        },
        "contains_all" => {
            writeln!(g, "fn needs<X: ContainsAll<R>>() {{}}").unwrap();
            writeln!(g, "pub fn q() {{ needs::<L>() }}").unwrap();
        },
        "pick_last" => {
            writeln!(g, "fn pick<M: Elem<Value = u32>, P: Place, X: Has<M, P>>(r: &X::Prod<Val>) -> u32 {{ *X::pick::<Val>(r) }}").unwrap();
            writeln!(g, "pub fn q(r: &Row) -> u32 {{ pick::<{last}, _, L>(r) }}").unwrap();
        },
        "pick_all" => {
            writeln!(g, "fn pick<M: Elem<Value = u32>, P: Place, X: Has<M, P>>(r: &X::Prod<Val>) -> u32 {{ *X::pick::<Val>(r) }}").unwrap();
            writeln!(g, "pub fn q(r: &Row) -> u32 {{ let mut s = 0u32;").unwrap();
            for name in &names {
                writeln!(g, "    s = s.wrapping_add(pick::<{name}, _, L>(r));").unwrap();
            }
            writeln!(g, "    s }}").unwrap();
        },
        "sub_rev" => {
            writeln!(g, "pub fn q(r: &Row) -> <R as Project>::Prod<Val> {{ <R as SubOf<L, _>>::sub::<Val>(r) }}").unwrap();
        },
        "project" => {
            writeln!(g, "pub fn q(r: &Row) -> u32 {{").unwrap();
            writeln!(g, "    let mut c = L::made::<4>(r);").unwrap();
            writeln!(g, "    L::write_row::<Arr<4>>(&mut c, 1, r);").unwrap();
            writeln!(g, "    L::read_row::<Arr<4>>(&c, 1).head").unwrap();
            writeln!(g, "}}").unwrap();
        },
        // The control: a member the list does not hold. A run that reports
        // success for this one checked nothing.
        "absent" => {
            writeln!(g, "pub struct Absent;").unwrap();
            writeln!(g, "fn needs<X: Contains<Absent>>() {{}}").unwrap();
            writeln!(g, "pub fn q() {{ needs::<L>() }}").unwrap();
        },
        other => panic!("unknown SPIKE_Q {other}"),
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("gen.rs");
    fs::write(out, g).unwrap();
}
