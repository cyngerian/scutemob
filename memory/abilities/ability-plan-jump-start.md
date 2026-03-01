# Ability Plan: Jump-Start

**Generated**: 2026-03-01
**CR**: 702.133
**Priority**: P4
**Batch**: 4.2 (Alt-cast graveyard)
**Similar abilities studied**: Flashback (CR 702.34) -- `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/tests/flashback.rs`

## CR Rule Text

> **702.133. Jump-Start**
>
> **702.133a** Jump-start appears on some instants and sorceries. It represents two static
> abilities: one that functions while the card is in a player's graveyard and another that
> functions while the card is on the stack. "Jump-start" means "You may cast this card from
> your graveyard if the resulting spell is an instant or sorcery spell by discarding a card
> as an additional cost to cast it" and "If this spell was cast using its jump-start ability,
> exile this card instead of putting it anywhere else any time it would leave the stack."
> Casting a spell using its jump-start ability follows the rules for paying additional costs
> in rules 601.2b and 601.2f-h.

## Key Edge Cases

1. **Jump-start is NOT an alternative cost** (ruling 2018-10-05 on Radical Idea): "If an
   effect allows you to pay an alternative cost rather than a spell's mana cost, you may pay
   that alternative cost when you jump-start a spell. You'll still discard a card as an
   additional cost to cast it." This means the player pays the card's normal mana cost plus
   discards a card. Alternative costs (e.g., cost reducers) can still be applied on top.

2. **Discard is an additional cost, not part of mana payment.** The discard happens during
   cost payment (CR 601.2f-h), before the spell goes on the stack. The discarded card can be
   ANY card in hand (contrast with Retrace which requires a LAND card).

3. **Timing restrictions still apply** (ruling on every jump-start card): "You must still
   follow any timing restrictions and permissions when casting a spell with jump-start,
   including those based on the card's type. For instance, you can cast a sorcery using
   jump-start only when you could normally cast a sorcery." Same as Flashback.

4. **Always exiled afterward** (ruling 2018-10-05): "A spell cast using jump-start will
   always be exiled afterward, whether it resolves, it's countered, or it leaves the stack
   in some other way." Identical to Flashback's departure behavior.

5. **Can be cast right away from graveyard** (ruling 2018-10-05): "If a card with jump-start
   is put into your graveyard during your turn, you'll be able to cast it right away if it's
   legal to do so, before an opponent can take any actions."

6. **Multiplayer**: No special multiplayer considerations beyond normal casting rules. The
   discard cost and exile replacement are player-local mechanics.

7. **Key difference from Flashback**: Flashback pays a Flashback-specific alternative cost;
   Jump-Start pays the card's regular mana cost plus discard-a-card. Flashback cannot combine
   with other alternative costs (CR 118.9a). Jump-Start CAN combine with alternative costs
   (since it is not one itself), but it always requires the discard additional cost.

8. **Key difference from Retrace**: Retrace requires discarding specifically a LAND card;
   Jump-Start accepts ANY card from hand.

9. **Interaction with commander tax**: Jump-Start casts from graveyard using the card's
   regular mana cost. Commander tax applies if the card is somehow both a commander and a
   jump-start spell (unusual edge case). The mana cost pipeline is: base mana cost (or
   whatever alternative cost the player chooses) + commander tax + kicker + cost modifiers.
   The discard is a separate additional cost on top of all mana.

10. **Madness interaction on discard**: If the card discarded as the jump-start additional
    cost has Madness (CR 702.35a), the madness replacement effect fires: the card goes to
    exile instead of graveyard, and a madness trigger is queued. The discard code must
    replicate the madness-check pattern from `abilities.rs` line 460-507 (cycling discard).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (`KeywordAbility::JumpStart`)
