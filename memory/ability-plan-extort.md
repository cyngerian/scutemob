# Ability Plan: Extort

**Generated**: 2026-02-27
**CR**: 702.101
**Priority**: P3
**Similar abilities studied**: Prowess (builder.rs trigger translation + controller-cast filtering in abilities.rs), Ward (MayPayOrElse pattern), Annihilator (ForEach with player targets), Afterlife (SelfDies trigger)

## CR Rule Text

**702.101.** Extort

**702.101a** Extort is a triggered ability. "Extort" means "Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain life equal to the total life lost this way."

**702.101b** If a permanent has multiple instances of extort, each triggers separately.

## Key Edge Cases

From CR and card rulings (Syndic of Tithes, Tithe Drinker):

1. **Extort resolves before the triggering spell** (ruling 2024-01-12): The extort ability resolves even if the triggering spell is countered. This is standard triggered ability behavior -- it goes on the stack above the spell and resolves first.

2. **Life gained equals total life ACTUALLY lost** (ruling 2024-01-12): "The amount of life you gain from extort is based on the total amount of life lost, not necessarily the number of opponents you have. For example, if your opponent's life total can't change (perhaps because that player controls Platinum Emperion), you won't gain any life." This means the life gain is NOT fixed at `opponents_count * 1` -- it depends on how much life was actually lost. The engine must track actual life lost.

3. **Does not target** (ruling 2024-01-12): "The extort ability doesn't target any player." This means hexproof/shroud/protection do not prevent extort from affecting opponents.

4. **One payment per trigger** (ruling 2024-01-12): "You may pay {W/B} a maximum of one time for each extort triggered ability. You decide whether to pay when the ability resolves."

5. **Multiple instances trigger separately** (CR 702.101b): If a permanent has multiple instances of extort, each triggers separately (each is a separate chance to pay and drain). Multiple permanents with extort also each trigger separately.

6. **Triggers on ANY spell, including creatures** (CR 702.101a): Unlike Prowess (noncreature only), extort triggers whenever "you cast a spell" with no type restriction.

7. **Multiplayer drain**: In a 4-player Commander game, each opponent (3 players) loses 1 life each. If all 3 lose 1, the controller gains 3. If 1 opponent's life can't change, controller gains 2 instead of 3.

8. **{W/B} is hybrid mana**: Can be paid with either white or black mana. The engine's ManaCost struct does not currently support hybrid mana -- model as generic for now (deferred to hybrid mana infrastructure).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

No existing Extort code anywhere in the engine.

## Implementation Steps

### Step 1: Enum Variant + Hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Extort` variant after `Afterlife(u32)` (around line 324).
**Pattern**: Follow `KeywordAbility::Exalted` (line 261) -- simple unit variant, no parameters.
**Doc comment**:
```rust
/// CR 702.101: Extort -- "Whenever you cast a spell, you may pay {W/B}.
/// If you do, each opponent loses 1 life and you gain life equal to the
/// total life lost this way."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.101b).
Extort,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `43u8` for `KeywordAbility::Extort` after the `Afterlife` arm (line ~375).
**Pattern**: Follow `KeywordAbility::Exalted => 34u8.hash_into(hasher)` -- simple unit variant.
```rust
// Extort (discriminant 43) -- CR 702.101
KeywordAbility::Extort => 43u8.hash_into(hasher),
```

### Step 2: New TriggerEvent Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `TriggerEvent::ControllerCastsSpell` variant after `ControllerSurveils` (line ~174).
**CR**: 702.101a -- "Whenever you cast a spell" means the trigger fires when the controller of the permanent with extort casts any spell.
**Rationale**: Cannot reuse existing variants:
- `AnySpellCast` (line 135) fires for ALL permanents regardless of controller -- no controller filter.
- `ControllerCastsNoncreatureSpell` (line 150) is Prowess-specific with a noncreature type filter.
- `OpponentCastsSpell` (line 164) has the inverse controller polarity.

```rust
/// CR 702.101a: Triggers when the controller of this permanent casts a spell
/// (any spell, including creature spells). Used by the Extort keyword.
/// The controller-match is verified at trigger-collection time in
/// `rules/abilities.rs`.
ControllerCastsSpell,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `13u8` for `TriggerEvent::ControllerCastsSpell` (after `ControllerSurveils` discriminant 12).
```rust
// CR 702.101a: Controller-casts-any-spell trigger -- discriminant 13
TriggerEvent::ControllerCastsSpell => 13u8.hash_into(hasher),
```

