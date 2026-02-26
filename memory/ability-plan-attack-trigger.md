# Ability Plan: Attack Trigger

**Generated**: 2026-02-26
**CR**: 508.1m, 508.2a, 508.2b, 508.3a, 603.2, 603.5
**Priority**: P1
**Similar abilities studied**: Dies Trigger (`WhenDies` -> `SelfDies` enrichment at `replay_harness.rs:408-424`, dispatch at `abilities.rs:431-469`, 10 unit tests in `tests/abilities.rs:1022-1609`)

## CR Rule Text

### CR 508.1m
> Any abilities that trigger on attackers being declared trigger.

### CR 508.2a
> Abilities that trigger on a creature attacking trigger only at the point the creature
> is declared as an attacker. They will not trigger if a creature attacks and then that
> creature's characteristics change to match the ability's trigger condition.

### CR 508.2b
> Any abilities that triggered on attackers being declared or that triggered during the
> process described in rules 508.1 are put onto the stack before the active player gets
> priority; the order in which they triggered doesn't matter. (See rule 603.)

### CR 508.3a
> An ability that reads "Whenever [a creature] attacks, . . ." triggers if that creature
> is declared as an attacker. Similarly, "Whenever [a creature] attacks [a player,
> planeswalker, or battle], . . ." triggers if that creature is declared as an attacker
> attacking that player or permanent. Such abilities won't trigger if a creature is put
> onto the battlefield attacking.

### CR 508.4 (key exclusion)
> If a creature is put onto the battlefield attacking, its controller chooses which
> defending player [...] it's attacking [...]. Such creatures are "attacking" but, for
> the purposes of trigger events and effects, they never "attacked."

### CR 603.2
> Whenever a game event or game state matches a triggered ability's trigger event, that
> ability automatically triggers. The ability doesn't do anything at this point.

### CR 603.5
> Some triggered abilities' effects are optional (they contain "may"). These abilities go
> on the stack when they trigger, regardless of whether their controller intends to
> exercise the ability's option or not.

## Key Edge Cases

- **CR 508.3a**: "Whenever ~ attacks" triggers ONLY when declared as attacker. Creatures put onto the battlefield attacking (e.g., Hero of Bladehold tokens, Geist of Saint Traft's Angel) do NOT trigger "whenever ~ attacks" on themselves.
- **CR 508.2a**: Trigger condition is checked at declaration time only. If the creature later gains/loses the trigger ability, it doesn't retroactively fire or un-fire.
- **CR 508.4**: Tokens created "tapped and attacking" were never "declared as attackers" -- they don't trigger "attacks" triggers. (Not currently relevant since the engine doesn't support "enters attacking" yet, but important for correctness documentation.)
- **APNAP ordering (CR 603.3b)**: In multiplayer, if multiple creatures from different players attack (unlikely in standard turn structure, but possible with extra combat effects), triggers go on stack in APNAP order. Currently only the active player can declare attackers, so this is a single-controller scenario, but the APNAP infrastructure already handles it.
- **Self-referential only**: The current `WhenAttacks` / `SelfAttacks` is for "When THIS creature attacks" only. "Whenever A creature attacks" would be a different `TriggerEvent` variant (not in scope).

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `TriggerCondition::WhenAttacks` exists at `cards/card_definition.rs:470`; `TriggerEvent::SelfAttacks` exists at `state/game_object.rs:111`; hash support at `state/hash.rs:890,1830`
- [x] Step 2: Rule enforcement -- `GameEvent::AttackersDeclared` emitted by `combat.rs:285-288`; `check_triggers` dispatches `SelfAttacks` at `abilities.rs:377-388`; triggers flushed at `combat.rs:296-298`
- [ ] Step 3: Enrichment wiring -- `enrich_spec_from_def` does NOT convert `WhenAttacks` to `SelfAttacks` (the gap)
- [ ] Step 4: Unit tests -- one basic test exists (`combat.rs:750`) but no enrichment-path test or effect-resolution test
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant (DONE)

Both enum variants already exist:
- `TriggerCondition::WhenAttacks` at `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs:470`
- `TriggerEvent::SelfAttacks` at `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs:111`
- Hash support at `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:890` (`TriggerEvent::SelfAttacks => 4u8`) and `hash.rs:1830` (`TriggerCondition::WhenAttacks => 2u8`)

