# Ability Plan: Partner With

**Generated**: 2026-02-28
**CR**: 702.124j
**Priority**: P3
**Similar abilities studied**: Partner (enum variant `KeywordAbility::Partner` at `state/types.rs:199`, deck validation at `rules/commander.rs:459`), Hideaway (ETB keyword trigger via `PendingTrigger` + `StackObjectKind::HideawayTrigger` at `rules/abilities.rs:920`, `rules/resolution.rs:1476`, `state/stack.rs:370`), SearchLibrary effect (`effects/mod.rs:1061`, uses `TargetFilter` + `matches_filter()`)

## CR Rule Text

**702.124. Partner**

> 702.124a Partner abilities are keyword abilities that modify the rules for deck construction in the Commander variant (see rule 903), and they function before the game begins. Each partner ability allows you to designate two legendary cards as your commander rather than one. Each partner ability has its own requirements for those two commanders. The partner abilities are: partner, partner--[text], partner with [name], choose a Background, and Doctor's companion.

> 702.124b Your deck must contain exactly 100 cards, including its two commanders. Both commanders begin the game in the command zone.

> 702.124c A rule or effect that refers to your commander's color identity refers to the combined color identities of your two commanders. See rule 903.4.

> 702.124d Except for determining the color identity of your commander, the two commanders function independently. When casting a commander with partner, ignore how many times your other commander has been cast (see rule 903.8). When determining whether a player has been dealt 21 or more combat damage by the same commander, consider damage from each of your two commanders separately (see rule 903.10a).

> 702.124e If an effect refers to your commander while you have two commanders, it refers to either one. If an effect causes you to perform an action on your commander and it could affect both, you choose which it refers to at the time the effect is applied.

> 702.124f Different partner abilities are distinct from one another and cannot be combined. For example, you cannot designate two cards as your commander if one of them has "partner" and the other has "partner with [name]."

> 702.124g If a legendary card has more than one partner ability, you may choose which one to use when designating your commander, but you can't use both. Notably, no partner ability or combination of partner abilities can ever let a player have more than two commanders.

> 702.124j "Partner with [name]" represents two abilities. It means "You may designate two legendary cards as your commander rather than one if each has a 'partner with [name]' ability with the other's name" and "When this permanent enters, target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle."

## Key Edge Cases

- **Two distinct abilities (CR 702.124j).** "Partner with [name]" represents TWO abilities: (1) a deck construction ability allowing the named pair as co-commanders, and (2) an ETB triggered ability that lets a target player search for the named card. Both must be implemented.
- **Target is ANY player, not just the controller.** The CR says "target player may search their library." In multiplayer, you could target an opponent (though rarely useful). This is a targeted trigger -- the player is chosen when the trigger goes on the stack.
- **Search is optional ("may search").** The target player is not obligated to search. Deterministic fallback: always search and find the card if it exists.
- **Card must be revealed.** Even though the reminder text omits "reveal," the full rule and rulings confirm the found card is revealed. The `SearchLibrary` effect already has a `reveal: bool` parameter.
- **Trigger still fires in Commander even if partner is in command zone.** Ruling: "The triggered ability of the 'partner with' keyword still triggers in a Commander game. If your other commander has somehow ended up in your library, you can find it." The search is NOT restricted to the library -- well, it IS restricted to the library, but the trigger fires regardless of where the partner currently is. If the partner is not in the library, the search finds nothing.
- **Can target another player.** Ruling: "You can also target another player who might have that card in their library." In multiplayer, targeting an opponent is legal if an opponent happens to have the named card in their library (rare but possible with theft/copy effects that shuffle cards into libraries).
- **Partner With cannot combine with plain Partner (CR 702.124f).** A creature with "Partner with [name]" can ONLY partner with the specific named creature. It cannot partner with a creature that has the generic "Partner" keyword. The deck validation must enforce this.
- **Independent commander tax and damage (CR 702.124d).** Same as plain Partner -- each commander tracks tax and combat damage independently. Already implemented for plain Partner.
- **Combined color identity (CR 702.124c).** Same as plain Partner -- the combined color identity is the union of both commanders' individual identities. Already implemented for plain Partner.
- **Search by exact card name.** The search filter must match by exact card name (not partial). The current `TargetFilter` struct does NOT have a `has_name` field -- this must be added.
- **Multiplayer: works with Stranglehold.** Ruling: "Note that the target player searches their library (which may be affected by effects such as that of Stranglehold)." Search prevention effects can block the search. The engine does not yet model Stranglehold-type effects, but the infrastructure should not preclude them.

