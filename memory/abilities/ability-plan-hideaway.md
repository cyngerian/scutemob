# Ability Plan: Hideaway

**Generated**: 2026-02-28
**CR**: 702.75
**Priority**: P3
**Similar abilities studied**: Foretell (face-down exile pattern, `rules/foretell.rs`), Exploit (ETB trigger via PendingTrigger + StackObjectKind, `rules/abilities.rs:850`, `rules/resolution.rs:986`, `state/stack.rs:255`), Cascade (library top-card exile + put rest on bottom, `rules/copy.rs:270`)

## CR Rule Text

**702.75. Hideaway**

> 702.75a Hideaway is a triggered ability. "Hideaway N" means "When this permanent enters, look at the top N cards of your library. Exile one of them face down and put the rest on the bottom of your library in a random order. The exiled card gains 'The player who controls the permanent that exiled this card may look at this card in the exile zone.'"

> 702.75b Previously, the rules for the hideaway ability caused the permanent to enter the battlefield tapped, and the number of cards the player looked at was fixed at four. Cards printed before this rules change had the printed text "Hideaway" with no numeral after the word. Those older cards have received errata in the Oracle card reference to have "Hideaway 4" and the additional ability "[This permanent] enters tapped."

## Key Edge Cases

- **"Enters tapped" is separate from Hideaway (CR 702.75b).** Older Hideaway lands (Windbrisk Heights, Mosswort Bridge, etc.) have an explicit "This land enters tapped" line. This is NOT part of the Hideaway keyword itself. The card definition must model this as a separate `Replacement` ability (ETB replacement, CR 614.1c) or the simpler `enters_tapped: true` flag if one exists.
- **Face-down exile (CR 406.3).** Exiled cards are face-down; opponents cannot examine them. Only the controlling player can look at them (via the ability granted by Hideaway). The engine already supports `face_down: bool` on `ObjectStatus` (used by Foretell).
- **Linked abilities (CR 607.2a).** The ETB exile ability and the "play the exiled card" ability are linked (CR 607.2a). The second ability refers ONLY to cards exiled by the first. Must track which cards were exiled by which Hideaway source. Currently NO `exiled_by` tracking exists on `GameObject` -- this is a new field.
- **"Play" not "cast" (older errata).** Windbrisk Heights says "you may play the exiled card." "Play" includes both casting spells and playing lands (CR 701.13). Newer Hideaway cards (Fight Rigging) also say "play." The implementation should support both.
- **Without paying its mana cost.** The Hideaway play ability typically lets you play the card without paying its mana cost. This is an alternative cost (CR 118.9).
- **Condition varies by card.** Windbrisk Heights: "if you attacked with three or more creatures this turn." Mosswort Bridge: "if creatures you control have total power 10 or greater." Fight Rigging: "if you control a creature with power 7 or greater." The condition is NOT part of the Hideaway keyword itself -- it's part of the card's separate activated/triggered ability.
- **The play ability is card-specific (not keyword-derived).** Hideaway as a keyword only defines the ETB trigger (look, exile, put back). The "play the exiled card" part is a separate ability on each card with its own cost and condition. Each card definition must model this as its own `AbilityDefinition::Activated` with a condition.
- **Deterministic fallback for card selection.** The engine uses deterministic fallback for interactive choices (Scry puts on bottom, Surveil mills all, Cascade exiles sequentially). For Hideaway, the deterministic fallback should exile the first card (top of the N) and put the rest on the bottom in a random (seeded) order.
- **Put the rest on bottom in random order.** CR 702.75a specifies "random order" (not "any order"). Use the engine's existing seeded shuffle approach.
- **Multiplayer: "The player who controls the permanent that exiled this card" (CR 702.75a).** If control of the Hideaway permanent changes, the NEW controller can look at the exiled card. This is already implied by the granted ability text.
- **Multiple Hideaway triggers (e.g., Panharmonicon + Hideaway permanent).** If Hideaway triggers multiple times, multiple cards are exiled face-down. The linked "play the exiled card" ability refers to all of them (CR 607.3).
- **Windbrisk Heights ruling (2007-10-01):** "At the time the ability resolves, you'll get to play the card if you declared three different creatures as attackers at any point in the turn. A creature declared as an attacker in two different attack phases counts only once. A creature that entered attacking (such as a token created by Militia's Pride) doesn't count because you never attacked with it." -- This condition tracking is card-specific.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