- [ ] Step 2: `AbilityDefinition::JumpStart` variant (no cost field -- uses card's regular mana cost)
- [ ] Step 3: Rule enforcement in `casting.rs` (graveyard casting + discard additional cost + exile on departure)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition (Radical Idea)
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::JumpStart` variant after `CommanderNinjutsu` (line 761).
**Doc comment**: `/// CR 702.133: Jump-start -- cast from graveyard by paying mana cost + discarding a card.`
**Pattern**: Follow `KeywordAbility::Flashback` at line 247.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `89u8` for `KeywordAbility::JumpStart` in the `HashInto for KeywordAbility` impl, after `CommanderNinjutsu => 88u8` (around line 512).
**Pattern**: `KeywordAbility::JumpStart => 89u8.hash_into(hasher),`

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::JumpStart => "Jump-Start".to_string(),` in the `keyword_display` match, after `CommanderNinjutsu` (around line 724).

### Step 2: AbilityDefinition Variant (NOT needed)

Unlike Flashback, Jump-Start does NOT use an alternative cost. It pays the card's regular
mana cost. Therefore, there is NO separate `AbilityDefinition::JumpStart { cost }` variant
needed. The keyword marker `KeywordAbility::JumpStart` is sufficient.

However, the jump-start discard cost needs to be paid during casting. Since this is an
additional cost that is intrinsic to the keyword (not configurable per-card), the casting
code can detect `JumpStart` and enforce the discard without a separate `AbilityDefinition`.

**Rationale**: Flashback needs `AbilityDefinition::Flashback { cost }` because each card
has a different flashback cost. Jump-Start always costs "the card's mana cost + discard
any card" -- there is no per-card variation to store.

### Step 3: Rule Enforcement in casting.rs

This is the main implementation step. Jump-Start modifies three aspects of casting:

#### Step 3a: Graveyard zone permission + discard cost field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `cast_with_jump_start: bool` and `jump_start_discard: Option<ObjectId>` fields
to `Command::CastSpell` after `cast_with_overload`.

```rust
/// CR 702.133a: If true, cast this spell from graveyard using jump-start.
/// The card's regular mana cost is paid, plus the player must discard a card
/// (specified in `jump_start_discard`). This is NOT an alternative cost --
/// it can combine with other alternative costs per the 2018-10-05 ruling.
///
/// If the spell resolves, is countered, or otherwise leaves the stack,
/// it is exiled instead of going to its normal destination.
#[serde(default)]
cast_with_jump_start: bool,
/// CR 702.133a: The card to discard as the jump-start additional cost.
/// Must be a card in the caster's hand (not the jump-start card itself,
/// which is in the graveyard). Required when `cast_with_jump_start` is true.
#[serde(default)]
jump_start_discard: Option<ObjectId>,
```

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Thread `cast_with_jump_start` and `jump_start_discard` through the
`Command::CastSpell` match arm to `handle_cast_spell`.

**File**: `crates/engine/src/rules/casting.rs`
**Action** (function signature): Add `cast_with_jump_start: bool` and
`jump_start_discard: Option<ObjectId>` parameters to `handle_cast_spell`.

#### Step 3b: Jump-Start detection and validation

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the initial card-fetch block (around line 90-200), add detection for jump-start.
Unlike flashback which auto-detects (any graveyard card with Flashback keyword), jump-start
requires the explicit `cast_with_jump_start: true` flag from the Command. This is because
jump-start is NOT an alternative cost -- if the card has both JumpStart and Escape keywords,
auto-detection could conflict. Explicit flags are safer.

```rust
// CR 702.133a: Jump-start -- allowed if card has the keyword and is in graveyard.
// Unlike flashback, jump-start requires the explicit cast_with_jump_start flag
// because it is not an alternative cost and can coexist with other abilities.
let casting_with_jump_start = cast_with_jump_start
    && casting_from_graveyard
    && card_obj.characteristics.keywords.contains(&KeywordAbility::JumpStart);
```

If `cast_with_jump_start` is true but the card doesn't have the keyword, reject:
```rust
if cast_with_jump_start && !card_obj.characteristics.keywords.contains(&KeywordAbility::JumpStart) {
    return Err(GameStateError::InvalidCommand(
        "jump-start: card does not have the JumpStart keyword (CR 702.133a)".into(),
    ));
}
```

Add `casting_with_jump_start` to the zone-permission guard (around line 184-191) so that
cards in the graveyard with JumpStart are allowed:
```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_jump_start  // NEW
    && !casting_with_madness
    && !casting_with_escape_auto
    && !cast_with_escape
    && !cast_with_foretell
{
```

Thread `casting_with_jump_start` through the return tuple.

#### Step 3c: Type validation (instants and sorceries only)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After the flashback type validation (around line 242-253), add:

```rust
// CR 702.133a: Jump-start -- type validation: only instants and sorceries.
if casting_with_jump_start {
    let is_instant_or_sorcery = chars.card_types.contains(&CardType::Instant)
        || chars.card_types.contains(&CardType::Sorcery);
    if !is_instant_or_sorcery {
        return Err(GameStateError::InvalidCommand(
            "jump-start can only be used on instants and sorceries".into(),
        ));
    }
}
```

#### Step 3d: Cost determination (uses card's mana cost, NOT an alternative cost)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Jump-Start does NOT change the cost selection. The card's regular mana cost is
used. No changes to the `base_cost_before_tax` chain (around line 537-600). This is the
key difference from Flashback.

Add mutual exclusion with flashback per cast (defense-in-depth -- a card shouldn't have
both, but if it did, the player chooses one):
```rust
if casting_with_jump_start && casting_with_flashback {
    return Err(GameStateError::InvalidCommand(
        "cannot use both jump-start and flashback on the same cast".into(),
    ));
}
```

**Important**: Jump-start suppresses flashback auto-detection. The flashback auto-detection
at line 112 checks `casting_from_graveyard && keywords.contains(Flashback)`. If a card has
BOTH keywords, flashback would auto-detect. To prevent this when the player explicitly
chose jump-start, add `&& !cast_with_jump_start` to the flashback auto-detection:
```rust
let casting_with_flashback = casting_from_graveyard
    && card_obj.characteristics.keywords.contains(&KeywordAbility::Flashback)
    && !cast_with_escape
    && !cast_with_jump_start;  // NEW: suppress flashback if player chose jump-start
```

#### Step 3e: Discard additional cost payment

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After mana payment (around line 880-920), before creating the StackObject,
add the jump-start discard cost payment.

**CRITICAL -- Madness interaction**: The discard MUST check whether the discarded card has
Madness (CR 702.35a). If it does, the card goes to exile (not graveyard) and a madness
trigger is queued. Follow the same pattern as the cycling discard in
`crates/engine/src/rules/abilities.rs` lines 460-507.

```rust
// CR 702.133a: Jump-start additional cost -- discard a card from hand.
// CR 601.2f-h: Additional costs are paid as part of casting.
let mut jump_start_events = Vec::new();
if casting_with_jump_start {
    let discard_id = jump_start_discard.ok_or_else(|| {
        GameStateError::InvalidCommand(
            "jump-start requires a card to discard (jump_start_discard must be Some)".into(),
        )
    })?;

    // Validate the discard card:
    // 1. Must be in the caster's hand
    let discard_obj = state.object(discard_id)?;
    if discard_obj.zone != ZoneId::Hand(player) {
        return Err(GameStateError::InvalidCommand(
            "jump-start discard card must be in caster's hand".into(),
        ));
    }

    // CR 702.35a: Check if the discarded card has madness.
    // If so, exile instead of graveyard and queue a madness trigger.
    let discard_card_id = discard_obj.card_id.clone();
    let has_madness = discard_obj
        .characteristics
        .keywords
        .contains(&KeywordAbility::Madness);

    let discard_destination = if has_madness {
        ZoneId::Exile
    } else {
        ZoneId::Graveyard(player)
    };

    let (new_discard_id, _) = state.move_object_to_zone(discard_id, discard_destination)?;
    jump_start_events.push(GameEvent::CardDiscarded {
        player,
        object_id: discard_id,
        new_id: new_discard_id,
    });

    // CR 702.35a: If madness, queue the madness trigger.
    if has_madness {
        let madness_cost = discard_card_id.as_ref().and_then(|cid| {
            state.card_registry.get(cid.clone()).and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Madness { cost } = a {
                        Some(cost.clone())
                    } else {
                        None
                    }
                })
            })
        });
        state.pending_triggers.push_back(PendingTrigger {
            source: new_discard_id,
            ability_index: 0,
            controller: player,
            triggering_event: None,
            entering_object_id: None,
            is_etb_trigger: false,
            is_enlist_trigger: false,
            stack_object_kind: Some(StackObjectKind::MadnessTrigger {
                source_object: new_discard_id,
                exiled_card: new_discard_id,
                madness_cost: madness_cost.unwrap_or_default(),
                owner: player,
            }),
        });
    }
}
```

Append `jump_start_events` to the cast events.

**Note**: The `PendingTrigger` struct fields and `StackObjectKind::MadnessTrigger` fields
must match the existing pattern in `abilities.rs` (cycling discard). Verify the exact
struct fields at implementation time by reading the `PendingTrigger` definition.

#### Step 3f: StackObject flag

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `cast_with_jump_start: bool` field to `StackObject` after `was_overloaded`:

```rust
/// CR 702.133a: If true, this spell was cast via jump-start from the graveyard.
/// When it leaves the stack (resolves, is countered, or fizzles), it is exiled
/// instead of going to any other zone. Same departure behavior as flashback.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub cast_with_jump_start: bool,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash for `cast_with_jump_start` in the `HashInto for StackObject` impl,
after `was_overloaded` (around line 1577):
```rust
// Jump-start (CR 702.133a) -- exiled instead of graveyard
self.cast_with_jump_start.hash_into(hasher);
```

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Set `cast_with_jump_start: casting_with_jump_start` when creating the
StackObject (around line 956). Also ensure all other StackObject creation sites set
`cast_with_jump_start: false` (storm copies at line ~1070, cascade at ~1113,
abilities at ~334/571/744/996/3078/3255/3607, resolution at ~1440).

