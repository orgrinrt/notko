//! Const-trait HasTrivialCtor declaration. See `ctor.rs` for the
//! cfg-gated module layout rationale.

/// The type ships a no-arg const constructor.
///
/// A type implementing `HasTrivialCtor` declares an associated
/// `const fn new() -> Self` callable in const contexts with no
/// arguments. Useful for marker types, phantom-data wrappers, and
/// any other type whose construction is a typestate signal rather
/// than runtime-data carrying.
///
/// The trait is fundamentally about a const-no-arg constructor
/// convention. Production wrappers in the workspace (Column, Virtual,
/// LinkedBin, and similar) impl it to give consumers a uniform
/// `Type::<...>::new()` spelling.
pub const trait HasTrivialCtor: Sized {
    /// Construct a value with no arguments.
    fn new() -> Self;
}
