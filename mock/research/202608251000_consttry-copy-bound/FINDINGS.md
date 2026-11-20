# The `const` feature does not narrow because it is const. It narrows because the bound is `Copy`.

Probe directory: `notko/research/202608251000_consttry-copy-bound/`
Toolchain: the pinned `nightly-2026-05-28` (`rustc 1.98.0-nightly (57d06900f 2026-05-27)`), plus
`stable 1.94.0` for the MSRV path and `nightly-2026-03-28` / `nightly-2026-06-18` for a dating question.
Nothing was committed. Nothing in `notko/src` or `notko/tests` was modified; all crate work happened in
`p04_real_crate/notko_fixed`, a detached copy, and `PROPOSED.diff` is the patch against the real tree.

## Gate outcomes, first

**Canon gate: no canon exists for this crate**, as the brief states, so there is nothing to defend and
existence and locus are open. The governing workspace rules I actually leaned on are
`unstable-features.md` (three-tier vetting, which bites hard here), `harness-the-type-system.md`
(the discipline ladder), and `a-refused-bound-wants-a-trait-not-a-feature.md` (which describes this wall
exactly). I am one expert; **the vetting verdict in section 7 is owed a second, independent read** before
anything ships on it.

**Test gate: the suite passes and does not cover the thing under investigation.** 129 tests green
before I touched anything. `tests/consttry_smoke.rs` proves the const path at `u32` and nothing else:
every one of its seven const proofs uses a `Copy` payload, there is no const proof for `Outcome`'s
`ConstFromResidual` at all, and the cross-error `E -> F` conversion is asserted nowhere in either
configuration. That is not a fabricated suite and nothing in it is tautological, so it is not
disqualifying. It is a **sampled law**: the one axis that decides this question is the one axis held
fixed, which is why a narrowing this large sat in a shipping crate with a green suite. I authored the
missing columns rather than only naming them (section 6).

---

## 1. The three questions, answered

### "What is the problem even?"

**It is real, and it is not the problem the source describes.** Reproduced with a control, twice, on
exit codes rather than on reading:

```
consumer(default-features = false) alone                      -> compiles
  + a sibling crate depending on notko with defaults          -> exit 101
same, against the patched crate                               -> exit 0
```

`p03_unification/`. Cargo unifies features across a dependency graph, so the escape hatch offered at
`src/consttry.rs:29-30` ("reach for the non-const variant via `default-features = false`") is not
reachable by the consumer it is written for. A consumer cannot control whether some other crate in the
graph turns the default on. The advice is sound in isolation and void in a dependency graph, and a
consumer only discovers this when a *third* crate they do not own is added.

So: yes, a problem. But the interesting part is that it is **not** a const-vs-runtime problem at all.

### "Copy on const time is a thing of itself that needs just wondering if it's meaningful."

**It is not meaningful. `Copy` is a proxy, and a strictly narrower one than the property that matters.**

What const evaluation actually forbids is running a destructor it cannot execute. That property has a
name in the language and it is `core::marker::Destruct`, not `Copy`. `Copy` implies it, so `Copy`
compiles; the implication does not run backwards, and the gap between them is not an edge case. It is
every ordinary non-`Copy` struct that has no `Drop` impl, which is most types people write.

Measured, `p01`:

| bound | `NotCopy` (no Drop) | `HasDrop` | generic `<T>`, no bound, runtime |
|---|---|---|---|
| `T: Copy` (shipped) | **refused**, E0599/E0277 | refused | **refused** |
| `T: [const] Destruct` | accepted | refused, E0277 | **accepted** |
| `T: Destruct` (non-const) | refused, E0493 | refused | refused |

The middle row is the correct behaviour on all three counts: it admits what const evaluation can
handle, refuses what it genuinely cannot, and disappears entirely for runtime callers. The third row is
worth keeping in view because it shows the bound has to be the *conditional* `[const]` one; a plain
`Destruct` bound holds for every type and therefore proves nothing (`p01/f_plain_destruct.rs`).

### "They are already different traits and impls, so..."

They are, and that is exactly why the narrowing was invisible. `ConstTry` is notko's own trait, so
nothing in the compiler or the ecosystem was ever going to compare its two impls against each other.
The only thing that would have caught the divergence is a test asserting the two configurations accept
the same types, and no such test existed. It does now (section 6).

