# Sketch N2: can the product of notko-hlist hold borrowed values of a frame lifetime

Hypothesis: a product over a notko-hlist list can hold a borrowed value of a frame lifetime and be read, and a twin whose borrow outlives the frame does not compile.

**Outcome:** WORKS, where the borrow sits on a carrier of the consumer's own and every member is held in the same borrowed form. The shipped member trait refuses a value that is itself a borrow, and the shipped carrier trait refuses a form chosen per member, which is what a read list of mixed members needs.

This is a feasibility sketch, written unattended on 15 November 2026 by a dispatched seat; the dispatcher opened the output files before writing this. Nothing in it is timed, and nothing in it is authority.

## What the audit read, and what holds of it

The audit reads the shipped product as holding `'static` values only (`research/202611140845_stack-audit/DOWNSTREAM.md:19-26`, `:45-49`). That is true of a member's value type and not of the product. The shipped `Col<'a>` already holds a borrow per member, and a consumer may add a carrier. What the shipped crate cannot do is let the carrier ask each member what it reads as.

## Attempts

Run by `run.sh`, toolchain `nightly-2026-05-28`, dev profile, notko-hlist with its default features (`const`, `membership`). The three sketches under `research/sketches/` dated 14 November do not touch the question.

### 1. The member's value is the borrow. Shipped crate. Refused.

`src/bin/a1_the_member_refuses.rs`, output `out_a1_the_member_refuses.txt`.

```
error[E0477]: the type `&'a Seq` does not fulfill the required lifetime
  --> src/bin/a1_the_member_refuses.rs:21:18
note: type must satisfy the static lifetime as required by this binding
  --> notko-hlist/src/carrier.rs:52:24
52 |     type Value: Copy + 'static;

error[E0478]: lifetime bound not satisfied
  --> src/bin/a1_the_member_refuses.rs:20:10
20 | impl<'a> Elem for Res<'a> {
   = note: but lifetime parameter must outlive the static lifetime

error[E0277]: the trait bound `Seq: Copy` is not satisfied
  --> src/bin/a1_the_member_refuses.rs:28:18
note: required by a bound in `notko_hlist::Elem::Value`
52 |     type Value: Copy + 'static;
```

Two bounds refuse the borrow: `'static` on `Elem::Value` and `'static` on `Elem` itself (`notko-hlist/src/carrier.rs:50-52`). A third refuses the other reading of the need: a resource that is not `Copy` cannot be a member's value at all.

### 2. The borrow on a consumer carrier. Shipped crate. Works.

`src/bin/a2_frame_carrier.rs`, output `out_a2_frame_carrier.txt`. A carrier `Frame<'a>` with `Of<E> = &'a E::Value`, the product through the shipped `Project::Prod`, read by member through the shipped `Has::pick`, cut through the shipped `SubOf::sub` with a `Dup` impl for the carrier.

```
table read by member through the borrowed product: [10, 20, 30, 40]
heat read by field: 40
a narrower product cut from it: table [10, 20, 30, 40], pos [1, 2, 3]
the frame ended and the product with it
exit status: 0
```

The program asserts by pointer that each field is the borrow of the frame's local and not a copy.

The case that must fail, `src/bin/a2_twin_outlives_the_frame.rs`, output `out_a2_twin_outlives_the_frame.txt`:

```
error[E0597]: `pos` does not live long enough
32 |             head: &pos,
38 |     }
   |     - `pos` dropped here while still borrowed
39 |     println!("{:?}", kept.tail.head);
   |                      -------------- borrow later used here
error[E0597]: `table` does not live long enough
```

### 3. A form per member. Shipped crate. Refused, three routes.

`src/bin/a3_form_per_member.rs`, output `out_a3_form_per_member.txt`. A read list wants one member by value, one as a window, one by reference. The carrier has to ask the member, or the member's value type, what it reads as.

```
error[E0277]: the trait bound `E: Readable` is not satisfied
26 |     type Of<E: Elem> = <E as Readable>::Rd<'a>;

error[E0276]: impl has stricter requirements than trait
36 |         E: Readable;
   |            ^^^^^^^^ impl has extra requirement `E: Readable`

