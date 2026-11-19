# The product of a list and its carrier

notko-hlist builds a product over a list of members, one field per member, and the carrier decides what each field is: a value, a shared or exclusive column, an owned array. Until now the carrier picked one form for all members at once, over each member's `Elem::Value`, so a product could not hold a value for one member, a window for another and a borrowed resource for a third. A read list of mixed members needs exactly that, and hilavitkutin's read lists are the first consumer waiting on it.

This design takes the carrier as a parameter of the product trait, the route the sketch `research/sketches/202611191400_carrier-as-a-typestate-parameter/` showed working, and holds it to the condition the maintainer set on 16 November: "If the parameterisation is done via typestate, that's fine, and things remain exactly explicit and nowhere ambiguous or allowing ambiguousness". The carrier is a typestate, and every member's form is fixed by the types.

notko declares no canon, so this design stands as the governing document for the product, below the maintainer's words and above the code.

## The carrier is a typestate

A carrier is a marker type, and its type is its state. `Val` is one value per member, `Col<'a>` a shared column borrowed for `'a`, a consumer's `Read<'f>` whatever that consumer reads for the frame `'f`. Nothing about the state lives at run time, and nothing picks it there.

A carrier declares its shape once, through `Shape` with an associated type `Is`, which is `Uniformly` or `PerMember` and nothing else. The shape is part of the carrier's type, so what the carrier holds follows from the type alone.

A uniform carrier holds every `Elem` member the same way, over its value, through `Uniform`, which has a generic associated type `Of<E: Elem>` and requires `Shape<Is = Uniformly>`. It holds nothing for a member that is not an `Elem`. The four carriers this crate ships, `Val`, `Col`, `ColMut` and `Arr`, are uniform.

A per-member carrier states one form for each member it holds, through `Form<M>` with an associated type `Of`, which requires `Shape<Is = PerMember>`. An associated type has one value per impl and coherence allows one impl per carrier and member, so the form is a function of the two types and of nothing else. Two forms for one member are refused (`E0119`).

`Carrier<M>`, what a product reads, is this crate's alone: one blanket impl routes it through the carrier's shape to `Uniform::Of<M>` or to `Form<M>::Of`, and the trait is sealed. That is what keeps a crate that owns a member from deciding what a shipped carrier holds for it. The orphan rule would let such a crate implement a trait keyed on its own member for `Val`, but a `Form<M>` for `Val` is refused at `Val`'s shape, and `Carrier<M>` cannot be implemented outside this crate at all. So a carrier is uniform or per member and never both, and a reader of `Val` knows what it holds for every member without looking further.

## The product names its carrier and every member

`Project<C>` is the product of a list through the carrier `C`, with the product type as `Prod`. It is sealed. `Empty` has `PNil` through every carrier, and `Cons<H, T>` has `PCons<C, H, <T as Project<C>>::Prod>` wherever `C: Carrier<H>` and `T: Project<C>`. A carrier with no form for one of the list's members has no product for that list, and the error names the member.

The cell `PCons<C, H, T>` holds `<C as Carrier<H>>::Of` and names both `C` and `H` in its own type. So a product never stands for another list or another carrier, even where the fields coincide: the product of `Cons<Pos, Empty>` through `Val` is not the product of `Cons<Depth, Empty>` through `Val` although both hold one `i32`, and it is not the product of `Cons<Pos, Empty>` through a consumer carrier that also holds an `i32` for `Pos`. Neither is ever inferred from what the fields hold.

A member is any type. `Elem` is what a uniform carrier asks of a member, since it builds the form over `Elem::Value`, and a per-member carrier asks nothing beyond its own `Form` impls. A resource that is not `Copy`, held by reference for a frame, is a member of a read list without being an `Elem`.

## Reaching into a product

`Has<C, M, P>` is the field of member `M` in a product through `C`, at the place `P` that `PlaceOf` answers, with the same refusals: no place when the member is absent, an ambiguous one when it is there twice. The carrier is a parameter of the trait and not of the method, because the recursion to the tail has to know that the tail has a product through the same carrier, and only a trait parameter can carry that into the impl.

`Vals<L>` is the product of values at one row, through `Val`, read and written by naming the member, as before.

`SubOf<C, S, Ps>` cuts the product of a narrower list out of the product of a wider one through the same carrier, each field duplicated. Duplication is `Dup<M>`, what the cut reads, and it is routed by the carrier's shape the way `Carrier<M>` is and sealed the same way. A uniform carrier duplicates every member or none, through `DupAll`, which has no member parameter: `Val` and `Col` duplicate every `Elem` member, `ColMut` and `Arr` none. A per-member carrier states `DupForm<M>` for each form it can duplicate, which requires the per-member shape, and that holds where the carrier and the member live in two crates, the arrangement a scheduler's read carrier and a consumer's members have. No crate can make `Arr` duplicate a member, and no uniform carrier can duplicate one member and not the rest.

## Walks between two carriers

The product's own walks move between the products of one list through two carriers: a row of values read out of columns or arrays and written back, a product of arrays seen as columns, a product of arrays built from one row. Each is a trait that takes its carriers as parameters and names both products as supertraits, which is what lets an impl for `Cons<H, T>` see the tail's two products.

- `Rows<C>`, over a uniform `C`: `read_row` where `C: ByRow`, `write_row` where `C: ByRowMut`, between `C` and `Val`.
- `Arrays<N>`, over `Arr<N>`: `cols` and `cols_mut` to `Col` and `ColMut` through a higher-ranked supertrait, `copy_from`, and `made` with `splat_in` from a row of `Val`.
- `PutRow<C, S, Ps>` and `PutVals<S, Ps>`: a narrower list's row of values written into a wider list's product, through `C` at a row and through `Val`.
- `Split`, the product of exclusive columns cut in two by rows, as before.

All of them are sealed, and all but `Split` have a const twin under the `const` feature, as the shipped walks do. A walk is reached through the trait that names its carriers, `<L as Rows<Arr<4>>>::read_row(&p, 2)`, where it used to be `L::read_row::<Arr<4>>(&p, 2)`.

## What this replaces

The shipped `Carrier`, a generic associated type over `Elem`, is renamed `Uniform`, with `Shape` and `Form<M>` beside it, and `Carrier<M>` is the sealed trait the product reads in its place. `Project` with its generic associated type `Prod<C: Carrier>` and its walk methods becomes `Project<C>` with `Prod`, and the walks move to the traits above. `Has<M, P>` becomes `Has<C, M, P>`, `SubOf<S, Ps>` becomes `SubOf<C, S, Ps>`, and its two writes move to `PutRow` and `PutVals`. `Dup` takes the member, and `DupAll` is what a uniform carrier implements. Nothing of the old shape is kept beside the new one. No crate in the workspace calls the product today, so nothing downstream breaks.

## What is not decided here

Whether a frame lifetime may sit on a member rather than on the carrier is left as the earlier sketch left it: it puts the lifetime in the list type and does not build as a small change (`research/sketches/202611152220_read-product-borrowed/FINDINGS.md`, attempt 4). The carrier is where the frame lives in this design.