#### 1a. Add `KeywordAbility::Hideaway(u32)` to types.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Hideaway(u32)` variant to the `KeywordAbility` enum. The `u32` is N (the number of cards to look at).
**Location**: After `KeywordAbility::Suspend` (line 543), before the closing `}` at line 544.
**Pattern**: Follow `KeywordAbility::Modular(u32)` at line 489 (keyword with numeric parameter).

```rust
/// CR 702.75: Hideaway N -- "When this permanent enters, look at the top N
/// cards of your library. Exile one of them face down and put the rest on
/// the bottom of your library in a random order."
///
/// Triggered ability keyword. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// The N parameter specifies how many cards to look at.
///
/// The "play the exiled card" part is card-specific and NOT derived from
/// this keyword -- each card defines its own activated/triggered ability
/// with a condition and "play without paying mana cost" effect.
Hideaway(u32),
```

#### 1b. Add hash discriminant in hash.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Hideaway(n)` arm with discriminant 66 (next after Suspend=65).
**Location**: After `KeywordAbility::Suspend => 65u8.hash_into(hasher)` (line 439).
**Pattern**: Follow `KeywordAbility::Modular(n)` hash pattern at line 424.

```rust
// Hideaway (discriminant 66) -- CR 702.75
KeywordAbility::Hideaway(n) => {
    66u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

#### 1c. Add `exiled_by_hideaway: Option<ObjectId>` to GameObject

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a new field to `GameObject` to track which permanent's Hideaway ability exiled this card. This enables the linked ability (CR 607.2a) -- the "play" ability refers only to cards exiled by this specific source.
**Location**: After `is_suspended: bool` (line 407), before the closing `}` at line 408.
**Pattern**: Follow `is_foretold: bool` / `foretold_turn: u32` pattern at line 371.

```rust
/// CR 702.75a / CR 607.2a: If set, this object in exile was exiled face-down
/// by a Hideaway trigger from the permanent with this ObjectId.
///
/// Used by the linked "play the exiled card" ability to identify which exiled
/// card belongs to which Hideaway source. Set when the Hideaway ETB trigger
/// resolves. Reset on zone changes (CR 400.7) -- but since hideaway cards are
/// already in exile, any zone change from exile clears this.
///
/// The ObjectId stored here is the Hideaway permanent's CURRENT ObjectId on
/// the battlefield at the time the trigger resolved.
#[serde(default)]
pub exiled_by_hideaway: Option<ObjectId>,
```

#### 1d. Hash the new field

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.exiled_by_hideaway.hash_into(hasher)` to the `HashInto for GameObject` impl.
**Location**: In the `HashInto for GameObject` block, after the other boolean fields (`is_suspended`, etc.).
**Pattern**: Follow how `is_suspended` or `is_foretold` is hashed.

#### 1e. Add `StackObjectKind::HideawayTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add a new variant for the Hideaway ETB trigger on the stack.
**Location**: After `SuspendCastTrigger` (line 351), before the closing `}` at line 352.
**Pattern**: Follow `ExploitTrigger` at line 255.

```rust
/// CR 702.75a: Hideaway ETB triggered ability on the stack.
///
/// "When this permanent enters, look at the top N cards of your library.
/// Exile one of them face down and put the rest on the bottom of your
/// library in a random order."
///
/// `source_object` is the Hideaway permanent's ObjectId on the battlefield.
/// `hideaway_count` is N (how many cards to look at).
///
/// When this trigger resolves:
/// 1. Look at top N cards of the controller's library.
/// 2. Exile one face-down (deterministic: exile the first/top card).
/// 3. Set `exiled_by_hideaway = source_object` on the exiled card.
/// 4. Put the rest on the bottom in a random order (seeded shuffle).
///
/// If the source has left the battlefield by resolution time (CR 400.7),
/// the trigger still resolves (it's already on the stack, CR 603.3).
HideawayTrigger {
    source_object: ObjectId,
    hideaway_count: u32,
},
```

