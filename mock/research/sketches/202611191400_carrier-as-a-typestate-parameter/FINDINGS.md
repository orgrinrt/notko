# Sketch: the product's carrier as a typestate parameter of the product trait

Hypothesis: notko-hlist's product can take its carrier as a parameter of the product trait so that a carrier states a form per member, and the parameterisation stays typestate throughout: every member's form is fixed by the types, no form can be stated twice or overridden, and no product stands for another list or another carrier.

Outcome: WORKS, for the model in `model/` over the shipped list, read from a second crate, `consumer/`. Every refusal the hypothesis names is a compile error, each shown below.

This is a feasibility sketch written on 19 November 2026 for the notko-hlist product round. It starts from `research/sketches/202611152220_read-product-borrowed/` (attempt 5 there), which showed a form per member is expressible beside the shipped list and left the typestate question and the walks open. Nothing here is timed, and nothing here is authority.

## The condition it is checked against

The maintainer's call on 16 November, recorded in clause-dev `.shared/research/202611150830_stack-path/57-op-calls-on-the-night.md`, call 2: "This sounds a whole lot like compromising the typestate if not done properly. If the parameterisation is done via typestate, that's fine, and things remain exactly explicit and nowhere ambiguous or allowing ambiguousness". Read as the condition: the carrier may become a parameter only where every member's form is stated in the types, with nowhere the form could be ambiguous.

## What the model is

`model/src/lib.rs`, over the shipped `Cons`, `Empty`, `List`, `Elem`, `Place`, `PlaceOf`, `Here` and `There`.

- `Carrier<M>`: carrier `Self` holds `Of` for member `M`. An associated type, so one impl per pair and one form per pair.
- `Uniform`: the shipped carrier trait as it is, a generic associated type `Of<E: Elem>`, one form for every `Elem` member over its value. A blanket impl makes every uniform carrier a `Carrier<E>` for every `E: Elem`. `Val`, `Col<'a>` and `Arr<N>` are uniform.
- `PCons<C: Carrier<H>, H, T>`: the cell, holding `<C as Carrier<H>>::Of`. It names its carrier and its member, as the shipped `PCons<C, H, T>` already does.
- `Project<C>: List`, sealed, with `type Prod`: the product of a list through `C`. `Cons<H, T>` has one where `C: Carrier<H>` and `T: Project<C>`. No bound on `H` itself, so a member that is not an `Elem` can be in a list a per-member carrier holds.
- `Has<C, M, P>`: the field of `M` through `C`, by place, const.
- The walks between two carriers' products, on traits that take the carriers as parameters and name both products as supertraits: `Rows<C: Uniform>: Project<C> + Project<Val>`, const, and `Cols<N>: Project<Arr<N>> + for<'a> Project<Col<'a>>`, higher-ranked.

## Attempts

Run by `run.sh`, toolchain `nightly-2026-05-28`, dev profile, notko-hlist at `1b2a572` with its default features.

### w1. A form per member, the uniform carriers and the walks. Works.

`consumer/src/bin/w1_form_per_member.rs`, output `out_w1_form_per_member.txt`. One read list of three members through the consumer's `Read<'f>`, a value, a window on a column and a reference to a resource that is not `Copy`, read back by member; then the model's `Arr<3>` read at a row in a `const` item, and the column walk from `Arr<2>` to `Col` read at a row.

```
resource read by member: [5, 6, 7] of 3
window read by member: centre 2 is 6
value read by member: 9
row read at compile time: 2 8
row through the column walk: 20 2
the frame ended and the read product with it
exit status: 0
```

The program asserts by pointer that the resource's field is the borrow and not a copy. That the consumer crate builds at all is the coherence result: its `impl Carrier<Pos> for Read<'f>` stands beside the model's blanket impl over uniform carriers, since `Read` is the consumer's own type and is not uniform.

