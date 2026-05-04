//! [`Maybe<T>`]: presence (replaces `Option<T>`).

use core::fmt;

/// Presence. Either carries a value ([`Maybe::Is`]) or doesn't ([`Maybe::Isnt`]).
///
/// Replaces `core::option::Option<T>` in the hilavitkutin stack's public APIs.
/// No dependency on `core::option`.
///
/// # Layout
///
/// `Maybe<T>` is a plain Rust-repr two-variant enum. When `T` has a niche
/// (function pointers, references `&T` / `&mut T`, `NonZero*`, `NonNull<T>`),
/// rustc applies niche-filling: `Maybe<T>` has identical size and alignment
/// to `T` itself, and `Maybe::Isnt` is represented by `T`'s invalid bit
/// pattern (null for pointer-shaped types). This is the same optimization
/// that gives `Option<T>` its null-niche layout for pointer types.
///
/// The optimization is applied automatically by the compiler on any
/// 2-variant enum with one unit variant and one payload-carrying variant
/// whose payload type has a niche. Size parity with the underlying
/// pointer type is verified at compile time below for the shapes the
/// stack's FFI boundaries rely on. If a future rustc changes its
/// niche-filling behavior for user enums, those assertions fail
/// compilation immediately.
///
/// The previous `#[repr(C)]` marker is removed: it was forcing a tagged
/// union layout (explicit discriminant + payload + padding) that blocked
/// niche-filling, and no consumer depended on the resulting layout.
/// Types that need explicit C-ABI representation at FFI boundaries
/// should use a `#[repr(C)]` struct with the shape they actually need,
/// not `Maybe` (tagged unions are not a native C construct anyway).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Maybe<T> {
    Is(T),
    Isnt,
}

// The layout contract lives on [`MaybeNull<T>`] further down, not on
// [`Maybe<T>`]. `Maybe` is the general vocabulary type; its layout
// depends on whether `T` happens to carry a niche. `MaybeNull<T>` is the
// FFI-intent wrapper that pins the layout per instantiation via
// sealed-trait bound + const assertion.

impl<T> Maybe<T> {
    /// `true` if this is [`Maybe::Is`].
    #[inline]
    pub const fn is(&self) -> bool {
        matches!(self, Maybe::Is(_))
    }

    /// `true` if this is [`Maybe::Isnt`].
    #[inline]
    pub const fn isnt(&self) -> bool {
        matches!(self, Maybe::Isnt)
    }

    /// Convert by value into a `&Maybe<&T>` (zero-cost).
    #[inline]
    pub const fn as_ref(&self) -> Maybe<&T> {
        match self {
            Maybe::Is(value) => Maybe::Is(value),
            Maybe::Isnt => Maybe::Isnt,
        }
    }

    /// Extract the inner value; panic if absent.
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Maybe::Is(value) => value,
            Maybe::Isnt => panic!("called `Maybe::unwrap` on an `Isnt` value"),
        }
    }

    /// Extract the inner value or return `fallback`.
    #[inline]
    pub fn unwrap_or(self, fallback: T) -> T {
        match self {
            Maybe::Is(value) => value,
            Maybe::Isnt => fallback,
        }
    }

    /// Map the inner value if present.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Maybe<U> {
        match self {
            Maybe::Is(value) => Maybe::Is(f(value)),
            Maybe::Isnt => Maybe::Isnt,
        }
    }

    /// Convert to [`crate::Outcome`], using `err` if [`Maybe::Isnt`].
    ///
    /// Mirrors `Option::ok_or` for the substrate vocabulary. The eager
    /// form takes `err` by value; for a closure-deferred form pair this
    /// with a manual `match` until `ok_or_else` ships.
    #[inline]
    pub fn ok_or<E>(self, err: E) -> crate::Outcome<T, E> {
        match self {
            Maybe::Is(value) => crate::Outcome::Ok(value),
            Maybe::Isnt => crate::Outcome::Err(err),
        }
    }

    /// Unwrap with a custom panic message.
    ///
    /// Panics with `msg` if [`Maybe::Isnt`]. Use over [`Maybe::unwrap`]
    /// when the panic context names the invariant that was violated.
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Maybe::Is(value) => value,
            Maybe::Isnt => panic!("{}", msg),
        }
    }
}

impl<T> Default for Maybe<T> {
    #[inline]
    fn default() -> Self {
        Maybe::Isnt
    }
}

impl<T: fmt::Debug> fmt::Debug for Maybe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Maybe::Is(value) => f.debug_tuple("Is").field(value).finish(),
            Maybe::Isnt => f.write_str("Isnt"),
        }
    }
}

#[cfg(feature = "try_trait_v2")]
mod try_impl {
    use super::Maybe;
    use core::convert::Infallible;
    use core::ops::{ControlFlow, FromResidual, Try};