error[E0277]: the trait bound `<E as Elem>::Value: Form` is not satisfied
43 |     type Of<E: Elem> = <E::Value as Form>::At<'a>;
```

The bound that refuses is the one on the generic associated type, `Carrier::Of<E: Elem>` (`notko-hlist/src/carrier.rs:61-64`). An impl may not add to it, so a carrier can use nothing of a member beyond `Elem::Value`.

### 4. Strike the two `'static` bounds. A copy of the crate. Refused.

`no_static/notko-hlist/` is the shipped `notko-hlist/src` with one change, in `src/carrier.rs`: `pub trait Elem { type Value: Copy; }`. Outputs `out_b0_copy_default_features.txt` and `out_b0_copy_no_default_features.txt`, the same on both feature paths.

```
error[E0309]: the associated type `<E as Elem>::Value` may not live long enough
  --> no_static/notko-hlist/src/carrier.rs:92:5
91 | impl<'a> Carrier for Col<'a> {
92 |     type Of<E: Elem> = &'a [E::Value];
error[E0309]: ... carrier.rs:97:5   (ColMut)
error[E0309]: ... project.rs:339:22 (Split::split_mut)
```

So the `'static` on the value is not only the preference stated at `carrier.rs:42-45`. The borrowed column carriers need the value to outlive a lifetime the carrier picks, and the trait gives them no place to say so except `'static` on every value. The repair the seat could see threads a value lifetime through `Elem`, `Carrier`, `PCons`, `Project`, `Has`, `SubOf` and `Split`; it was not built, and no smaller change was found or ruled out.

### 5. The carrier as a parameter of the product trait. A model. Works.

`src/form.rs` and `src/lib.rs`, over the shipped `Cons`, `Empty`, `List`, `Place`, `Here`, `There` and `PlaceOf`. The carrier states per member what it holds, as `impl Form<M> for Carrier` under any bound it likes, and the product is `ProjectBy<C>` for the list and the carrier together. The read list names no lifetime; the frame is on the carrier.

`src/bin/c1_form_per_member.rs`, output `out_c1_form_per_member.txt`: one list of three members, read as a value, a window on a column and a reference to a resource that is not `Copy`, each found by member.

```
resource read by member: [5, 6, 7] of 3
window read by member: centre 2 is 6
value read by member: 9
the write side through the same model: 11
the frame ended and the read product with it
exit status: 0
```

The fourth line is the shipped one-value-per-member carrier written in the model's terms, over a member that implements the shipped `Elem`.

The case that must fail, `src/bin/c1_twin_outlives_the_frame.rs`, output `out_c1_twin_outlives_the_frame.txt`:

```
error[E0597]: `column` does not live long enough
error[E0597]: `seq` does not live long enough
28 |     }
   |     - `seq` dropped here while still borrowed
29 |     let s = <Reads as HasBy<Read<'_>, Store, _>>::pick(&kept);
```

This is the shape hilavitkutin's own read product has in code (`hilavitkutin-api/src/values.rs:187-201`, `:238-257`), stated once over any carrier. It is a spike: one instance showing the shape is expressible beside the shipped list, and its names and incidental choices decide nothing.

## Predicate

Holds for: toolchain `nightly-2026-05-28`; notko at `f5d8f56`; notko-hlist features `const` and `membership` on for attempts 1, 2, 3 and 5, both on and both off for attempt 4; dev profile; x86-64 Linux.

- A product through a consumer carrier holds a borrow of a frame lifetime: holds where every member is held as the same form over its `Elem::Value`, and the value type is `Copy + 'static`.
- A member's value that is itself a borrow: does not hold on the shipped crate, and does not hold with the two `'static` bounds struck.
- A form per member on the shipped `Carrier`: does not hold.
- A form per member where the carrier is a trait parameter: holds in the model, for a list of three members.

Not checked: the model inside notko-hlist, under the `const` path as a const trait, its sealing, its diagnostics, a long list, its compile-time cost; hilavitkutin built against any of it.

## Handed back

The design is silent on where a frame lifetime should live. Attempt 2 puts it on the carrier with one form for all members. Attempt 5 puts it on the carrier with a form per member and needs a second product trait in notko-hlist or a reshaping of `Project`. A lifetime on the member, attempt 4's direction, puts it in the list type and does not build as a small change. Which of these notko-hlist should offer is not decided here.