## Current State (from ability-wip.md)

The WIP file currently tracks Bolster, not Partner With. Partner With has no existing work:

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (deck validation + ETB trigger resolution)
- [ ] Step 3: Trigger wiring (ETB -> search library)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

#### 1a. Add `KeywordAbility::PartnerWith(String)` to types.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `PartnerWith(String)` variant to the `KeywordAbility` enum. The `String` holds the exact name of the partner card (e.g., `"Toothy, Imaginary Friend"`).
**Location**: After `KeywordAbility::Partner` (line 199).
**Pattern**: Follow `KeywordAbility::Dredge(u32)` at line 244 (keyword with inner data), but with `String` instead of `u32`.

```rust
/// CR 702.124j: "Partner with [name]" represents two abilities:
/// (1) Deck construction: allows this card and the named card as co-commanders.
/// (2) ETB trigger: "When this permanent enters, target player may search
///     their library for a card named [name], reveal it, put it into their
///     hand, then shuffle."
///
/// The `String` is the exact name of the partner card. The deck validation
/// in `commander.rs` checks that both commanders have matching PartnerWith
/// names. The ETB trigger is wired in `abilities.rs` via PendingTrigger.
///
/// CR 702.124f: PartnerWith cannot combine with plain Partner or other
/// partner variants.
PartnerWith(String),
```

**Note on `Ord`/`Hash`**: `String` already derives `Ord` and `Hash`, so the existing `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]` on `KeywordAbility` will work without changes.

#### 1b. Add hash discriminant in hash.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::PartnerWith(name)` arm with discriminant 67 (next after Hideaway=66).
**Location**: After the `KeywordAbility::Hideaway(n)` arm at line 441.
**Pattern**: Follow `KeywordAbility::Dredge(n)` hash pattern at line 349, but hash the string bytes instead of a u32.

```rust
// PartnerWith (discriminant 67) -- CR 702.124j
KeywordAbility::PartnerWith(name) => {
    67u8.hash_into(hasher);
    name.as_bytes().hash_into(hasher);
}
```

**Note**: Check how `String` is hashed elsewhere in `hash.rs`. If there is a `HashInto for String` impl, use `name.hash_into(hasher)` directly. Otherwise, hash the bytes.

#### 1c. Add `has_name: Option<String>` to TargetFilter

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add an optional `has_name` field to the `TargetFilter` struct. This enables `SearchLibrary` to filter by exact card name, which is needed for "search for a card named [name]."
**Location**: After `has_subtype: Option<SubType>` (line 672).

```rust
/// Must have exactly this name (exact match). None = no restriction.
/// Used by "Partner with" ETB search (CR 702.124j) and similar
/// "search for a card named [name]" effects.
#[serde(default)]
pub has_name: Option<String>,
```

**Also update `matches_filter()` in `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`** (around line 2608, after the `has_subtype` check):

```rust
if let Some(name) = &filter.has_name {
    if chars.name != *name {
        return false;
    }
}
```

This is a minimal, backward-compatible addition. All existing `TargetFilter` uses have `has_name: None` (via `Default`), so no existing behavior changes.

#### 1d. Add `StackObjectKind::PartnerWithTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add a new variant for the Partner With ETB trigger on the stack.
**Location**: After `HideawayTrigger` (line 373), before the closing `}`.
**Pattern**: Follow `HideawayTrigger` at line 370.