#### 1f. Hash `StackObjectKind::HideawayTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `HideawayTrigger` arm with discriminant 16 (next after SuspendCastTrigger=15).
**Location**: After `SuspendCastTrigger` hash arm (around line 1301).

```rust
// HideawayTrigger (discriminant 16) -- CR 702.75a
StackObjectKind::HideawayTrigger {
    source_object,
    hideaway_count,
} => {
    16u8.hash_into(hasher);
    source_object.hash_into(hasher);
    hideaway_count.hash_into(hasher);
}
```

#### 1g. Add `PendingTrigger` fields for Hideaway

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_hideaway_trigger: bool` and `hideaway_count: Option<u32>` fields.
**Location**: After the last pending trigger field (around line 200, after `is_suspend_cast_trigger` / `suspend_card_id` fields).
**Pattern**: Follow `is_exploit_trigger: bool` at line 140.

```rust
/// CR 702.75a: If true, this pending trigger is a Hideaway ETB trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::HideawayTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `hideaway_count` carries the N parameter. The `ability_index` field
/// is unused when this is true.
#[serde(default)]
pub is_hideaway_trigger: bool,
/// CR 702.75a: Number of cards to look at from the top of the library.
///
/// Only meaningful when `is_hideaway_trigger` is true.
#[serde(default)]
pub hideaway_count: Option<u32>,
```

#### 1h. Add new Effect variant: `Effect::HideawayExile`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: This step is NOT needed. The Hideaway ETB trigger resolution will be handled directly in `resolution.rs` as a `StackObjectKind::HideawayTrigger` match arm (like Exploit, Evolve, Modular), not via an `Effect` variant. The trigger logic (look at top N, exile one face-down, put rest on bottom) is specific enough to warrant a dedicated resolution path.

**Rationale**: Existing keyword triggers (Exploit, Evolve, Modular, Myriad) all use dedicated `StackObjectKind` variants with bespoke resolution logic in `resolution.rs`. Hideaway follows this same pattern. A generic `Effect` variant would not cleanly capture the face-down exile + linked-ability tracking.

#### 1i. Add new Effect variant: `Effect::PlayExiledCard`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add a new `Effect::PlayExiledCard` variant for the "play the exiled card without paying its mana cost" part of Hideaway cards. This effect is used in the card-specific activated abilities.
**Location**: After `Effect::Nothing` (line 533), or in the Zone section.

```rust
/// CR 702.75a / CR 607.2a: Play the card exiled face-down by this
/// permanent's Hideaway trigger without paying its mana cost.
///
/// At resolution: find the card in exile that has
/// `exiled_by_hideaway == Some(source_id)`, then cast it without paying
/// its mana cost (alternative cost, CR 118.9). If the exiled card is a
/// land, play it instead of casting (CR 701.13). If no matching exiled
/// card exists, the ability does nothing.
///
/// Deterministic fallback: always plays the card (does not decline).
PlayExiledCard,
```

**Hash**: Add to `Effect` hash impl with the next available discriminant.

#### 1j. Add new Condition variant: `Condition::HideawayCondition`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Rather than adding a single generic Hideaway condition, model the card-specific conditions using EXISTING `Condition` variants where possible. For the initial implementation (Windbrisk Heights), the condition "attacked with three or more creatures this turn" requires a new game state tracker.

For the simplified M1 implementation, add a single condition:

```rust
/// CR 702.75: Custom condition for Hideaway play abilities.
/// Check varies by card. This is a placeholder for the initial
/// implementation that always evaluates to true (deterministic: always play).
HideawayPlayable,
```

**Note**: This is a simplification. The real condition (e.g., "attacked with 3+ creatures this turn") would require tracking `creatures_attacked_this_turn: u32` on `PlayerState`. For the initial validated implementation, use `Condition::Always` or a deterministic `HideawayPlayable` that always passes, and document the gap. A card-specific condition can be added when the card definition is authored.

**Alternative**: Skip this entirely and use `Condition::Always` in the card definition's `Conditional` effect. The runner can upgrade later.

### Step 2: Rule Enforcement (Hideaway ETB Trigger Resolution)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a match arm for `StackObjectKind::HideawayTrigger` in the main `resolve_stack_object` function.
**Location**: After the `SuspendCastTrigger` resolution arm.
**CR**: 702.75a -- "look at the top N cards of your library. Exile one of them face down and put the rest on the bottom of your library in a random order."

**Resolution logic:**

```rust
StackObjectKind::HideawayTrigger {
    source_object,
    hideaway_count,
} => {
    let controller = stack_obj.controller;

    // Step 1: Get the top N cards from the controller's library.
    let lib_zone = ZoneId::Library(controller);
    let top_ids: Vec<ObjectId> = state
        .zones
        .get(&lib_zone)
        .map(|z| z.cards_ordered().iter().rev().take(hideaway_count as usize).cloned().collect())
        .unwrap_or_default();

    if top_ids.is_empty() {
        // Library has no cards; trigger resolves with no effect.
        events.push(GameEvent::AbilityResolved {
            controller,
            stack_object_id: stack_obj.id,
        });
    } else {
        // Step 2: Deterministic fallback -- exile the first card (top of library).
        let exile_card_id = top_ids[0];
        let remaining = top_ids[1..].to_vec();

        // Move chosen card to exile face-down.
        let (new_exile_id, _old) = state.move_object_to_zone(exile_card_id, ZoneId::Exile)?;
        if let Some(exile_obj) = state.objects.get_mut(&new_exile_id) {
            exile_obj.status.face_down = true;
            exile_obj.exiled_by_hideaway = Some(source_object);
        }

        // Step 3: Put remaining cards on bottom of library in random order.
        // Use seeded shuffle (timestamp_counter as seed) for determinism.
        let seed = state.timestamp_counter;
        let mut shuffled = remaining.clone();
        // Simple Fisher-Yates with seed
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher as StdHasher};
        let mut rng_state = seed;
        for i in (1..shuffled.len()).rev() {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (rng_state as usize) % (i + 1);
            shuffled.swap(i, j);
        }
        // Move each remaining card to the bottom of the library.
        for card_id in &shuffled {
            // Remove from current position in library; re-add at bottom.
            if let Some(zone) = state.zones.get_mut(&lib_zone) {
                zone.remove_card(*card_id);
                zone.push_bottom(*card_id);
            }
        }

        // Emit events.
        events.push(GameEvent::HideawayExiled {
            player: controller,
            source: source_object,
            exiled_card: new_exile_id,
            remaining_count: shuffled.len() as u32,
        });

        events.push(GameEvent::AbilityResolved {
            controller,
            stack_object_id: stack_obj.id,
        });
    }
}
```

**Important pattern note**: Look at how Cascade (`copy.rs:270`) gets library cards. The library zone stores cards with `top()` as the top card. Use the same access pattern: `z.cards_ordered().iter().rev().take(n)` or the equivalent `z.top_n(n)` if such a method exists. If not, use the pattern from Scry (`effects/mod.rs:1115`).

**New GameEvent**: Add `GameEvent::HideawayExiled` variant.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add event variant.

```rust
/// CR 702.75a: A card was exiled face-down by a Hideaway trigger.
HideawayExiled {
    player: PlayerId,
    source: ObjectId,
    exiled_card: ObjectId,
    remaining_count: u32,
},
```

**Hash**: Add to `GameEvent` hash impl with the next available discriminant.

### Step 3: Trigger Wiring

#### 3a. ETB trigger generation in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `PermanentEnteredBattlefield` event handler within `check_triggers`, add a block that generates a `PendingTrigger` with `is_hideaway_trigger = true` when the entering permanent has `KeywordAbility::Hideaway(n)`.
**Location**: After the Exploit trigger generation (around line 850).
**CR**: 702.75a -- "When this permanent enters..."
**Pattern**: Follow the Exploit trigger generation at line 850.

```rust
// CR 702.75a: If the permanent has Hideaway(N), generate the hideaway ETB trigger.
// "When this permanent enters, look at the top N cards of your library..."
if let Some(obj) = state.objects.get(object_id) {
    for kw in obj.characteristics.keywords.iter() {
        if let KeywordAbility::Hideaway(n) = kw {
            let controller = obj.controller;
            let hideaway_trigger = PendingTrigger {
                source: *object_id,
                ability_index: 0,
                controller,
                triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
                entering_object_id: Some(*object_id),
                is_hideaway_trigger: true,
                hideaway_count: Some(*n),
                // All other fields default to false/None
                ..Default::default()
            };
            triggers.push(hideaway_trigger);
        }
    }
}
```

**Note**: `PendingTrigger` must derive or implement `Default` (it likely already does given the pattern of `..Default::default()` usage -- verify). If it doesn't, add a manual default impl or set all fields explicitly (follow the pattern from Exploit trigger construction at lines 882-898).

#### 3b. Flush pending trigger to stack in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `flush_pending_triggers` function, add a branch for `is_hideaway_trigger` that creates a `StackObjectKind::HideawayTrigger`.
**Location**: After the `is_exploit_trigger` branch (around line 1825).
**Pattern**: Follow the exploit branch.

```rust
} else if trigger.is_hideaway_trigger {
    // CR 702.75a: Hideaway ETB trigger -- "When this permanent enters,
    // look at the top N cards..."
    StackObjectKind::HideawayTrigger {
        source_object: trigger.source,
        hideaway_count: trigger.hideaway_count.unwrap_or(4),
    }
}
```

#### 3c. builder.rs keyword-to-trigger translation (Optional)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: The Hideaway trigger does NOT need to be wired through `builder.rs` triggered_abilities because it uses the `PendingTrigger` path directly (like Exploit, Evolve). The keyword `Hideaway(N)` is checked at trigger-scan time in `abilities.rs`, not at object-construction time in `builder.rs`.

**Note**: Some keywords (Living Weapon, Persist) use `builder.rs` to create `TriggeredAbilityDef` entries, while others (Exploit, Evolve, Modular) use direct `PendingTrigger` construction in `abilities.rs`. The PendingTrigger path is used when the trigger needs a dedicated `StackObjectKind` variant. Hideaway needs a dedicated stack variant (to carry the N count), so the `abilities.rs` path is correct.

### Step 4: PlayExiledCard Effect Execution

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add a match arm for `Effect::PlayExiledCard` in `execute_effect`.
**CR**: 702.75a, 118.9 (alternative cost), 607.2a (linked abilities).

**Resolution logic:**

```rust
Effect::PlayExiledCard => {
    // Find the card in exile that was exiled by this source's Hideaway trigger.
    let source_id = ctx.source;
    let controller = ctx.controller;

    // Search exile zone for a card with exiled_by_hideaway == Some(source_id).
    let hideaway_card = state
        .objects
        .values()
        .find(|obj| {
            obj.zone == ZoneId::Exile
                && obj.exiled_by_hideaway == Some(source_id)
                && obj.status.face_down
        })
        .map(|obj| obj.id);

    if let Some(card_id) = hideaway_card {
        // Deterministic fallback: always play the card.
        // Check if the card is a land (play) or spell (cast).
        let is_land = state
            .objects
            .get(&card_id)
            .map(|obj| obj.characteristics.card_types.contains(&CardType::Land))
            .unwrap_or(false);

        if is_land {
            // Play as land: move from exile to battlefield.
            // Turn face-up first (CR 406.3a).
            if let Some(obj) = state.objects.get_mut(&card_id) {
                obj.status.face_down = false;
                obj.exiled_by_hideaway = None;
            }
            let (new_id, _) = state.move_object_to_zone(card_id, ZoneId::Battlefield)?;
            events.push(GameEvent::PermanentEnteredBattlefield {
                player: controller,
                object_id: new_id,
            });
        } else {
            // Cast as spell without paying mana cost.
            // Turn face-up (CR 406.3a).
            if let Some(obj) = state.objects.get_mut(&card_id) {
                obj.status.face_down = false;
                obj.exiled_by_hideaway = None;
            }
            // Move to stack and resolve.
            // NOTE: This is a simplified implementation. The full version
            // would go through the casting pipeline with cost = 0.
            // For now, move directly to stack as a spell.
            let (stack_id, _) = state.move_object_to_zone(card_id, ZoneId::Stack)?;
            // ... create StackObject, resolve normally.
            // This is complex -- defer full casting pipeline integration to Step 5 (card def).
        }
    }
    // If no matching exiled card found, the ability does nothing.
}
```

**Important simplification note**: The full "cast without paying mana cost from exile" pipeline is complex (it needs to go through `casting.rs` with cost overridden to zero). For the initial validated implementation, the `PlayExiledCard` effect can:

1. Find the hideaway-exiled card.
2. Turn it face-up.
3. Move it directly to the appropriate zone (battlefield for permanents, graveyard for instants/sorceries after "resolving").

This skips the full cast pipeline (no stack interaction, no triggers for "whenever you cast a spell"). The gap is documented and can be closed in a later iteration. Alternatively, integrate with the existing "cast from exile without paying mana cost" pattern used by Suspend cast triggers (`SuspendCastTrigger` resolution in `resolution.rs`).

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/hideaway.rs`
**Tests to write**:

1. **`test_hideaway_etb_trigger_fires`** -- CR 702.75a: A permanent with `Hideaway(4)` enters the battlefield; verify a HideawayTrigger is placed on the stack.

2. **`test_hideaway_trigger_resolution_exiles_one_face_down`** -- CR 702.75a: When the trigger resolves, one card from the top 4 of the library is exiled face-down (`status.face_down == true`), and the remaining 3 are put on the bottom of the library.

3. **`test_hideaway_exiled_card_tracked_by_source`** -- CR 607.2a: The exiled card's `exiled_by_hideaway` field matches the source permanent's ObjectId.

4. **`test_hideaway_empty_library`** -- Edge case: If the library has fewer than N cards (or is empty), the trigger resolves with no effect (or exiles however many are available).

5. **`test_hideaway_face_down_exile_is_hidden`** -- CR 406.3: The exiled card has `face_down = true`, verifying hidden information enforcement.

6. **`test_hideaway_play_exiled_card`** -- CR 702.75a + card-specific: Activating the "play" ability finds the matching exiled card, turns it face-up, and plays it.

7. **`test_hideaway_negative_no_keyword`** -- A permanent without Hideaway does NOT generate a HideawayTrigger when it enters.

**Pattern**: Follow tests for Exploit in `crates/engine/tests/exploit.rs` (or whichever file tests Exploit -- Grep for the test file).

