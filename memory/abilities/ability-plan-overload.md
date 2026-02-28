# Ability Plan: Overload

**Generated**: 2026-02-28
**CR**: 702.96
**Priority**: P3
**Similar abilities studied**: Evoke (alternative cost, `AbilityDefinition::Evoke`, `cast_with_evoke` flag), Kicker (conditional effect dispatch via `Condition::WasKicked`), Burst Lightning (card definition pattern for conditional effects)

## CR Rule Text

**702.96. Overload**

> **702.96a** Overload is a keyword that represents two static abilities that function while the spell with overload is on the stack. Overload [cost] means "You may choose to pay [cost] rather than pay this spell's mana cost" and "If you chose to pay this spell's overload cost, change its text by replacing all instances of the word 'target' with the word 'each.'" Casting a spell using its overload ability follows the rules for paying alternative costs in rules 601.2b and 601.2f-h.

> **702.96b** If a player chooses to pay the overload cost of a spell, that spell won't require any targets. It may affect objects that couldn't be chosen as legal targets if the spell were cast without its overload cost being paid.

> **702.96c** Overload's second ability creates a text-changing effect. See rule 612, "Text-Changing Effects."

**Related: CR 612.1** (text-changing effects), **CR 601.2b** (alternative cost announcement), **CR 601.2f** (total cost determination), **CR 118.9** (alternative costs -- only one allowed per spell).

## Key Edge Cases

1. **No targets when overloaded (CR 702.96b)**: An overloaded spell has NO targets at all. It cannot fizzle (the fizzle check at `resolution.rs:52` only fires when `!targets.is_empty()`). It also bypasses hexproof, shroud, and protection-from-targeting (since it does not target).

2. **Alternative cost exclusivity (CR 118.9a)**: Overload is an alternative cost. It cannot be combined with flashback, evoke, bestow, madness, miracle, escape, or foretell. Only one alternative cost per spell.

3. **Cannot overload when cast "without paying mana cost" (Cyclonic Rift ruling 2024-01-12)**: "If you are instructed to cast a spell with overload 'without paying its mana cost,' you can't choose to pay its overload cost instead." This means suspend, cascade, and similar "cast free" effects cannot be overloaded. Overload IS the cost -- you must choose it at cast time instead of paying the normal mana cost.

4. **Mana value unchanged (Mizzium Mortars ruling 2024-01-12)**: "The mana value of the spell remains unchanged, no matter what the total cost to cast it was." The spell's printed mana cost determines its mana value, not the overload cost.

5. **Protection still prevents damage (Mizzium Mortars ruling 2024-01-12)**: "Note that if the spell with overload is dealing damage, protection from that spell's color will still prevent that damage." Protection prevents damage from matching sources regardless of targeting. Overloaded spells bypass targeting-protection but NOT damage-prevention-protection.

6. **Text-changing effect on the spell itself (CR 702.96c / 612.1)**: The text change is a text-changing effect on the spell. However, in our engine, this is modeled semantically via conditional effect dispatch (like kicker), not via literal text replacement.

7. **Multiplayer implications**: Overloaded Cyclonic Rift returns EACH nonland permanent the caster doesn't control to its owner's hand -- across all opponents. This makes it one of the most powerful Commander cards. The `EffectTarget::AllPermanentsMatching(filter)` pattern with `controller: TargetController::Opponent` handles this correctly.

8. **Teleportal "can't be blocked" ruling**: For overloaded Teleportal, the set of creatures that gets +1/+0 is determined on resolution, but "can't be blocked" is continuously updated (any creature you control at the moment blockers are chosen can't be blocked). This is modeled by the continuous effect system already.

## Current State (from ability-wip.md)

The WIP file is currently tracking Bolster (unrelated). Overload has no partial work.

- [ ] 1. Enum variant (`KeywordAbility::Overload`)
- [ ] 2. Rule enforcement (alternative cost in `casting.rs`, `was_overloaded` flag)
- [ ] 3. Condition variant (`Condition::WasOverloaded`)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and AbilityDefinition

**Files**:
- `crates/engine/src/state/types.rs` (KeywordAbility enum, ~line 560 after Hideaway)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition enum, ~line 254 after Suspend; Condition enum, ~line 777 after WasKicked)