No changes needed.

### Step 2: Rule Enforcement / Dispatch (DONE)

The runtime dispatch already exists:
- `GameEvent::AttackersDeclared` emitted by `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs:285-288`
- `check_triggers` match arm at `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:377-388` iterates each attacker and calls `collect_triggers_for_event(state, &mut triggers, TriggerEvent::SelfAttacks, Some(*attacker_id), None)`
- Trigger flush at `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs:290-298` calls `check_triggers` + `flush_pending_triggers` after the `AttackersDeclared` event

No changes needed.

### Step 3: Enrichment Wiring (THE GAP)

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `WhenAttacks -> SelfAttacks` conversion block in `enrich_spec_from_def`, immediately after the existing `WhenDies -> SelfDies` block (lines 408-425).

**Pattern**: Follow the exact `WhenDies` pattern at lines 408-425:

```rust
// CR 508.1m / CR 508.3a: Convert "When ~ attacks" card-definition triggers into
// runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
// This covers self-referential attack triggers (e.g. Audacious Thief).
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenAttacks,
        effect,
        ..
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfAttacks,
            intervening_if: None,
            description: "When ~ attacks (CR 508.3a)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

**Insertion point**: After line 425 (closing brace of the `WhenDies` block) in `enrich_spec_from_def`.

**CR**: 508.1m ("Any abilities that trigger on attackers being declared trigger"), 508.3a ("Whenever [a creature] attacks" triggers if declared as attacker).

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/abilities.rs`
**Tests to write** (add after the dies trigger tests, around line 1609):

#### Test 1: `test_attack_trigger_fires_on_declare_attackers`
- **CR**: 508.1m, 508.3a, 603.2
- **What**: A creature with a manually-constructed `SelfAttacks` trigger is declared as an attacker. Verify `AbilityTriggered` event is emitted and the triggered ability is on the stack.
- **Note**: This partially overlaps with `test_603_self_attacks_trigger_fires` in `combat.rs:750`, but that test uses 2 players. This test uses 4 players for multiplayer coverage and verifies more assertions (stack object kind, controller).
- **Pattern**: Follow `test_dies_trigger_fires_on_lethal_damage_sba` at line 1024.

#### Test 2: `test_attack_trigger_via_card_definition_enrich_path`
- **CR**: 508.3a, enrichment path
- **What**: Build a creature using `enrich_spec_from_def` from a card definition that has `AbilityDefinition::Triggered { trigger_condition: WhenAttacks, effect: DrawCards { ... } }`. Declare it as an attacker. Verify the trigger fires.
- **This is the critical test** -- it validates that the enrichment gap is fixed.
- **Pattern**: Follow `test_dies_trigger_via_card_definition_enrich_path` at line 1522.
- **Approach**: Build a temporary `CardDefinition` with `WhenAttacks` trigger (since no card in `definitions.rs` uses it yet). Or, if Audacious Thief has been added by step 5, use that. Since step 5 may run after step 4, build the CardDefinition inline in the test (like the dies trigger test uses `all_cards()` for Solemn Simulacrum -- but since no attack-trigger card exists yet, create a custom def).

#### Test 3: `test_attack_trigger_resolves_draws_card`
- **CR**: 508.3a, 603 (trigger resolution)
- **What**: A creature with a `SelfAttacks` trigger that has `Effect::DrawCards { amount: 1 }`. Declare it as attacker, then all players pass priority to resolve the trigger. Verify the controller draws a card (hand count increases by 1).
- **Pattern**: Follow `test_dies_trigger_resolves_draws_card` at line 1204.

#### Test 4: `test_attack_trigger_does_not_fire_for_non_attacker`
- **CR**: 508.3a (negative test)
- **What**: A creature with `SelfAttacks` trigger exists on the battlefield, but a DIFFERENT creature is declared as the attacker. Verify the trigger does NOT fire on the non-attacking creature.
- **Pattern**: Custom negative test; verify no `AbilityTriggered` event for the trigger-bearing creature.

#### Test 5: `test_attack_trigger_multiple_attackers`
- **CR**: 508.1m, 603.3b (APNAP)
- **What**: Two creatures with `SelfAttacks` triggers are declared as attackers simultaneously. Verify both triggers fire (2 `AbilityTriggered` events, 2 stack objects).
- **Pattern**: Follow `test_dies_trigger_multiple_creatures_simultaneous_sba` at line 1390.