    impl<T> Try for Maybe<T> {
        type Output = T;
        type Residual = Maybe<Infallible>;

        #[inline]
        fn from_output(output: Self::Output) -> Self {
            Maybe::Is(output)
        }

        #[inline]
        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                Maybe::Is(value) => ControlFlow::Continue(value),
                Maybe::Isnt => ControlFlow::Break(Maybe::Isnt),
            }
        }
    }

    impl<T> FromResidual<Maybe<Infallible>> for Maybe<T> {
        #[inline]
        fn from_residual(residual: Maybe<Infallible>) -> Self {
            match residual {
                Maybe::Isnt => Maybe::Isnt,
                Maybe::Is(never) => match never {},
            }
        }
    }
}

#[cfg(feature = "const")]
#[path = "maybe_consttry_const.rs"]
mod consttry_const_impl;

#[cfg(not(feature = "const"))]
#[path = "maybe_consttry_plain.rs"]
mod consttry_plain_impl;

mod niche {
    //! Sealed marker trait for types that carry a bit-pattern-zero niche.
    //!
    //! Rustc's niche-filling optimisation applies to any 2-variant enum
    //! where one variant is unit and the other carries a type with an
    //! invalid bit pattern. This module enumerates the types the stack
    //! relies on as having a documented all-zeros invalid bit pattern:
    //! function pointers (non-null by language contract), references
    //! (non-null by language contract), [`core::ptr::NonNull`], and
    //! [`core::num::NonZero*`]. In every case the niche value is the
    //! all-zeros bit pattern (null pointer or integer zero), so a
    //! single sealed trait captures the full set.
    //!
    //! The trait is sealed: `Sealed` is private to this module, and no
    //! downstream crate can impl it. The exported [`NicheFilled`]
    //! supertrait shows the bound shape at the call site but cannot be
    //! implemented outside notko.

    trait Sealed {}
    /// Marker: `T` carries a bit-pattern-zero niche. Sealed; downstream
    /// crates cannot extend.
    ///
    /// The sealed-trait pattern deliberately uses a private supertrait
    /// (`Sealed`). `#[allow(private_bounds)]` suppresses rustc's
    /// warning about the public trait having a private supertrait:
    /// that is exactly the point. External code sees `NicheFilled` as the
    /// bound, cannot reach `Sealed`, and therefore cannot impl
    /// `NicheFilled` for new types.
    #[allow(private_bounds)]
    pub trait NicheFilled: Sealed {}

    // References.
    impl<T: ?Sized> Sealed for &T {}
    impl<T: ?Sized> NicheFilled for &T {}
    impl<T: ?Sized> Sealed for &mut T {}
    impl<T: ?Sized> NicheFilled for &mut T {}

    // NonNull.
    impl<T: ?Sized> Sealed for core::ptr::NonNull<T> {}
    impl<T: ?Sized> NicheFilled for core::ptr::NonNull<T> {}

    // NonZero integers (unsigned + signed + usize + isize).
    macro_rules! impl_nz {
        ($($ty:path),* $(,)?) => {
            $(
                impl Sealed for $ty {}
                impl NicheFilled for $ty {}
            )*
        };
    }
    impl_nz!(
        core::num::NonZeroU8,
        core::num::NonZeroU16,
        core::num::NonZeroU32,
        core::num::NonZeroU64,
        core::num::NonZeroU128,
        core::num::NonZeroUsize,
        core::num::NonZeroI8,
        core::num::NonZeroI16,
        core::num::NonZeroI32,
        core::num::NonZeroI64,
        core::num::NonZeroI128,
        core::num::NonZeroIsize,
    );

    // Function pointers at arities 0..=8 with every qualifier
    // combination (safe / unsafe x plain / extern "C").
    macro_rules! impl_fn_niche {
        ($($args:ident),*) => {
            impl<R, $($args),*> Sealed for fn($($args),*) -> R {}
            impl<R, $($args),*> NicheFilled for fn($($args),*) -> R {}
            impl<R, $($args),*> Sealed for unsafe fn($($args),*) -> R {}
            impl<R, $($args),*> NicheFilled for unsafe fn($($args),*) -> R {}
            impl<R, $($args),*> Sealed for extern "C" fn($($args),*) -> R {}
            impl<R, $($args),*> NicheFilled for extern "C" fn($($args),*) -> R {}
            impl<R, $($args),*> Sealed for unsafe extern "C" fn($($args),*) -> R {}
            impl<R, $($args),*> NicheFilled for unsafe extern "C" fn($($args),*) -> R {}
        };
    }
    impl_fn_niche!();
    impl_fn_niche!(A);
    impl_fn_niche!(A, B);
    impl_fn_niche!(A, B, C);
    impl_fn_niche!(A, B, C, D);
    impl_fn_niche!(A, B, C, D, E);
    impl_fn_niche!(A, B, C, D, E, F);
    impl_fn_niche!(A, B, C, D, E, F, G);
    impl_fn_niche!(A, B, C, D, E, F, G, H);
}

