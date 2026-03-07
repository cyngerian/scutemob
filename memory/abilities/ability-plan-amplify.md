# Ability Plan: Amplify

**Generated**: 2026-03-06
**CR**: 702.38
**Priority**: P4
**Similar abilities studied**: Modular (CR 702.43) — ETB counter placement in `resolution.rs:596-638`, keyword enum at `types.rs:556-563`, hash at `hash.rs:441-444`, tests at `tests/modular.rs`; Graft (CR 702.58) — same ETB counter pattern at `resolution.rs:640+` and `lands.rs:179+`

## CR Rule Text

702.38. Amplify

702.38a: Amplify is a static ability. "Amplify N" means "As this object enters, reveal any number of cards from your hand that share a creature type with it. This permanent enters with N +1/+1 counters on it for each card revealed this way. You can't reveal this card or any other cards that are entering the battlefield at the same time as this card."

702.38b: If a creature has multiple instances of amplify, each one works separately.

## Key Edge Cases

- **Self-exclusion**: Cannot reveal the Amplify card itself (CR 702.38a "You can't reveal this card"). Since the creature is on the stack being resolved (not in hand), this is naturally handled -- the card is not in the hand zone.
- **Simultaneous ETB exclusion**: "or any other cards that are entering the battlefield at the same time as this card" -- relevant for mass ETB effects (e.g., Living End). For the deterministic engine, this means filtering out any objects currently being resolved from the same batch. In practice, single-creature resolution means this rarely applies.
- **Creature type sharing**: The revealed cards must share at least one creature type with the entering creature. Use layer-resolved subtypes for the entering creature (post-Changeling, etc.). Cards in hand use their printed characteristics (no layer system in hand, but CDAs like Changeling DO apply in all zones via `calculate_characteristics`).
- **Multiple instances (CR 702.38b)**: Each Amplify instance works separately. Amplify 1 + Amplify 2 on the same creature: reveal for each separately. The same hand card can be revealed for multiple Amplify instances (the CR does not say "each card can only be revealed once"). For deterministic bot play, auto-reveal all eligible for each instance.
- **Reveal is optional**: Player CHOOSES which cards to reveal (may reveal zero). For bot/auto-resolve: reveal all eligible creatures from hand (maximizes counters).
- **"Cards" not "creature cards" in CR**: The CR 702.38a says "reveal any number of cards from your hand that share a creature type with it." The reminder text on cards says "creature cards" but the actual rule says "cards that share a creature type." A card shares a creature type if it has at least one creature subtype in common. Non-creature cards cannot share creature types (they have no creature subtypes), so the effect is the same.
- **Multiplayer**: No special multiplayer considerations. The ability only cares about the controller's own hand.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Amplify is a static/replacement ability, not a trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Amplify(u32)` variant after `Outlast` (line ~1117).
The `u32` parameter is the N value (number of +1/+1 counters per revealed card).

```
/// CR 702.38: Amplify N -- "As this creature enters, reveal any number of cards
/// from your hand that share a creature type with it. This permanent enters with
/// N +1/+1 counters on it for each card revealed this way."
///
/// Static ability. Multiple instances work separately (CR 702.38b).
///
/// Discriminant 122.
Amplify(u32),
```

**Pattern**: Follow `KeywordAbility::Modular(u32)` at `types.rs:563`