```rust
/// CR 702.124j: Partner With ETB triggered ability on the stack.
///
/// "When this permanent enters, target player may search their library
/// for a card named [name], reveal it, put it into their hand, then
/// shuffle."
///
/// `source_object` is the permanent with "Partner with [name]" on the
/// battlefield.
/// `partner_name` is the exact name of the card to search for.
/// `target_player` is the targeted player who will search their library.
///
/// When this trigger resolves:
/// 1. The target player searches their library for a card with the exact
///    name `partner_name`.
/// 2. If found, reveal it, put it into their hand.
/// 3. Shuffle the target player's library.
/// 4. If not found (or if the player declines the "may" search), just
///    shuffle the target player's library.
///
/// CR 603.3: The trigger goes on the stack and can be countered.
/// If the source has left the battlefield by resolution time (CR 400.7),
/// the trigger still resolves (it is already on the stack).
PartnerWithTrigger {
    source_object: ObjectId,
    partner_name: String,
    target_player: PlayerId,
},
```

#### 1e. Hash `StackObjectKind::PartnerWithTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `PartnerWithTrigger` arm to the `HashInto for StackObjectKind` impl. Find the next available discriminant (check what HideawayTrigger uses -- it should be 16; use 17 for PartnerWithTrigger).
**Location**: After the `HideawayTrigger` hash arm in the `StackObjectKind` hash impl.

```rust
// PartnerWithTrigger (discriminant 17) -- CR 702.124j
StackObjectKind::PartnerWithTrigger {
    source_object,
    partner_name,
    target_player,
} => {
    17u8.hash_into(hasher);
    source_object.hash_into(hasher);
    partner_name.as_bytes().hash_into(hasher);
    target_player.hash_into(hasher);
}
```

#### 1f. Add `PendingTrigger` fields for Partner With

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_partner_with_trigger: bool` and `partner_with_name: Option<String>` fields to `PendingTrigger`.
**Location**: After the `hideaway_count` field (end of the struct).
**Pattern**: Follow `is_hideaway_trigger: bool` / `hideaway_count: Option<u32>` at the end of the struct.

```rust
/// CR 702.124j: If true, this pending trigger is a Partner With ETB trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::PartnerWithTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `partner_with_name` carries the partner's card name. The `ability_index`
/// field is unused when this is true.
#[serde(default)]
pub is_partner_with_trigger: bool,
/// CR 702.124j: The exact name of the partner card to search for.
///
/// Only meaningful when `is_partner_with_trigger` is true.
#[serde(default)]
pub partner_with_name: Option<String>,
```

#### 1g. Add PendingTrigger hash fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: In the `HashInto for PendingTrigger` impl, add hashing for the two new fields.
**Location**: After the existing hideaway fields in the PendingTrigger hash block.

```rust
self.is_partner_with_trigger.hash_into(hasher);
self.partner_with_name.hash_into(hasher);
```

**Note**: Check whether `Option<String>` has a `HashInto` impl. If not, hash it manually: `if let Some(name) = &self.partner_with_name { 1u8.hash_into(hasher); name.as_bytes().hash_into(hasher); } else { 0u8.hash_into(hasher); }`.

#### 1h. Update view_model.rs format_keyword

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `PartnerWith(name)` arm to the `format_keyword` function.
**Location**: After `KeywordAbility::Partner => "Partner".to_string()` at line 625.

```rust
KeywordAbility::PartnerWith(name) => format!("Partner with {name}"),
```

#### 1i. Update countered-spell handling in resolution.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `PartnerWithTrigger { .. }` to the exhaustive match arm that handles countered stack objects (around line 1647).
**Location**: In the `| StackObjectKind::HideawayTrigger { .. }` list at line 1662.

```rust
| StackObjectKind::PartnerWithTrigger { .. }
```

### Step 2: Rule Enforcement

#### 2a. Deck validation for Partner With pairs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/commander.rs`
**Action**: Extend `validate_partner_commanders` to handle the `PartnerWith` variant. Currently it only checks for plain `Partner`. The updated function must:
1. Check if both commanders have `PartnerWith` with matching names (cmd1 has `PartnerWith("cmd2_name")` and cmd2 has `PartnerWith("cmd1_name")`).
2. Reject mismatched combinations: `Partner` + `PartnerWith` (CR 702.124f).
3. Continue to accept two `Partner` commanders (existing behavior).