pub use niche::NicheFilled;

/// Size-parity wrapper for [`Maybe<T>`] where `T` carries a niche.
///
/// `MaybeNull<T>` is a `#[repr(transparent)]` newtype over [`Maybe<T>`]
/// constrained by the sealed [`NicheFilled`] trait. `T` must be one of:
///
/// - A function pointer type (`fn(...)`, `unsafe fn(...)`,
///   `extern "C" fn(...)`, `unsafe extern "C" fn(...)`) at arity
///   0..=8. Niche: the null function pointer bit pattern.
/// - A reference `&T` or `&mut T`. Niche: the null pointer bit pattern.
/// - [`core::ptr::NonNull<T>`]. Niche: null.
/// - [`core::num::NonZeroU*`] / [`core::num::NonZeroI*`] /
///   `NonZeroUsize` / `NonZeroIsize`. Niche: integer zero.
///
/// Every case shares the same invalid bit pattern: all zeros. MaybeNull's
/// [`MaybeNull::null`] constructor uses that pattern, giving `MaybeNull<T>`
/// the same in-memory size as `T` itself. Instantiating `MaybeNull<T>`
/// with a `T` outside the set fails to compile on the trait bound.
///
/// The `#[repr(transparent)]` marker makes `MaybeNull<T>` identical in
/// size, alignment, and ABI to `Maybe<T>`, which for niche-carrying
/// `T` is identical to `T`. A per-instantiation const assertion
/// verifies `size_of::<MaybeNull<T>>() == size_of::<T>()` at compile time.
///
/// Use at FFI boundaries where the pointer-sized-or-integer-sized
/// nullable representation IS the point:
///
/// ```ignore
/// use notko::MaybeNull;
///
/// #[repr(C)]
/// pub struct Descriptor {
///     pub init_fn: MaybeNull<unsafe extern "C" fn() -> u32>,
/// }
/// ```
///
/// For general-purpose presence without an FFI boundary, use
/// [`Maybe<T>`] directly.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MaybeNull<T: NicheFilled>(Maybe<T>);

impl<T: NicheFilled> MaybeNull<T> {
    /// Compile-time layout assertion evaluated per instantiation.
    /// Referenced by [`Self::new`] and [`Self::null`] to force
    /// evaluation so a size regression breaks the build at the
    /// introduction site.
    const _LAYOUT_ASSERT: () = assert!(
        core::mem::size_of::<MaybeNull<T>>() == core::mem::size_of::<T>(),
        "MaybeNull<T> layout regression: niche-filling does not apply to T",
    );

    /// Construct the null variant: the all-zeros bit pattern (null for
    /// pointer-shaped `T`, integer zero for `NonZero*`).
    #[inline]
    pub const fn null() -> Self {
        let _ = Self::_LAYOUT_ASSERT;
        Self(Maybe::Isnt)
    }

    /// Wrap `value`. Round-trips through the underlying layout without
    /// discriminant overhead.
    #[inline]
    pub const fn new(value: T) -> Self {
        let _ = Self::_LAYOUT_ASSERT;
        Self(Maybe::Is(value))
    }

    /// `true` if this is the null variant.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0.isnt()
    }

    /// `true` if this carries a value.
    #[inline]
    pub const fn is_non_null(&self) -> bool {
        self.0.is()
    }

    /// View as the underlying [`Maybe`].
    #[inline]
    pub const fn as_maybe(&self) -> &Maybe<T> {
        &self.0
    }

    /// Consume into the underlying [`Maybe`].
    #[inline]
    pub fn into_maybe(self) -> Maybe<T> {
        self.0
    }
}

impl<T: NicheFilled> From<Maybe<T>> for MaybeNull<T> {
    #[inline]
    fn from(m: Maybe<T>) -> Self {
        Self(m)
    }
}

impl<T: NicheFilled> From<MaybeNull<T>> for Maybe<T> {
    #[inline]
    fn from(n: MaybeNull<T>) -> Self {
        n.0
    }
}

// Per-shape layout verification pinned to `MaybeNull<T>` (not `Maybe<T>`).
// Each line forces const evaluation of `MaybeNull::<T>::_LAYOUT_ASSERT`
// for a specific T, so a size regression surfaces at compile time at
// notko's foundation rather than at a far-away FFI crash site.
const _: () = MaybeNull::<unsafe extern "C" fn()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<extern "C" fn()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<unsafe fn()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<fn()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<&'static ()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<&'static mut ()>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::ptr::NonNull<()>>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroU8>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroU16>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroU32>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroU64>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroU128>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroUsize>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroI8>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroI16>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroI32>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroI64>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroI128>::_LAYOUT_ASSERT;
const _: () = MaybeNull::<core::num::NonZeroIsize>::_LAYOUT_ASSERT;