**Test file location**: Look for existing ability test files.

```
Grep pattern="test_exploit" path="crates/engine/tests/" output_mode="files_with_matches"
```

If exploit tests are in a single file, create `hideaway.rs` in the same directory. If they're in a combined file, add to the same file.

### Step 6: Card Definition

**Suggested card**: Windbrisk Heights (simplest Hideaway land, widely played in Commander)
**Oracle text**:
```
Hideaway 4
This land enters tapped.
{T}: Add {W}.
{W}, {T}: You may play the exiled card without paying its mana cost
if you attacked with three or more creatures this turn.
```
**Color identity**: W

**Card definition structure:**

```rust
CardDefinition {
    card_id: cid("windbrisk-heights"),
    name: "Windbrisk Heights".to_string(),
    mana_cost: None, // Land
    types: types(&[CardType::Land]),
    oracle_text: "Hideaway 4\nThis land enters tapped.\n{T}: Add {W}.\n{W}, {T}: You may play the exiled card without paying its mana cost if you attacked with three or more creatures this turn.".to_string(),
    color_identity: colors(&[Color::White]),
    abilities: vec![
        // 1. Hideaway 4 keyword
        AbilityDefinition::Keyword(KeywordAbility::Hideaway(4)),
        // 2. Enters tapped (separate from Hideaway keyword per CR 702.75b)
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::EntersBattlefield,
            modification: ReplacementModification::EntersTapped,
            is_self: true,
        },
        // 3. {T}: Add {W} -- mana ability (handled separately in mana_abilities)
        // 4. {W}, {T}: Play the exiled card (activated ability with condition)
        AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost { white: 1, ..Default::default() }),
                Cost::Tap,
            ]),
            effect: Effect::Conditional {
                condition: Condition::Always, // Simplified; real condition: attacked with 3+ creatures
                if_true: Box::new(Effect::PlayExiledCard),
                if_false: Box::new(Effect::Nothing),
            },
            timing_restriction: None, // Can be activated at any time (not sorcery speed)
        },
    ],
    mana_abilities: vec![
        ManaAbility {
            cost: ActivationCost { requires_tap: true, ..Default::default() },
            mana_produced: ManaPool { white: 1, ..Default::default() },
            any_color: false,
        },
    ],
    ..Default::default()
}
```

