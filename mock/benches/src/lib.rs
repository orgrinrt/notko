//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The hooks of notko's bench tree. `on_init` empties the call record of `product-build`, and `after_cell` copies it
//! into the cell's run directory as `calls.tsv` and refuses the cell when any call did not build the fixture alone
//! from clean (`product_build_call::verdict`).

use mockspace_bench_harness::driver::{AfterCell, CellVerdict, Hooks, InitContext, InitVerdict};

fn on_init(_: &InitContext<'_>) -> InitVerdict {
    product_build_call::reset_log();
    InitVerdict::Proceed
}

fn after_cell(cell: &AfterCell<'_>) -> CellVerdict {
    let text = std::fs::read_to_string(product_build_call::log_path()).unwrap_or_default();
    let calls = product_build_call::calls();
    product_build_call::reset_log();
    if let Err(e) = std::fs::write(
        cell.out_dir.join("calls.tsv"),
        format!("{}{text}", product_build_call::LOG_HEADER),
    ) {
        return CellVerdict::Fail(format!("calls.tsv was not written: {e}"));
    }
    match product_build_call::verdict(&calls) {
        Ok(note) => CellVerdict::Note(note),
        Err(why) => CellVerdict::Fail(why),
    }
}

pub fn hooks() -> Hooks {
    Hooks {
        on_init: Some(on_init),
        after_cell: Some(after_cell),
        ..Hooks::default()
    }
}
