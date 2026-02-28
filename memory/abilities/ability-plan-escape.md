# Ability Plan: Escape

**Generated**: 2026-02-27
**CR**: 702.138
**Priority**: P3
**Similar abilities studied**: Flashback (CR 702.34) in `casting.rs`, `stack.rs`, `resolution.rs`, `tests/flashback.rs`; Delve (CR 702.66) exile-cards-as-cost pattern in `casting.rs`

## CR Rule Text

```
702.138. Escape

702.138a Escape represents a static ability that functions while the card with escape is
in a player's graveyard. "Escape [cost]" means "You may cast this card from your graveyard
by paying [cost] rather than paying its mana cost." Casting a spell using its escape ability
follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

702.138b A spell or permanent "escaped" if that spell or the spell that became that
permanent as it resolved was cast from a graveyard with an escape ability.

702.138c An ability that reads "[This permanent] escapes with [one or more of a kind of
counter]" means "If this permanent escaped, it enters with [those counters]." That ability
may have a triggered ability linked to it that triggers "When it enters this way."
(See rule 603.11.) Such a triggered ability triggers when that permanent enters the
battlefield after its replacement effect was applied, even if that replacement effect had
no effect.

702.138d An ability that reads "[This permanent] escapes with [ability]" means "If this
permanent escaped, it has [ability]."
```

Related CR rules:
- **CR 118.9**: Alternative costs. Only one alternative cost per spell (CR 118.9a). Does not change mana cost (CR 118.9c). Additional costs/increases/reductions still apply (CR 118.9d).
- **CR 601.2b**: Announce intention to pay alternative cost at cast time.
- **CR 601.2f**: Total cost = alternative cost + additional costs + increases - reductions.
- **CR 601.2h**: Pay all costs (non-random first, then library-moving costs).

## Key Edge Cases

From card rulings (Ox of Agonas, Uro, Kroxa):