#[cfg(test)]
mod niche_layout_tests {
    use super::*;

    /// `Maybe::Isnt` for a function-pointer payload is the null bit
    /// pattern. This is what allows `Maybe<unsafe extern "C" fn(...)>`
    /// to cross a C ABI boundary as a single nullable function pointer,
    /// matching the historical `Option<fn>` idiom.
    #[test]
    fn maybe_fn_isnt_is_null_bit_pattern() {
        let m: Maybe<unsafe extern "C" fn()> = Maybe::Isnt;
        // SAFETY: compile-time assertions above verify that
        // `Maybe<fn>` has the same size + alignment as `*const ()`.
        // Transmuting reads the bit pattern.
        let bits: usize = unsafe { core::mem::transmute(m) };
        assert_eq!(
            bits, 0,
            "Maybe::Isnt for unsafe extern \"C\" fn() must be null",
        );
    }

    #[test]
    fn maybe_ref_isnt_is_null_bit_pattern() {
        let m: Maybe<&'static u32> = Maybe::Isnt;
        let bits: usize = unsafe { core::mem::transmute(m) };
        assert_eq!(bits, 0, "Maybe::Isnt for &T must be null");
    }

    #[test]
    fn maybe_nonnull_isnt_is_null_bit_pattern() {
        let m: Maybe<core::ptr::NonNull<u32>> = Maybe::Isnt;
        let bits: usize = unsafe { core::mem::transmute(m) };
        assert_eq!(bits, 0, "Maybe::Isnt for NonNull<T> must be null");
    }

    #[test]
    fn maybe_nonzero_isnt_is_zero_bit_pattern() {
        let m: Maybe<core::num::NonZeroU32> = Maybe::Isnt;
        let bits: u32 = unsafe { core::mem::transmute(m) };
        assert_eq!(bits, 0, "Maybe::Isnt for NonZeroU32 must be zero");
    }

    /// Round-trip: a `Some`-analog value transmuted through Maybe
    /// recovers the original bit pattern. Proves `Maybe::Is(v)` is
    /// just `v`'s bit pattern when niche-filling applies.
    #[test]
    fn maybe_fn_is_roundtrips_fn_pointer() {
        unsafe extern "C" fn marker() {}
        let original = marker as unsafe extern "C" fn();
        let m: Maybe<unsafe extern "C" fn()> = Maybe::Is(original);
        let bits: usize = unsafe { core::mem::transmute(m) };
        assert_ne!(bits, 0, "Maybe::Is(fn) must not be null");
        assert_eq!(
            bits, marker as usize,
            "Maybe::Is(fn) must be fn's bit pattern",
        );
    }

    /// [`MaybeNull<T>`] is `#[repr(transparent)]` over `Maybe<T>` and
    /// inherits its niche layout.
    #[test]
    fn maybe_null_fn_has_pointer_layout() {
        let n = MaybeNull::<unsafe extern "C" fn()>::null();
        assert_eq!(
            core::mem::size_of_val(&n),
            core::mem::size_of::<*const ()>(),
        );
        let bits: usize = unsafe { core::mem::transmute(n) };
        assert_eq!(bits, 0);
    }

    #[test]
    fn maybe_null_nonzero_has_integer_layout() {
        let n = MaybeNull::<core::num::NonZeroU32>::null();
        assert_eq!(
            core::mem::size_of_val(&n),
            core::mem::size_of::<u32>(),
        );
        let bits: u32 = unsafe { core::mem::transmute(n) };
        assert_eq!(bits, 0);
    }

    #[test]
    fn maybe_null_from_maybe_roundtrip() {
        let m: Maybe<&'static u32> = Maybe::Isnt;
        let n: MaybeNull<&'static u32> = MaybeNull::from(m);
        assert!(n.is_null());
        let back: Maybe<&'static u32> = n.into_maybe();
        assert_eq!(back, Maybe::Isnt);
    }

    // Compile-fail boundary documentation. Uncommenting these should
    // fail with "the trait bound `u32: NicheFilled` is not satisfied"
    // and similar. The sealed trait keeps the set closed.
    //
    // fn _reject_u32() { let _ = MaybeNull::<u32>::null(); }
    // fn _reject_struct() {
    //     pub struct Plain(u32);
    //     let _ = MaybeNull::<Plain>::null();
    // }
    // fn _reject_i64() { let _ = MaybeNull::<i64>::null(); }
}
