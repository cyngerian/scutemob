# Finding: Recursive Characteristic Evaluation in Conditional Static Effects

**Date:** 2026-08-02  
**Severity:** HIGH  
**Status:** Confirmed from static code review  
**Primary files reviewed:**

- `crates/engine/src/rules/layers.rs`
- `crates/engine/src/effects/mod.rs`
- `crates/card-defs/.../indomitable_archangel.rs`
- `crates/engine/src/state/mod.rs`

## Executive Summary

`calculate_characteristics` can recurse without bound when a conditional static
ability evaluates a condition that itself queries layer-resolved
characteristics.

`Indomitable Archangel` is a confirmed legal-deck reproducer. Its Metalcraft
condition counts artifacts by calling `expect_characteristics` for battlefield
objects. That nested characteristic calculation reevaluates every conditional
continuous effect, including the same Archangel effect, which evaluates the
Metalcraft condition again.

The result is an unbounded cycle ending in stack overflow.

The immediate repair should introduce a characteristic-evaluation context and
exclude a conditional effect while that same effect's condition is being
evaluated recursively. This removes the crash without falling back to printed
characteristics.

The durable repair should also separate:

1. effect duration/source liveness;
2. conditional applicability; and
3. per-object effect filtering.

Conditional characteristic queries should be explicitly layer-bounded rather
than always requesting fully resolved Layer 7d characteristics.

---

## Confirmed Call Chain

### 1. `calculate_characteristics` globally evaluates effect activity

At the start of `calculate_characteristics`, before the layer loop, the engine
collects every active continuous effect:

```rust
let active_effects: Vec<&ContinuousEffect> = state
    .continuous_effects
    .iter()
    .filter(|e| is_effect_active(state, e))
    .collect();
```

This means the condition of every conditional continuous effect is evaluated
for every characteristic query, regardless of which object is being queried.

### 2. `is_effect_active` evaluates conditional static abilities

After checking duration/source liveness, `is_effect_active` calls:

```rust
check_static_condition(state, condition, source_id, controller)
```

The nearby comment says conditions are evaluated "at layer-application time,"
but this call occurs while constructing `active_effects`, before any layer is
processed.

### 3. Indomitable Archangel uses a characteristic-based condition

The card definition registers a Layer 6 ability grant:

```rust
AbilityDefinition::Static {
    continuous_effect: ContinuousEffectDef {
        layer: EffectLayer::Ability,
        modification: LayerModification::AddKeyword(
            KeywordAbility::Shroud,
        ),
        filter: EffectFilter::ArtifactsYouControl,
        duration: EffectDuration::WhileSourceOnBattlefield,
        condition: Some(
            Condition::YouControlNOrMoreWithFilter {
                count: 3,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Artifact),
                    ..Default::default()
                },
            },
        ),
    },
}
```

### 4. The condition recursively requests full characteristics

`check_static_condition` handles
`YouControlNOrMoreWithFilter` by iterating controlled battlefield objects and
calling:

```rust
let chars =
    crate::rules::layers::expect_characteristics(state, obj.id);
```

It then applies `matches_filter(&chars, filter)`.

### 5. The recursive cycle

```text
calculate_characteristics(object A)
└─ collect active effects
   └─ is_effect_active(Indomitable Archangel effect)
      └─ check_static_condition(Metalcraft)
         └─ expect_characteristics(candidate object B)
            └─ calculate_characteristics(object B)
               └─ collect active effects
                  └─ is_effect_active(Indomitable Archangel effect)
                     └─ check_static_condition(Metalcraft)
                        └─ expect_characteristics(candidate object B)
                           └─ ...
```

The cycle does not require the outer query to be for Archangel or for an
artifact. Any characteristic query while the conditional effect is registered
can enter the same global activity evaluation.

---

## Incorrect Existing Termination Argument

The comment above the nested `expect_characteristics` call acknowledges that the
call is reentrant but claims it is safe because:

1. persistent data structures are immutable; and
2. it is checking other battlefield objects rather than the object currently
   being calculated.

Neither claim establishes termination.

Immutability prevents observing partial mutation. It does not prevent a pure
function from recursively calling itself forever.

The recursive call also need not return to the original queried object. Each
nested `calculate_characteristics` call reevaluates the same global conditional
effect, which is sufficient to repeat the cycle.

The comment should be removed or rewritten as part of the fix.

---

## Scope and Risk

### Confirmed impact

- A legal card definition can trigger a stack overflow.
- The failure is reachable through ordinary characteristic queries.
- The crash is not restricted to one UI, simulator, or networking path.
- The crash occurs in core rules evaluation.

### Likely broader impact

Any static condition that:

1. is evaluated from `is_effect_active`; and
2. calls `calculate_characteristics` or `expect_characteristics`

can create the same class of recursion.

This should be audited with searches such as:

```bash
rg -n \
  'check_static_condition|calculate_characteristics|expect_characteristics' \
  crates/engine/src/effects \
  crates/engine/src/rules
```

This finding is broader than the `Indomitable Archangel` card definition. The
card is a reproducer, not the root cause.

---

## Immediate Remediation

### Goal

Stop recursive reevaluation of the same conditional effect while preserving
layer-resolved characteristic checks for the condition.

### Recommended shape

Introduce an evaluation context that is created by the public entry point and
threaded through all nested characteristic calculations.

```rust
#[derive(Default)]
struct CharacteristicEvalContext {
    evaluating_conditions: HashSet<EffectId>,
}

pub fn calculate_characteristics(
    state: &GameState,
    object_id: ObjectId,
) -> Option<Characteristics> {
    let mut ctx = CharacteristicEvalContext::default();
    calculate_characteristics_inner(state, object_id, &mut ctx)
}
```

The internal calculation should use a context-aware activity check:

```rust
fn calculate_characteristics_inner(
    state: &GameState,
    object_id: ObjectId,
    ctx: &mut CharacteristicEvalContext,
) -> Option<Characteristics> {
    // Existing base-characteristic and layer logic.

    let active_effects: Vec<&ContinuousEffect> = state
        .continuous_effects
        .iter()
        .filter(|effect| {
            is_effect_active_with_context(state, effect, ctx)
        })
        .collect();

    // Existing layer processing.
}
```

Before evaluating a condition, mark the effect as in progress. A nested query
must not reevaluate that same effect's condition.

Conceptually:

```rust
fn is_effect_active_with_context(
    state: &GameState,
    effect: &ContinuousEffect,
    ctx: &mut CharacteristicEvalContext,
) -> bool {
    if !is_effect_duration_active(state, effect) {
        return false;
    }

    let Some(condition) = &effect.condition else {
        return true;
    };

    if !ctx.evaluating_conditions.insert(effect.id) {
        // This condition is already being evaluated higher in the query tree.
        // Do not allow the effect to bootstrap itself.
        return false;
    }

    let result = evaluate_static_condition_with_context(
        state,
        condition,
        effect,
        ctx,
    );

    ctx.evaluating_conditions.remove(&effect.id);
    result
}
```

The recursive branch in `YouControlNOrMoreWithFilter` must call the internal,
context-aware calculation:

```rust
let Some(chars) =
    calculate_characteristics_inner(state, obj.id, ctx)
else {
    return false;
};
```

### Cleanup safety

The in-progress marker must always be removed, including on early returns. Use a
small helper, guard object, or closure pattern so a later refactor cannot leave
an `EffectId` permanently marked within one evaluation.

### Semantic limitation of the immediate patch

Treating a recursively encountered effect as inactive is appropriate for the
Archangel reproducer because granting shroud does not change whether the player
controls three artifacts.

This should not be declared a complete implementation of every possible
self-referential conditional continuous effect. Same-layer or self-supporting
conditions need explicit rules review and tests before the behavior is
generalized.

---

## Durable Architectural Repair

