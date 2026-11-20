//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The call record, read back, and the verdict the hooks give a cell from it.

use std::path::PathBuf;

/// The header of the record, as the hooks library writes it into a run directory.
pub const LOG_HEADER: &str = "pid\tcall\tcolumns\tfixture_built\tothers_built\tabsent_before\tpresent_after\twaited_on_lock\texit\n";

/// The record of every call since the last reset.
pub fn log_path() -> PathBuf {
    crate::call::own_dir().join("calls.tsv")
}

/// Empty the record, before a run and after each cell has taken its copy.
pub fn reset_log() {
    let _ = std::fs::remove_file(log_path());
}

/// One call, read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub columns:       usize,
    pub fixture_built: u64,
    pub others_built:  u64,
    pub absent_before: bool,
    pub present_after: bool,
    pub waited:        bool,
    pub exit:          String,
}

fn parse(line: &str) -> Option<Call> {
    let f: Vec<&str> = line.split('\t').collect();
    (f.len() == 9).then(|| {
        Call {
            columns:       f[2].parse().unwrap_or(0),
            fixture_built: f[3].parse().unwrap_or(0),
            others_built:  f[4].parse().unwrap_or(u64::MAX),
            absent_before: f[5] == "true",
            present_after: f[6] == "true",
            waited:        f[7] == "true",
            exit:          f[8].to_string(),
        }
    })
}

/// Every call recorded since the last reset.
pub fn calls() -> Vec<Call> {
    std::fs::read_to_string(log_path())
        .unwrap_or_default()
        .lines()
        .filter_map(parse)
        .collect()
}

/// `Ok` with a note when every call built the fixture alone, from clean, at one point, without waiting on a lock
/// and with cargo succeeding; `Err` naming every call that did not, or saying no call was recorded.
pub fn verdict(calls: &[Call]) -> Result<String, String> {
    let Some(first) = calls.first() else {
        return Err("no call was recorded".into());
    };
    let bad: Vec<String> = calls
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.fixture_built != 1
                || c.others_built != 0
                || !c.absent_before
                || !c.present_after
                || c.waited
                || c.exit != "0"
                || c.columns != first.columns
        })
        .map(|(i, c)| format!("line {}: {c:?}", i + 1))
        .collect();
    if bad.is_empty() {
        Ok(format!(
            "{} calls, each built the fixture of {} columns alone from clean",
            calls.len(),
            first.columns
        ))
    } else {
        Err(format!(
            "calls that did not build the fixture alone from clean: {}",
            bad.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Call {
        Call {
            columns:       128,
            fixture_built: 1,
            others_built:  0,
            absent_before: true,
            present_after: true,
            waited:        false,
            exit:          "0".into(),
        }
    }

    #[test]
    fn calls_that_each_built_the_fixture_alone_pass() {
        assert!(verdict(&[good(), good(), good()]).is_ok());
    }

    #[test]
    fn no_call_fails() {
        assert!(verdict(&[]).is_err());
    }

    /// The case each check exists for: a call that found the fixture already built, one that rebuilt the product
    /// as well, one that started with the library present, one that left none, one that waited on the build
    /// directory's lock, one where cargo failed, and one from another point. Each fails the cell alone.
    #[test]
    fn every_call_that_did_not_build_the_fixture_alone_fails_the_cell() {
        let cases: [fn(&mut Call); 7] = [
            |c| c.fixture_built = 0,
            |c| c.others_built = 1,
            |c| c.absent_before = false,
            |c| c.present_after = false,
            |c| c.waited = true,
            |c| c.exit = "101".into(),
            |c| c.columns = 64,
        ];
        for (i, spoil) in cases.iter().enumerate() {
            let mut bad = good();
            spoil(&mut bad);
            assert!(verdict(&[good(), bad, good()]).is_err(), "case {i} passed");
        }
    }

    #[test]
    fn a_record_line_reads_back() {
        let line = "7\t1\t128\t1\t0\ttrue\ttrue\tfalse\t0";
        assert_eq!(parse(line), Some(good()));
        assert_eq!(parse("7\t1\t128"), None);
    }
}
