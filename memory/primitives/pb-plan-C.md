# Primitive Batch Plan: PB-C — Extra Turns

**Generated**: 2026-04-05
**Primitive**: `Effect::ExtraTurn` — DSL variant for granting extra turns
**CR Rules**: 500.7 (extra turn creation, LIFO ordering), 702.174g (Gift an extra turn)
**Cards affected**: 5 (2 existing fixes + 3 new)
**Dependencies**: None — turn queue infrastructure already exists (TurnState.extra_turns, advance_turn, ExtraTurnAdded event)
**Deferred items from prior PBs**: None relevant (activated_ability_cost_reductions off-by-one and Cavern CounterRestriction are unrelated, carried forward)

## Primitive Specification

Add `Effect::ExtraTurn { player: PlayerTarget, count: EffectAmount }` to the Effect enum.
This allows card definitions to grant one or more extra turns to a specified player.

**Why it's needed**: The turn queue infrastructure already exists (`TurnState.extra_turns: Vector<PlayerId>`, `advance_turn()` pops LIFO, `GameEvent::ExtraTurnAdded`), but there is no Effect variant to push turns onto the queue from card definitions. Four cards and one partial fix are blocked on this.

**How it fits**: The new variant dispatches in `execute_effect_inner()` by resolving the player target, resolving the count, and calling `state.turn.extra_turns.push_back(player_id)` N times (LIFO: each push_back adds to end, pop_back retrieves last). Emits `GameEvent::ExtraTurnAdded { player }` for each turn added.

**GiftType::ExtraTurn wiring**: The `execute_gift_effect()` function in resolution.rs currently has a no-op arm for `GiftType::ExtraTurn`. This PB wires it up to push the recipient onto `extra_turns` and emit the event.

## CR Rule Text

**CR 500.7**: "Some effects can give a player extra turns. They do this by adding the turns directly after the specified turn. If a player is given multiple extra turns, the extra turns are added one at a time. If multiple players are given extra turns, the extra turns are added one at a time, in APNAP order (see rule 101.4). The most recently created turn will be taken first."

**CR 702.174g**: "'Gift an extra turn' means the effect is 'The chosen player takes an extra turn after this one.'"

## Engine Changes

### Change 1: Add `Effect::ExtraTurn` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant to the `Effect` enum, after `SolveCase` (line ~1853)
**Pattern**: Follow `Effect::AdditionalCombatPhase` at line ~1214 (similar turn-structure effect)

```rust
/// CR 500.7: Grant extra turns to a player. The specified player takes
/// `count` extra turns after this one. Multiple turns are added one at a
/// time (LIFO — most recently added goes first per CR 500.7).
///
/// Used by Nexus of Fate, Temporal Trespass, Temporal Mastery,
/// Teferi Master of Time (-10), Emrakul the Promised End (partial).
ExtraTurn {
    /// The player who takes the extra turn(s).
    player: PlayerTarget,
    /// Number of extra turns to grant (usually Fixed(1), Teferi uses Fixed(2)).
    count: EffectAmount,
},
```

### Change 2: Dispatch in `execute_effect_inner()`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `Effect::SolveCase` (line ~4526), before the closing `}`
**CR**: 500.7 — extra turns are added to the LIFO queue

```rust
// CR 500.7: Grant extra turns to a player.
Effect::ExtraTurn { player, count } => {
    let resolved_count = resolve_amount(state, count, ctx).max(0) as u32;
    let targets = resolve_player_target(state, player, ctx);
    for pid in targets {
        for _ in 0..resolved_count {
            state.turn.extra_turns.push_back(pid);
            events.push(GameEvent::ExtraTurnAdded { player: pid });
        }
    }
}
```

Note: `resolve_player_target` and `resolve_amount` are existing helper functions in effects/mod.rs. The runner should verify exact names — they may be `resolve_player_target_list` or similar.

### Change 3: Hash the new variant

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `Effect::ExtraTurn` in the `impl HashInto for Effect` block (after line 5154, discriminant 76)

```rust
// CR 500.7: ExtraTurn (discriminant 76)
Effect::ExtraTurn { player, count } => {
    76u8.hash_into(hasher);
    player.hash_into(hasher);
    count.hash_into(hasher);
}
```

### Change 4: Wire up `GiftType::ExtraTurn` in resolution.rs

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Replace the no-op arm for `GiftType::ExtraTurn` at line ~7208 with actual logic
**CR**: 702.174g — "The chosen player takes an extra turn after this one."

