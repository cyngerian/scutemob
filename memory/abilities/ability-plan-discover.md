# Ability Plan: Discover

**Generated**: 2026-03-07
**CR**: 701.57
**Priority**: P4
**Similar abilities studied**: Cascade (CR 702.85) in `crates/engine/src/rules/copy.rs:319-507`, `crates/engine/src/rules/casting.rs:3312-3380`, `crates/engine/src/rules/resolution.rs:1501-1515`, `crates/engine/tests/cascade.rs`

## CR Rule Text

> 701.57. Discover
>
> 701.57a "Discover N" means "Exile cards from the top of your library until you exile a nonland card with mana value N or less. You may cast that card without paying its mana cost if the resulting spell's mana value is less than or equal to N. If you don't cast it, put that card into your hand. Put the remaining exiled cards on the bottom of your library in a random order."
>
> 701.57b A player has "discovered" after the process described in 701.57a is complete, even if some or all of those actions were impossible.
>
> 701.57c If the final card exiled during the process described in rule 701.57a has mana value N or less, it is the "discovered card," regardless of whether it was cast or put into a player's hand.

## Cascade CR (for comparison)

> 702.85a Cascade is a triggered ability that functions only while the spell with cascade is on the stack. "Cascade" means "When you cast this spell, exile cards from the top of your library until you exile a nonland card whose mana value is less than this spell's mana value. You may cast that card without paying its mana cost if the resulting spell's mana value is less than this spell's mana value. Then put all cards exiled this way that weren't cast on the bottom of your library in a random order."

## Key Differences: Discover vs Cascade

1. **Discover is a keyword action (701.57), NOT a triggered keyword ability (702.85).** Cascade is a triggered ability on a spell ("When you cast this spell..."). Discover is an action performed by other abilities — e.g., "When this creature enters, discover 3." The discover action is performed during the resolution of whatever ability triggers it, not as its own separate triggered ability on the stack.

2. **MV comparison is <= N (Discover) vs < spell_MV (Cascade).** Cascade: "mana value is less than this spell's mana value" (strictly less). Discover: "mana value N or less" (less than or equal).

3. **If you don't cast it, put it into your hand (Discover only).** Cascade: cards not cast go to library bottom. Discover: the discovered card specifically goes to your hand if you decline to cast it; the remaining exiled cards go to library bottom.

4. **Discover uses a fixed N parameter, not the spell's MV.** "Discover 3" always uses 3. Cascade uses the cascade spell's own mana value dynamically.

5. **Discover is NOT inherently a cast trigger.** It is a keyword action invoked by other abilities. Cards like Geological Appraiser have "When this creature enters, if you cast it, discover 3" — the ETB trigger invokes the discover action during its resolution.

## Key Edge Cases

