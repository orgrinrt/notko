//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! What `product-build` does on each call, and what its hooks read back.
//!
//! The fixture crate beside the bench includes a source file this crate writes for the call's point: a registry of
//! that many columns and what a consumer asks of the product over it (`source`). Before the timed build, untimed,
//! the source is written, `notko-hlist` is brought up to date and the fixture package alone is cleaned, so the
//! timed build compiles the fixture and nothing else (`prepare`). The timed block is one `cargo build` of the
//! fixture (`build`). After it, untimed, cargo's own messages say what it compiled and one line goes to the call
//! record (`record`). The hooks library reads the record back and refuses a cell where any call built other than
//! the fixture alone (`verdict`).
//!
//! Everything sits under the build directory the run inherits (`CARGO_TARGET_DIR`), in `notko-product-build/`, so
//! cleaning the fixture never touches the profile directories the harness built its driver and arms into.

mod call;
mod record;
mod source;

pub use call::{build, fixture_dir, prepare, record};
pub use record::{Call, LOG_HEADER, calls, log_path, reset_log, verdict};
pub use source::{columns_of_system, source};
