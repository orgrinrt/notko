//! Const-callable parallels of `core::ops::Try` and `core::ops::FromResidual`.
//!
//! `core::ops::Try` is not `pub const trait` as of 2026-05 nightly, so
//! `?` on `Just<T>`, `Maybe<T>`, `Outcome<T, E>`, and `Bool` cannot be
//! used in const context. This module ships a substrate-internal
//! parallel surface: explicit `match x.branch() { ... }` calls work in
//! const fn bodies.
//!
//! `?` syntax itself stays non-const because rustc desugars `?`
//! directly to `core::ops::Try::branch`. Adopting `ConstTry` does not
//! change `?`-syntax behaviour. When core eventually lifts `Try` to
//! `pub const trait Try`, this bridge can be removed and existing
//! `impl Try` blocks rewritten to `impl const Try` (no consumer-API
//! migration required since `?` desugaring switches automatically).
//!
//! Both traits are gated behind feature `const` (default-on). Without
//! the feature, the traits exist as regular `pub trait`s; impls drop
//! the `const` keyword. This lets notko consumers on stable Rust opt
//! out of the const-trait machinery via `default-features = false`.
//!
//! The const-variant impls in `just.rs` / `maybe.rs` / `outcome.rs`
//! carry an extra `T: Copy` bound (and `E: Copy` for Outcome). The
//! restriction exists because const fn cannot evaluate destructors
//! for arbitrary generic `T` under current rustc nightly. Consumers
//! pushing non-Copy types through `branch` / `from_output` reach for
//! the non-const variant via `default-features = false`. Substrate
//! consumers (Bool, USize, Cap, Bits, NUSize) are all Copy, so the
//! restriction is invisible at the typical call site.

use core::ops::ControlFlow;

#[cfg(feature = "const")]
/// Const-callable parallel of `core::ops::Try`.
pub const trait ConstTry {
    /// The "successful" type emerging from the `?` operator.
    type Output;

    /// The "residual" type carrying the early-return information.
    type Residual;

    /// Construct the value back from a successful Output.
    fn from_output(output: Self::Output) -> Self;

    /// Decide whether to short-circuit (Break) or continue (Continue).
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

#[cfg(not(feature = "const"))]
/// Const-callable parallel of `core::ops::Try`.
pub trait ConstTry {
    /// The "successful" type emerging from the `?` operator.
    type Output;

    /// The "residual" type carrying the early-return information.
    type Residual;

    /// Construct the value back from a successful Output.
    fn from_output(output: Self::Output) -> Self;

    /// Decide whether to short-circuit (Break) or continue (Continue).
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

#[cfg(feature = "const")]
/// Const-callable parallel of `core::ops::FromResidual`.
pub const trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    /// Construct Self from a residual value.
    fn from_residual(residual: R) -> Self;
}

#[cfg(not(feature = "const"))]
/// Const-callable parallel of `core::ops::FromResidual`.
pub trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    /// Construct Self from a residual value.
    fn from_residual(residual: R) -> Self;
}
