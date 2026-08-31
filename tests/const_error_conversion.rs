//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! Cross-error conversion through the const path, which the documentation said
//! was missing.
//!
//! `ConstFromResidual` carried a divergence note saying the const variant on
//! `Outcome` omitted the `F: From<E>` case, because `From` in a const bound was
//! not stable, and that a consumer wanting the conversion had to write
//! `Outcome::Err(e.into())` by hand. The bound is in the impl and has been for
//! a while. The note outlived it, which is what a note does.
//!
//! Only the const configuration has any of this, so the body sits in a module
//! that goes away when the feature is off, which is what a consumer building
//! without it gets. That arm cannot be run from inside this workspace: a
//! sibling crate dev-depends on `notko` and takes the defaults, and cargo
//! unifies features across the graph, so `--no-default-features` here still
//! leaves `const` on. `consttry_parity.rs` says the same thing from the other
//! side and is why parity is asserted rather than documented.

// An integration test is its own crate, so it turns the gates on itself. That
// is not incidental: it is what a consumer of the const path has to do too, and
// this file is the smallest demonstration of the whole arrangement.
#![cfg_attr(feature = "const", feature(const_trait_impl))]
#![cfg_attr(feature = "const", feature(const_convert))]

#[cfg(feature = "const")]
mod when_const_is_on {
    use core::convert::Infallible;

    use notko::{ConstFromResidual, Outcome};

    /// The error a step reports.
    #[derive(PartialEq, Eq, Debug)]
    pub struct Narrow(pub u32);

    /// The error the caller reports, which the narrow one converts into.
    #[derive(PartialEq, Eq, Debug)]
    pub struct Wide(pub u32);

    const impl From<Narrow> for Wide {
        fn from(n: Narrow) -> Self {
            Wide(n.0 + 100)
        }
    }

    /// The claim, and it is a compile-time one: this body runs in a const context
    /// and converts `Narrow` into `Wide` on the way out, which is exactly what the
    /// note said could not be written.
    const fn widen(residual: Outcome<Infallible, Narrow>) -> Outcome<u32, Wide> {
        <Outcome<u32, Wide> as ConstFromResidual<Outcome<Infallible, Narrow>>>::from_residual(
            residual,
        )
    }

    /// Evaluated at compile time rather than called, so the const-ness is the
    /// assertion rather than a claim about it.
    const WIDENED: Outcome<u32, Wide> = widen(Outcome::Err(Narrow(1)));

    #[test]
    fn a_const_context_converts_one_error_into_another() {
        assert_eq!(WIDENED, Outcome::Err(Wide(101)));
    }

    #[test]
    fn the_same_conversion_runs_outside_const_too() {
        // The control for the constant above. A `const` item that happened to be
        // folded from a runtime path would still read as evaluated at compile time
        // here, so the runtime arm is checked separately and has to agree.
        assert_eq!(widen(Outcome::Err(Narrow(41))), Outcome::Err(Wide(141)));
    }
}