**Action 1a**: Add `KeywordAbility::Overload` variant to the `KeywordAbility` enum.

```rust
/// CR 702.96: Overload [cost] -- alternative cost; replaces "target" with "each".
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The overload cost itself is stored in `AbilityDefinition::Overload { cost }`.
///
/// When cast with overload: the spell has no targets and affects all valid
/// objects instead of one target. Implements CR 702.96a/b via conditional
/// effect dispatch (like Kicker), not literal text replacement.
Overload,
```

**Pattern**: Follow `KeywordAbility::Evoke` at line 315 (marker keyword for alternative cost).

**Action 1b**: Add `AbilityDefinition::Overload { cost: ManaCost }` variant.

```rust
/// CR 702.96: Overload [cost]. The card may be cast by paying this cost instead
/// of its mana cost (alternative cost, CR 118.9). When overloaded, the spell's
/// text replaces all instances of "target" with "each" -- modeled as conditional
/// effect dispatch via `Condition::WasOverloaded`.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Overload)` for quick
/// presence-checking without scanning all abilities.
Overload { cost: ManaCost },
```

**Pattern**: Follow `AbilityDefinition::Evoke { cost }` at line 175.

**Action 1c**: Add `Condition::WasOverloaded` variant to the `Condition` enum.

```rust
/// CR 702.96a: "if this spell's overload cost was paid" -- true when
/// `was_overloaded` is set on the EffectContext or StackObject.
///
/// Checked at resolution time. Used in card definitions to branch between
/// single-target and all-matching-permanents effects (analogous to WasKicked).
WasOverloaded,
```

**Pattern**: Follow `Condition::WasKicked` at line 777.

### Step 2: StackObject Flag

**File**: `crates/engine/src/state/stack.rs` (~line 127 after `was_suspended`)

**Action**: Add `was_overloaded: bool` field to `StackObject`.

```rust
/// CR 702.96a: If true, this spell was cast by paying its overload cost
/// (an alternative cost). At resolution, the spell's effect uses the
/// "each" (all-matching) branch instead of the "target" (single-target)
/// branch. The spell has no targets and cannot fizzle.
///
/// Must always be false for copies (`is_copy: true`) -- copies are not cast.
#[serde(default)]
pub was_overloaded: bool,
```

**Pattern**: Follow `was_buyback_paid: bool` at line 118.

**Note**: Also add `was_overloaded: false` to ALL existing `StackObject` struct literals in `casting.rs` (two: the main spell creation at ~line 876 and the two cascade/storm helper sites at ~lines 988 and 1030).

### Step 3: Command Flag

**File**: `crates/engine/src/rules/command.rs` (~line 156 after `cast_with_buyback`)

**Action**: Add `cast_with_overload: bool` field to `Command::CastSpell`.

```rust
/// CR 702.96a: If true, cast this spell by paying its overload cost instead
/// of its mana cost. This is an alternative cost (CR 118.9) -- cannot
/// combine with flashback, evoke, bestow, madness, miracle, escape, foretell,
/// or other alternative costs.
///
/// When true, the spell has no targets (do NOT pass any in `targets`).
/// The spell's effect will use the overloaded branch (affecting all valid
/// objects instead of a single target).
#[serde(default)]
cast_with_overload: bool,
```

### Step 4: Casting Logic

**File**: `crates/engine/src/rules/casting.rs`

**Action 4a**: Add `cast_with_overload` parameter to `handle_cast_spell` function signature (~line 52).

Add after the `cast_with_buyback` parameter:
```rust
cast_with_overload: bool,
```

**Action 4b**: Add overload validation in the alternative cost mutual exclusion section (~line 262). Follow the Evoke pattern (Step 1/1c/1d/1e).

```rust
// Step 1g: Validate overload exclusion (CR 118.9a).
let casting_with_overload = if cast_with_overload {
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_bestow {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with bestow (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_madness {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with madness (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if cast_with_miracle {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with miracle (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_escape {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with escape (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_foretell {
        return Err(GameStateError::InvalidCommand(
            "cannot combine overload with foretell (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if get_overload_cost(&card_id, &state.card_registry).is_none() {
        return Err(GameStateError::InvalidCommand(
            "spell does not have overload".into(),
        ));
    }
    true
} else {
    false
};
```

Also add overload to the existing mutual exclusion checks for evoke, bestow, madness, miracle, escape, foretell (add `if casting_with_overload { ... }` blocks in each of those validation sections).

**Action 4c**: Add overload cost to Step 2 (base cost selection, ~line 485).

Add a new branch in the `if casting_with_evoke { ... } else if ...` chain:
```rust
} else if casting_with_overload {
    // CR 702.96a: Pay overload cost instead of mana cost.
    // CR 118.9c: The spell's printed mana cost is unchanged; only the payment differs.
    get_overload_cost(&card_id, &state.card_registry)
```

**Action 4d**: When overloading, enforce that targets are empty (CR 702.96b).

After target validation (~line 676), add:
```rust
// CR 702.96b: When overloaded, the spell has no targets.
if casting_with_overload && !targets.is_empty() {
    return Err(GameStateError::InvalidCommand(
        "overloaded spells have no targets (CR 702.96b)".into(),
    ));
}
```

Also, when overloading, override the requirements to empty so that target validation does not complain about missing targets:
```rust
// CR 702.96b: Override target requirements when overloaded -- spell has no targets.
let requirements = if casting_with_overload {
    vec![] // No targets required
} else {
    requirements
};
```

**Action 4e**: Set `was_overloaded` on the `StackObject` (~line 882).

```rust
was_overloaded: casting_with_overload,
```

**Action 4f**: Add `get_overload_cost` helper function (after `get_evoke_cost`, ~line 1142).

```rust
/// CR 702.96a: Look up the overload cost from the card's `AbilityDefinition`.
///
/// Returns the `ManaCost` stored in `AbilityDefinition::Overload { cost }`, or `None`
/// if the card has no definition or no overload ability defined.
fn get_overload_cost(
    card_id: &Option<crate::state::CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Overload { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

### Step 5: Engine Command Dispatch

**File**: `crates/engine/src/rules/engine.rs`

**Action**: Add `cast_with_overload` to the `Command::CastSpell` destructuring (~line 81) and pass it through to `casting::handle_cast_spell` (~line 100).

In the pattern match:
```rust
cast_with_overload,
```

In the function call:
```rust
casting::handle_cast_spell(
    &mut state, player, card, targets,
    convoke_creatures, improvise_artifacts, delve_cards,
    kicker_times, cast_with_evoke, cast_with_bestow,
    cast_with_miracle, cast_with_escape, escape_exile_cards,
    cast_with_foretell, cast_with_buyback, cast_with_overload,
)?;
```

### Step 6: EffectContext -- WasOverloaded

**File**: `crates/engine/src/effects/mod.rs`

**Action 6a**: Add `was_overloaded: bool` field to `EffectContext` (~line 65 after `kicker_times_paid`).

```rust
/// CR 702.96a: If true, this spell was cast with its overload cost paid.
/// Used by `Condition::WasOverloaded`. Set from `StackObject.was_overloaded`
/// at spell resolution.
pub was_overloaded: bool,
```

**Action 6b**: Update `EffectContext::new` and `EffectContext::new_with_kicker` to include `was_overloaded: false`.

**Action 6c**: Add a new constructor or extend `new_with_kicker` to accept `was_overloaded`:

```rust
/// Build a context with kicker and overload status.
pub fn new_with_cast_flags(
    controller: PlayerId,
    source: ObjectId,
    targets: Vec<SpellTarget>,
    kicker_times_paid: u32,
    was_overloaded: bool,
) -> Self {
    Self {
        controller,
        source,
        targets,
        target_remaps: HashMap::new(),
        kicker_times_paid,
        was_overloaded,
    }
}
```

**Alternative approach (simpler)**: Keep `new_with_kicker` and just add `was_overloaded: false` to it, then set `ctx.was_overloaded = stack_obj.was_overloaded` after construction in resolution.rs. This is simpler and avoids changing all existing call sites of `new_with_kicker`.

**Recommended approach**: Set `was_overloaded` directly after context creation in `resolution.rs`:

```rust
let mut ctx = EffectContext::new_with_kicker(
    controller, source_object, legal_targets, stack_obj.kicker_times_paid,
);
ctx.was_overloaded = stack_obj.was_overloaded;
```

**Action 6d**: Add `Condition::WasOverloaded` evaluation in the `evaluate_condition` function (~line 2666 after `WasKicked`).

```rust
// CR 702.96a: "if this spell's overload cost was paid" -- true when overloaded.
Condition::WasOverloaded => ctx.was_overloaded,
```

### Step 7: Hash Updates

**File**: `crates/engine/src/state/hash.rs`

**Action 7a**: Add `was_overloaded` hash to `StackObject` HashInto impl (~line 1384 after `was_suspended`).

```rust
// Overload (CR 702.96a) -- spell was cast with overload cost
self.was_overloaded.hash_into(hasher);
```

**Action 7b**: Add `Condition::WasOverloaded` hash to the Condition HashInto impl (~line 2425 after `WasKicked`).

```rust
// Overload condition (discriminant 8) -- CR 702.96a
Condition::WasOverloaded => 8u8.hash_into(hasher),
```

**Action 7c**: Add `AbilityDefinition::Overload` hash to the AbilityDefinition HashInto impl. Follow the pattern of `AbilityDefinition::Evoke`. Find the match arm and add:

```rust
AbilityDefinition::Overload { cost } => {
    <NEXT_DISCRIMINANT>u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

### Step 8: Replay Harness

**File**: `crates/engine/src/testing/replay_harness.rs`

**Action 8a**: Add `"cast_spell_overload"` action type to `translate_player_action` (~after `"cast_spell_foretell"` at ~line 647).

```rust
// CR 702.96a: Cast a spell with overload from the player's hand.
// The overload cost (an alternative cost) is paid instead of the mana cost.
// The spell has no targets -- it affects all valid objects.
"cast_spell_overload" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    // CR 702.96b: Overloaded spells have no targets.
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: vec![],
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
        cast_with_overload: true,
    })
}
```

**Action 8b**: Add `cast_with_overload: false` to ALL existing `Command::CastSpell` construction sites in the harness (approximately 10 sites). Grep for `cast_with_buyback:` and add `cast_with_overload: false,` after each.

### Step 9: Replay Viewer

**File**: `tools/replay-viewer/src/view_model.rs`

**Action**: Add `KeywordAbility::Overload` to the `format_keyword` match (~line 672 after `Hideaway`).

```rust
KeywordAbility::Overload => "Overload".to_string(),
```

### Step 10: Unit Tests

**File**: `crates/engine/tests/overload.rs` (new file)

**Tests to write**:

1. **`test_overload_normal_cast_targets_single`** -- CR 702.96a: Cast Vandalblast normally ({R}) targeting a single artifact. Verify only that artifact is destroyed.
   - Setup: P1 has Vandalblast in hand. P2 has 2 artifacts on battlefield. P1 has {R} in pool.
   - Cast with target on one artifact. Resolve. Assert only that artifact is destroyed. The other survives.

2. **`test_overload_cast_destroys_all_matching`** -- CR 702.96a/b: Cast Vandalblast with overload ({4}{R}). Verify ALL artifacts opponents control are destroyed.
   - Setup: P1 has Vandalblast in hand. P2 has 2 artifacts, P3 has 1 artifact, P1 has 1 artifact. P1 has {4}{R} in pool.
   - Cast with `cast_with_overload: true`, no targets. Resolve. Assert: P2's 2 artifacts destroyed, P3's 1 artifact destroyed, P1's artifact SURVIVES (Vandalblast says "you don't control").

3. **`test_overload_no_targets_cannot_fizzle`** -- CR 702.96b: Overloaded spell has no targets. It cannot be countered by the fizzle rule (CR 608.2b).
   - Setup: Cast overloaded spell. Even if there are no valid "each" objects, it still resolves (with no effect).
   - Cast Vandalblast overloaded when no opponents control artifacts. Verify spell resolves (SpellResolved event, not SpellFizzled).

4. **`test_overload_bypasses_hexproof`** -- CR 702.96b: "It may affect objects that couldn't be chosen as legal targets." Overloaded spell affects creatures/permanents with hexproof.
   - Setup: P2 has a creature with Hexproof on battlefield. P1 casts Cyclonic Rift overloaded.
   - Verify the hexproof creature is bounced (returned to hand).

5. **`test_overload_alternative_cost_exclusivity`** -- CR 118.9a: Cannot combine overload with flashback, evoke, etc.
   - Setup: Try to cast with `cast_with_overload: true` and `cast_with_evoke: true`. Verify error.

6. **`test_overload_pays_overload_cost`** -- CR 702.96a / 601.2f: The overload cost is paid instead of the mana cost.
   - Setup: P1 has Vandalblast in hand. P1 has {4}{R} in pool.
   - Cast with overload. Verify mana pool is depleted correctly ({4}{R} consumed).
   - Also test that casting with only {R} in pool (normal cost) and `cast_with_overload: true` fails (insufficient mana).

7. **`test_overload_no_targets_allowed`** -- CR 702.96b: If cast with overload, no targets may be declared.
   - Setup: Try to cast with `cast_with_overload: true` AND a target in the `targets` vec.
   - Verify error: "overloaded spells have no targets".

8. **`test_overload_commander_tax_applies`** -- CR 118.9d: Additional costs (commander tax) apply on top of alternative costs.
   - Setup: P1's commander has overload. Cast it from the command zone with overload. Verify overload cost + commander tax is paid.

**Pattern**: Follow tests for Evoke in `crates/engine/tests/evoke.rs` and Kicker in `crates/engine/tests/kicker.rs` for setup patterns.

### Step 11: Card Definition (later phase)

**Suggested cards** (in order of implementation utility):

1. **Vandalblast** ({R}, Sorcery) -- "Destroy target artifact you don't control. Overload {4}{R}."
   - Simplest Overload card: single effect, clear "target" -> "each" mapping.
   - Normal: `EffectTarget::DeclaredTarget { index: 0 }`, targets: `[TargetArtifact]`
   - Overloaded: `Effect::ForEach { over: ForEachTarget::EachPermanentMatching(filter), effect: DestroyPermanent { target: Source } }` OR simpler: `DestroyPermanent { target: AllPermanentsMatching(filter) }`

   Card definition structure:
   ```rust
   AbilityDefinition::Keyword(KeywordAbility::Overload),
   AbilityDefinition::Overload {
       cost: ManaCost { generic: 4, red: 1, ..Default::default() },
   },
   AbilityDefinition::Spell {
       effect: Effect::Conditional {
           condition: Condition::WasOverloaded,
           if_true: Box::new(Effect::DestroyPermanent {
               target: EffectTarget::AllPermanentsMatching(TargetFilter {
                   has_card_type: Some(CardType::Artifact),
                   controller: TargetController::Opponent,
                   ..Default::default()
               }),
           }),
           if_false: Box::new(Effect::DestroyPermanent {
               target: EffectTarget::DeclaredTarget { index: 0 },
           }),
       },
       targets: vec![TargetRequirement::TargetArtifact],
       modes: None,
       cant_be_countered: false,
   },
   ```

   **Key design note**: The `targets` field on `AbilityDefinition::Spell` lists the requirements for the NORMAL (non-overloaded) cast. When overloaded, the casting logic overrides `requirements` to `vec![]` (Step 4d), so no targets are validated. The `Condition::WasOverloaded` branch in the effect uses `AllPermanentsMatching` instead of `DeclaredTarget`, so it never reads from the (empty) target list.

2. **Cyclonic Rift** ({1}{U}, Instant) -- "Return target nonland permanent you don't control to its owner's hand. Overload {6}{U}."
   - The Commander staple. Normal: bounce one nonland permanent opponent controls. Overloaded: bounce ALL nonland permanents opponents control.

3. **Mizzium Mortars** ({1}{R}, Sorcery) -- "Deals 4 damage to target creature you don't control. Overload {3}{R}{R}{R}."
   - Tests damage-based overload across multiple creatures.

### Step 12: Game Script (later phase)

**Suggested scenario**: "Vandalblast Overload destroys all opponent artifacts in Commander"

**Subsystem directory**: `test-data/generated-scripts/stack/`

**Scenario outline**:
1. 4-player game. P1 has Vandalblast in hand. Give P1 {4}{R} mana.
2. P2 has Sol Ring + Arcane Signet on battlefield. P3 has Lightning Greaves on battlefield. P1 has Mind Stone on battlefield.
3. P1 casts Vandalblast with overload (`cast_spell_overload` action, no targets).
4. All players pass priority. Spell resolves.
5. Assert: P2's Sol Ring and Arcane Signet are destroyed (in graveyard). P3's Lightning Greaves is destroyed. P1's Mind Stone survives (P1 controls it; Vandalblast says "you don't control").

## Interactions to Watch

### Targeting System
- **Normal cast**: `targets: vec![TargetRequirement::TargetArtifact]` + `EffectTarget::DeclaredTarget { index: 0 }` -- standard single-target behavior. Hexproof/shroud/protection block targeting. Fizzle if target is illegal at resolution.
- **Overloaded cast**: No targets required. Effect uses `EffectTarget::AllPermanentsMatching(filter)` which resolves all matching permanents at resolution time. No hexproof/shroud/protection check (no targeting). Cannot fizzle (no targets to become illegal).

### Alternative Cost Pipeline
- Overload inserts into the existing alternative cost chain in `casting.rs`:
  `base_cost = overload_cost if casting_with_overload else ...`
- Commander tax, convoke, improvise, delve all apply on top (CR 118.9d).
- Kicker can be combined with overload (kicker is an additional cost, not alternative) -- but currently no card has both.

### Resolution Pipeline
- The fizzle check (`!targets.is_empty()`) naturally handles overloaded spells (no targets = no fizzle check).
- Effect dispatch uses `Condition::WasOverloaded` to branch between single-target and all-matching effects.
- `EffectContext.was_overloaded` is set from `StackObject.was_overloaded` at resolution time.

### Copy Effects (Storm, Cascade)
- Copies of overloaded spells: CR does not explicitly address this, but per CR 707.10, copies copy the characteristics of the original. The `was_overloaded` flag should NOT be copied (copies are not cast -- `is_copy: true`). However, if `was_overloaded` is on the stack object and copied via `copy_stack_object`, it should be evaluated. **Initial implementation**: set `was_overloaded: false` on copies (the conservative approach). This matches how `was_evoked`, `was_buyback_paid`, and `was_suspended` are handled -- all false for copies.

### Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Overload` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Overload { cost }`, `Condition::WasOverloaded` |
| `crates/engine/src/state/stack.rs` | Add `was_overloaded: bool` to `StackObject` |
| `crates/engine/src/rules/command.rs` | Add `cast_with_overload: bool` to `Command::CastSpell` |
| `crates/engine/src/rules/casting.rs` | Add overload validation, cost selection, `get_overload_cost` helper |
| `crates/engine/src/rules/engine.rs` | Pass `cast_with_overload` through |
| `crates/engine/src/effects/mod.rs` | Add `was_overloaded` to `EffectContext`, evaluate `Condition::WasOverloaded` |
| `crates/engine/src/state/hash.rs` | Hash `was_overloaded`, `Condition::WasOverloaded`, `AbilityDefinition::Overload` |
| `crates/engine/src/testing/replay_harness.rs` | Add `"cast_spell_overload"` action, `cast_with_overload: false` to all existing sites |
| `tools/replay-viewer/src/view_model.rs` | Add `KeywordAbility::Overload` to `format_keyword` |
| `crates/engine/tests/overload.rs` | New test file with 8 tests |