**File**: `crates/engine/src/rules/copy.rs`
**Action**: Set `cast_with_jump_start: false` when creating spell copies (around line 174).
CR 707.10: copies are never cast.

#### Step 3g: Exile on departure (resolution, fizzle, counter)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: In all four destination-determination checks, add `cast_with_jump_start`
alongside `cast_with_flashback`:

1. **Fizzle** (around line 89):
```rust
let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start {
    ZoneId::Exile
} else {
    ZoneId::Graveyard(owner)
};
```

2. **Normal resolution** (around line 536):
```rust
let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start {
    ZoneId::Exile
} else if stack_obj.was_buyback_paid {
    ZoneId::Hand(owner)
} else {
    ZoneId::Graveyard(owner)
};
```

3. **Counter by spell/ability** (around line 2280):
```rust
let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start {
    ZoneId::Exile
} else {
    ZoneId::Graveyard(owner)
};
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: In the `CounterSpell` effect handler (around line 804):
```rust
let destination = if stack_obj.cast_with_flashback || stack_obj.cast_with_jump_start {
    crate::state::zone::ZoneId::Exile
} else {
    ZoneId::Graveyard(owner)
};
```

#### Step 3h: Replay harness support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_jump_start"` action type after `"cast_spell_flashback"`:

```rust
"cast_spell_jump_start" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let discard_name = action.get("discard_card")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "jump-start requires 'discard_card' field".to_string())?;
    let discard_id = find_in_hand(state, player, discard_name)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        cast_with_evoke: false,
        cast_with_bestow: false,
        cast_with_miracle: false,
        cast_with_escape: false,
        escape_exile_cards: vec![],
        cast_with_foretell: false,
        cast_with_buyback: false,
        cast_with_overload: false,
        cast_with_jump_start: true,
        jump_start_discard: Some(discard_id),
    })
}
```

