//! Ad-hoc spike for the stack audit of 14 November 2026. See FINDINGS.md.
//!
//! Two questions. Whether a caller of a `#[profile(Hot)]` function, written
//! the ordinary way, builds in both arms the attribute emits; and what one
//! fallible step costs in a loop under each carrier, inlined and not.

use notko::{Just, Maybe, Outcome, profile};

/// A small error, one word of payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Small {
    TooBig(u32),
}

/// A wide error, four words of payload, so a non-inlined return goes through
/// memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wide {
    pub at:    u64,
    pub value: u64,
    pub limit: u64,
    pub pad:   u64,
}

#[inline(always)]
fn body(x: u32) -> u32 {
    x.wrapping_mul(3).wrapping_add(1)
}

// ---------------------------------------------------------------------------
// The arms, written by hand so inlining is the only thing that differs
// between the two of each pair.
// ---------------------------------------------------------------------------

macro_rules! pair {
    ($inl:ident, $noinl:ident, ($x:ident, $lim:ident) -> $ret:ty $body:block) => {
        #[inline(always)]
        pub fn $inl($x: u32, $lim: u32) -> $ret $body
        #[inline(never)]
        pub fn $noinl($x: u32, $lim: u32) -> $ret $body
    };
}

pair!(raw_inl, raw_noinl, (x, _lim) -> u32 { body(x) });

pair!(result_inl, result_noinl, (x, lim) -> Result<u32, Small> {
    if x > lim { Err(Small::TooBig(x)) } else { Ok(body(x)) }
});

pair!(outcome_inl, outcome_noinl, (x, lim) -> Outcome<u32, Small> {
    if x > lim { Outcome::Err(Small::TooBig(x)) } else { Outcome::Ok(body(x)) }
});

pair!(option_inl, option_noinl, (x, lim) -> Option<u32> {
    if x > lim { None } else { Some(body(x)) }
});

pair!(maybe_inl, maybe_noinl, (x, lim) -> Maybe<u32> {
    if x > lim { Maybe::Isnt } else { Maybe::Is(body(x)) }
});

// What the release arm of `#[profile(Hot)]` emits for the same source: the
// check stays, the failure is a panic, the return is the bare value.
pair!(just_inl, just_noinl, (x, lim) -> Just<u32> {
    if x > lim { panic!("hot path invariant violated: {:?}", Small::TooBig(x)) } else { Just::new(body(x)) }
});

pair!(result_wide_inl, result_wide_noinl, (x, lim) -> Result<u32, Wide> {
    if x > lim {
        Err(Wide { at: 0, value: x as u64, limit: lim as u64, pad: 0 })
    } else {
        Ok(body(x))
    }
});

pair!(outcome_wide_inl, outcome_wide_noinl, (x, lim) -> Outcome<u32, Wide> {
    if x > lim {
        Outcome::Err(Wide { at: 0, value: x as u64, limit: lim as u64, pad: 0 })
    } else {
        Outcome::Ok(body(x))
    }
});

// ---------------------------------------------------------------------------
// The same source through the attribute itself. `Hot` carries `#[inline]`,
// `Cold` and `Warm` carry nothing, as `notko-macros-core/src/tiers.rs` sets
// them.
// ---------------------------------------------------------------------------

#[profile(Hot)]
pub fn macro_hot(x: u32, lim: u32) -> Result<u32, Small> {
    if x > lim {
        return Err(Small::TooBig(x));
    }
    Ok(body(x))
}

#[profile(Cold)]
pub fn macro_cold(x: u32, lim: u32) -> Result<u32, Small> {
    if x > lim {
        return Err(Small::TooBig(x));
    }
    Ok(body(x))
}

#[profile(Warm)]
pub fn macro_warm(x: u32, lim: u32) -> Result<u32, Small> {
    if x > lim {
        return Err(Small::TooBig(x));
    }
    Ok(body(x))
}

// ---------------------------------------------------------------------------
// The probes: callers of the hot function, each the ordinary way to call
// something whose source says `-> Result<u32, Small>`.
// ---------------------------------------------------------------------------

/// `?` on the hot function, from a function that returns an `Outcome`.
#[cfg(feature = "probe_question_mark")]
pub fn caller_question_mark(x: u32, lim: u32) -> Outcome<u32, Small> {
    let v = macro_hot(x, lim)?;
    Outcome::Ok(v + 1)
}

/// A `match` on the hot function's two outcomes.
#[cfg(feature = "probe_match")]
pub fn caller_match(x: u32, lim: u32) -> u32 {
    match macro_hot(x, lim) {
        Outcome::Ok(v) => v,
        Outcome::Err(_) => 0,
    }
}

/// The mirrored predicate, which builds in both arms. What it does in each is
/// the question: the binary `recover` prints it.
#[cfg(feature = "probe_is_err")]
pub fn caller_recovers(x: u32, lim: u32) -> u32 {
    let r = macro_hot(x, lim);
    if r.is_err() { 0 } else { r.unwrap() }
}