---

## 2. The finding, in one line

**Nothing about const evaluation requires `Copy`, and with the right bound the `const` feature stops
removing anything at all.** A single `const impl` carrying `[const] Destruct` serves both audiences:
const callers get every type without a destructor, and runtime callers get *every type, full stop*,
including ones with a real `Drop`.

`p01/m_relaxation.rs`, all four accepted in one compilation:

1. runtime caller, payload with a non-const `Drop` — accepted
2. runtime caller, non-`Copy` payload — accepted
3. **const** caller, non-`Copy` payload — accepted
4. runtime caller, fully generic `<T>` with **no bound written at all** — accepted

The negative control is what makes that mean something: `p01/k_propagation_nobound.rs`, a *const*
generic fn with no bound, is refused. The check is live, not vacuous.

That is the whole of it. A `[const]` bound is conditionally const by design: it must hold constly only
where the callee is invoked in a const context. The shipped code spends a permanent, unconditional
`Copy` tax to buy a property that was only ever needed conditionally.

---

## 3. The second narrowing, which is independent and also removable

`outcome_consttry_const.rs:32-37` documents dropping the `E -> F` conversion because "`From` in const
trait bounds is not yet stable". **That is false on the pinned toolchain.** `core::ops::Deref`,
`From` and friends are const behind `rustc_const_unstable(feature = "const_convert", issue = "143773")`,
and `const_convert` is **already ALLOWED** in the workspace vetting table, required by arvo-storage.

This compiles (`p02/b_const_from_destruct.rs`):

```
const impl<T, E, F: [const] From<E> + [const] Destruct>
    ConstFromResidual<Outcome<Infallible, E>> for Outcome<T, F>
where E: [const] Destruct
```

and it is strictly better than both shipped impls at once, verified in `p02/d_from_relaxation.rs`:

- reflexive `E == F` in **const** context — works (core has `impl<T> const From<T> for T`,
  `convert/mod.rs:785`), so the case the shipped const impl handles is not regressed
- cross-conversion `ErrA -> ErrB` at **runtime** with a deliberately non-const `From` — works, so the
  case the plain impl handles is restored to the const path

Controls, because "both work" is the shape of a vacuous bound: `p02/e_negative_control.rs` puts a
non-const `From` in a **const** context and is refused (`ErrB: [const] From<ErrA> is not satisfied`);
`p02/f_const_from_positive.rs` is identical but with `const impl From` and compiles. The bound
discriminates.

**Cost, and it is a real one:** a consumer writing their own `impl const From` needs
`#![feature(const_convert)]` **in their own crate**. I hit this myself: the test crate would not build
until I gated it, which is the same wall a consumer meets. Nightly-only, and it does not affect
consumers who only *use* the conversion at runtime.

---

## 4. Op's `ConstCopyish`, tested rather than substituted

The instinct is right and needs one specific thing filled in, which is what "with the bounds filled that
need" was already reaching for.

- **Read literally, a plain marker trait does not work.** `pub trait ConstCopyish {}` with
  `impl<T: Copy> ConstCopyish for T {}`, used as the impl bound, is **refused with E0493**
  (`p05/a_plain_marker.rs`). A user-defined marker carries no information the const drop checker
  consumes. Typestate that "purely lowers out" cannot, on its own, discharge a drop obligation: the
  checker accepts exactly one thing, and that thing is `Destruct`.
- **With `[const] Destruct` as its supertrait it works** (`p05/b_const_marker_destruct.rs`):

```
pub const trait ConstCopyish: [const] Destruct {}
const impl<T: [const] Destruct> ConstCopyish for T {}
```

Blanket, so every eligible type gets it with no consumer action, and it admits `NotCopy` in const while
still taking `HasDrop` at runtime.

**Whether it is worth having is a genuine choice, and I measured the cost rather than guessing.**
Diagnostics, `p05/c_diag_*`: both spellings root the error at the *same* message,
`the trait bound HasDrop: [const] Destruct is not satisfied`. The named marker adds one extra note hop
(`required for HasDrop to implement [const] ConstCopyish`).