The current `is_effect_active` combines two different questions:

```text
Is the effect still present and running?
Is its conditional static ability currently satisfied?
```

These should be separated.

```rust
fn is_effect_duration_active(
    state: &GameState,
    effect: &ContinuousEffect,
) -> bool;

fn is_effect_condition_satisfied(
    state: &GameState,
    effect: &ContinuousEffect,
    ctx: &mut CharacteristicEvalContext,
) -> bool;
```

Duration/source liveness can usually be evaluated from raw state:

- source exists;
- source is on the battlefield;
- source is phased in;
- until-end-of-turn effect has not expired;
- pairing/control-duration markers remain valid.

Condition evaluation may require derived characteristics and therefore must
participate in the characteristic evaluation graph.

### Layer-bounded characteristic queries

`YouControlNOrMoreWithFilter` currently asks for fully resolved
characteristics even when it only needs a card type.

For Metalcraft, the relevant question is whether the objects are artifacts.
That requires type-changing effects through Layer 4, not Layer 6 abilities or
Layer 7 power/toughness.

Add an internal API such as:

```rust
fn calculate_characteristics_through(
    state: &GameState,
    object_id: ObjectId,
    through_layer: EffectLayer,
    ctx: &mut CharacteristicEvalContext,
) -> Option<Characteristics>;
```

The filter evaluator can determine the highest layer it requires.

Example policy:

```rust
impl TargetFilter {
    fn required_characteristic_layer(&self) -> EffectLayer {
        if self.max_power.is_some() || self.min_power.is_some() {
            EffectLayer::PtSwitch
        } else if !self.has_keywords.is_empty() {
            EffectLayer::Ability
        } else if self.has_color_requirement() {
            EffectLayer::ColorChange
        } else {
            EffectLayer::TypeChange
        }
    }
}
```

The exact method must account for every `TargetFilter` field used by
`matches_filter`, including:

- power/toughness;
- card types;
- supertypes;
- subtypes;
- colors;
- keywords.

Raw `GameObject` properties such as counters, controller, tapped status, combat
status, and token status should remain separate from characteristic-layer
requirements.

### Same-layer conditions

A condition controlling a Layer 6 effect may itself ask whether an object has a
keyword, which also requires Layer 6.

That is not an ordinary lower-layer query. It may require dependency or
fixed-point semantics.

Until supported intentionally, the engine should detect this class through
validation or debug assertions rather than permitting unconstrained recursion.

A useful invariant is:

```text
A condition controlling an effect in layer L may freely query characteristics
through layers strictly earlier than L.

Queries requiring layer L or later must use an explicitly supported
same-layer/dependency path.
```

This should be verified against the Comprehensive Rules and real card examples
before being enforced globally.

---

## Rejected Fixes

### Use printed/base characteristics in the condition

Do not replace the nested call with:

```rust
obj.characteristics
```

That prevents recursion but is wrong when continuous effects add or remove the
Artifact type in Layer 4.

### Add an arbitrary recursion-depth limit

A depth limit converts a deterministic stack overflow into a
board-size-dependent fallback. It does not define correct rules behavior.

A depth limit may be useful as a final panic-prevention assertion, but it is not
the semantic fix.

### Special-case Indomitable Archangel

The card exposes a general engine defect. A card-specific branch would leave
other characteristic-based conditional effects vulnerable.

### Use thread-local or global mutable recursion state

The evaluation state belongs to one characteristic query tree. Thread-local or
global state complicates tests, parallelism, reentrancy, and future caching.

Pass the context explicitly.

---

## Required Regression Tests

### Crash reproducer

```rust
#[test]
fn indomitable_archangel_characteristics_terminate() {
    // Archangel plus three controlled artifacts.
    // Query characteristics for every battlefield object.
    // Pre-fix behavior: stack overflow.
}
```

This must be a true discriminating regression test: reverting the fix should
reproduce the failure.

### Metalcraft true

