//--------------------------------------------------------------------------------------------------
// Copyright (c) 2026                   orgrinrt                 ort@hiisi.digital
// SPDX-License-Identifier: MPL-2.0     https://mozilla.org/MPL/2.0        contact@hiisi.digital
//--------------------------------------------------------------------------------------------------

//! The lists the length targets count, shared between them.
//!
//! Type aliases and marker structs only, so this module is the same in every
//! configuration and the two paths are counting the same things rather than
//! two sets of lists that happen to be written alike.
//!
//! Not a test target: cargo discovers `tests/*.rs` and not `tests/*/mod.rs`.

#![allow(dead_code)]

use notko_hlist::{Cons, Empty};

pub struct A;
pub struct B;
pub struct C;
pub struct D;
pub struct E;

pub type L0 = Empty;
pub type L1 = Cons<A, Empty>;
pub type L2 = Cons<A, Cons<B, Empty>>;
pub type L3 = Cons<A, Cons<B, Cons<C, Empty>>>;
pub type L4 = Cons<A, Cons<B, Cons<C, Cons<D, Empty>>>>;
pub type L5 = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Empty>>>>>;

/// The same type three times over. A list is a list rather than a set, so this
/// is length three and not length one.
pub type Repeated = Cons<A, Cons<A, Cons<A, Empty>>>;

/// A head that is itself a list. Nothing bounds a head, so a nested list is an
/// ordinary member and contributes one to the length rather than its own.
pub type Nested = Cons<L3, Cons<L2, Empty>>;

/// A head that borrows. The heads are never constructed and a length is a fact
/// about types, so a head carrying a lifetime is an ordinary member. It is
/// worth counting because the phantom in a cell is `(H, T)` rather than
/// `fn() -> (H, T)`, which is what makes a cell covariant in its head and keeps
/// one from outliving what its head borrows.
pub struct Borrowing<'a>(pub &'a u8);
pub type WithBorrowing = Cons<Borrowing<'static>, Cons<A, Empty>>;

/// Eight cells in front of whatever `T` is, so a long list is built by nesting
/// rather than by writing thirty-two `Cons` and hoping the reader counts them
/// the same way twice.
pub type Eight<T> = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<A, Cons<B, Cons<C, T>>>>>>>>;

/// Long enough that the recursion is doing real work rather than unfolding
/// twice, and inside the default recursion limit, which is what a consumer
/// gets before it reaches for the raised one.
pub type L8 = Eight<Empty>;
pub type L32 = Eight<Eight<Eight<Eight<Empty>>>>;