```rust
GiftType::ExtraTurn => {
    // CR 702.174g: "The chosen player takes an extra turn after this one."
    state.turn.extra_turns.push_back(recipient);
    events.push(GameEvent::ExtraTurnAdded { player: recipient });
}
```

Also update the `GiftType::TappedFish | GiftType::Octopus | GiftType::ExtraTurn` combined match arm to split out `ExtraTurn`. The remaining deferred arms become `GiftType::TappedFish | GiftType::Octopus`.

### Change 5: Exhaustive match updates

Files requiring new match arms for the new Effect variant:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `match effect` | L216 | Add dispatch arm (Change 2) |
| `crates/engine/src/state/hash.rs` | `impl HashInto for Effect` | L4532 | Add hash arm, discriminant 76 (Change 3) |

No other files have exhaustive matches on `Effect`. The tools (replay-viewer, TUI) do not match on Effect variants.

## Card Definition Fixes

### teferi_master_of_time.rs

**Oracle text**: "You may activate loyalty abilities of Teferi on any player's turn any time you could cast an instant. +1: Draw a card, then discard a card. -3: Target creature you don't control phases out. -10: Take two extra turns after this one."
**Loyalty**: 3

**Current state**: Only +1 (draw-only, no discard). Missing -3 (PhaseOut not in DSL) and -10 (ExtraTurn not in DSL). Three TODOs.

**Fix**: Add the -10 loyalty ability using `Effect::ExtraTurn { player: PlayerTarget::Controller, count: EffectAmount::Fixed(2) }`. The +1 "then discard" and -3 "phase out" remain TODOs (those are separate DSL gaps). Update TODO comments to reflect that the -10 is now implemented.

```rust
abilities: vec![
    // +1: Draw a card, then discard a card.
    // TODO: "then discard a card" — forced discard on self not easily expressible.
    AbilityDefinition::LoyaltyAbility {
        cost: LoyaltyCost::Plus(1),
        effect: Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        targets: vec![],
    },
    // -3: Target creature you don't control phases out.
    // TODO: Phase-out target effect — no Effect::PhaseOut variant.
    // -10: Take two extra turns after this one.
    AbilityDefinition::LoyaltyAbility {
        cost: LoyaltyCost::Minus(10),
        effect: Effect::ExtraTurn {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(2),
        },
        targets: vec![],
    },
],
```

### emrakul_the_promised_end.rs

**Oracle text**: "This spell costs {1} less to cast for each card type among cards in your graveyard. When you cast this spell, you gain control of target opponent during that player's next turn. After that turn, that player takes an extra turn. Flying, trample, protection from instants"

**Current state**: Has keywords (Flying, Trample, ProtectionFrom(Instant)), cost reduction. Missing cast trigger with gain-control + extra turn. One TODO.

**Fix**: Add a partial cast trigger that grants the extra turn to the target opponent. The "gain control of target opponent during that player's next turn" part remains a TODO (player-control effect not in DSL). The extra turn part is expressible.

Note: Emrakul's "that player takes an extra turn" fires AFTER the controlled turn, which means it's a delayed effect. Since we can't model "after that turn" precisely without the player-control infrastructure, the pragmatic fix is to add the extra turn effect as a comment/TODO noting it's partially resolved, with the extra turn being grantable once the cast trigger framework supports it. The runner should add the `ExtraTurn` effect as the body of a cast trigger (if cast triggers support `Effect`), or note it as blocked.

Actually, looking at Emrakul more carefully: the entire cast trigger is "gain control of target opponent during that player's next turn. After that turn, that player takes an extra turn." The extra turn is tied to the gain-control effect completing. Without player control, we can't properly implement the sequencing. **Mark the ExtraTurn part as known-expressible but keep the TODO since the trigger as a whole is blocked on player-control.**

Update the TODO comment to: `TODO: Cast trigger — gain-control blocked (PB-A/PB-E); extra turn part expressible via Effect::ExtraTurn (PB-C).`

## New Card Definitions

### nexus_of_fate.rs

**Oracle text**: "Take an extra turn after this one. If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's library instead."
**Mana Cost**: {5}{U}{U}
**Type**: Instant
**Color Identity**: U

**CardDefinition sketch**:

```rust
CardDefinition {
    card_id: cid("nexus-of-fate"),
    name: "Nexus of Fate".to_string(),
    mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
    types: types(&[CardType::Instant]),
    oracle_text: "Take an extra turn after this one.\nIf Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's library instead.".to_string(),
    abilities: vec![
        // CR 614.1a / CR 614.15: Self-replacement — if this would go to graveyard
        // from anywhere, shuffle into owner's library instead.
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldChangeZone {
                from: None, // from anywhere
                to: ZoneType::Graveyard,
                filter: ObjectFilter::Any, // substituted with SpecificObject at registration
            },
            modification: ReplacementModification::ShuffleIntoOwnerLibrary,
            is_self: true,
            unless_condition: None,
        },
    ],
    spell_effect: Some(Effect::ExtraTurn {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    }),
    ..Default::default()
}
```

**IMPORTANT**: Nexus of Fate's self-replacement effect says "from anywhere" — it applies when the card is discarded, milled, or would go to graveyard after resolving. For permanents, `register_permanent_replacement_abilities` handles this. For instants/sorceries, the replacement needs to be checked at the resolution destination selection (resolution.rs ~line 1683-1698) AND in any generic zone-change path.

The runner must verify that the self-replacement registration infrastructure handles non-permanent CardDef replacements. If `register_permanent_replacement_abilities` is only called for permanents entering the battlefield, the runner needs to:
1. Check if `check_zone_change_replacement` already scans CardDef-level replacements on the object being moved, OR
2. Add a check in the resolution.rs instant/sorcery destination selection (lines 1683-1698) that reads the resolving spell's CardDef for self-replacement effects and applies `ShuffleIntoOwnerLibrary` when matched.

Option 2 is simpler and more contained. The runner should check the `AbilityDefinition::Replacement` on the CardDef at the same site where flashback/jump-start/cipher/adventure redirects are checked.

### temporal_trespass.rs

**Oracle text**: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.) Take an extra turn after this one. Exile Temporal Trespass."
**Mana Cost**: {8}{U}{U}{U}
**Type**: Sorcery
**Color Identity**: U

**CardDefinition sketch**:

```rust
CardDefinition {
    card_id: cid("temporal-trespass"),
    name: "Temporal Trespass".to_string(),
    mana_cost: Some(ManaCost { generic: 8, blue: 3, ..Default::default() }),
    types: types(&[CardType::Sorcery]),
    oracle_text: "Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nTake an extra turn after this one. Exile Temporal Trespass.".to_string(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Delve),
        // "Exile ~" — self-replacement: when this would go to graveyard after
        // resolving, exile it instead. Modeled as Replacement.
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldChangeZone {
                from: None,
                to: ZoneType::Graveyard,
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
            is_self: true,
            unless_condition: None,
        },
    ],
    spell_effect: Some(Effect::ExtraTurn {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    }),
    ..Default::default()
}
```

Note: The "Exile ~" text is a self-replacement — when the spell resolves and would go to graveyard (CR 608.2n), it goes to exile instead. Same registration concern as Nexus of Fate. The runner should apply the same approach.

### temporal_mastery.rs

**Oracle text**: "Take an extra turn after this one. Exile Temporal Mastery. Miracle {1}{U} (You may cast this card for its miracle cost when you draw it if it's the first card you drew this turn.)"
**Mana Cost**: {5}{U}{U}
**Type**: Sorcery
**Color Identity**: U

**CardDefinition sketch**:

```rust
CardDefinition {
    card_id: cid("temporal-mastery"),
    name: "Temporal Mastery".to_string(),
    mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
    types: types(&[CardType::Sorcery]),
    oracle_text: "Take an extra turn after this one. Exile Temporal Mastery.\nMiracle {1}{U} (You may cast this card for its miracle cost when you draw it if it's the first card you drew this turn.)".to_string(),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Miracle {
            cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
        }),
        // "Exile ~" — self-replacement effect (same as Temporal Trespass).
        AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldChangeZone {
                from: None,
                to: ZoneType::Graveyard,
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
            is_self: true,
            unless_condition: None,
        },
    ],
    spell_effect: Some(Effect::ExtraTurn {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    }),
    ..Default::default()
}
```

## Infrastructure Note: Non-Permanent Self-Replacement Effects

The three new cards (Nexus of Fate, Temporal Trespass, Temporal Mastery) all have `is_self: true` replacement effects on instants/sorceries. The existing `register_permanent_replacement_abilities` only registers replacements when permanents enter the battlefield. For instants/sorceries, these replacements must be checked at the resolution destination selection site in resolution.rs (lines ~1683-1698).

**Implementation approach**: At the resolution destination selection site, after checking flashback/jump-start/cipher/adventure/buyback, add a check that reads the resolving spell's CardDef (via card registry lookup on `card_id`) for `AbilityDefinition::Replacement { is_self: true, .. }` entries. If a `WouldChangeZone { to: Graveyard }` replacement with `ShuffleIntoOwnerLibrary` is found, redirect to shuffle-into-library. If `RedirectToZone(Exile)` is found, redirect to exile.