**Hash**: Add to `state/hash.rs` after Outlast (discriminant 121, line ~613):
```
// Amplify (discriminant 122) -- CR 702.38
KeywordAbility::Amplify(n) => {
    122u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions and add the new arm. Key locations:
- `state/hash.rs` (KeywordAbility hash -- above)
- `state/builder.rs` (keyword-driven trigger generation -- no trigger needed for Amplify, but verify the match is non-exhaustive or add a no-op arm)
- `rules/layers.rs` / `calculate_characteristics` (if any keyword-specific Layer logic exists)
- `rules/combat.rs` (keyword checks -- Amplify has no combat effect, add no-op arm if exhaustive)

### Step 2: Rule Enforcement (ETB Counter Placement)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add an Amplify ETB counter block in the permanent-enters-battlefield section, after the existing Modular/Graft/Riot blocks (around line ~640).

**CR**: 702.38a -- "As this object enters, reveal any number of cards from your hand that share a creature type with it. This permanent enters with N +1/+1 counters on it for each card revealed this way."

**Pattern**: Follow the Modular block at `resolution.rs:596-638`.

**Logic**:
1. Look up the card definition to find all `Amplify(n)` instances.
2. For each `Amplify(n)` instance:
   a. Get the entering creature's subtypes (from the card definition's `types.subtypes`, since the object is just entering and layer-resolved chars may not be fully available yet -- BUT CDAs like Changeling must apply, so use `calculate_characteristics(state, new_id)` if the object is already in `state.objects`, or fall back to card def subtypes).
   b. Scan the controller's hand for cards that share at least one creature subtype with the entering creature. Use `calculate_characteristics` for hand objects to respect CDAs (Changeling).
   c. Exclude the entering creature itself (it's on the battlefield now, not in hand, so this is naturally handled).
   d. Count the number of eligible cards (auto-reveal all for deterministic play).
   e. Place `n * eligible_count` +1/+1 counters on the entering creature.
3. Emit `GameEvent::CounterAdded` for the total counters placed.

**Implementation detail -- finding hand cards sharing creature types**:
```rust
// Pseudo-code for the core logic:
let entering_subtypes: OrdSet<SubType> = calculate_characteristics(state, new_id)
    .map(|c| c.subtypes.clone())
    .unwrap_or_default();

let controller_hand_cards: Vec<ObjectId> = state.objects.iter()
    .filter(|(_, obj)| obj.zone == ZoneId::Hand(controller) && obj.is_phased_in())
    .map(|(id, _)| *id)
    .collect();

let eligible_count = controller_hand_cards.iter()
    .filter(|&&hand_obj_id| {
        let hand_subtypes = calculate_characteristics(state, hand_obj_id)
            .map(|c| c.subtypes.clone())
            .unwrap_or_default();
        // Share at least one creature subtype
        !entering_subtypes.intersection(hand_subtypes).is_empty()
    })
    .count() as u32;
```

**IMPORTANT**: This block must appear in ALL ETB sites:
1. `resolution.rs` -- main permanent resolution (line ~750 area, inside the permanent-enters block)
2. `lands.rs` -- `handle_play_land` (line ~100 area). Amplify is creature-only so this will be a no-op in practice, but for consistency with the Graft pattern, include it. Since lands never have creature subtypes, the eligible count will always be 0.

**Multiple ETB sites in resolution.rs**: There are ~7 calls to `apply_self_etb_from_definition`. The Amplify logic should be placed in the SAME block as Modular/Graft/Riot (the main permanent-enters section around line 550-650), which is called once per resolved permanent. Verify that all other ETB sites (token creation, Ninjutsu, etc.) also get the Amplify block OR confirm they go through the same code path.

### Step 3: Trigger Wiring

**N/A** -- Amplify is a static ability that functions as an ETB replacement effect (CR 702.38a: "As this object enters"). It does not use the trigger/stack system. No `StackObjectKind` variant needed. No `PendingTrigger` wiring needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/amplify.rs`
**Tests to write**:

1. `test_amplify_basic_one_revealed` -- Amplify 1 creature enters with one matching creature card in hand. Assert: 1 +1/+1 counter placed. CR 702.38a.

2. `test_amplify_multiple_revealed` -- Amplify 1 creature enters with 3 matching creature cards in hand. Assert: 3 +1/+1 counters placed. CR 702.38a.

3. `test_amplify_no_matching_cards` -- Amplify 1 creature enters with no matching creature cards in hand (different creature types). Assert: 0 counters, creature has base P/T only. CR 702.38a.