**Note**: The condition "attacked with three or more creatures this turn" is simplified to `Condition::Always` for the initial implementation. Adding proper attack tracking would require:
- A `creatures_declared_as_attackers_this_turn: u32` counter on `PlayerState`
- Reset in `reset_turn_state`
- Incremented in `handle_declare_attackers`
- A new `Condition::AttackedWithNOrMoreCreatures(u32)` variant

This is documented as a gap for future work.

### Step 7: Game Script

**Suggested scenario**: "Hideaway land enters, trigger exiles a card face-down, then player activates play ability"
**Subsystem directory**: `test-data/generated-scripts/replacement/` (or a new `abilities/` directory if one exists)

**Script outline:**
1. Player A plays Windbrisk Heights (land enters tapped via replacement effect).
2. Hideaway trigger fires and resolves: top 4 cards are examined, one exiled face-down, rest put on bottom.
3. On a later turn, Player A activates the {W},{T} ability.
4. The exiled card is played without paying its mana cost.

**Assertions:**
- After step 2: exactly one card in exile zone with `face_down: true`
- Library count decreased by 4 (one exiled, three moved to bottom... but wait, they're still in the library, just at the bottom -- so library count decreased by 1)
- After step 4: exiled card is on the battlefield (if permanent) or in graveyard (if instant/sorcery)

## Interactions to Watch

- **Panharmonicon + Hideaway**: If Panharmonicon is on the battlefield and the Hideaway permanent is an artifact or creature, the Hideaway trigger fires twice (CR 603.2d). Two cards are exiled face-down. The "play" ability can play both (CR 607.3).
- **Blink effects**: If the Hideaway permanent is blinked (exiled and returned), it's a new object (CR 400.7). The new object has no relation to the previously exiled card. A new Hideaway trigger fires. The old exiled card's `exiled_by_hideaway` still references the OLD ObjectId, so the new permanent's play ability won't find it.
- **Replacement effects on ETB (enters tapped)**: The "enters tapped" on old Hideaway cards is a separate replacement effect, not part of the keyword. The engine's existing ETB replacement infrastructure handles this.
- **Face-down cards and characteristics (CR 406.3a)**: A face-down exiled card has no characteristics. But the spell/ability that exiled it allows it to be played. When played, it's turned face up just before being put on the stack (CR 406.3a).
- **Hidden information**: The exiled card must be hidden from opponents. The engine's `face_down` flag + `GameEvent::private_to()` filtering handles this. The `HideawayExiled` event should be private to the controller.
- **"Play" vs "cast"**: Hideaway says "play," which includes lands. If the exiled card is a land, it bypasses the stack entirely. This matters for timing (you can only play a land during your main phase if you haven't played one this turn).

## Files Changed Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `Hideaway(u32)` to `KeywordAbility` enum |
| `crates/engine/src/state/hash.rs` | Add hash arms for `Hideaway(u32)`, `HideawayTrigger`, `HideawayExiled` event, `PlayExiledCard` effect, `exiled_by_hideaway` field |
| `crates/engine/src/state/game_object.rs` | Add `exiled_by_hideaway: Option<ObjectId>` field |
| `crates/engine/src/state/stack.rs` | Add `HideawayTrigger` variant |
| `crates/engine/src/state/stubs.rs` | Add `is_hideaway_trigger: bool` and `hideaway_count: Option<u32>` fields to `PendingTrigger` |
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::PlayExiledCard` variant |
| `crates/engine/src/rules/abilities.rs` | Add Hideaway trigger generation in `check_triggers` + flush in `flush_pending_triggers` |
| `crates/engine/src/rules/resolution.rs` | Add `HideawayTrigger` resolution arm |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::HideawayExiled` variant |
| `crates/engine/src/effects/mod.rs` | Add `Effect::PlayExiledCard` execution |
| `crates/engine/tests/hideaway.rs` | New test file with 7 tests |
| `crates/engine/src/cards/definitions.rs` | Add Windbrisk Heights card definition (Step 5) |