```rust
#[test]
fn indomitable_archangel_grants_shroud_with_three_artifacts() {
    // The controller has at least three layer-resolved artifacts.
    // Each controlled artifact has Shroud.
}
```

### Metalcraft false

```rust
#[test]
fn indomitable_archangel_does_not_grant_shroud_with_two_artifacts() {
    // No artifact receives Shroud.
}
```

### Query unrelated object

```rust
#[test]
fn conditional_static_effect_does_not_recurse_for_unrelated_query() {
    // Archangel is present.
    // Query a land or nonartifact creature.
    // The query still terminates.
}
```

### Layer 4 adds Artifact

```rust
#[test]
fn metalcraft_counts_permanent_made_artifact_in_layer_four() {
    // One of the three qualifying artifacts is not printed as an artifact,
    // but becomes one through a Layer 4 continuous effect.
    // Metalcraft is satisfied.
}
```

### Layer 4 removes Artifact

```rust
#[test]
fn metalcraft_does_not_count_permanent_that_loses_artifact_type() {
    // A printed artifact loses Artifact in Layer 4.
    // It does not contribute to Metalcraft.
}
```

### Multiple conditional effects

```rust
#[test]
fn nested_distinct_conditional_effects_terminate_deterministically() {
    // Two different conditional static effects both inspect characteristics.
    // Ensures the guard is keyed by EffectId rather than a single boolean.
}
```

### No stale context entry

```rust
#[test]
fn failed_or_false_condition_does_not_poison_later_queries() {
    // Evaluate a false condition, then change the test state or query another
    // object in the same evaluation path.
    // Confirms cleanup of the in-progress marker.
}
```

---

## Acceptance Criteria

The remediation is complete when:

1. The Archangel legal-deck reproducer no longer stack-overflows.
2. Reverting the cycle-control change makes the regression test fail or crash.
3. Metalcraft uses layer-resolved Artifact status, not printed type alone.
4. Nested characteristic queries reuse one explicit evaluation context.
5. The same `EffectId` cannot recursively evaluate its own condition.
6. Distinct conditional effects can nest without being incorrectly suppressed.
7. Existing layer, dependency, affected-set, copy, LKI, and replay tests remain
   green.
8. A code search identifies and reviews every characteristic query reachable
   from `check_static_condition`.
9. Documentation no longer claims that immutability guarantees recursive
   termination.
10. The implementation documents what happens when a condition requires the
    same layer as the effect it controls.

---

## Suggested Agent Task Brief

> Fix the confirmed unbounded recursion between
> `rules::layers::calculate_characteristics`,
> `rules::layers::is_effect_active`, and
> `effects::check_static_condition`.
>
> The reproducer is `Indomitable Archangel`:
> `YouControlNOrMoreWithFilter` calls `expect_characteristics` while the
> Archangel effect's own activity is being evaluated.
>
> Implement an explicit per-query characteristic evaluation context keyed by
> `EffectId`. Nested characteristic calculations must reuse the context and
> must not reevaluate a conditional effect whose condition is already active in
> the current query tree.
>
> Do not fix this by using printed characteristics, adding a depth limit, or
> special-casing the card.
>
> First ship the minimal cycle-safe repair with discriminating regression tests.
> Then split duration liveness from condition satisfaction and introduce a
> layer-bounded internal characteristic query for `TargetFilter` evaluation.
>
> Audit every `calculate_characteristics` or `expect_characteristics` call
> reachable from `check_static_condition`.
>
> Preserve existing dependency ordering, `affected_set` behavior, LKI behavior,
> protocol/hash invariants, and card-definition isolation.

---

## Review Notes

This report is based on static inspection of the uploaded files. It identifies
the direct recursion and the relevant architectural boundary.

It does not include:

- a compiled patch;
- a runtime backtrace;
- a complete audit of every `Condition` variant;
- a Comprehensive Rules determination for every possible same-layer
  self-referential condition.

Those should be completed by the project agent during implementation and
review.