4. `test_amplify_n_multiplier` -- Amplify 3 creature (like Kilnmouth Dragon) enters with 2 matching cards in hand. Assert: 6 +1/+1 counters (3 * 2). CR 702.38a.

5. `test_amplify_multiple_instances` -- Creature with both Amplify 1 and Amplify 2 enters with 2 matching cards in hand. Assert: 6 counters total (1*2 + 2*2). CR 702.38b: "each one works separately."

6. `test_amplify_empty_hand` -- Amplify creature enters with empty hand. Assert: 0 counters. CR 702.38a (may reveal zero).

7. `test_amplify_changeling_in_hand` -- Amplify creature enters; hand contains a Changeling creature (every creature type). Assert: Changeling card counts as sharing a type. Tests CDA interaction.

8. `test_amplify_non_creature_in_hand` -- Hand has non-creature cards with no creature subtypes (e.g., instants, lands). Assert: these are not counted. CR 702.38a.

**Pattern**: Follow tests in `crates/engine/tests/modular.rs` for setup (GameStateBuilder, card definitions, pass_all helpers, find_object_on_battlefield).

**Card definitions for tests**: Create inline test card definitions:
- `amplify_1_soldier`: 1/1 Soldier creature with Amplify 1 (like Daru Stinger)
- `amplify_3_dragon`: 5/5 Dragon creature with Amplify 3 (like Kilnmouth Dragon)
- `amplify_dual`: creature with both Amplify 1 and Amplify 2 (artificial for testing)
- `soldier_card`: 2/2 Soldier creature (hand fodder)
- `dragon_card`: 2/2 Dragon creature (hand fodder)
- `changeling_card`: 1/1 creature with Changeling keyword (shares all types)
- `elf_card`: 1/1 Elf creature (does NOT share types with Soldier or Dragon)

### Step 5: Card Definition (later phase)

**Suggested card**: Canopy Crawler -- 3G, 2/2 Beast with Amplify 1. Simple creature type (Beast), clean oracle text. The activated ability ({T}: Target creature gets +1/+1 until end of turn for each +1/+1 counter on this creature) can be deferred.

**Alternative**: Kilnmouth Dragon -- 5RR, 5/5 Dragon with Amplify 3 and Flying. More iconic but more complex (activated ability deals damage equal to counters).

**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: Cast an Amplify creature with 2 matching creature cards in hand. Verify the creature enters with the correct number of +1/+1 counters and has the expected P/T.

**Subsystem directory**: `test-data/generated-scripts/baseline/`

## Interactions to Watch

- **Changeling (CR 702.73a)**: Cards with Changeling are every creature type. They always share a creature type with any creature. `calculate_characteristics` handles this via CDA in all zones.
- **Humility (Layer 6)**: If Humility is on the battlefield when the Amplify creature enters, does Amplify still function? Amplify is a static ability that applies "as this object enters" -- replacement effects apply before the permanent is on the battlefield. By the time Humility's Layer 6 effect would remove abilities, the ETB replacement has already applied. The counters stay. This is the same as Modular ETB counters surviving Humility.
- **ETB doublers (Panharmonicon, Yarok)**: Amplify is NOT a triggered ability -- it's a replacement effect / static ability. Panharmonicon does not double it. Same reasoning as Modular ETB counters.
- **Doubling Season / Vorinclex**: These double counters being placed. They ARE replacement effects on counter-placement events. If Doubling Season is out, Amplify's +1/+1 counters would be doubled. The existing `EntersWithCounters` / `CounterAdded` pipeline should handle this if counter-doubling replacements are wired.
- **Copy effects**: A copy of an Amplify creature entering would also have Amplify (copied from the original). The copy would check its own controller's hand for matching types.

## Discriminant Values

- **KeywordAbility**: discriminant **122** (after Outlast=121)
- **AbilityDefinition**: NOT needed -- Amplify uses `AbilityDefinition::Keyword(KeywordAbility::Amplify(n))`, no separate AbilDef variant required (same as Modular)
- **StackObjectKind**: NOT needed -- Amplify has no trigger/stack component