**Location**: The `validate_partner_commanders` function at line 459.
**CR**: 702.124f, 702.124j

**Updated logic:**

```rust
pub fn validate_partner_commanders(
    cmd1: &CardDefinition,
    cmd2: &CardDefinition,
) -> Result<(), String> {
    use crate::state::KeywordAbility;

    // Check for plain Partner
    let cmd1_has_partner = cmd1.abilities.iter().any(|a| {
        matches!(a, AbilityDefinition::Keyword(KeywordAbility::Partner))
    });
    let cmd2_has_partner = cmd2.abilities.iter().any(|a| {
        matches!(a, AbilityDefinition::Keyword(KeywordAbility::Partner))
    });

    // Check for Partner With
    let cmd1_partner_with: Option<&str> = cmd1.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Keyword(KeywordAbility::PartnerWith(name)) = a {
            Some(name.as_str())
        } else {
            None
        }
    });
    let cmd2_partner_with: Option<&str> = cmd2.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Keyword(KeywordAbility::PartnerWith(name)) = a {
            Some(name.as_str())
        } else {
            None
        }
    });

    // Case 1: Both have plain Partner
    if cmd1_has_partner && cmd2_has_partner {
        return Ok(());
    }

    // Case 2: Both have PartnerWith -- verify names match cross-wise
    if let (Some(pw1), Some(pw2)) = (cmd1_partner_with, cmd2_partner_with) {
        if pw1 == cmd2.name && pw2 == cmd1.name {
            return Ok(());
        } else {
            return Err(format!(
                "'{}' has partner with '{}' but '{}' has partner with '{}' \
                 -- names don't match (CR 702.124j)",
                cmd1.name, pw1, cmd2.name, pw2
            ));
        }
    }

    // Case 3: Mixed Partner + PartnerWith -- not allowed (CR 702.124f)
    if (cmd1_has_partner && cmd2_partner_with.is_some())
        || (cmd2_has_partner && cmd1_partner_with.is_some())
    {
        return Err(format!(
            "'Partner' and 'Partner with [name]' cannot be combined (CR 702.124f): \
             '{}' and '{}'",
            cmd1.name, cmd2.name
        ));
    }

    // Case 4: One has PartnerWith but the other has nothing
    if cmd1_partner_with.is_some() || cmd2_partner_with.is_some() {
        return Err(format!(
            "partner with pairing incomplete: '{}' and '{}' (CR 702.124j)",
            cmd1.name, cmd2.name
        ));
    }

    // Case 5: Neither has any partner ability
    Err(format!(
        "neither '{}' nor '{}' has partner",
        cmd1.name, cmd2.name
    ))
}
```

#### 2b. Partner With trigger resolution in resolution.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a match arm for `StackObjectKind::PartnerWithTrigger` in the main stack resolution function.
**Location**: After the `StackObjectKind::HideawayTrigger` resolution arm (around line 1550).
**CR**: 702.124j -- "target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle."

**Resolution logic:**