The recursion of the walks is the part attempt 5 of the earlier sketch did not reach. It holds because the carrier is a trait parameter and both products are supertraits: inside `impl Rows<C> for Cons<H, T>`, `T: Rows<C>` gives `T: Project<C>` and `T: Project<Val>`, which is what normalising the cell's tail needs. A walk written as a method generic over the carrier, as the shipped `Project::read_row::<C>` is, would need `T: Project<C>` for a `C` the trait cannot name, and was not tried for that reason.

### r1. Two forms for one member. Refused.

`out_r1_two_forms_for_one_member.txt`:

```
error[E0119]: conflicting implementations of trait `Carrier<Pos>` for type `Two`
```

### r2. A uniform carrier given another form for one member. Refused.

`out_r2_a_uniform_carrier_overridden.txt`:

```
error[E0119]: conflicting implementations of trait `Carrier<Pos>` for type `Mine`
   = note: conflicting implementation in crate `model`:
           - impl<C, E> Carrier<E> for C
             where C: Uniform, E: Elem;
```

So a per-member exception to a uniform carrier cannot be written for an `Elem` member. That is all r2 shows. A member that is not an `Elem`, owned by the consumer, was not tried, and the first review of the branch built it: the blanket impl covers `Elem` members only, so the consumer's `impl Carrier<Res> for Val` is accepted and `Val` then holds whatever that crate says for `Res`. The model leaves that open. The design closes it with a declared shape and a sealed `Carrier<M>` (`docs/design/notko-hlist-product-carrier.md`), and the crate pins it with `tests/compile_fail/a_shipped_carrier_holds_nothing_for_a_member_of_yours.rs`.

### r3. A carrier with no form for a member. Refused.

`out_r3_no_form_for_a_member.txt`:

```
error[E0277]: the trait bound `consumer::Read<'static>: Carrier<Heat>` is not satisfied
   = note: required for `Cons<Heat, model::Empty>` to implement `Project<consumer::Read<'static>>`
```

The diagnostic then suggests the uniform carriers, through the blanket impl, which a real crate quiets with `#[diagnostic::do_not_recommend]` and says right with `on_unimplemented` on `Carrier`.

### r4. Two lists with the same field types. Refused.

`out_r4_same_fields_other_list.txt`: a product of `Cons<Pos, Empty>` assigned to one of `Cons<Depth, Empty>`, both `i32` through `Val`, is `E0308`.

### r5. One list through two carriers with the same field type. Refused.

`out_r5_same_list_other_carrier.txt`: a product of `Cons<Pos, Empty>` through `Val` assigned to one through `Read<'static>`, both `i32`, is `E0308`. The carrier is in the product's type, so its state is never inferred from the fields.

### r6. A read product kept past its frame. Refused.

`out_r6_twin_outlives_the_frame.txt`: `E0597`, `seq` does not live long enough.

## Predicate

Holds for: toolchain `nightly-2026-05-28`; notko at `1b2a572`; notko-hlist features `const` and `membership` on; the model built with `const_trait_impl`; dev profile; x86-64 Linux; a consumer in a crate apart from the model.

- A form per member through a carrier of the consumer's own, beside uniform carriers holding one form over every `Elem` member: holds, for a list of three members.
- A row walk between two carriers' products, by recursion on the list, at compile time: holds, for `Arr<3>` to `Val` over two members.
- A walk to a carrier with a lifetime through a higher-ranked supertrait: holds, for `Arr<2>` to `Col`.
- Two forms for one member, a uniform carrier overridden for one member, a missing form, a product taken for another list's or another carrier's: each refused at compile time.

Not checked: `SubOf`, `Vals` and `Split` in the model's terms; the plain path without `const`; a long list; compile-time cost; hilavitkutin built against any of it.

## Handed back

The design can take the carrier as a parameter of `Project` with `Carrier<M>` as the per-member form and `Uniform` as what the shipped `Carrier` is today. The cost is the call shape: a walk is reached through a trait naming its carriers, `<L as Rows<Arr<3>>>::read_row(..)`, where the shipped crate takes `L::read_row::<Arr<3>>(..)`.