### Step 3: New Effect Variant — DrainLife

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::DrainLife { amount: EffectAmount }` variant after the existing life effects section (around line 223, after `LoseLife`).
**CR**: 702.101a -- "each opponent loses 1 life and you gain life equal to the total life lost this way."
**Rationale**: Cannot decompose into existing primitives. A `Sequence([LoseLife { EachOpponent, 1 }, GainLife { Controller, ??? }])` doesn't work because the `GainLife` amount depends on the actual life lost by opponents (which could be less than `opponents_count` if any opponent's life total can't change). A dedicated effect tracks the actual total.

```rust
/// CR 702.101a: Each opponent of the controller loses `amount` life, and the
/// controller gains life equal to the total life actually lost by all opponents.
///
/// This is NOT the same as LoseLife + GainLife because the gain depends on
/// the actual life change, not the intended loss (relevant when an opponent's
/// life total can't change, e.g., Platinum Emperion).
///
/// The "controller" is the controller of the spell or ability that created
/// this effect (from EffectContext).
DrainLife {
    amount: EffectAmount,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add handler for `Effect::DrainLife` in `execute_effect_inner`. Place after the `LoseLife` handler (after line ~260).
**Implementation**:
```rust
Effect::DrainLife { amount } => {
    let loss = resolve_amount(state, amount, ctx).max(0) as u32;
    if loss == 0 {
        return;
    }
    // Collect opponents of the controller.
    let opponents: Vec<PlayerId> =
        resolve_player_target_list(state, &PlayerTarget::EachOpponent, ctx);
    let mut total_lost: u32 = 0;
    for p in &opponents {
        if let Some(ps) = state.players.get_mut(p) {
            // Track life before to compute actual loss.
            let before = ps.life_total;
            ps.life_total -= loss as i32;
            let actual_loss = (before - ps.life_total).max(0) as u32;
            total_lost += actual_loss;
        }
        events.push(GameEvent::LifeLost {
            player: *p,
            amount: loss,
        });
    }
    // Controller gains life equal to total actually lost.
    if total_lost > 0 {
        if let Some(ps) = state.players.get_mut(&ctx.controller) {
            ps.life_total += total_lost as i32;
        }
        events.push(GameEvent::LifeGained {
            player: ctx.controller,
            amount: total_lost,
        });
    }
},
```
**Note**: In the current engine, `LifeLost` events always carry the *intended* loss amount (not actual). The `total_lost` is computed from the actual life total change. This distinction matters only when "life total can't change" effects exist. For now, the implementation computes actual loss from the delta. If/when Platinum Emperion is implemented, the prevention would happen before the life total mutation, and the delta would correctly be 0.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `33u8` for `Effect::DrainLife` (after `Surveil` discriminant 32).
```rust
Effect::DrainLife { amount } => {
    33u8.hash_into(hasher);
    amount.hash_into(hasher);
}
```

### Step 4: Trigger Wiring — abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `check_triggers`, inside the `GameEvent::SpellCast { player, .. }` arm (line ~592-667), add a new block for `ControllerCastsSpell` after the `ControllerCastsNoncreatureSpell` block and before the `OpponentCastsSpell` block.
**CR**: 702.101a -- "Whenever you cast a spell" = controller of the permanent == caster.
**Pattern**: Follow Prowess wiring (lines 622-637) but without the noncreature filter.

```rust
// CR 702.101a: Extort -- "Whenever you cast a spell."
// Collect triggers only for permanents controlled by the caster.
// No type restriction (unlike Prowess which requires noncreature).
{
    let controller_sources: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();

    for obj_id in controller_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::ControllerCastsSpell,
            Some(obj_id),
            None,
        );
    }
}
```

**Note**: The `only_object: Some(obj_id)` pattern ensures each object is checked individually, matching the Prowess pattern. Each extort instance on each permanent generates its own trigger (CR 702.101b).

### Step 5: Builder.rs Keyword Translation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the `for kw in spec.keywords.iter()` loop (starting line ~350), add a new block for `KeywordAbility::Extort` after the `Afterlife` block (after line ~560).
**Pattern**: Follow Exalted/Annihilator pattern -- simple `if matches!` + `triggered_abilities.push`.
**CR**: 702.101a, 702.101b.

```rust
// CR 702.101a: Extort -- "Whenever you cast a spell, you may pay
// {W/B}. If you do, each opponent loses 1 life and you gain life
// equal to the total life lost this way."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.101b).
// The "may pay" optional cost is deferred to interactive mode (M10+);
// deterministic fallback always resolves the drain effect.
if matches!(kw, KeywordAbility::Extort) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::ControllerCastsSpell,
        intervening_if: None,
        description: "Extort (CR 702.101a): Whenever you cast a spell, \
                      you may pay {W/B}. If you do, each opponent loses 1 \
                      life and you gain that much life."
            .to_string(),
        effect: Some(Effect::DrainLife {
            amount: EffectAmount::Fixed(1),
        }),
    });
}
```

**Note on optional payment**: The `{W/B}` payment is modeled as deterministic-always-pay. The engine's deterministic fallback means triggered abilities always resolve their effects. When interactive mode is added (M10+), this should be wrapped in a `MayPay` or equivalent construct where the player chooses. For now, every extort trigger resolves the drain.

### Step 6: Replay Viewer — view_model.rs

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Extort` arm to `format_keyword` function (after `Afterlife` arm, line ~609).
```rust
KeywordAbility::Extort => "Extort".to_string(),
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/extort.rs`
**Pattern**: Follow `crates/engine/tests/prowess.rs` for test structure, `crates/engine/tests/annihilator.rs` for attack/trigger patterns.

**Tests to write**:

1. **`test_extort_basic_drain_on_spell_cast`** -- CR 702.101a
   - Setup: 4 players, P1 has a creature with Extort on battlefield, P1 has an instant in hand.
   - P1 casts the instant. Extort triggers, goes on stack. Pass priority to resolve extort.
   - Assert: each opponent (P2, P3, P4) lost 1 life, P1 gained 3 life (total actually lost).

2. **`test_extort_triggers_on_creature_spell`** -- CR 702.101a ("any spell")
   - Setup: 4 players, P1 has a creature with Extort on battlefield, P1 has a creature in hand.
   - P1 casts the creature spell. Extort triggers.
   - Assert: extort trigger exists on the stack (the drain fires). This verifies Extort triggers on creature spells (unlike Prowess).

3. **`test_extort_does_not_trigger_for_opponent_spell`** -- CR 702.101a ("you cast")
   - Setup: 4 players, P1 has a creature with Extort. P2 casts a spell during P1's turn (instant).
   - Assert: NO extort trigger fires. Extort only triggers for the controller's spells.

4. **`test_extort_multiple_instances_trigger_separately`** -- CR 702.101b
   - Setup: 4 players, P1 has two permanents each with Extort on battlefield (or one permanent with 2 Extort instances via ObjectSpec).
   - P1 casts a spell. Both extort instances trigger.
   - Assert: each opponent lost 2 life (1 per trigger), P1 gained 6 life (3 per trigger x 2).

5. **`test_extort_does_not_target`** -- Ruling: "doesn't target any player"
   - Setup: 4 players, P1 has Extort creature. P2 has hexproof (or shroud, via a permanent).
   - P1 casts a spell. Extort triggers.
   - Assert: P2 still loses 1 life (hexproof doesn't prevent extort because it doesn't target).

6. **`test_extort_resolves_before_triggering_spell`** -- Ruling: "resolves before the spell"
   - Setup: 2 players, P1 has Extort creature, P1 casts an instant.
   - Assert: after all players pass priority once (letting extort resolve), the triggering spell is still on the stack. Life drain has already occurred.

7. **`test_extort_multiplayer_4_player_drain`** -- Multiplayer specific
   - Setup: 4 players, P1 has Extort, all players at 40 life.
   - P1 casts a spell, extort triggers and resolves.
   - Assert: P2=39, P3=39, P4=39, P1=43 (gained 3).

### Step 8: Card Definition (later phase)

**Suggested card**: Syndic of Tithes
- **Name**: Syndic of Tithes
- **Mana cost**: {1}{W}
- **Types**: Creature -- Human Cleric
- **P/T**: 2/2
- **Keywords**: Extort
- **Oracle text**: "Extort (Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain that much life.)"
- **Color identity**: W (note: the {W/B} in reminder text does NOT affect color identity per CR 903.4b)
- **Card lookup**: use `card-definition-author` agent

**Alternative card**: Tithe Drinker ({W}{B}, 2/1, Lifelink + Extort) -- good for testing keyword interaction with Lifelink.

### Step 9: Game Script (later phase)

**Suggested scenario**: "Extort drain in 4-player Commander"
- P1 controls Syndic of Tithes (2/2 Extort), casts Lightning Bolt targeting P2.
- Extort triggers, goes on stack above Lightning Bolt.
- All players pass priority on extort trigger -- it resolves: P2/P3/P4 each lose 1 life, P1 gains 3.
- All players pass priority on Lightning Bolt -- it resolves: P2 takes 3 damage.
- Final life totals: P1=43, P2=36, P3=39, P4=39.

**Subsystem directory**: `test-data/generated-scripts/stack/` (extort interacts with stack ordering)

## Interactions to Watch

1. **Stack ordering**: Extort trigger goes on the stack above the spell that caused it. It resolves first. The triggering spell resolves even if the extort trigger was countered. The extort trigger resolves even if the triggering spell was countered (it's independent).

2. **Storm interaction** (CR 702.40c from gotchas-rules.md): Storm copies are NOT cast. Extort should NOT trigger for storm copies. Since `GameEvent::SpellCast` is only emitted by `handle_cast_spell` (not by storm copy creation), this is already correct. Verify in tests if storm is available.

3. **Split Second** (CR 702.61): If the triggering spell has Split Second, extort still triggers (CR 702.61b: "triggered abilities still trigger and resolve normally"). No special handling needed.

4. **Cascade interaction**: When cascade resolves and the player casts the cascaded spell, that IS a new cast and DOES trigger extort again. This is correct because the cascade cast goes through `handle_cast_spell`.

5. **Multiple extort sources**: If P1 controls 3 permanents each with extort and casts 1 spell, 3 separate extort triggers go on the stack. Each resolves independently. In 4-player, that's 3 * 3 = 9 total life lost by opponents combined, and P1 gains 9.

6. **{W/B} hybrid mana**: The engine's ManaCost struct doesn't support hybrid mana. The optional payment is deferred to interactive mode entirely (deterministic mode auto-resolves the drain). When interactive mode is added, a `HybridManaCost` or equivalent will be needed.

7. **Life total can't change**: Per rulings, if an opponent's life total can't change, the actual loss is 0 for that player, and the controller gains less. The `DrainLife` implementation tracks actual deltas. This edge case is not testable until Platinum Emperion or similar is implemented, but the infrastructure is correct.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Extort` variant |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::ControllerCastsSpell` variant |
| `crates/engine/src/state/hash.rs` | Add hash arms for `Extort` (43u8), `ControllerCastsSpell` (13u8), `DrainLife` (33u8) |
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::DrainLife { amount }` variant |
| `crates/engine/src/effects/mod.rs` | Add `DrainLife` handler |
| `crates/engine/src/rules/abilities.rs` | Add `ControllerCastsSpell` trigger collection in SpellCast handler |
| `crates/engine/src/state/builder.rs` | Add Extort keyword-to-trigger translation |
| `tools/replay-viewer/src/view_model.rs` | Add `Extort` arm in `format_keyword` |
| `crates/engine/tests/extort.rs` | New test file with 7 tests |