1. **Escape does NOT change the card's destination after resolution** (unlike Flashback). A permanent enters the battlefield normally; an instant/sorcery goes to the graveyard normally. If the permanent dies later, it goes to the graveyard. It can escape again.
2. **The escape cost includes BOTH the mana component AND the exile-cards component.** The exile of other cards from your graveyard is part of the alternative cost (CR 118.9), not a separate mechanic. Both must be paid during 601.2h.
3. **Once casting begins, the card immediately moves to the stack** (same as all spells per CR 601.2a). Players cannot take actions until casting is complete.
4. **Escape's permission doesn't change when you may cast** -- timing restrictions (sorcery speed for creatures/sorceries) still apply (ruling 2020-01-24).
5. **Cannot combine escape with another alternative cost** (CR 118.9a) -- cannot use both escape and flashback, evoke, bestow, miracle, or madness simultaneously.
6. **If a card has escape and flashback**, the player chooses which one to apply (ruling 2020-01-24). Since both cast from graveyard, the engine must distinguish which alternative cost is being used.
7. **Additional costs (commander tax, kicker) apply on top of escape cost** (CR 118.9d).
8. **Mana value is based on printed mana cost, not escape cost** (CR 118.9c).
9. **Escape works on ALL card types** -- unlike Flashback (instants/sorceries only), Escape applies to creatures, enchantments, etc.
10. **"Escaped" status must be tracked on the permanent** (CR 702.138b) for "escapes with" abilities (counters, abilities). The permanent needs a `was_escaped: bool` field.
11. **"Escapes with [counter]"** (CR 702.138c) is a replacement effect on ETB -- the permanent enters with those counters. This is NOT a triggered ability (it's a replacement, like `kicker_times_paid` driving extra effects at resolution).
12. **Multiplayer**: Escape cost (exiling cards) is from the caster's own graveyard only. No interaction with opponents' graveyards.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `KeywordAbility::Escape` does not exist yet
- [ ] Step 2: Rule enforcement -- no escape detection in casting.rs
- [ ] Step 3: Trigger wiring -- n/a (escape itself is not a trigger; "escapes with" is a replacement)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and AbilityDefinition

#### 1a: KeywordAbility::Escape

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Escape` variant after `Miracle` (line ~384).
**Pattern**: Follow `KeywordAbility::Flashback` at line 219.

```rust
/// CR 702.138: Escape [cost] -- static ability from graveyard.
/// "You may cast this card from your graveyard by paying [cost] rather
/// than paying its mana cost."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The escape cost (mana + exile count) is stored in
/// `AbilityDefinition::Escape { cost, exile_count }`.
///
/// Unlike Flashback, Escape does NOT exile the card on resolution.
/// The spell resolves normally (permanent to battlefield, instant/sorcery
/// to graveyard). The permanent tracks `was_escaped` for "escapes with"
/// abilities (CR 702.138b-d).
Escape,
```

#### 1b: AbilityDefinition::Escape

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Escape` variant to `AbilityDefinition` enum after `Miracle` (line ~201).
**Pattern**: Follow `AbilityDefinition::Flashback { cost }` at line 145.

```rust
/// CR 702.138: Escape [mana cost], Exile [N] other cards from your graveyard.
/// The card may be cast from its owner's graveyard by paying this alternative cost.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Escape)` for quick
/// presence-checking without scanning all abilities.
///
/// `exile_count` is the number of OTHER cards that must be exiled from the
/// graveyard as part of the escape cost. These are exiled during cost payment
/// (CR 601.2h), similar to how delve exiles cards for cost reduction.
Escape { cost: ManaCost, exile_count: u32 },
```

#### 1c: Hash discriminant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant 50 for `KeywordAbility::Escape` in the `HashInto` impl (after `Miracle => 49u8` at line ~389).
**Pattern**: Follow `KeywordAbility::Miracle => 49u8` at line 389.

```rust
// Escape (discriminant 50) -- CR 702.138
KeywordAbility::Escape => 50u8.hash_into(hasher),
```

#### 1d: StackObject field `was_escaped`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `was_escaped: bool` field to `StackObject` after `cast_with_miracle` (line ~92).
**Pattern**: Follow `cast_with_miracle: bool` at line 92.

```rust
/// CR 702.138b: If true, this spell was cast via escape from the graveyard.
/// The spell's escape cost (mana + exiling other cards) was paid as an
/// alternative cost. Unlike flashback, escape does NOT change where the
/// spell goes on resolution -- it resolves normally.
///
/// This flag is propagated to the permanent as `was_escaped` at resolution
/// time (for "escapes with [counter]" and "escapes with [ability]" effects).
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_escaped: bool,
```

**Hash**: Add `self.was_escaped.hash_into(hasher);` in `hash.rs` StackObject impl (after `cast_with_miracle` hash at line ~1201).

#### 1e: GameObject field `was_escaped`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `was_escaped: bool` field to `GameObject` after `is_bestowed` (line ~333).
**Pattern**: Follow `was_evoked: bool` at line 322.

```rust
/// CR 702.138b: If true, this permanent "escaped" -- it entered the battlefield
/// from a spell that was cast from the graveyard using an escape ability.
///
/// Used by "escapes with [counter]" (CR 702.138c) and "escapes with [ability]"
/// (CR 702.138d) effects at resolution time.
///
/// Set during spell resolution when the permanent enters the battlefield.
/// Reset to false on zone changes (CR 400.7).
#[serde(default)]
pub was_escaped: bool,
```

**Hash**: Add `self.was_escaped.hash_into(hasher);` in `hash.rs` GameObject impl (after `is_bestowed` hash at line ~539).

### Step 2: Rule Enforcement (Casting)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Multiple modifications to `handle_cast_spell`.

#### 2a: Escape zone detection

In the zone-detection block (line ~87-159), add `casting_with_escape` detection alongside `casting_with_flashback`:

```rust
// CR 702.138a: Escape -- allowed if card has the escape keyword and is in graveyard.
let casting_with_escape = casting_from_graveyard
    && card_obj
        .characteristics
        .keywords
        .contains(&KeywordAbility::Escape);
```

Update the zone legality check (line ~144-152) to include `!casting_with_escape`:

```rust
if card_obj.zone != ZoneId::Hand(player)
    && !casting_from_command_zone
    && !casting_with_flashback
    && !casting_with_madness
    && !casting_with_escape  // NEW
{
```

Update the return tuple to include `casting_with_escape`.

**CR**: 702.138a -- "You may cast this card from your graveyard..."

#### 2b: Handle Escape + Flashback disambiguation

When a card has BOTH Escape and Flashback keywords and is in the graveyard, the player must
choose which to use. The engine currently auto-detects flashback from the graveyard.

**Design decision**: Since both escape and flashback are detected automatically from the graveyard,
and a card could have both, we need a way to disambiguate. The cleanest approach:
- Escape has its own `cast_with_escape: bool` flag in the `CastSpell` command (like `cast_with_evoke`, `cast_with_bestow`).
- However, Flashback does NOT have such a flag -- it is auto-detected.
- For simplicity and consistency: **keep the auto-detection pattern**. If the card has BOTH keywords in the graveyard, prefer Escape if `escape_exile_cards` (new command field) is non-empty; otherwise fall back to Flashback.
- Actually, the simpler approach: **add `cast_with_escape: bool` to the Command**, just like `cast_with_evoke`. When true, use escape cost. When false and in graveyard with flashback, use flashback cost. When both false and card is in graveyard, check if it has escape and auto-detect (for backward compat with existing harness).

**Revised design**: Add `cast_with_escape: bool` to `Command::CastSpell`. When true:
1. Validate card is in graveyard and has Escape keyword.
2. Use escape mana cost (from `AbilityDefinition::Escape { cost, exile_count }`).
3. Validate and exile `escape_exile_cards` (new field) from graveyard.

If `cast_with_escape: false` and card is in graveyard with Flashback keyword, auto-detect flashback (existing behavior preserved).

If `cast_with_escape: false` and card is in graveyard with Escape but NOT Flashback, auto-detect escape.

#### 2c: New command fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add two fields to `Command::CastSpell` (after `cast_with_miracle`, line ~121):

```rust
/// CR 702.138a: If true, cast this spell by paying its escape cost
/// (mana + exiling cards from graveyard) instead of its mana cost.
/// This is an alternative cost (CR 118.9) -- cannot combine with
/// flashback, evoke, bestow, madness, miracle, or other alternative costs.
///
/// When true, `escape_exile_cards` must contain exactly the number of
/// ObjectIds specified by the card's `AbilityDefinition::Escape { exile_count }`.
/// Each card must be in the caster's graveyard and must not be the card
/// being cast (it says "other cards").
#[serde(default)]
cast_with_escape: bool,
/// CR 702.138a: Cards in the caster's graveyard to exile as part of the
/// escape cost. Must be exactly `exile_count` cards (from AbilityDefinition::Escape).
/// Each card must be in the caster's graveyard, must not be the card being
/// cast (the spell itself), and must not be duplicated.
///
/// Empty vec when `cast_with_escape` is false.
#[serde(default)]
escape_exile_cards: Vec<ObjectId>,
```

#### 2d: Alternative cost validation

In the alternative-cost validation section (line ~218-289), add escape validation:

```rust
// Step 1e: Validate escape (CR 702.138a / CR 118.9a).
let casting_with_escape = if cast_with_escape {
    if casting_with_flashback { // auto-detected
        return Err(GameStateError::InvalidCommand(
            "cannot combine escape with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine escape with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_bestow {
        return Err(GameStateError::InvalidCommand(
            "cannot combine escape with bestow (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_madness {
        return Err(GameStateError::InvalidCommand(
            "cannot combine escape with madness (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if cast_with_miracle {
        return Err(GameStateError::InvalidCommand(
            "cannot combine escape with miracle (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // Card must be in graveyard with Escape keyword (already validated above in zone detection).
    if !casting_from_graveyard || !card_has_escape_keyword {
        return Err(GameStateError::InvalidCommand(
            "escape: card must be in your graveyard with the Escape keyword (CR 702.138a)".into(),
        ));
    }
    true
} else if casting_from_graveyard && card_has_escape_keyword && !casting_with_flashback {
    // Auto-detect: card in graveyard with Escape but no Flashback and cast_with_escape not set.
    // Auto-enable escape for backward compatibility / convenience.
    true
} else {
    false
};
```

Also add escape to all existing alt-cost mutual exclusion checks (evoke, bestow, madness, miracle, flashback all need `casting_with_escape` exclusion).

#### 2e: Escape mana cost lookup

Add a helper function `get_escape_cost` (following `get_flashback_cost` pattern at line ~796):

```rust
/// CR 702.138a / CR 118.9: Look up the escape cost from the card's AbilityDefinition.
///
/// Returns the ManaCost and exile_count from `AbilityDefinition::Escape { cost, exile_count }`,
/// or `None` if the card has no escape ability definition.
fn get_escape_cost(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<(ManaCost, u32)> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Escape { cost, exile_count } = a {
                    Some((cost.clone(), *exile_count))
                } else {
                    None
                }
            })
        })
    })
}
```

#### 2f: Escape cost selection in mana cost determination

In the alternative-cost selection block (line ~317-340), add escape:

```rust
} else if casting_with_escape {
    // CR 702.138a: Pay escape mana cost instead of printed mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged.
    get_escape_cost(&card_id, &state.card_registry).map(|(cost, _)| cost)
}
```

#### 2g: Escape exile cost payment

After the delve reduction block (line ~545-558), add escape exile payment. This is
different from delve -- delve reduces generic mana, while escape's exile is a FIXED
cost (exactly N cards, not variable). The exile happens during cost payment (CR 601.2h).

Add a new function `apply_escape_exile_cost`:

```rust
/// CR 702.138a: Validate and exile cards for escape cost.
///
/// Unlike delve (which reduces generic mana), escape's exile is a fixed count
/// that is part of the alternative cost. The player must exile exactly
/// `exile_count` other cards from their graveyard.
///
/// Validation:
/// - Each card must be in the caster's graveyard.
/// - Each card must NOT be the spell being cast (the escape card itself is
///   already on the stack at this point -- but check old IDs defensively).
/// - No duplicates.
/// - Count must match exactly `exile_count`.
fn apply_escape_exile_cost(
    state: &mut GameState,
    player: PlayerId,
    escape_cards: &[ObjectId],
    required_count: u32,
    events: &mut Vec<GameEvent>,
) -> Result<(), GameStateError> {
    if escape_cards.len() as u32 != required_count {
        return Err(GameStateError::InvalidCommand(format!(
            "escape requires exactly {} cards to exile, but {} were provided",
            required_count,
            escape_cards.len()
        )));
    }
    // Validate uniqueness.
    let mut seen = std::collections::HashSet::new();
    for &id in escape_cards {
        if !seen.insert(id) {
            return Err(GameStateError::InvalidCommand(format!(
                "duplicate card {:?} in escape_exile_cards", id
            )));
        }
    }
    // Validate each card is in caster's graveyard.
    for &id in escape_cards {
        let obj = state.objects.get(&id).ok_or_else(|| {
            GameStateError::InvalidCommand(format!("escape exile card {:?} not found", id))
        })?;
        if obj.zone != ZoneId::Graveyard(player) {
            return Err(GameStateError::InvalidCommand(format!(
                "escape exile card {:?} is not in your graveyard", id
            )));
        }
    }
    // Exile each card.
    for &id in escape_cards {
        let (new_exile_id, _) = state.move_object_to_zone(id, ZoneId::Exile)?;
        events.push(GameEvent::ObjectExiled {
            player,
            object_id: new_exile_id,
        });
    }
    Ok(())
}
```

Call this function in `handle_cast_spell` right after the mana cost is determined and
delve/convoke/improvise are applied, but before `pay_cost`:

```rust
// CR 702.138a: Exile cards from graveyard as part of escape cost.
let mut escape_events: Vec<GameEvent> = Vec::new();
if casting_with_escape && !escape_exile_cards.is_empty() {
    let (_, exile_count) = get_escape_cost(&card_id, &state.card_registry)
        .ok_or_else(|| GameStateError::InvalidCommand(
            "escape: card has no AbilityDefinition::Escape".into(),
        ))?;
    apply_escape_exile_cost(state, player, &escape_exile_cards, exile_count, &mut escape_events)?;
}
```

**Important timing note**: The escape spell itself has already moved to the stack by this
point (CR 601.2a moves the card to the stack before costs are paid). So the card being
cast is no longer in the graveyard -- the exile cards list cannot include the cast card.
This is naturally enforced since the cast card is on the stack, not in the graveyard.

#### 2h: Flashback type restriction does NOT apply to Escape

**CR 702.34a** restricts flashback to instants and sorceries. **CR 702.138a** has no such
restriction. The existing flashback type check (line ~197-208) must be gated:

```rust
// CR 702.34a: Flashback -- type validation: only instants and sorceries.
// CR 702.138a: Escape has NO type restriction -- creatures, enchantments, etc. are valid.
if casting_with_flashback && !casting_with_escape {
    // ... existing flashback type check ...
}
```

Actually, this is simpler: if `casting_with_escape` is true, `casting_with_flashback` is
false (they are mutually exclusive alternative costs). So the existing flashback check
already doesn't fire. No change needed here -- just verify this is the case.

#### 2i: StackObject construction

In the StackObject construction (line ~610-630), add `was_escaped`:

```rust
was_escaped: casting_with_escape,
```

Also update all other StackObject construction sites (storm copies at ~720, cascade free cast at ~758) to include `was_escaped: false`.

#### 2j: Command dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` (or wherever Command::CastSpell is dispatched)

Update the match arm for `Command::CastSpell` to pass the new `cast_with_escape` and `escape_exile_cards` fields to `handle_cast_spell`.

Add `cast_with_escape: bool` and `escape_exile_cards: Vec<ObjectId>` parameters to the
`handle_cast_spell` function signature.

### Step 3: Resolution -- Propagate `was_escaped` to Permanent

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Propagate `was_escaped` from StackObject to the permanent at resolution time.
**Pattern**: Follow `was_evoked` propagation at line ~218.
**CR**: 702.138b -- a permanent "escaped" if the spell that became it was cast from graveyard with escape.

```rust
obj.was_escaped = stack_obj.was_escaped;
```

Add this line at line ~218, right after `obj.was_evoked = stack_obj.was_evoked;`.

#### 3b: "Escapes with [counter]" replacement effect (CR 702.138c)

For Ox of Agonas's "+1/+1 counter" on escape, this is a replacement effect on ETB.
It follows the same pattern as `kicker_times_paid` driving extra effects.

**Implementation approach**: In `resolution.rs`, after setting `was_escaped`, check if
the card definition has an "escapes with counter" ability and add the counter:

```rust
// CR 702.138c: "escapes with [counter]" -- if this permanent escaped,
// it enters with the specified counters.
if stack_obj.was_escaped {
    if let Some(cid) = &card_id_for_registry {
        if let Some(def) = state.card_registry.get(cid.clone()) {
            for ability in &def.abilities {
                if let AbilityDefinition::EscapeWithCounter { counter_type, count } = ability {
                    if let Some(obj) = state.objects.get_mut(&new_id) {
                        let current = obj.counters.get(counter_type).copied().unwrap_or(0);
                        obj.counters = obj.counters.update(*counter_type, current + count);
                    }
                }
            }
        }
    }
}
```

**New AbilityDefinition variant needed**:

```rust
/// CR 702.138c: "This permanent escapes with [N] [counter type] counter(s) on it."
/// If the permanent escaped, it enters the battlefield with the specified counters.
/// This is a replacement effect on the ETB event.
EscapeWithCounter { counter_type: CounterType, count: u32 },
```

Add to `AbilityDefinition` enum in `card_definition.rs`.

### Step 4: Replay Harness Action

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_escape"` action type.
**Pattern**: Follow `"cast_spell_flashback"` at line ~257.

```rust
// CR 702.138a: Cast a spell with escape from the player's graveyard.
// The engine uses cast_with_escape: true and escape_exile_cards for the
// exiled cards. The action JSON provides `escape_exile_names: ["card1", "card2", ...]`.
"cast_spell_escape" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    // Parse escape_exile_names from action JSON.
    let exile_names: Vec<String> = action
        .get("escape_exile_names")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let exile_ids: Vec<ObjectId> = exile_names
        .iter()
        .filter_map(|name| find_in_graveyard(state, player, name))
        .collect();
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
        cast_with_escape: true,
        escape_exile_cards: exile_ids,
    })
}
```

Also update ALL existing `Command::CastSpell` construction sites in the harness to include
`cast_with_escape: false, escape_exile_cards: vec![]`.

### Step 5: Update All CastSpell Construction Sites

Every place that constructs `Command::CastSpell` or `StackObject` must include the new fields.

Search for all sites:

```
Grep pattern="Command::CastSpell" path="crates/engine/src/" output_mode="files_with_matches"
Grep pattern="StackObject {" path="crates/engine/src/" output_mode="files_with_matches"
Grep pattern="Command::CastSpell" path="crates/engine/tests/" output_mode="files_with_matches"
```

Files that will need `cast_with_escape: false` / `escape_exile_cards: vec![]` added:

- `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs` (Command enum definition)
- `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs` (StackObject construction, storm/cascade copies)
- `/home/airbaggie/scutemob/crates/engine/src/rules/copy.rs` (cascade free cast)
- `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` (all action arms)
- `/home/airbaggie/scutemob/crates/engine/tests/flashback.rs`
- `/home/airbaggie/scutemob/crates/engine/tests/bestow.rs`
- `/home/airbaggie/scutemob/crates/engine/tests/evoke.rs`
- `/home/airbaggie/scutemob/crates/engine/tests/miracle.rs`
- `/home/airbaggie/scutemob/crates/engine/tests/effects.rs`
- Any other test file that constructs `Command::CastSpell`

Also for `StackObject`:
- `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs` (struct definition)
- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` (HashInto impl)
- `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` (propagate to permanent)

**Strategy**: Run `cargo test --all` after adding the field to find ALL compilation errors from missing fields. Fix each one by adding the default value.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/escape.rs`
**Pattern**: Follow `tests/flashback.rs` structure and conventions.

Tests to write:

1. **`test_escape_basic_creature_cast_from_graveyard`**
   - CR 702.138a: Cast a creature with escape from graveyard.
   - Set up: creature with Escape in graveyard, enough mana + enough other cards in graveyard.
   - Verify: SpellCast event, spell on stack, mana paid = escape cost, cards exiled from graveyard.

2. **`test_escape_exile_cost_paid`**
   - CR 702.138a / CR 601.2h: Verify exactly N other cards are exiled from graveyard as part of cost.
   - Set up: card with escape (exile 5 others), 5 other cards in graveyard.
   - Verify: 5 cards moved to exile, ObjectExiled events emitted.

3. **`test_escape_permanent_resolves_normally`**
   - CR 702.138a (ruling): Unlike flashback, an escaped permanent enters the battlefield normally and stays there. NOT exiled.
   - Set up: creature with escape in graveyard, cast it, resolve.
   - Verify: creature is on the battlefield (not exile, not graveyard).

4. **`test_escape_sorcery_goes_to_graveyard`**
   - If escape were on a sorcery: after resolution, it goes to graveyard (not exile).
   - This verifies escape does NOT behave like flashback's exile-on-resolution.

5. **`test_escape_was_escaped_flag_set`**
   - CR 702.138b: Permanent that escaped has `was_escaped: true`.
   - Verify the flag is set on the StackObject and propagated to the permanent.

6. **`test_escape_with_counter`**
   - CR 702.138c: "escapes with a +1/+1 counter" -- permanent enters with the counter.
   - Set up: creature with Escape + EscapeWithCounter, cast and resolve.
   - Verify: permanent on battlefield with +1/+1 counter.

7. **`test_escape_insufficient_graveyard_cards`**
   - Negative: Not enough other cards in graveyard to exile.
   - Set up: card with escape (exile 5), only 3 other cards in graveyard.
   - Verify: CastSpell fails with an error.

8. **`test_escape_wrong_exile_count`**
   - Negative: Player provides wrong number of exile cards.
   - Set up: escape requires 5, player provides 3.
   - Verify: error.

9. **`test_escape_cannot_exile_self`**
   - The card being cast is "other cards" -- you cannot exile the escape card itself.
   - This is naturally enforced since the card moves to the stack before exile payment.
   - Verify: the card is on the stack, the exile list only contains graveyard cards.

10. **`test_escape_cannot_combine_with_flashback`**
    - CR 118.9a: Only one alternative cost per spell.
    - Set up: card with both Escape and Flashback, try to cast with escape while also
      auto-detecting flashback. Or explicitly try to combine.
    - Verify: error about alternative cost conflict.

11. **`test_escape_timing_restriction_applies`**
    - CR 702.138a (ruling): Escape doesn't change timing restrictions.
    - A creature with escape cannot be cast during opponent's turn at sorcery speed.

12. **`test_escape_mana_value_unchanged`**
    - CR 118.9c: Mana value is based on printed cost, not escape cost.
    - Verify the card on the stack has its printed mana cost, not the escape cost.

13. **`test_escape_commander_tax_applies`**
    - CR 118.9d: Commander tax applies on top of escape cost.
    - Set up: commander with escape, cast from graveyard (commander dies, goes to graveyard, then escape).
    - Verify: total cost = escape mana cost + commander tax.
    - Note: This is a stretch goal -- may require commander in graveyard setup.

### Step 7: Card Definition (later phase)

**Suggested card**: Ox of Agonas
- CardId: `"ox-of-agonas"`
- Mana Cost: {3}{R}{R}
- Type: Creature -- Ox, 4/2
- Oracle: "When this creature enters, discard your hand, then draw three cards. Escape -- {R}{R}, Exile eight other cards from your graveyard. This creature escapes with a +1/+1 counter on it."
- Abilities:
  - `AbilityDefinition::Keyword(KeywordAbility::Escape)`
  - `AbilityDefinition::Escape { cost: ManaCost { red: 2, ..default }, exile_count: 8 }`
  - `AbilityDefinition::EscapeWithCounter { counter_type: CounterType::PlusOnePlusOne, count: 1 }`
  - `AbilityDefinition::Triggered` for the ETB discard+draw effect

### Step 8: Game Script (later phase)

**Suggested scenario**: `escape_basic_ox_of_agonas`
- p1 starts with Ox of Agonas in graveyard + 8 other cards in graveyard + {R}{R} mana.
- p1 casts Ox via escape, exiling 8 named cards from graveyard.
- Spell resolves: Ox enters battlefield with a +1/+1 counter (5/3 total).
- ETB trigger: p1 discards hand, draws 3 cards.
- Assert: Ox on battlefield with +1/+1 counter, 8 cards in exile, 3 cards in hand, graveyard reduced.

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Escape + Flashback on same card**: CR 118.9a -- player chooses one. Engine must not auto-detect both simultaneously. The `cast_with_escape` flag on the Command disambiguates.

2. **Escape + Commander Tax**: CR 118.9d -- additional costs stack on top of escape cost. If a commander with escape has been cast 2 times before, the escape mana cost is escape_cost + {2} tax.

3. **Escape + Delve/Convoke/Improvise**: These are NOT alternative costs (CR 702.66b, 702.51b, 702.126a). They apply AFTER the total cost (including escape alternative cost) is determined. A spell with both escape and delve would: pay escape mana + exile escape cards + exile delve cards. However, delve exiles reduce generic mana from the escape cost. The escape exile cards and delve exile cards must not overlap (each card can only be exiled once).

4. **Escape + Kicker**: Kicker is an additional cost (CR 702.33a). It applies on top of escape cost per CR 118.9d. Total = escape_mana_cost + kicker_cost.

5. **"Escapes with" counter + Humility**: If Humility is on the battlefield, the "escapes with" is a replacement effect at ETB (CR 702.138c). Humility removes abilities in Layer 6, but the replacement on ETB fires before the permanent is on the battlefield for Layer 6 to apply. The counter still gets placed. (This is analogous to how ETB replacement effects like "enters tapped" work -- they modify the ETB event itself.)

6. **Escape after death**: A creature that dies goes to the graveyard. If it has escape, it can be cast again from the graveyard. This is the core gameplay loop -- no special wiring needed; the zone detection in casting.rs handles it naturally.

7. **Multiplayer**: No special multiplayer considerations. Escape only uses the caster's own graveyard. The exile cost is paid by the caster. Other players cannot interact with the exile payment (it happens during casting, not on the stack).