- **If you can't cast the discovered card** (no legal targets, timing restrictions don't apply since it's "without paying mana cost" but targeting still matters), put it into your hand (ruling 2023-11-10).
- **X in mana cost = 0** when cast without paying mana cost (ruling 2023-11-10).
- **Split cards**: MV is combined halves; may cast either half if that half's MV <= N (ruling 2023-11-10). Split cards not yet implemented in engine.
- **All exiling is mandatory; only the cast is optional** (ruling 2023-11-10).
- **Cards are exiled face up** (ruling 2023-11-10).
- **701.57b**: A player has "discovered" even if some actions were impossible (empty library, etc.). This matters for "whenever you discover" triggers.
- **701.57c**: The "discovered card" identity is the last nonland card exiled with MV <= N, regardless of whether it was cast or put into hand.
- **Multiplayer**: No special multiplayer considerations beyond standard priority/APNAP. Discover is performed by one player.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (KeywordAbility::Discover not yet added)
- [ ] Step 2: Rule enforcement (resolve_discover function)
- [ ] Step 3: Trigger wiring (N/A as keyword action — wired via Effect::Discover)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Discover` variant with discriminant 136. Discover is listed as a keyword on cards per Scryfall (Geological Appraiser has `Keywords: ["Discover"]`), but in the CR it is a keyword action (701.57), not a keyword ability. Add it as a `KeywordAbility` variant anyway for card definition fidelity (cards do bear it as a keyword). It will NOT trigger cascade-style behavior from the keyword alone — it is purely a marker. The actual discover action is invoked via `Effect::Discover`.

```
/// CR 701.57: Discover N — keyword action. Exile cards from top of library
/// until you exile a nonland card with MV <= N. Cast it for free or put it
/// into your hand. Put the rest on the bottom in a random order.
///
/// Discover is a keyword action, not a triggered ability like Cascade.
/// Cards bear this keyword but the action is invoked by their triggered
/// abilities (e.g., ETB triggers) via Effect::Discover.
///
/// Discriminant 136.
Discover,
```

**Hash**: Add to `crates/engine/src/state/hash.rs` — add `KeywordAbility::Discover` arm (no-op pattern match, same as other parameterless keywords).

**Match arms to update**:
- `tools/replay-viewer/src/view_model.rs`: KeywordAbility match — add `KeywordAbility::Discover => "Discover"` arm.
- No StackObjectKind needed (discover does not add its own SOK — it executes inline during resolution of the parent trigger).

### Step 2: Effect::Discover + resolve_discover function

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `Effect::Discover { player: PlayerTarget, n: u32 }` variant to the `Effect` enum. When executed, calls `resolve_discover(state, player_id, n)`.

**File**: `crates/engine/src/rules/copy.rs`
**Action**: Add `pub fn resolve_discover(state: &mut GameState, player: PlayerId, discover_n: u32) -> Vec<GameEvent>` function, modeled closely on `resolve_cascade` (line 319) but with these differences:

1. **MV check**: `card_mv <= discover_n` (not `card_mv < spell_mana_value`).
2. **Optional cast (deterministic)**: In the current deterministic engine, always cast if possible (matching cascade's behavior at line 381). The "put into hand" fallback happens if the card is a land or otherwise uncastable — but for deterministic M9.5, always cast nonland cards found. Add a comment noting that when player choice is implemented, this should become optional.
3. **Hand fallback**: If the player declines to cast (future) OR if the card cannot be cast, move it to the player's hand instead of leaving it in exile. This is the key behavioral difference from cascade.
4. **Events**: Emit `GameEvent::DiscoverExiled` and `GameEvent::DiscoverCast` (or `GameEvent::DiscoverToHand`) — new event variants modeled on `CascadeExiled`/`CascadeCast`.
5. **Library bottom**: Same as cascade — remaining exiled cards go to bottom in deterministic order.

**CR**: 701.57a — the entire procedure.

### Step 3: GameEvent variants

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add three new `GameEvent` variants:

```rust
/// Cards exiled during discover resolution (CR 701.57a).
DiscoverExiled {
    player: PlayerId,
    cards_exiled: Vec<ObjectId>,
},

/// A card was cast without paying its mana cost via discover (CR 701.57a).
DiscoverCast {
    player: PlayerId,
    card_id: ObjectId,
},

