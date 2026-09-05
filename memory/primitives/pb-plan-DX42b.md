# PB-DX42b — implementation plan (`scutemob-233`)

Authority: `docs/audits/mtg-characteristics-recursion-adjudication.md` §3.2(iii), §3.3, §5.2;
`memory/primitives/seed-rerank-2026-08-14.md` §4 row 18. Stage-0 measurements:
`memory/primitives/pb-DX42b-stage0-census.md`. Wire prediction: `pb-DX42b-wire-prediction.md`.

Closes **`OOS-ADJ-1` ≡ `OOS-DX19-2`** as ONE defect, plus **`OOS-DX19-1`**'s residue. Rider:
**`OOS-ADJ-2`** (the PB-DX42a `t7` widening + `t9`'s missing `TargetFilter` half).

---

## 0. The defect in one paragraph

`rules::layers::characteristics_for_condition` (`layers.rs:111`) returns **printed**
`obj.characteristics` for **any** condition evaluated inside a `calculate_characteristics` walk,
because the only thing it can consult is an ambient `thread_local!` depth counter that says
"somewhere inside the layer system" and nothing more. That is a CR 613.1d deviation on **7**
deck-legal `Complete` pairs (`indomitable_archangel` × the seven Artifact-moving supply cards) and
on the unbounded CR 708.2a face-down class. A depth counter suppresses **the entire layer system**
where an `EffectId` set suppresses **the one self-referential effect** — that difference *is* the
seven pairs.

## 1. The three engine steps, in this order (§5.2)

### Step 1 — split `is_effect_active` at its existing seam

`layers.rs:661`. Two free functions, no behaviour change:

* `is_effect_duration_active(state, effect) -> bool` — the whole `match effect.duration` block.
  **Verified free of any characteristics query**; it is what makes step 3 non-circular.
* `is_effect_condition_satisfied(state, effect, eval) -> bool` — the CR 604.2 `condition` half.

Keep `pub fn is_effect_active(state, effect) -> bool` as the composition of the two, because
`rules/copy.rs:117`, `layers.rs:2080` (`abilities_are_blanked`) and `layers.rs:3004`
(`recompute_object_controller`) call it and none of them is inside a bounded walk. Their
`eval` is a fresh `CharacteristicEvalContext::outside_layer_walk()`.

### Step 2 — an explicit per-`EffectId` context replaces the ambient depth counter

New in `layers.rs`:

```rust
pub struct CharacteristicEvalContext {
    /// `EffectId`s whose CONDITION is being evaluated higher in this call stack.
    in_flight: std::collections::BTreeSet<EffectId>,
    /// `Some(l)` — inside a layer walk bounded through layer `l`. `None` — outside.
    bound: Option<EffectLayer>,
}
```

* `CharacteristicEvalContext::outside_layer_walk()` — empty set, `bound: None`.
* Two RAII guards, because `calculate_characteristics_through` has an early `return None` and can
  unwind: `BoundGuard` saves/restores `bound`; `InFlightGuard` inserts/removes one `EffectId`.
  This is the shape `LayerWalkGuard` already had and the adjudication explicitly asks for.
* **`LAYER_WALK_DEPTH` / `LayerWalkGuard` / `in_layer_walk` are RETIRED** — see §4 for the
  decision and what replaces the `process_command` balance assert.

Threading. `check_static_condition` and `check_condition` are `pub` with 63 call sites between
them. **Do not thread `&mut` through those signatures.** Instead:

* rename the real bodies to `check_static_condition_ctx(state, cond, source, controller, eval)` and
  `check_condition_ctx(state, cond, ctx, eval)`, both `pub(crate)`;
* keep `pub fn check_static_condition(..)` / `pub fn check_condition(..)` as **thin wrappers** that
  call the `_ctx` form with `&mut CharacteristicEvalContext::outside_layer_walk()`. Every one of
  the four safe caller classes (`activation_condition`, `intervening_if`, `Effect::Conditional`,
  `unless_condition`) keeps its exact signature and its exact behaviour — full layer resolution.
* the recursive arms inside the evaluators (`Not`/`Or`/`And`, and the five
  `check_condition` → `check_static_condition` self-delegations) call the `_ctx` form and pass
  `eval` down, so the context survives a combinator.

`characteristics_for_condition` becomes:

```rust
pub(crate) fn characteristics_for_condition_ctx(
    state: &GameState,
    obj: &GameObject,
    required: EffectLayer,
    eval: &mut CharacteristicEvalContext,
) -> Characteristics {
    match eval.bound {
        // Outside any walk: CR 613.1d in full. Unchanged for the four safe callers.
        None => expect_characteristics(state, obj.id),
        // Inside a walk: resolve THROUGH the layer this filter actually needs.
        Some(_) => calculate_characteristics_through(state, obj.id, required, eval)
            .unwrap_or_default(),
    }
}
```

Keep `pub fn characteristics_for_condition(state, obj)` as a compat shim
(`..._ctx(state, obj, EffectLayer::PtSwitch, &mut outside_layer_walk())`) **only if some caller
still needs it**; it is re-exported from `lib.rs:28`. If nothing needs it, delete it and the
re-export — but then say so, because the deviation's name disappearing from the tree is a fact the
next reader needs.

### Step 3 — the bounded query, with the activity sweep bounded by the SAME layer

```rust
pub fn calculate_characteristics_through(
    state: &GameState,
    object_id: ObjectId,
    through: EffectLayer,
    eval: &mut CharacteristicEvalContext,
) -> Option<Characteristics>
```

`calculate_characteristics(state, id)` becomes exactly
`calculate_characteristics_through(state, id, EffectLayer::PtSwitch, &mut outside_layer_walk())`.

Inside:

1. `let _bound = BoundGuard::enter(eval, through);`
2. the pre-loop base rewrites (suspected, ring-bearer, DFC, meld, face-down, mutate) run
   **unchanged** — they are base-characteristic rewrites, conceptually before Layer 1, and the
   face-down one at `:333` is why the CR 708.2a over-count closes for free;
3. **the activity sweep is bounded**:
   `.filter(|e| e.layer <= through && is_effect_duration_active(..) && is_effect_condition_satisfied(.., eval))`.
   **This is the load-bearing precondition of §3.2(iii)** — a bounded query over a *global*
   activity sweep is the original recursion with an extra parameter, because the sweep would still
   evaluate the Layer-6 Archangel condition and re-enter the same bounded query. Put that sentence
   in the code as a comment;
4. `layers_in_order` is filtered to `l <= through`;
5. the post-loop steps (mutate ability union, derived attack triggers) run unchanged.

`is_effect_condition_satisfied(state, effect, eval)`:

```
None condition                       -> true
eval.in_flight.contains(&effect.id)  -> false   // the labelled deviation, see §3
otherwise:
    debug_assert!(required < effect.layer)      // see below
    let _g = InFlightGuard::enter(eval, effect.id);
    check_static_condition_ctx(state, cond, source, controller, eval)
```

**Termination is by construction and must be stated as such in the doc comment.** An effect at
layer `L` whose condition requires layer `R < L` produces a nested walk bounded at `R`, which
sweeps only effects at layers `<= R < L`. The bound strictly decreases at every level and
`EffectLayer` is finite and totally ordered, so the recursion is finite **without** any
cycle-breaker on this corpus — which is exactly §3.2(iii)'s claim.

### `TargetFilter::required_characteristic_layer`

On `TargetFilter` in `crates/card-types/src/cards/card_definition.rs`. **Computed per filter
INSTANCE, never per type**, and returning `Option<EffectLayer>` (`None` = this filter reads no
characteristic at all, e.g. only `exclude_self` / `has_counter_type` / `controller`).

**It MUST be written as an exhaustive destructure** —
`let TargetFilter { max_power, min_power, ..every one of the 33 fields.. } = self;` — so that a
field added later is a **compile error** rather than a silent omission. That is `OOS-DX28-1`'s
`TOKEN_SPEC_FIELDS` lesson and PB-DX43's exhaustive-match lesson, and it is the whole reason this
is not a `match` on a handful of `Option`s.

The mapping over the nine `Characteristics` fields `matches_filter` reads
(`effects/mod.rs:10522`), highest layer wins:

| filter fields | `Characteristics` field | layer |
|---|---|---|
| `has_name` | `name` | **`Text`** (CR 613.1a copy sets it, 613.1c text-change can) |
| `max_cmc`, `min_cmc`, `max_cmc_amount`, `min_cmc_amount` | `mana_cost` | **`Text`** |
| `has_card_type`, `has_card_types`, `non_creature`, `non_land`, `has_subtype`, `has_subtypes`, `exclude_subtypes`, `basic`, `nonbasic`, `legendary`, `has_chosen_subtype`, `exclude_chosen_subtype` | `card_types` / `subtypes` / `supertypes` | **`TypeChange`** |
| `colors`, `exclude_colors` | `colors` | **`ColorChange`** |
| `has_keywords` | `keywords` | **`Ability`** |
| `max_power`, `min_power`, `max_toughness` | `power` / `toughness` | **`PtSwitch`** (7d) |
| `controller`, `is_token`, `is_nontoken`, `is_attacking`, `is_blocking`, `is_tapped`, `is_untapped`, `has_counter_type`, `exclude_self`, `owner` | not read by `matches_filter` — `GameObject` state | contribute **nothing** |

The two `*_cmc_amount` fields are not read by `matches_filter` today but are the same
characteristic; include them, over-collecting deliberately, and say why (over-collection can only
raise the required layer, which can only make the `debug_assert` louder — it fails safe).

**`Condition::required_characteristic_layer`** — a sibling on `Condition`, exhaustive with **no
wildcard arm** (SR-5 / PB-DX43 shape), returning `Option<EffectLayer>`:

* `YouControlNOrMoreWithFilter { filter, .. }` → `filter.required_characteristic_layer()`
* the ten other layer-querying variants enumerated in adjudication §2.2 (`YouControlPermanent`,
  `OpponentControlsPermanent`, `ControlLandWithSubtypes`, `ControlAtMostNOtherLands`,
  `ControlBasicLandsAtLeast`, `ControlAtLeastNOtherLands`, `ControlAtLeastNOtherLandsWithSubtype`,
  `ControlLegendaryCreature`, `ControlCreatureWithSubtype`, `OpponentControlsMoreLandsThanYou`) →
  `Some(EffectLayer::TypeChange)` — every one of them tests card types, subtypes or supertypes
* `Not`/`Or`/`And` → the **max** of their operands
* everything else → `None`

### The `debug_assert`

In `is_effect_condition_satisfied`, before recursing:

```rust
debug_assert!(
    required < effect.layer,
    "CR 613.1d: {:?}'s condition requires characteristics resolved through layer {:?}, \
     which is at or after its own layer {:?}. ...",
    effect.id, required, effect.layer
);
```

That class is **empty in the corpus** (stage-0 census: the one deck-legal condition needs
`TypeChange` and sits on an `Ability` effect) and the assert is how it stays visible. It must have
its own test: build a synthetic same-layer case and prove the assert fires
(`#[should_panic]`, `#[cfg(debug_assertions)]`).

---

## 2. Files expected to move

* `crates/engine/src/rules/layers.rs` — steps 1-3, the context, the guards, the retirement
* `crates/engine/src/effects/mod.rs` — `_ctx` renames, the 11 `characteristics_for_condition` call
  sites gain a `required` argument and `eval`
* `crates/card-types/src/cards/card_definition.rs` — the two `required_characteristic_layer` impls
* `crates/engine/src/rules/engine.rs` — the retired balance assert
* `crates/engine/src/lib.rs` — the re-export
* tests: `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs` (invert + reword),
  `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs` (the rider)

**`crates/view-model`, `crates/simulator/src` and `tools/` are expected to be EXACTLY 0** — every
consumer of characteristics already calls `calculate_characteristics`, whose signature does not
change. Measure this against the final tree rather than asserting it (`crates/simulator/src` does
call `check_condition`, whose public signature is deliberately preserved).

## 3. The surviving cycle-breaker ships LABELLED

`eval.in_flight` still suppresses an effect whose condition re-enters its own evaluation. With the
bound strictly decreasing that is unreachable on this corpus, and it is a **backstop for the
release build** where the `debug_assert` is compiled out. Per adjudication §3.2(ii) and §5.3 it
must ship as a **documented deviation**, in the same register as the CR 700.5a devotion note:

> The CR is silent on condition-evaluation cycles. CR 613.8b is evidence about the CR's
> disposition when it faces an unresolvable circularity — it picks a **total order**, not
> suppression — but CR 613.8a(a) confines 613.8b to a single layer, and the live case is
> strictly cross-layer, so 613.8b does not govern. Treat-as-inactive is therefore an
> **undocumented deviation** and is labelled as one here.

It needs a **wrong-way-round pin**: a test that builds a same-layer self-referential effect,
asserts the suppressed (deviating) outcome, and says in its own message that the batch which
implements a timestamp-ordered tiebreak should INVERT it.

## 4. `LayerWalkGuard` — the decision, stated

**RETIRED entirely**, together with `LAYER_WALK_DEPTH`, `in_layer_walk`, the `lib.rs` re-export,
and the `process_command` balance `debug_assert!` (`engine.rs:369-380`).

The reason, and it is a reason rather than a tidy-up: that assert existed because ambient
thread-local depth **can** leak across a command boundary (a `mem::forget`, an `enter()` outside
`calculate_characteristics`) and a leaked depth is sticky for the rest of the thread. A
`&mut CharacteristicEvalContext` created at the top of `calculate_characteristics_through` and
passed down by reference **cannot outlive that call** — the borrow checker says so — so the hazard
the assert guarded is no longer representable rather than merely unlikely. `OOS-DX19-4` (the depth
tripwire) is closed by construction and its row must say that.

What replaces it is a probe on the invariant that actually can break now: that `BoundGuard` and
`InFlightGuard` restore their state on an early return and on unwind.

## 5. Tests (AC 7386 / 7387)

1. **INVERT** `deviation_animated_nexus_does_not_count_toward_metalcraft` — never delete. Rename to
   say what now holds; keep the fixture; flip the assertion; the new message must say the
   deviation is CLOSED and cite CR 613.1d.
2. **UNDER-count, real channel.** `blinkmoth_nexus` (or `inkmoth_nexus` / `darksteel_mutation`)
   animated beside two plain artifacts and the Archangel, driven on a real `LocalGame` /
   `HumanChoice` seat, with every permanent reaching the battlefield through a **real cast / ETB**
   rather than `GameStateBuilder` placement (`OOS-DX43-6`: `GameStateBuilder::build()` registers no
   static continuous effects, so a conferring permanent placed straight on the battlefield confers
   nothing and the probe fails for a reason it does not describe). Assert by a targeted spell being
   **REFUSED** under the Archangel's shroud.
3. **OVER-count, same channel.** `eaten_by_piranhas` / `kenriths_transformation` /
   `imprisoned_in_the_moon` over one of the 28 `Complete` artifact creatures, so Metalcraft turns
   OFF and the same targeted spell is **ACCEPTED**. **Not** `darksteel_mutation` (its payload *is*
   `[Artifact, Creature]`, so an enchanted artifact stays one).
4. **`thaumatic_compass`** — its own test. A DFC face swap is a pre-loop base rewrite at
   `layers.rs:219`, not a Layer-4 continuous effect, so it does not discriminate the Layer-4 path
   and must not be folded into (3).
5. **Two distinct conditional effects nest without mutual suppression** — the discriminating probe
   for step 2 and **unwritable against a depth counter**. Must be RED under a depth-counter revert.
6. **The `debug_assert` fires** on a synthetic same-layer case.
7. **`no_condition_evaluator_resolves_characteristics_directly` re-keyed with its reason.** After
   the refactor the two `pub fn` wrappers are three lines each, so the gate as written would go
   **vacuously green** — *a gate you edit prose to satisfy has stopped measuring*, and here the
   prose is a function name. Re-key it onto `check_static_condition_ctx` / `check_condition_ctx`
   (the bodies that actually evaluate), keep scanning the two wrappers as well, and add a
   **non-vacuity floor on body SIZE** so a body that shrinks to nothing fails loudly. The permitted
   route is now `characteristics_for_condition_ctx`; `expect_characteristics` and any spelling of
   `calculate_characteristics` stay forbidden.
8. **Reword** `the_deviation_is_scoped_to_the_layer_walk_only` — the ambient flag it reads is gone.
   It becomes the statement that the four safe callers get full CR 613.1d resolution while a
   condition inside the walk gets a **layer-BOUNDED** one, which is the new boundary.
9. **Rider `OOS-ADJ-2`.** `pb_dx42a_continuous_condition_roster`:
   * `t7` widened from the one-variant `ControlLandWithSubtypes` pin to **all eight**
     non-`TargetFilter` layer-querying variants (§2.2's eleven minus the three that carry one);
   * `t9` gains its **missing `TargetFilter` half** — it currently asserts
     `CONTINUOUS_EFFECT_DEF_FIELDS == declared("ContinuousEffectDef")` and does **not** make the
     same assertion for `TARGET_FILTER_FIELDS`; the constant has **20 entries visible in the first
     block and 33 fields are declared**, so re-key the constant against the declaration and let the
     new assertion hold it.
   * both revert-proven.

## 6. Wire

**Predicted UNMOVED for both, in writing, at `d90b7994`, before any production line.** Nothing here
is a type, a variant or a field on anything serialized. Gate-execute both against the final tree;
if either moves, STOP and report rather than bumping.

## 7. Hazards, named

* **The load-bearing precondition.** Bounding the query without bounding the sweep does not
  terminate. If the implementation adds `through_layer` to the query and leaves
  `.filter(|e| is_effect_active(state, e))` global, it has built the original recursion with an
  extra parameter.
* **`unwrap_or_default()` on the nested query.** `calculate_characteristics_through` returns `None`
  only when the object does not exist, which inside a battlefield sweep cannot happen — but
  defaulting silently is how a fizzle becomes a wrong answer. Prefer skipping the object.
* **The four safe callers.** PB-DX19's first attempt read base characteristics unconditionally and
  broke all four to fix the one. `garruks_uprising`'s intervening-if on a counter-pumped creature,
  `bloodline_keeper`'s changeling activation cost, and `mox_opal` over a face-down manifest are the
  three worked cases; the existing `non_layer_path_reads_layer_resolved_power` and
  `non_layer_path_reads_layer_resolved_subtypes` must stay green.
* **`sibling_condition_on_a_continuous_effect_terminates`** must stay green — it is the reviewer's
  reproduction of the original SIGABRT and the only thing standing between "no corpus def puts
  `ControlAtLeastNOtherLands` on a continuous effect" and a regression.