That result cuts against the marker's main selling point. The reason to own the name is insulation from
an unstable upstream one, and **the insulation is partial: `Destruct` appears in consumer error messages
either way.** What it does buy is that consumers' *written* bounds say `ConstCopyish`, so if upstream
renames `Destruct` at stabilisation (an open question on #133214), notko changes two lines and no
consumer changes any. With six impl sites the internal saving is nil; the consumer-facing saving is the
whole case for it.

My reading, offered as a suggestion rather than a call: **use `[const] Destruct` directly.** It is one
fewer public trait, the diagnostics are one hop shorter, and it is what core itself writes in 158
places. If the rename risk is judged the larger cost, the marker is a sound and tested alternative and
costs almost nothing.

---

## 5. Routes attacked and closed, with what closed each

Each of these was built and compiled. None is closed by argument alone.

| # | Route | Outcome | Closed by |
|---|---|---|---|
| 1 | Delete `T: Copy`, change nothing else | refused | **E0493** `destructor of Just<T> cannot be evaluated at compile-time` |
| 2 | Full destructure `let Just(x) = self` instead of the partial move `self.0` | refused | **E0493**. The check is on `self` as a by-value local, not on how it is consumed |
| 3 | `match self` on the enums (already a full move in every arm) | refused | **E0493** on `Maybe<T>`. Same reason |
| 4 | Plain `T: Destruct` bound | refused | **E0493**. Non-const `Destruct` holds for every type, so it constrains nothing |
| 5 | Plain user-defined marker trait (op's, read literally) | refused | **E0493** |
| 6 | Two impls, `T: Copy` const + unbounded plain | refused | **E0119** conflicting implementations |
| 7 | `ManuallyDrop` + `ptr::read` | refused, then **reopened** | `Deref is not yet stable as a const trait` — which turned out to mean it *is* one, behind a gate |
| 8 | `ManuallyDrop::into_inner` (avoids `Deref`) | refused | **E0493**. `into_inner` hands back a `Just<T>` that still needs dropping |
| 9 | Union transmute `ManuallyDrop<Just<T>>` -> `ManuallyDrop<T>` | **compiles** | works, but see below |
| 10 | Same union trick on `Maybe<T>` | refused | **E0493**. No layout guarantee on a `repr(Rust)` enum, and the discriminant must be read first |
| 11 | `ManuallyDrop` with `const_convert` enabled | **compiles** | subsumed by route 12 |
| 12 | `[const] Destruct` | **compiles** | the recommendation |
| 13 | Adopt core's const `Try` and drop `ConstTry` | compiles, **rejected** | section 8 |

**Route 9 deserves a note because it is the tempting one.** It needs no `const_destruct` at all, which
matters given the vetting gate. It works only because `Just` is `#[repr(transparent)]` (`just.rs:21`),
so the transmute is sound *there* and nowhere else: `Maybe` and `Outcome` are plain `repr(Rust)` enums
with no layout guarantee, and route 10 confirms the trick does not extend to them. It also costs
`unsafe` and silently depends on a `repr` attribute staying put. A fix that covers one of three types
and is UB if copied to the other two is not a fix. Recorded so nobody re-derives it.

---

## 6. Tests authored

Two new targets, in `p04_real_crate/notko_fixed/tests/`. Both are in `PROPOSED.diff`.

**`consttry_parity.rs` — ungated, and that is the whole design.** No `#[cfg]`, no feature gate, so it
must build in both configurations or neither. A bound that one path carries and the other does not makes
it **fail to build in exactly one**, and that build failure is the assertion. Against the shipped code:

```
shipped, const OFF : 6 passed
shipped, const ON  : could not compile, 7 errors
patched, const ON  : 6 passed
patched, const OFF : 6 passed
```

That is the defect stated as a test and the fix verified against it. It covers `Just`/`Maybe`/`Outcome`
with a non-`Copy` payload, `Outcome` with a non-`Copy` **error**, a payload with a real `Drop` at
runtime, and a fully generic helper writing no bound.

**`consttry_notcopy.rs` — const proofs, `required-features = ["const"]`.** These cannot be
configuration-neutral, and finding out why was itself a result: run ungated against the plain path they
fail with `E0015 cannot call non-const method in constants`, because the plain path genuinely is not
const-callable. So const-callability is what the feature legitimately switches; **type reach is what it
must not**, and the two targets separate those cleanly. Contents: non-`Copy` payloads, a nested
non-`Copy`, `Outcome`'s `ConstFromResidual` (which had no const proof anywhere), and the cross-error
`E -> F` conversion (which had no proof anywhere in either configuration).

One deliberate touch: a target whose assertions are all `const _` blocks reports `0 tests`, which reads
like a skip. I added one runtime mirror so the count is non-zero and the target is visibly alive.

**`just_deref.rs`** — section 9. Seven tests, ungated, passing in both configurations.

Final state, patched copy: **129 passed** (nightly + const), **126** (nightly, `--no-default-features`),
**126** (stable 1.94, `--no-default-features`). Zero failures.

---

## 7. `unstable-features.md`: what this needs, and what is owed

`const_destruct` is **in none of the vetting tables**, and that rule says a feature absent from them is
unvetted and may not ship. Running the procedure:

- **Tracking issue:** rust-lang/rust#133214, **open**, filed 2024-11-19. Re-implemented for the
  next-generation solver in PR #132329.
- **Soundness:** no `I-unsound` label, no soundness discussion found. No sound-subset feature was split
  off it, which is the signal that identified full `specialization` as structurally unsound. The trait
  carries `#[rustc_deny_explicit_impl]` (`core/src/marker.rs:1059-1061`), so no crate can write its own
  impl; membership is entirely compiler-determined, which removes the obvious hazard class.
- **Staleness:** no stabilisation PR yet; blocked behind the const-traits umbrella #143874, which this
  workspace already carries at WATCH. One open design question, whether the name `Destruct` survives
  stabilisation.
- **std-internal carve-out:** **applies, decisively.** On the pinned nightly both `core` and `alloc`
  enable `#![feature(const_destruct)]` (`core/src/lib.rs:104`, `alloc/src/lib.rs:106`), with **158**
  `[const] Destruct` bound sites across the two.
- **Does a stable wrapper suffice?** No, and this is proven rather than assumed: `Copy` is the only
  stable proxy and the table in section 1 shows it is strictly narrower. That is precisely the case the
  carve-out contemplates.

**Proposed row — ALLOWED, on the std-internal carve-out, with no soundness hole found.** Alongside
`const_convert`, which is already ALLOWED and which the `From` half needs.

**This is one expert's reading and a second independent one is owed** before it lands, per the
two-expert rule. I have not written into `unstable-features.md`.

**SAFETY-style justifications, since the carve-out requires them at each use site and I am not editing
in place.** Op should write these properly; this is the substance they need to carry:

- Above the gate in `src/lib.rs`:
  `const_destruct` (#133214) names `core::marker::Destruct` so a `[const]` bound can discharge the
  const-evaluation drop obligation. Enabled under the std-internal carve-out: `core` and `alloc` both
  enable it, with 158 bound sites between them. No `I-unsound`; the trait is
  `#[rustc_deny_explicit_impl]`, so no crate supplies its own impl. The stable alternative, `Copy`, is
  strictly narrower and excludes every non-`Copy` type that has no destructor. Revisit if the name
  changes at stabilisation.
- Above the gate for `const_convert`: already ALLOWED in the workspace table (#143773). Needed for
  `[const] From` on `Outcome`'s residual conversion and for `const Deref`.
- At each of the six `[const] Destruct` impl sites: the bound is the exact obligation the const
  evaluator imposes on a by-value `self`, and it is conditional, so runtime callers are unconstrained.

Both `[const] Destruct` and the older `~const Destruct` compile on the pin (`p01/e`, `p01/g`). `[const]`
is the current spelling; `~const` may reach a wider band of older nightlies if that is ever wanted.

---

## 8. `ConstTry`'s stated reason is wrong. Its real reason is better, and it should stay.

Existence was in scope, so I tested it, and op's pushback on the result was correct.

**`src/consttry.rs:8` says `core::ops::Try` "is not `pub const trait` as of 2026-05 nightly". It is.**
`core/src/ops/try_trait.rs:132-133` on the pinned nightly:
`#[rustc_const_unstable(feature = "const_try", issue = "74935")] pub const trait Try: [const] FromResidual`.
`FromResidual` likewise at :309-310.

**And `src/consttry.rs:13-18` says `?` "stays non-const" because rustc desugars it to
`Try::branch`. The premise is right and the conclusion is false**: since `Try` is const, that
desugaring is const-callable. `p07/c_full.rs` compiles `let v = m?;` **inside a `const fn`**, on a
non-`Copy` payload, and const-evaluates the result.

The charitable explanation — that this was true when written and went stale — **is refuted**:
`nightly-2026-03-28` already had `pub const trait Try`, and so does `2026-06-18`. The claim was not
true on any nightly I can check.

**But the trait should stay**, and reading the code rather than assuming is what showed why. Three
reasons, none of them the one that is written down:

1. **Different feature, different default.** notko's `core::ops::Try` impls are gated
   `#[cfg(feature = "try_trait_v2")]` (`just.rs:293`, `maybe.rs:420`, `outcome.rs:325`), which is
   **default-off**. `ConstTry` provides const branching under the **default** feature set. Routing
   through core's `Try` would make const branching require `try_trait_v2`.
2. **Gate count.** `ConstTry` needs `const_trait_impl`. Going via core needs `const_try` (#74935) **and**
   `const_try_residual` (#91285) **and** `try_trait_v2` — two more unvetted features, on top of
   `Residual` impls notko does not currently have for the const path. I found both by hitting them and
   supplying them one at a time (`p07/a`, `p07/b`, `p07/c`).
3. **MSRV and nightly-range tolerance, which is op's point and the strongest of the three.** Owning the
   trait keeps notko independent of how new a nightly is and of how the upstream const-`Try` shape
   settles. Pinning to whatever landed most recently is the fragile option.

Also worth recording: `just.rs:290-292` documents that orphan rules forbid implementing core's
`Residual` for `Infallible`, which is why notko carries a bespoke `JustResidual`. That constraint is
real and is another reason the parallel surface earns its place.

**So: keep `ConstTry`, and fix the two false sentences.** The trait's justification is genuinely
stronger than the one it currently gives for itself.

---

## 9. `Deref`, and the rest of the `core` surface

**There is no `ConstDeref` in notko and there should not be one.** `core::ops::Deref` is *already*
`pub const trait` (`core/src/ops/deref.rs:138-139`), gated by `const_convert`, which the workspace
already vets as ALLOWED — and `impl<T: ?Sized> const Deref for ManuallyDrop<T>` exists at
`manually_drop.rs:272-273`. `*m` in a `const fn` compiles once the gate is on (`p06/g6`). The general
rule this suggests: **before writing a const parallel of a core trait, check whether core's version is
already `const trait` behind a gate.** `Try` is the case where a parallel earns its place, for the
reasons in section 8; `Deref` is not.

**`Just` should have `Deref`, and I implemented and tested it** rather than recommending it. It is
`#[repr(transparent)]` over one private field (`just.rs:21-24`) and its own doc calls it "a no-op
extraction of the inner value" — that is the `Deref` shape. Added following notko's own file-level
gating convention (`just_deref_const.rs` / `just_deref_plain.rs`), because the crate documents at
`consttry.rs:36-45` why inline `cfg` does not work for const-trait items.

**Two things I got wrong and the code corrected, both from op's instruction to read rather than assume:**

- I expected a `Deref` + `Iterator` autoderef hazard. **There is none.** `Iterator` is implemented for
  `JustIter<T>`, not for `Just<T>` (`just.rs:226`). My earlier grep truncated the name.
- I proposed `AsRef` / `AsMut` and **withdrew them**. `Just` already has *inherent* `as_ref` and
  `as_mut` returning `Just<&T>` / `Just<&mut T>` (`just.rs:56`, `just.rs:69`) — the functor shape, which
  is deliberate and good. An inherent method wins method resolution over a trait one, so the trait impls
  would compile, appear in the docs, and be **unreachable** through `j.as_ref()`. That is a trap, not a
  feature. Caught by E0308 when the test asserted the wrong return type. There is now a test pinning the
  inherent behaviour so nobody adds them later.

Shipped: `Deref`, `DerefMut`, `Borrow`, `BorrowMut` for `Just` (no name collisions, verified against the
full inherent surface). Seven tests, ungated, green in both configurations.

**`Maybe` and `Outcome` must not get `Deref`.** `Deref` is total — it must always produce a target — and
neither has one for its empty/error variant. Anything else would panic inside a coercion, which is the
worst place for it.

**On the wider `core` surface**, an honest scoping note rather than a survey I did not do: `Just` also
lacks `Display`. I did not add it, because `Display` is a deliberate design choice about user-facing
output rather than an omission, and it is outside what this dispatch established. A proper pass over
the remaining `core` traits (the `ops` arithmetic family, `Not`/`BitAnd`/`BitOr` on the boolean-shaped
types, `Extend`, `FromIterator`) wants its own dispatch with the same read-the-inherent-surface-first
discipline that caught the `AsRef` collision here. **The `AsRef` case is the reason: a plausible-looking
core impl can be actively harmful, and the only way to know is to enumerate the inherent methods first.**

---

## 10. README changes needed

Not applied; I worked in a copy. Line numbers against the current `notko/README.md`.

1. **README:32-34 contradicts README:243, and :33 is the wrong one.** ":33 says `--no-default-features`
   leaves "the const paths absent". :243 says they "exist in plain non-const form". The code does the
   latter: `consttry_plain_path.rs` and the three `*_consttry_plain.rs` files ship the traits and impls
   as plain items. Fix :33 to match :243.
2. **README:237**, the feature table row, says the traits "become `const trait`s". Today that is
   incomplete, because the feature also silently narrows which types the impls accept. After the patch
   it becomes true as written, so this line is fixed *by* the patch rather than by an edit.
3. **Add an explicit parity sentence**, near the feature table, because it is the property consumers
   need and cannot infer: the `const` feature decides whether the traits are const-callable, and does
   not change which types they accept. Worth stating precisely because cargo unifies features and a
   consumer cannot keep the default off.
4. **State the nightly gates the default set turns on**: `const_trait_impl`, plus `const_destruct` and
   `const_convert` after this patch. README:32 currently says only "const-trait machinery", and a
   consumer hitting E0658 in their own crate (which they will, if they write `impl const From` or want
   `*just` in a `const fn`) has nothing to search for.
5. **If the `Deref` work lands**, add `Deref`/`DerefMut`/`Borrow`/`BorrowMut` for `Just` wherever the
   README lists trait impls, and say explicitly that `Maybe` and `Outcome` do not get `Deref` and why.
6. **Optional but worth it:** README:76 says `?` "needs a nightly compiler, since that is what
   `try_trait_v2` is". True. It is now also true that `?` can work in a `const fn` (section 8), which
   the crate documentation currently denies at `consttry.rs:13-18`. If that capability is not being
   adopted, the README need not mention it, but the module doc should stop asserting the opposite.

---

## 11. What I could not reach

- **No aggregator or SerpAPI search was needed beyond one query**; the decisive evidence was local. I
  read `core` and `alloc` from the pinned toolchain's own `rust-src` rather than trusting web summaries,
  which is why the `Try`-is-already-const and `Deref`-is-already-const findings are citable to
  `file:line` in the toolchain rather than to a blog.
- **I did not open #143874, #74935 or #91285 directly.** #133214 I did fetch. The status claims for the
  other three rest on the `rustc_const_unstable` attributes in the pinned source, which is a fact about
  the toolchain notko compiles against and is the thing that actually governs; it is not a substitute
  for reading their stabilisation discussions, and section 7's proposed row should not be taken as
  vetting `const_try` or `const_try_residual`, which I recommend against adopting anyway.
- **I did not run `scripts/feature-matrix.sh`**, per the brief; I ran the configurations by hand
  (`--features const`, `--no-default-features`, `--features const,try_trait_v2`, and stable).
- **Not benchmarked, and nothing here claims a performance number.** The `[const] Destruct` change
  should be codegen-neutral, since it removes a bound rather than adding a runtime operation, but that
  is an expectation and not a measurement. If it matters, it needs the bench harness.

## 12. One instrument failure, recorded because it nearly produced a false finding

Two crates both named `notko v0.0.1` at different paths, sharing one `CARGO_TARGET_DIR`, produced a
**false red**: the patched crate reported `NotCopy: Copy is not satisfied` while its own source plainly
read `[const] Destruct`. Rebuilding in an isolated target directory cleared it. Every result in this
file was re-run under per-variant isolated target directories after that was found.

Separately, my first unification probe was broken (`Just`'s field is private, so `Just(x)` does not
construct) and my control caught it; and one shell conditional reported "STILL BROKEN" on a build that
had succeeded, because `grep | head` exits 0 regardless. Both were caught by looking at raw output
instead of a summarised exit. The lesson is the cheap one: check that the instrument can produce the
other answer before believing the answer it gives.
