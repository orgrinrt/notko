//! Sketch N3: one function under `#[profile(Hot)]`, asked the same question in
//! every build. See FINDINGS.md.
//!
//! The function admits a value at or under a limit and refuses one over it.
//! The refusal is an ordinary input-dependent error, the kind a debug build
//! hands back to its caller.

use notko::profile;

/// The refusal: the value was over the limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverLimit {
    pub value: u32,
    pub limit: u32,
}

/// Admits `value` when it is at or under `limit`.
#[profile(Hot)]
pub fn admit(value: u32, limit: u32) -> Result<u32, OverLimit> {
    if value > limit {
        return Err(OverLimit {
            value,
            limit,
        });
    }
    Ok(value)
}
