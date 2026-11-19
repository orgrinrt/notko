//! Attempt 5: a product whose carrier is a parameter of the trait.
//!
//! A model written in this sketch, over the shipped crate's list, places and
//! `PlaceOf`. It is not in notko-hlist. The shipped `Project::Prod<C: Carrier>`
//! is a generic associated type, and `Carrier::Of<E: Elem>` fixes what a carrier
//! may ask of a member to `Elem` alone (attempt 3). Here the carrier says, per
//! member, what it holds, as an impl of `Form<M>` with whatever bounds it
//! wants, and the product is found for a list and a carrier together.

use notko_hlist::{Cons, Empty, Here, List, Place, PlaceOf, There};

/// Carrier `Self` holds `Of` for member `M`.
pub trait Form<M> {
    /// What is held for `M`.
    type Of: Copy;
}

/// A cell of the product: the head member's field, then the rest.
#[derive(Clone, Copy)]
pub struct Cell<V, T> {
    pub head: V,
    pub tail: T,
}

/// The end of the product.
#[derive(Clone, Copy)]
pub struct End;

/// The product of a list through the carrier `C`.
pub trait ProjectBy<C>: List {
    /// `Cell` per member, `End` at the end.
    type Prod: Copy;
}

impl<C> ProjectBy<C> for Empty {
    type Prod = End;
}

impl<C: Form<H>, H, T: ProjectBy<C>> ProjectBy<C> for Cons<H, T> {
    type Prod = Cell<<C as Form<H>>::Of, T::Prod>;
}

/// The field of member `M` in a product of `Self` through `C`, at place `P`.
pub trait HasBy<C: Form<M>, M, P: Place>: ProjectBy<C> + PlaceOf<M, P> {
    /// The field of `M` in `p`.
    fn pick(p: &Self::Prod) -> <C as Form<M>>::Of;
}

impl<C: Form<M>, M, T: ProjectBy<C>> HasBy<C, M, Here> for Cons<M, T> {
    fn pick(p: &Self::Prod) -> <C as Form<M>>::Of {
        p.head
    }
}

impl<C: Form<H> + Form<M>, H, M, P: Place, T: HasBy<C, M, P>> HasBy<C, M, There<P>> for Cons<H, T> {
    fn pick(p: &Self::Prod) -> <C as Form<M>>::Of {
        T::pick(&p.tail)
    }
}