**File**: `/home/airbaggie/scutemob/crates/engine/tests/combat.rs`
- The existing `test_603_self_attacks_trigger_fires` at line 750 already covers the basic case. No changes needed to this file.

### Step 5: Card Definition (later phase)

**Suggested card**: Audacious Thief
- **Oracle text**: "Whenever this creature attacks, you draw a card and you lose 1 life."
- **Mana cost**: {2}{B}
- **Type**: Creature -- Human Rogue
- **P/T**: 2/2
- **Why**: Simplest possible attack trigger -- draws a card and loses 1 life. Both `DrawCards` and `LoseLife` effects already exist in the engine. No tokens, no counters, no complex conditions.
- **Card lookup**: Use `card-definition-author` agent with "Audacious Thief".
- **File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
- **Card definition sketch**:
  ```rust
  CardDefinition {
      card_id: cid("audacious-thief"),
      name: "Audacious Thief".to_string(),
      types: TypeLine { card_types: vec![CardType::Creature], ..Default::default() },
      cost: ManaCost { generic: 2, black: 1, ..Default::default() },
      oracle_text: "Whenever Audacious Thief attacks, you draw a card and you lose 1 life.".to_string(),
      abilities: vec![
          AbilityDefinition::Triggered {
              trigger_condition: TriggerCondition::WhenAttacks,
              effect: Effect::Sequence(vec![
                  Effect::DrawCards { amount: EffectAmount::Fixed(1) },
                  Effect::LoseLife { amount: EffectAmount::Fixed(1) },
              ]),
              intervening_if: None,
          },
      ],
      subtypes: vec![SubType::Human, SubType::Rogue],
      power: Some(2),
      toughness: Some(2),
      timing: TimingRestriction::SorcerySpeed,
      ..Default::default()
  }
  ```

### Step 6: Game Script (later phase)

**Suggested scenario**: "Audacious Thief attack trigger draws a card"
**Subsystem directory**: `/home/airbaggie/scutemob/test-data/generated-scripts/combat/`
**Script outline**:
1. p1 controls Audacious Thief on battlefield (enriched from card def)
2. Advance to DeclareAttackers step
3. p1 declares Audacious Thief attacking p2
4. Attack trigger fires (AbilityTriggered event)
5. All players pass priority -> trigger resolves
6. Assert: p1's hand count increased by 1 (drew a card)
7. Assert: p1's life total decreased by 1 (lost 1 life from trigger)

### Step 7: Coverage Doc Update

**File**: `/home/airbaggie/scutemob/docs/mtg-engine-ability-coverage.md`
**Action**: Update the "Attack trigger" row from `partial` to `validated` (after all tests pass and script is approved).

## Interactions to Watch

- **Trigger fires in `handle_declare_attackers`, not in a separate turn-based action.** The `check_triggers` + `flush_pending_triggers` calls at `combat.rs:290-298` already handle this correctly. The trigger goes on the stack BEFORE the active player receives priority (CR 508.2b).
- **Vigilance interaction**: Vigilance creatures don't tap when attacking (CR 508.1f). The trigger fires regardless of whether the creature taps -- attacking is the trigger, not tapping. The engine already handles this correctly (the trigger dispatch checks `SelfAttacks`, not `SelfBecomesTapped`).
- **Summoning sickness**: A creature with summoning sickness and no haste can't attack at all. If it can't declare as an attacker, the trigger never fires. This is enforced by `handle_declare_attackers` validation at `combat.rs:112-117`.
- **Protection from the defender**: Protection prevents blocking (the B in DEBT) but does NOT prevent attacking. A creature with protection can still declare as an attacker, and its "when attacks" trigger still fires.
- **Multiple combat phases**: If a player gets extra combat phases (not currently supported), the creature could trigger multiple times. Not relevant for current engine scope but worth noting.
- **"Put onto the battlefield attacking" (CR 508.4)**: Tokens/creatures that enter attacking without being "declared as attackers" should NOT trigger "whenever ~ attacks." The current engine does not support "enters attacking" so this is not a current concern, but the dispatch in `abilities.rs:377-388` correctly only fires on `AttackersDeclared` events, not on any other mechanism.
