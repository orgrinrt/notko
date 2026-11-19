// PROBE p02-e: NEGATIVE CONTROL for p02-d.
// A CONST caller doing a cross-conversion whose `From` impl is NOT const.
// This MUST be refused. If it compiles, the [const] From bound in p02-d is
// vacuous and that probe proved nothing.
#![feature(const_trait_impl, const_destruct, const_convert)]
#![no_std]
#![crate_type = "lib"]
use core::convert::Infallible;
use core::marker::Destruct;
pub enum Outcome<T, E> { Ok(T), Err(E) }
pub const trait ConstTry { type Output; type Residual;
    fn from_output(o: Self::Output) -> Self;
    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output>; }
pub const trait ConstFromResidual<R = <Self as ConstTry>::Residual> {
    fn from_residual(residual: R) -> Self; }
const impl<T, E, F: [const] From<E> + [const] Destruct> ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, F>
where E: [const] Destruct {
    fn from_residual(r: Outcome<Infallible, E>) -> Self {
        match r { Outcome::Err(e) => Outcome::Err(F::from(e)), Outcome::Ok(n) => match n {} }
    }
}
pub struct ErrA(pub u32);
pub struct ErrB(pub u32);
impl From<ErrA> for ErrB { fn from(a: ErrA) -> ErrB { ErrB(a.0) } }  // NOT const

// must be REFUSED: const context, non-const From
pub const fn cross_const_nonconst_from(r: Outcome<Infallible, ErrA>) -> Outcome<u8, ErrB> {
    <Outcome<u8, ErrB> as ConstFromResidual<Outcome<Infallible, ErrA>>>::from_residual(r)
}