Also update all existing `Command::CastSpell` constructions in the harness to include
`cast_with_jump_start: false, jump_start_discard: None`.

### Step 3 (not applicable): Trigger Wiring

Jump-Start has no triggered abilities. The discard-as-additional-cost is handled during
casting (Step 3e), and the exile replacement is handled during resolution (Step 3g).

However, the `CardDiscarded` event emitted during the discard cost may trigger
"whenever a player discards a card" abilities (e.g., Waste Not, Bone Miser). These triggers
are already wired via the existing `check_triggers` infrastructure that fires after
`handle_cast_spell` completes. No additional trigger wiring is needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/jump_start.rs`
**Tests to write** (following `flashback.rs` pattern closely):

1. **`test_jump_start_basic_cast_from_graveyard`** -- CR 702.133a: Card with JumpStart in
   graveyard can be cast by paying mana cost + discarding a card. Verify SpellCast event,
   spell on stack, mana pool depleted, discard card in graveyard, CardDiscarded event emitted.

2. **`test_jump_start_exile_on_resolution`** -- CR 702.133a: After jump-start spell resolves,
   it goes to exile (not graveyard). Verify spell in exile zone, not in graveyard.

3. **`test_jump_start_exile_on_counter`** -- CR 702.133a: When jump-start spell is countered,
   it goes to exile. Follow flashback test 3 pattern with Counterspell.

4. **`test_jump_start_sorcery_timing`** -- CR 702.133a + rulings: Sorcery with jump-start
   cannot be cast during opponent's turn. Set active_player to p2, have p1 try to cast --
   should fail.

5. **`test_jump_start_non_jump_start_card_cannot_cast`** -- Negative: Card without JumpStart
   in graveyard with `cast_with_jump_start: true` should be rejected.

6. **`test_jump_start_pays_mana_cost_not_alternative`** -- CR 702.133a: Jump-start pays the
   card's regular mana cost. Radical Idea costs {1}{U}. Verify that exactly {1}{U} is
   consumed (not some different amount).

7. **`test_jump_start_discard_required`** -- CR 702.133a: Jump-start with no discard card
   (`jump_start_discard: None`) should be rejected.

8. **`test_jump_start_discard_must_be_in_hand`** -- CR 601.2f-h: The card to discard must
   be in the caster's hand. Providing an ObjectId for a card in graveyard should fail.

9. **`test_jump_start_discard_any_card`** -- CR 702.133a: Any card type can be discarded
   (not just lands). Discard a creature card -- should succeed.

10. **`test_jump_start_normal_hand_cast_not_exiled`** -- CR 702.133a: Casting a jump-start
    card normally from hand (without using jump-start) goes to graveyard on resolution,
    not exile.

11. **`test_jump_start_flag_set_on_stack`** -- CR 702.133a: The `cast_with_jump_start` flag
    is `true` on the StackObject. `cast_with_flashback` should be `false`.

12. **`test_jump_start_insufficient_mana_rejected`** -- CR 601.2f-h: With insufficient mana
    for the card's mana cost, the cast should fail even with a valid discard card.

**Test helper definitions**:
- `radical_idea_def()` -- Instant {1}{U}, "Draw a card. Jump-start."
- `generic_sorcery_jump_start_def()` -- Sorcery {2}{R}, "Deal 2 damage. Jump-start." (for sorcery timing tests)
- `lightning_bolt_def()` -- Instant {R}, no jump-start (negative test)
- `counterspell_def()` -- Instant {U}{U}, "Counter target spell." (for counter test)

### Step 5: Card Definition (later phase)

**Suggested card**: Radical Idea
**Oracle text**: "Draw a card. Jump-start (You may cast this card from your graveyard by
discarding a card in addition to paying its other costs. Then exile this card.)"
**Type**: Instant, {1}{U}
**Card definition file**: `crates/engine/src/cards/defs/radical_idea.rs`

```rust
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("radical-idea"),
        name: "Radical Idea".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.\nJump-start".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::JumpStart),
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
```

### Step 6: Game Script (later phase)

**Suggested scenario**: "Jump-start basic flow"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Scenario description**: Player 1 has Radical Idea in graveyard and a random card in hand.
Player 1 casts Radical Idea from graveyard using jump-start (discarding the hand card),
spell resolves, Radical Idea ends up in exile, discarded card in graveyard, player 1 drew
a card.

**Script sequence**:
1. Initial state: Radical Idea in p1 graveyard, "Expendable Card" in p1 hand, library has 1+ cards
2. `cast_spell_jump_start` with `discard_card: "Expendable Card"`
3. All players pass priority
4. Assert: Radical Idea in exile, Expendable Card in p1 graveyard, p1 hand count increased by 1
   (drew a card, but discarded one -- net 0, but with a library card now in hand)

## Interactions to Watch

- **Flashback auto-detection**: The existing code auto-detects flashback when a card in the
  graveyard has `KeywordAbility::Flashback`. Must ensure that `JumpStart` cards in the
  graveyard are NOT auto-detected as flashback. The existing check
  (`casting_from_graveyard && keywords.contains(Flashback)`) will correctly NOT match
  JumpStart cards. But if `cast_with_jump_start` is false and the card has JumpStart but
  not Flashback, the zone check should reject it. Additionally, the flashback auto-detection
  must be suppressed when `cast_with_jump_start` is true (see Step 3d).

- **Escape auto-detection**: Similar concern. Cards with JumpStart but not Escape should
  not auto-detect as escape casts. The existing check uses `card_has_escape_keyword` which
  specifically checks for `KeywordAbility::Escape`, so JumpStart will not collide.

- **Discard triggers**: The `CardDiscarded` event from the jump-start additional cost will
  fire "whenever a player discards a card" triggers (Waste Not, etc.). These triggers should
  fire correctly via the existing `check_triggers` call after `handle_cast_spell`.

- **Madness interaction**: If the discarded card has Madness, the madness replacement
  fires: card goes to exile instead of graveyard, and a MadnessTrigger is queued. The discard
  code in Step 3e MUST replicate the madness-check pattern from `abilities.rs` lines 460-507
  (the cycling discard handler). See Step 3e for the full code pattern.

- **All StackObject creation sites**: Every place that creates a `StackObject` must include
  `cast_with_jump_start: false`. Grep for `cast_with_flashback: false` to find all sites --
  they are the same files. Sites include:
  - `casting.rs`: main spell cast (~line 956), storm copies (~1070), cascade (~1113)
  - `abilities.rs`: activated abilities (~334, ~571, ~744, ~996, ~3078, ~3255, ~3607)
  - `resolution.rs`: token/trigger stack objects (~1440)
  - `copy.rs`: spell copy (~174), cascade free cast (~359)

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::JumpStart` |
| `crates/engine/src/state/stack.rs` | Add `cast_with_jump_start: bool` field |
| `crates/engine/src/state/hash.rs` | Add hash discriminants for JumpStart keyword (89u8) + StackObject field |
| `crates/engine/src/rules/command.rs` | Add `cast_with_jump_start`, `jump_start_discard` to CastSpell |
| `crates/engine/src/rules/engine.rs` | Thread new fields through to `handle_cast_spell` |
| `crates/engine/src/rules/casting.rs` | Zone permission, type validation, discard cost (with madness check), mutual exclusion |
| `crates/engine/src/rules/resolution.rs` | Exile on resolve/fizzle/counter (4 sites) |
| `crates/engine/src/effects/mod.rs` | Exile on CounterSpell effect (1 site) |
| `crates/engine/src/rules/copy.rs` | `cast_with_jump_start: false` on copies |
| `crates/engine/src/rules/abilities.rs` | `cast_with_jump_start: false` on all StackObject creations |
| `crates/engine/src/testing/replay_harness.rs` | `cast_spell_jump_start` action + update all existing CastSpell |
| `tools/replay-viewer/src/view_model.rs` | `JumpStart => "Jump-Start"` display |
| `crates/engine/tests/jump_start.rs` | 12 unit tests (new file) |

## Discriminant Summary

- `KeywordAbility::JumpStart` = hash discriminant `89u8`
- `StackObject.cast_with_jump_start` = new bool field (hashed after `was_overloaded`)
- No new `AbilityDefinition` variant needed
- No new `StackObjectKind` variant needed (uses regular `Spell`)
- TUI `stack_view.rs` does NOT need changes (no new StackObjectKind)