This is the same pattern as the existing flashback/jump-start checks — just driven by CardDef metadata instead of stack object flags.

Alternatively, the runner could add a simpler boolean flag `self_exile_on_resolution: bool` to `CardDefinition` (for Temporal Trespass/Mastery) and `self_shuffle_on_resolution: bool` (for Nexus of Fate). This avoids the replacement infrastructure entirely. The flags are checked at the destination selection site in resolution.rs. This is the recommended simpler approach.

## Unit Tests

**File**: `crates/engine/tests/extra_turns.rs` (existing file, add new tests)
**Tests to write**:

- `test_effect_extra_turn_basic` — CR 500.7: Effect::ExtraTurn grants one extra turn to the controller. Build a 4-player game, execute the effect, verify `state.turn.extra_turns` contains the player and `ExtraTurnAdded` event is emitted.
- `test_effect_extra_turn_two_turns` — CR 500.7: Effect::ExtraTurn with count=2 (Teferi -10) grants two extra turns. Verify both are added LIFO (second push_back is taken first).
- `test_effect_extra_turn_opponent` — CR 702.174g: ExtraTurn targeting an opponent (Gift an extra turn). Verify the opponent's PlayerId is pushed onto extra_turns.
- `test_effect_extra_turn_resolves_and_taken` — End-to-end: cast a spell with ExtraTurn effect, pass priority through resolution, verify the controller gets the next turn.
- `test_gift_extra_turn` — CR 702.174g: GiftType::ExtraTurn correctly grants the chosen opponent an extra turn. Verify extra_turns queue and event emission.
- `test_nexus_of_fate_shuffle_replacement` — CR 614.1a: When Nexus of Fate would go to graveyard after resolving, it is shuffled into its owner's library instead. Verify it is NOT in the graveyard.
- `test_temporal_trespass_exile_self` — Temporal Trespass exiles itself after resolving instead of going to graveyard. Verify it is in exile, not graveyard.
- `test_temporal_mastery_miracle_extra_turn` — Temporal Mastery cast for Miracle cost still grants an extra turn and exiles itself.

**Pattern**: Follow existing tests in `crates/engine/tests/extra_turns.rs` (use `four_player_with_libraries`, `complete_turn` helpers, direct state manipulation for unit tests, `process_command` for integration).

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (Teferi -10, Emrakul comment update)
- [ ] New card defs authored (Nexus of Fate, Temporal Trespass, Temporal Mastery)
- [ ] Non-permanent self-replacement infrastructure works (Nexus shuffle, Temporal exile)
- [ ] GiftType::ExtraTurn wired up in resolution.rs
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except known blocked items: Teferi +1 discard, Teferi -3 phase-out, Emrakul gain-control)

## Risks & Edge Cases

- **Non-permanent self-replacement registration**: The biggest implementation risk. The current replacement infrastructure is permanent-centric. Instants/sorceries with `is_self: true` replacements (Nexus of Fate, Temporal Trespass, Temporal Mastery) need a new check at the resolution destination selection site. The simpler flag-based approach (`self_exile_on_resolution` / `self_shuffle_on_resolution`) avoids this entirely and is recommended.
- **ExtraTurn count resolution**: `EffectAmount::Fixed(2)` for Teferi is straightforward, but if `resolve_amount` returns a negative number, clamp to 0 to avoid underflow.
- **LIFO ordering with count > 1**: When Teferi grants 2 extra turns, they are pushed one at a time. Since they're for the same player, order doesn't matter. But verify that both are correctly popped.
- **Gift extra turn + eliminated player**: If the gift recipient has been eliminated before the extra turn is taken, `advance_turn()` should skip them. Verify existing tests cover this (the `next_player_in_turn_order` function may handle this, but extra turns pop directly without active-player check). This is an edge case worth testing.
- **Nexus of Fate infinite loop**: In a real game, Nexus of Fate can create deterministic infinite loops (cast → shuffle → draw → cast). The existing `loop_detection.rs` mandatory-loop detection (CR 104.4b: draw) should handle this. No new work needed, but worth a note.
- **Emrakul partial fix**: Only updating the TODO comment, not adding functional extra turn logic. The cast trigger's extra turn is sequenced after the gain-control turn completes, which requires infrastructure that doesn't exist yet (player control + delayed extra turn). Documenting this clearly prevents confusion.
