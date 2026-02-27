# Ability Plan: Afterlife

**Generated**: 2026-02-27
**CR**: 702.135
**Priority**: P3
**Similar abilities studied**: Persist (SelfDies trigger in `state/builder.rs:470-495`), Annihilator (parameterized keyword in `state/builder.rs:423-436`), Swan Song (CreateToken with flying keyword in `cards/definitions.rs:944-956`)

## CR Rule Text

> **702.135.** Afterlife
>
> **702.135a** Afterlife is a triggered ability. "Afterlife N" means "When this permanent is put into a graveyard from the battlefield, create N 1/1 white and black Spirit creature tokens with flying."
>
> **702.135b** If a permanent has multiple instances of afterlife, each triggers separately.

## Key Edge Cases

- **Multiple instances (CR 702.135b)**: If a creature has both Afterlife 1 and Afterlife 2 (e.g., from an ability-granting effect), each triggers separately, producing 1 + 2 = 3 tokens total. Already handled by builder.rs generating one `TriggeredAbilityDef` per keyword instance.
- **Token is multi-color**: The Spirit token is both white AND black (two colors in `OrdSet<Color>`). Existing `TokenSpec.colors` field supports `OrdSet<Color>`, so `[Color::White, Color::Black]` works natively.
- **Token has flying**: The Spirit token has the Flying keyword. Existing `TokenSpec.keywords` supports `OrdSet<KeywordAbility>`, so `[KeywordAbility::Flying]` works natively.
- **"Dies" equivalence (CR 700.4)**: "Put into a graveyard from the battlefield" = "dies". The existing `TriggerEvent::SelfDies` trigger covers this exactly.
- **No intervening-if**: Unlike Persist/Undying, Afterlife has no intervening-if condition. The trigger always fires when the permanent dies, regardless of counters or other state.
- **Tokens created under controller at death time (CR 603.3a)**: The `PendingTrigger` captures the death-time controller, not the owner. If Player A controls Player B's creature with Afterlife and it dies, Player A gets the Spirit tokens. Already handled by the `death_controller` field in the `CreatureDied` event path.
- **Token with Afterlife**: A token creature with Afterlife dies, trigger fires, Spirit tokens are created. The original token ceases to exist in the graveyard (SBA CR 704.5d), but the trigger was already queued and the Spirit tokens are created normally.
- **Afterlife on noncreature permanents**: CR 702.135a says "permanent" not "creature". In practice, all printed Afterlife cards are creatures, so `CreatureDied` covers them. If a noncreature permanent somehow had Afterlife, `AuraFellOff` would cover Auras, but other permanent types (artifact, enchantment) would need a separate "noncreature permanent dies" event. This is a pre-existing gap not specific to Afterlife; no action needed now.
- **Blockers ruling**: "You can't block with a creature with afterlife, wait for it to die, then block with the resulting Spirit tokens" -- blockers are declared simultaneously (CR 509.1a). Not a rules enforcement issue; just a reminder for script authors.
- **Multiplayer APNAP**: Multiple Afterlife creatures dying simultaneously from different players produce triggers ordered by APNAP (CR 603.3). Already handled by the existing trigger dispatch in `abilities.rs`.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (builder.rs trigger generation)
- [ ] Step 3: Trigger wiring (no additional wiring needed -- uses existing SelfDies path)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Afterlife(u32)` variant after `BattleCry` (line 316), before the closing `}` of the `KeywordAbility` enum.
**Pattern**: Follow `Annihilator(u32)` at line 269 (parameterized keyword).

```rust
/// CR 702.135: Afterlife N -- "When this permanent is put into a graveyard
/// from the battlefield, create N 1/1 white and black Spirit creature tokens
/// with flying."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.135b).
Afterlife(u32),
```

**Hash file**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant 42 for `Afterlife(n)` after `BattleCry` (line 370).

```rust
// Afterlife (discriminant 42) -- CR 702.135
KeywordAbility::Afterlife(n) => {
    42u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**View model file**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add display string after `BattleCry` (line 608).

```rust
KeywordAbility::Afterlife(n) => format!("Afterlife {n}"),
```

**Match arm audit**: Grep for exhaustive `match` expressions on `KeywordAbility` across the codebase and add the `Afterlife(n)` arm to each. The compiler will flag these as errors if missed (non-exhaustive pattern), so this is automatically enforced.

### Step 2: Builder Trigger Generation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add Afterlife trigger generation block after the Undying block (after line 528), inside the same keyword-iteration loop.
**Pattern**: Follow Persist at lines 470-495 for the SelfDies trigger pattern, combined with Annihilator at lines 423-436 for the parameterized `n` pattern.
**CR**: 702.135a -- the trigger fires on SelfDies; the effect is CreateToken with the Spirit token spec.

```rust
// CR 702.135a: Afterlife N -- "When this permanent is put into a
// graveyard from the battlefield, create N 1/1 white and black Spirit
// creature tokens with flying."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.135b).
if let KeywordAbility::Afterlife(n) = kw {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: format!(
            "Afterlife {n} (CR 702.135a): When this permanent dies, \
             create {n} 1/1 white and black Spirit creature token(s) \
             with flying."
        ),
        effect: Some(Effect::CreateToken {
            spec: crate::cards::card_definition::TokenSpec {
                name: "Spirit".to_string(),
                power: 1,
                toughness: 1,
                colors: [Color::White, Color::Black].into_iter().collect(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                keywords: [KeywordAbility::Flying].into_iter().collect(),
                count: *n,
                tapped: false,
                mana_color: None,
                mana_abilities: vec![],
            },
        }),
    });
}
```

**Note**: The `TokenSpec.count` field is set to `*n` (the Afterlife parameter). The `CreateToken` effect handler in `effects/mod.rs:293` already loops `0..spec.count` to create multiple tokens, so no changes are needed in the effect execution layer.

### Step 3: Trigger Wiring

**No additional wiring needed.** Afterlife uses the existing `TriggerEvent::SelfDies` trigger pathway, which is already fully implemented for `CreatureDied` events (in `abilities.rs:783-832`) and `AuraFellOff` events (in `abilities.rs:835-871`). The builder generates the `TriggeredAbilityDef` with `trigger_on: TriggerEvent::SelfDies`, and the trigger dispatch in `check_triggers` matches it automatically.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/afterlife.rs`
**Pattern**: Follow `crates/engine/tests/persist.rs` for SelfDies trigger test structure. Use the same helper functions (`find_by_name`, `find_by_name_in_zone`, `count_on_battlefield`, `pass_all`).

**Tests to write**:

1. **`test_afterlife_basic_creates_spirit_token`** -- CR 702.135a
   - Setup: 2/2 creature with `KeywordAbility::Afterlife(1)`, lethal damage marked.
   - Expected: After SBA, dies trigger queued. After trigger resolves, one 1/1 Spirit token on battlefield with flying, colors white+black, subtype Spirit.
   - Verify token characteristics: name "Spirit", power 1, toughness 1, colors contain White and Black, keywords contain Flying, subtypes contain "Spirit".

2. **`test_afterlife_n_creates_n_tokens`** -- CR 702.135a (N > 1)
   - Setup: 2/1 creature with `KeywordAbility::Afterlife(3)`, lethal damage marked.
   - Expected: After trigger resolves, exactly 3 Spirit tokens on battlefield.
   - Verify: `count_on_battlefield(state, "Spirit") == 3`.

3. **`test_afterlife_no_intervening_if`** -- CR 702.135a (contrast with Persist)
   - Setup: 2/2 creature with `KeywordAbility::Afterlife(1)` AND a -1/-1 counter AND lethal damage.
   - Expected: Afterlife still triggers (no intervening-if condition). One Spirit token created.
   - Contrast: Persist would NOT trigger in this scenario.

4. **`test_afterlife_token_dies_still_creates_spirits`** -- CR 702.135a + CR 704.5d
   - Setup: Token creature with `KeywordAbility::Afterlife(1)`, lethal damage.
   - Expected: Token dies, Afterlife trigger fires, Spirit token created. Original token ceases to exist (SBA), but Spirit token persists.

5. **`test_afterlife_multiple_instances_trigger_separately`** -- CR 702.135b
   - Setup: 2/2 creature with both `KeywordAbility::Afterlife(1)` AND `KeywordAbility::Afterlife(2)`, lethal damage.
   - Expected: Two triggers on stack. After both resolve, 1 + 2 = 3 Spirit tokens total.

6. **`test_afterlife_multiplayer_apnap`** -- CR 603.3
   - Setup: 4 players. P1 and P3 each have a creature with Afterlife 1, both with lethal damage.
   - Expected: Both triggers on stack in APNAP order. After resolution, each controller has one Spirit token.

### Step 5: Card Definition (later phase)

**Suggested card**: Ministrant of Obligation
- Type: Creature -- Human Cleric
- Mana cost: {2}{W}
- P/T: 2/1
- Keywords: Afterlife 2
- Oracle: "Afterlife 2 (When this creature dies, create two 1/1 white and black Spirit creature tokens with flying.)"
- Simple stats, no other abilities, good for testing Afterlife 2 (N > 1).

**Alternative card**: Orzhov Enforcer (1B, 1/2, Deathtouch + Afterlife 1) -- tests interaction with another keyword.

**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Afterlife creature dies in combat, Spirit tokens created"
- P1 attacks with Ministrant of Obligation (2/1, Afterlife 2).
- P2 blocks with a 2/2 creature.
- Combat damage kills Ministrant. Afterlife 2 triggers.
- Trigger resolves: two 1/1 white and black Spirit tokens with flying appear on P1's battlefield.
- Verify: 2 Spirit tokens, original creature in graveyard.

**Subsystem directory**: `test-data/generated-scripts/combat/` (dies in combat) or `test-data/generated-scripts/stack/` (trigger resolution).

## Interactions to Watch

- **Afterlife + Sacrifice**: When a creature with Afterlife is sacrificed (e.g., as part of an effect or cost), it dies, so Afterlife triggers. The trigger fires regardless of how the creature was put into the graveyard from the battlefield.
- **Afterlife + Exile instead of die**: If the creature is exiled instead of dying (e.g., Swords to Plowshares), Afterlife does NOT trigger because the creature was not "put into a graveyard from the battlefield." The `SelfDies` trigger only fires on `CreatureDied` events, not exile events.
- **Afterlife + Persist/Undying**: A creature with both Afterlife and Persist that has no -1/-1 counters and dies will trigger BOTH Afterlife and Persist. Both triggers go on the stack. The controller chooses the order. If Persist resolves first, the creature returns to the battlefield (and Afterlife's CreateToken still resolves, creating the tokens). If Afterlife resolves first, tokens are created, then Persist returns the creature.
- **Afterlife + Commander (CR 903.9a)**: If a commander creature has Afterlife and dies, the commander zone-change SBA fires AFTER the creature moves to the graveyard. The Afterlife trigger was already queued when `CreatureDied` fired (before the SBA moves the commander to the command zone). The trigger still resolves normally and creates Spirit tokens, even though the source card is now in the command zone rather than the graveyard. This is consistent with how Persist's `MoveZone` effect would silently fail if the source is no longer in the graveyard.
- **Multiplayer implications**: All standard -- APNAP ordering for simultaneous deaths, controller at time of death gets the tokens.