```rust
StackObjectKind::PartnerWithTrigger {
    source_object: _,
    partner_name,
    target_player,
} => {
    let controller = stack_obj.controller;

    // CR 702.124j: Target player searches their library for a card
    // named [partner_name].
    let lib_zone = ZoneId::Library(target_player);

    // Find the first card in the target player's library that matches
    // the exact name.
    let matching_card: Option<ObjectId> = state
        .objects
        .iter()
        .filter(|(_, obj)| {
            obj.zone == lib_zone && obj.characteristics.name == partner_name
        })
        .map(|(id, _)| *id)
        .min_by_key(|id| id.0); // Deterministic: pick lowest ObjectId

    if let Some(card_id) = matching_card {
        // Found -- reveal it, put it into target player's hand.
        let hand_zone = ZoneId::Hand(target_player);
        match state.move_object_to_zone(card_id, hand_zone) {
            Ok((new_id, _)) => {
                events.push(GameEvent::CardRevealed {
                    player: target_player,
                    object_id: new_id,
                });
                events.push(GameEvent::CardSearchedToHand {
                    player: target_player,
                    object_id: new_id,
                    card_name: partner_name.clone(),
                });
            }
            Err(_) => {
                // Zone move failed; trigger resolves with no effect.
            }
        }
    }
    // Whether found or not, shuffle the target player's library.
    // CR 701.20: Shuffle the library.
    let seed = state.timestamp_counter;
    if let Some(zone) = state.zones.get_mut(&lib_zone) {
        zone.shuffle_seeded(seed);
    }
    events.push(GameEvent::LibraryShuffled {
        player: target_player,
    });

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Important**: Check which `GameEvent` variants already exist. The engine likely has:
- `GameEvent::LibraryShuffled { player }` -- verify existence via grep.
- `GameEvent::CardRevealed { player, object_id }` -- may need to be added.
- `GameEvent::CardSearchedToHand` -- may need to be added, or reuse the existing SearchLibrary event pattern.

If these events do not exist, the resolution can use the simpler approach of just moving the card and emitting `AbilityResolved`. The important behavior is the zone move, not the event granularity.

**Alternative simpler approach**: Since the engine already has `Effect::SearchLibrary` which does exactly this (search library for a card matching a filter, move to hand, shuffle), the resolution could delegate to `execute_effect` with a constructed `SearchLibrary` effect. This avoids duplicating search logic. The `TargetFilter` would use the new `has_name` field:

```rust
StackObjectKind::PartnerWithTrigger {
    source_object: _,
    partner_name,
    target_player,
} => {
    let controller = stack_obj.controller;
    let filter = TargetFilter {
        has_name: Some(partner_name.clone()),
        ..Default::default()
    };
    let search_effect = Effect::SearchLibrary {
        player: PlayerTarget::SpecificPlayer(target_player),
        filter,
        reveal: true,
        destination: ZoneTarget::Hand,
    };
    let mut ctx = EffectContext::new(controller, source_object, vec![]);
    let search_events = execute_effect(&search_effect, &mut ctx, state);
    events.extend(search_events);

    // Shuffle (SearchLibrary may already shuffle -- check)
    // If not, shuffle explicitly:
    let lib_zone = ZoneId::Library(target_player);
    let seed = state.timestamp_counter;
    if let Some(zone) = state.zones.get_mut(&lib_zone) {
        zone.shuffle_seeded(seed);
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Decision for the runner**: Use the `Effect::SearchLibrary` delegation approach if possible, as it avoids duplicating zone-move and event logic. The `SearchLibrary` effect already handles the "find first matching card, move to destination" pattern. The only new piece is `TargetFilter.has_name`. However, note that `SearchLibrary` does NOT currently shuffle after searching (the `Shuffle` effect is separate). Also check whether `PlayerTarget::SpecificPlayer` exists -- if not, the runner may need to use a different approach.

### Step 3: Trigger Wiring

#### 3a. ETB trigger generation in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `PermanentEnteredBattlefield` event handler within `check_triggers`, add a block that generates a `PendingTrigger` with `is_partner_with_trigger = true` when the entering permanent has `KeywordAbility::PartnerWith(name)`.
**Location**: After the Hideaway trigger generation (around line 973, after the `hideaway_keywords` block).
**CR**: 702.124j -- "When this permanent enters..."
**Pattern**: Follow the Hideaway trigger generation at line 920.

```rust
// CR 702.124j: If the permanent has PartnerWith(name), generate the
// partner-with ETB trigger. "When this permanent enters, target player
// may search their library for a card named [name], reveal it, put it
// into their hand, then shuffle."
//
// Target player: deterministic fallback targets the controller (owner).
// In a real interactive implementation, the player would choose a target.
if let Some(obj) = state.objects.get(object_id) {
    let controller = obj.controller;
    let partner_with_names: Vec<String> = obj
        .characteristics
        .keywords
        .iter()
        .filter_map(|kw| {
            if let KeywordAbility::PartnerWith(name) = kw {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();
    for name in partner_with_names {
        triggers.push(PendingTrigger {
            source: *object_id,
            ability_index: 0, // unused for partner-with triggers
            controller,
            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
            entering_object_id: Some(*object_id),
            targeting_stack_id: None,
            triggering_player: None,
            exalted_attacker_id: None,
            defending_player_id: None,
            is_evoke_sacrifice: false,
            is_madness_trigger: false,
            madness_exiled_card: None,
            madness_cost: None,
            is_miracle_trigger: false,
            miracle_revealed_card: None,
            miracle_cost: None,
            is_unearth_trigger: false,
            is_exploit_trigger: false,
            is_modular_trigger: false,
            modular_counter_count: None,
            is_evolve_trigger: false,
            evolve_entering_creature: None,
            is_myriad_trigger: false,
            is_suspend_counter_trigger: false,
            is_suspend_cast_trigger: false,
            suspend_card_id: None,
            is_hideaway_trigger: false,
            hideaway_count: None,
            is_partner_with_trigger: true,
            partner_with_name: Some(name),
        });
    }
}
```

**Note**: The target player for the trigger is the controller (deterministic fallback). CR 702.124j says "target player," meaning any player can be targeted. For the deterministic engine, always targeting the controller is correct because the controller is the one most likely to have the partner card in their library.

#### 3b. Flush pending trigger to stack in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `flush_pending_triggers` function, add a branch for `is_partner_with_trigger` that creates a `StackObjectKind::PartnerWithTrigger`.
**Location**: After the `is_hideaway_trigger` branch (around line 2019).
**Pattern**: Follow the hideaway branch.

```rust
} else if trigger.is_partner_with_trigger {
    // CR 702.124j: Partner With ETB trigger -- "When this permanent
    // enters, target player may search their library for a card
    // named [name]..."
    StackObjectKind::PartnerWithTrigger {
        source_object: trigger.source,
        partner_name: trigger.partner_with_name.clone().unwrap_or_default(),
        target_player: trigger.controller, // Deterministic: target self
    }
}
```

**Important ordering**: This branch must be checked BEFORE the generic `TriggeredAbility` fallback at the end of the chain. Insert it in the `if/else if` chain after `is_hideaway_trigger` and before the final `else`.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/partner_with.rs`
**Tests to write**:

1. **`test_partner_with_deck_validation_matching_pair`** -- CR 702.124j: Two commanders with matching `PartnerWith` names pass validation. Pir has `PartnerWith("Toothy, Imaginary Friend")` and Toothy has `PartnerWith("Pir, Imaginative Rascal")`.

2. **`test_partner_with_deck_validation_mismatched_names`** -- CR 702.124j: Two commanders with non-matching `PartnerWith` names fail validation (e.g., Pir + some other creature with `PartnerWith("Wrong Name")`).

3. **`test_partner_with_cannot_combine_with_plain_partner`** -- CR 702.124f: A creature with `PartnerWith` and a creature with plain `Partner` fail validation.

4. **`test_partner_with_etb_trigger_fires`** -- CR 702.124j: A permanent with `PartnerWith("X")` enters the battlefield; verify a `PartnerWithTrigger` is placed on the stack with the correct `partner_name`.

5. **`test_partner_with_trigger_finds_partner_in_library`** -- CR 702.124j: When the trigger resolves, if the named card is in the target player's library, it is moved to that player's hand. Library is then shuffled.

6. **`test_partner_with_trigger_partner_not_in_library`** -- CR 702.124j: When the trigger resolves, if the named card is NOT in the library (e.g., already in hand, on battlefield, or in command zone), the library is still shuffled but no card is moved.

7. **`test_partner_with_trigger_fires_even_if_partner_on_battlefield`** -- CR 702.124j ruling: The trigger still fires regardless of where the partner currently is. If the partner is already on the battlefield, the search finds nothing in the library, but the trigger fires and resolves (shuffling the library).

8. **`test_partner_with_independent_tax`** -- CR 702.124d: Casting one Partner With commander does not affect the other's tax. (May reuse logic from existing `test_partner_commanders_separate_tax_tracking` if it works with PartnerWith too.)

9. **`test_partner_with_combined_color_identity`** -- CR 702.124c: The combined color identity of two Partner With commanders is the union of both. (Similar to existing `test_partner_commanders_combined_color_identity` but using PartnerWith.)

10. **`test_partner_with_negative_no_keyword`** -- A permanent without `PartnerWith` does NOT generate a PartnerWithTrigger when it enters.

**Pattern**: Follow tests for Exploit in `crates/engine/tests/abilities.rs` or the existing partner tests in `crates/engine/tests/commander.rs`. Deck validation tests go alongside the existing partner validation tests in `commander.rs`. ETB trigger tests go in the new `partner_with.rs` file.

**Test setup pattern:**

```rust
// For ETB trigger tests:
// 1. Create CardDefinitions for both partners
// 2. Build state with one partner in library, the other as a card to enter
// 3. Move the card to the battlefield
// 4. Check that a PartnerWithTrigger PendingTrigger was generated
// 5. Flush triggers to stack
// 6. Resolve the stack object
// 7. Verify the partner card moved from library to hand
// 8. Verify the library was shuffled (check count or order change)

// For deck validation tests:
// 1. Create CardDefinitions with the appropriate keyword abilities
// 2. Call validate_partner_commanders
// 3. Assert Ok(()) or Err as expected
```

### Step 5: Card Definition (later phase)

**Suggested card pair**: Pir, Imaginative Rascal + Toothy, Imaginary Friend

These are the most iconic and widely-played Partner With pair in Commander. They have straightforward abilities beyond the Partner With keyword itself.

**Pir, Imaginative Rascal**
- {2}{G}
- Legendary Creature -- Human
- 1/1
- Partner with Toothy, Imaginary Friend
- If one or more counters would be put on a permanent your team controls, that many plus one of each of those kinds of counters are put on that permanent instead.

**Toothy, Imaginary Friend**
- {3}{U}
- Legendary Creature -- Illusion
- 1/1
- Partner with Pir, Imaginative Rascal
- Whenever you draw a card, put a +1/+1 counter on Toothy.
- When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it.

**Note**: Pir's counter-doubling replacement effect is complex (affects all counter placements on permanents your team controls). For the initial implementation, the card definition can include the `PartnerWith` keyword and basic stats, with the counter-doubling effect as a `// TODO` or simplified version.

**Card lookup**: Use `card-definition-author` agent for both cards.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Partner With ETB search -- Pir enters, finds Toothy in library"

**Subsystem directory**: `test-data/generated-scripts/commander/`

**Script outline:**
1. Player 1 has Pir, Imaginative Rascal as commander (in command zone). Toothy, Imaginary Friend is in their library.
2. Player 1 casts Pir from the command zone for {2}{G}.
3. Pir enters the battlefield; Partner With ETB trigger goes on the stack.
4. All players pass priority; trigger resolves.
5. Player 1 searches their library, finds Toothy, reveals it, puts it into hand.
6. Player 1's library is shuffled.

**Assertions:**
- After step 5: Toothy is in Player 1's hand.
- After step 5: Library count decreased by 1 (one card moved to hand).
- After step 6: Library was shuffled.

**Alternative scenario (partner not in library):**
1. Same setup, but Toothy is already in Player 1's hand (not in library).
2. Pir enters, trigger fires and resolves.
3. Search finds nothing; library is shuffled.
4. Assert: Toothy is still in hand (not duplicated). Library count unchanged.

### Step 7: All Other Match Arms

When adding `PartnerWith(String)` as a new `KeywordAbility` variant, the following exhaustive match expressions in the codebase must be updated:

1. **`state/hash.rs`**: `HashInto for KeywordAbility` -- add arm (Step 1b)
2. **`tools/replay-viewer/src/view_model.rs`**: `format_keyword` -- add arm (Step 1h)
3. **Any other exhaustive matches on `KeywordAbility`**: Grep for `KeywordAbility::` match patterns and add the new arm.

Run `cargo build` after adding the enum variant to get compiler errors for all missing match arms.

## Interactions to Watch

- **Partner With and Panharmonicon**: Panharmonicon doubles ETB triggers from artifacts/creatures. If Pir enters (creature), the Partner With trigger fires twice. The target player searches twice -- but the second search will likely find nothing (the partner was already moved to hand by the first). Net result: one search succeeds, one finds nothing.
- **Partner With and blink effects**: If the Partner With creature is blinked (exiled and returned), it enters the battlefield again as a new object (CR 400.7). The Partner With trigger fires again, allowing another search. If the partner is already on the battlefield, the search finds nothing in the library.
- **Partner With and copy effects**: If a creature copies a creature with Partner With (e.g., Clone copying Pir), the copy gains `PartnerWith("Toothy, Imaginary Friend")`. The ETB trigger fires, and the copy's controller can search for Toothy. However, the copy itself does NOT have the deck-construction partner ability -- it only has the ETB search ability (since the copy was not designated as a commander).
- **Partner With and Stranglehold**: Stranglehold prevents opponents from searching libraries. If an opponent controls Stranglehold and the Partner With trigger targets the controller (who is not the Stranglehold controller's opponent), the search proceeds normally. If the trigger targets the Stranglehold controller, the search is blocked. The engine does not yet model Stranglehold; document as a known gap.
- **Partner With in non-Commander formats**: The ETB trigger still fires in any format (it's a triggered ability on a creature card). However, the deck-construction part (allowing two commanders) only applies in Commander. The validation logic is already Commander-specific.
- **Multiple PartnerWith keywords on one card**: Extremely unlikely, but if a card had two `PartnerWith` keywords (e.g., through a copy effect granting a second), each would generate its own ETB trigger. The `partner_with_names` collection in Step 3a handles this.

## Files Changed Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `PartnerWith(String)` to `KeywordAbility` enum |
| `crates/engine/src/state/hash.rs` | Add hash arms for `PartnerWith(String)`, `PartnerWithTrigger`, new `PendingTrigger` fields |
| `crates/engine/src/state/stack.rs` | Add `PartnerWithTrigger` variant to `StackObjectKind` |
| `crates/engine/src/state/stubs.rs` | Add `is_partner_with_trigger: bool` and `partner_with_name: Option<String>` to `PendingTrigger` |
| `crates/engine/src/cards/card_definition.rs` | Add `has_name: Option<String>` to `TargetFilter` |
| `crates/engine/src/effects/mod.rs` | Add `has_name` check to `matches_filter()` |
| `crates/engine/src/rules/commander.rs` | Extend `validate_partner_commanders` for `PartnerWith` pairs and mixed-partner rejection |
| `crates/engine/src/rules/abilities.rs` | Add Partner With trigger generation in `check_triggers` + flush in `flush_pending_triggers` |
| `crates/engine/src/rules/resolution.rs` | Add `PartnerWithTrigger` resolution arm + countered-spell arm |
| `tools/replay-viewer/src/view_model.rs` | Add `PartnerWith(name)` to `format_keyword` |
| `crates/engine/tests/partner_with.rs` | New test file with ~10 tests |
| `crates/engine/tests/commander.rs` | Add 2-3 deck validation tests for Partner With |