/// The discovered card was put into hand instead of being cast (CR 701.57a).
DiscoverToHand {
    player: PlayerId,
    card_id: ObjectId,
},
```

**Hash**: Add arms for these events in `crates/engine/src/state/hash.rs`.

### Step 4: Trigger Wiring

Discover is NOT a triggered ability itself — it is a keyword action performed during resolution of other abilities. No `StackObjectKind::DiscoverTrigger` is needed.

Cards that discover do so via their own triggered abilities. For example, Geological Appraiser has an ETB trigger whose effect includes `Effect::Discover { player: PlayerTarget::Controller, n: 3 }`.

The `Effect::Discover` execution handler in `effects/mod.rs` calls `resolve_discover` directly during effect resolution. No separate stack object or trigger dispatch is needed.

### Step 5: Unit Tests

**File**: `crates/engine/tests/discover.rs`
**Tests to write**:

1. **`test_discover_basic_finds_and_casts_card`** — CR 701.57a: Library has land, land, 2-MV spell. Discover 3 exiles the lands, finds the spell (MV 2 <= 3), casts it. Remaining lands go to library bottom.

2. **`test_discover_mv_equal_to_n_is_valid`** — CR 701.57a: Card with MV exactly equal to N qualifies (MV <= N, not MV < N like cascade). This is the key difference from cascade.

3. **`test_discover_empty_library`** — CR 701.57b: Discover with empty library completes without error. Player has "discovered" even though no card was found.

4. **`test_discover_all_lands_in_library`** — CR 701.57a: Library contains only lands. All are exiled, no qualifying nonland found. All exiled cards go to library bottom.

5. **`test_discover_put_into_hand_fallback`** — CR 701.57a: When the discovered card cannot be cast (future: player choice), it goes to hand. For now, test the hand-fallback path by discovering a card that the engine's deterministic mode would skip (or test via a card type that can't be "cast for free" in the current implementation — e.g., if the nonland card has MV > N, which shouldn't happen by definition, or test with a scenario where no valid target exists).

6. **`test_discover_remaining_cards_go_to_library_bottom`** — CR 701.57a: After discover resolves, exiled cards that weren't the discovered card go to the bottom of the library in a deterministic order.

7. **`test_discover_vs_cascade_mv_threshold`** — Confirm that cascade uses `<` (strict) while discover uses `<=` for the same MV boundary. Set up identical libraries; cascade spell with MV 3 should NOT find a 3-MV card, but discover 3 SHOULD find it.

**Pattern**: Follow `crates/engine/tests/cascade.rs` structure — same `pass_all` helper, similar card definition builders, same assertion patterns.

### Step 6: Card Definition (later phase)

**Suggested card**: Geological Appraiser
- **Mana cost**: {2}{R}{R} (MV 4)
- **Type**: Creature - Human Artificer
- **P/T**: 3/2
- **Oracle**: "When this creature enters, if you cast it, discover 3."
- **Abilities**:
  - `AbilityDefinition::Keyword(KeywordAbility::Discover)` (marker keyword)
  - `AbilityDefinition::Triggered` with `TriggerCondition::SelfEntersBattlefield` and `InterveningIf::WasCast` (if available, otherwise condition on `cast_alt_cost` tracking), effect `Effect::Discover { player: PlayerTarget::Controller, n: 3 }`
- **Card lookup**: use `card-definition-author` agent

### Step 7: Game Script (later phase)

**Suggested scenario**: Geological Appraiser ETB discover 3
- Player casts Geological Appraiser (4 mana)
- ETB trigger goes on stack
- ETB resolves, performing discover 3
- Library is pre-seeded with known card order: land, land, 2-MV instant
- Discover exiles lands, finds the instant, casts it for free
- Assert: instant's effect resolves, exiled lands on library bottom, Geological Appraiser on battlefield

**Subsystem directory**: `test-data/generated-scripts/stack/` (discover is a stack-related mechanic)

## Interactions to Watch

- **Cascade vs Discover**: The `resolve_discover` function must NOT reuse `resolve_cascade` directly because of the MV comparison difference (`<=` vs `<`) and the hand-fallback behavior. Extract shared code if desired, but the behavioral differences require a separate function.
- **"Whenever you discover" triggers (701.57b)**: Some cards trigger on discover completion. The `DiscoverCast`/`DiscoverToHand` events can serve as trigger points. Not needed for the initial implementation but the event design should support it.
- **Free-cast restrictions**: Same as cascade — no alternative costs, mandatory additional costs must be paid, X = 0. The `StackObject` construction should mirror cascade's (all alt-cost flags = false).
- **Effect resolution context**: `Effect::Discover` executes during effect resolution (not as a separate stack object). This means it happens inline while resolving the parent triggered ability. The stack is not empty during discover — the parent trigger is still resolving.
- **`tools/replay-viewer/src/view_model.rs`**: Must add `KeywordAbility::Discover` arm to the keyword display match. No SOK arm needed since discover doesn't add a stack object kind.
- **`tools/tui/src/play/panels/stack_view.rs`**: No SOK arm needed for the same reason.
- **`crates/engine/src/state/hash.rs`**: Must add hash arms for `KeywordAbility::Discover` and the three new `GameEvent` variants.

## File Change Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Discover` (disc 136) |
| `crates/engine/src/state/hash.rs` | Hash arms for new KW + GameEvent variants |
| `crates/engine/src/effects/mod.rs` | Add `Effect::Discover { player, n }` + execution handler |
| `crates/engine/src/rules/copy.rs` | Add `pub fn resolve_discover()` |
| `crates/engine/src/rules/events.rs` | Add `DiscoverExiled`, `DiscoverCast`, `DiscoverToHand` |
| `tools/replay-viewer/src/view_model.rs` | `KeywordAbility::Discover` arm |
| `crates/engine/tests/discover.rs` | New test file with 7 tests |
