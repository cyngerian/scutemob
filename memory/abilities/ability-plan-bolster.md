# Ability Plan: Bolster

**Generated**: 2026-02-28
**CR**: 701.39 (Keyword Actions)
**Priority**: P3
**Similar abilities studied**: Connive (`Effect::Connive` in `card_definition.rs:416`, `effects/mod.rs:1809`), Proliferate (`Effect::Proliferate` in `card_definition.rs:531`, `effects/mod.rs:1518`), Investigate (`Effect::Investigate` in `card_definition.rs:340`, `effects/mod.rs:490`), AddCounter (`Effect::AddCounter` in `card_definition.rs:368`, `effects/mod.rs:920`)

## CR Rule Text

```
701.39. Bolster

701.39a "Bolster N" means "Choose a creature you control with the least
toughness or tied for least toughness among creatures you control. Put N
+1/+1 counters on that creature."
```

## Key Edge Cases

1. **Bolster does NOT target (CR 701.39a).** The creature is chosen on resolution, not when
   the spell/ability is put on the stack. This means protection from the source's color does
   not prevent bolster from placing counters (Abzan Skycaptain ruling 2014-11-24: "you could
   put counters on a creature with protection from white").

2. **Toughness is determined at resolution time (ruling 2014-11-24).** "You determine which
   creature to put counters on as the spell or ability that instructs you to bolster
   resolves." This means the toughness comparison must use `calculate_characteristics`
   (layer-aware toughness), not raw `characteristics.toughness`.

3. **Tied for least toughness -- controller chooses.** If multiple creatures are tied for
   the least toughness, the controller chooses which one gets the counters. Deterministic
   fallback: choose the creature with the smallest `ObjectId` among those tied for least
   toughness.

4. **No creatures you control -- bolster does nothing.** If the controller has no creatures
   on the battlefield when bolster resolves, nothing happens. No error.

5. **Multiple bolster triggers resolve independently (Dromoka ruling 2014-11-24).** "The
   creature you control with the lowest toughness as the first such ability resolves may
   not have the lowest toughness as the second such ability resolves." Each bolster
   recalculates at its own resolution time.

6. **Bolster can target the source itself.** If the creature with the bolster ETB trigger
   has the least toughness among creatures the controller controls, the counters go on it.

7. **Multiplayer:** Bolster only considers creatures controlled by the bolster ability's
   controller, not all creatures on the battlefield.

## Current State (from ability-wip.md)

- [ ] Step 1: Effect variant
- [ ] Step 2: Rule enforcement (effect execution)
- [ ] Step 3: Trigger wiring (n/a -- bolster is an effect used inside triggered/spell abilities)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Effect Variant

Bolster is a **keyword action** (CR 701.39), not a keyword ability. Per the gotchas in
`memory/gotchas-infra.md` ("Keyword actions (Surveil, Scry, etc.) are Effects, NOT
`KeywordAbility` enum variants"), it should be added as an `Effect` variant.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Effect::Bolster` variant in the Counters section (after `RemoveCounter` at ~line 378)
**Pattern**: Follow `Effect::Connive` at line 416 and `Effect::Proliferate` at line 531

```rust
/// CR 701.39: Bolster N -- "Choose a creature you control with the least
/// toughness or tied for least toughness among creatures you control. Put
/// N +1/+1 counters on that creature."
///
/// Bolster does NOT target (ruling 2014-11-24). The creature is chosen at
/// resolution time. If the controller has no creatures, nothing happens.
/// Deterministic fallback for tied toughness: choose smallest ObjectId.
Bolster {
    /// The player who controls the bolster effect (determines which
    /// creatures are eligible).
    player: PlayerTarget,
    /// Number of +1/+1 counters to place.
    count: EffectAmount,
},
```

**Note**: Unlike `AddCounter` which takes an `EffectTarget` (a pre-declared target),
`Bolster` takes a `PlayerTarget` because the creature is chosen during resolution based on
the controller's creatures, not targeted at cast time. The `player` field identifies whose
creatures to consider (always the controller in practice, but using `PlayerTarget` keeps it
consistent with other keyword-action effects like `Scry` and `Surveil`).

**No KeywordAbility variant needed.** Bolster is not a static/triggered/activated keyword
that sits on a permanent. Cards say "bolster N" in their effect text, and the card definition
will reference `Effect::Bolster { ... }` inside an `AbilityDefinition::Triggered` or
`AbilityDefinition::Spell`.

**No hash.rs update needed.** `Effect` variants are not hashed directly -- they are part of
`CardDefinition` (which is registered in the `CardRegistry` and not part of mutable game
state). The `GameEvent` variant (Step 2) does need to be covered in `hash.rs` if it carries
data.

### Step 2: Rule Enforcement (Effect Execution)

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add handling for `Effect::Bolster` in the main `execute_effect` match
**Location**: After the `Effect::RemoveCounter` handler (~line 970), in the Counters section
**CR**: 701.39a -- choose creature with least toughness, put N +1/+1 counters on it

Implementation logic:

```rust
Effect::Bolster { player, count } => {
    let n = resolve_amount(state, count, ctx).max(0) as u32;
    if n == 0 {
        // Bolster 0 does nothing.
        return events;
    }
    let players = resolve_player_target_list(state, player, ctx);
    for p in players {
        // CR 701.39a: Find all creatures controlled by this player on the
        // battlefield, then select the one with the least toughness.
        // Use calculate_characteristics for layer-aware toughness (ruling 2014-11-24).
        let creatures: Vec<(ObjectId, i32)> = state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller == p
                    && obj.characteristics.card_types.contains(&CardType::Creature)
            })
            .filter_map(|(&id, _)| {
                let chars = crate::rules::layers::calculate_characteristics(state, id)?;
                // Only creatures with calculable toughness
                chars.toughness.map(|t| (id, t))
            })
            .collect();

        if creatures.is_empty() {
            // No creatures -- bolster does nothing.
            continue;
        }

        // Find the minimum toughness value.
        let min_toughness = creatures.iter().map(|(_, t)| *t).min().unwrap();

        // Among tied creatures, choose the one with the smallest ObjectId
        // (deterministic fallback -- interactive choice deferred to M10+).
        let chosen_id = creatures
            .iter()
            .filter(|(_, t)| *t == min_toughness)
            .map(|(id, _)| *id)
            .min_by_key(|id| id.0)
            .unwrap();

        // Place N +1/+1 counters on the chosen creature.
        if let Some(obj) = state.objects.get_mut(&chosen_id) {
            let cur = obj.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
            obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, cur + n);
            events.push(GameEvent::CounterAdded {
                object_id: chosen_id,
                counter: CounterType::PlusOnePlusOne,
                count: n,
            });
        }
    }
}
```

**Key design decisions:**

1. **Layer-aware toughness**: Must use `calculate_characteristics` (not raw
   `obj.characteristics.toughness`) because continuous effects, counters, and Auras can
   modify toughness. The ruling explicitly says "as the ability resolves."

2. **`card_types.contains(&CardType::Creature)` check on base characteristics**: This
   identifies creatures on the battlefield. Note: we should also check via
   `calculate_characteristics` in case a non-creature was animated (e.g., Mutavault). Use
   the layer-aware check from the `chars` we already compute.

3. **Deterministic tie-breaking**: When multiple creatures share the least toughness, the
   controller should choose. Since interactive choice is deferred to M10+, use smallest
   `ObjectId` as a deterministic fallback (matches the pattern used by other effects like
   Connive's discard and sacrifice permanents).

4. **No separate GameEvent for Bolster**: Unlike Surveil/Investigate/Connive which have
   dedicated events for "whenever you X" triggers, there are no cards with "whenever you
   bolster" triggers in MTG. The `CounterAdded` event is sufficient. If such a trigger is
   ever needed, a `Bolstered` event can be added later.

**Imports needed in `effects/mod.rs`**: `CounterType` and `ZoneId` are already imported.
`CardType` may need to be added to the import block if not already present. Check at
implementation time.

### Step 3: Trigger Wiring

**Not applicable.** Bolster is a keyword action (effect primitive), not a triggered or
static ability. It is used **inside** triggered abilities (ETB triggers) or spell effects
on individual cards. The cards themselves define the trigger; bolster is just the effect
payload.

For example, Sandsteppe Mastodon's card definition will have:
```rust
AbilityDefinition::Triggered {
    trigger: TriggerCondition::WhenSelfEnters,
    effect: Effect::Bolster {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(5),
    },
    ..
}
```

No wiring is needed in `abilities.rs`, `builder.rs`, `stack.rs`, or `resolution.rs`.
No new `StackObjectKind` variant is needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/bolster.rs`
**Pattern**: Follow the Evolve test file at `crates/engine/tests/evolve.rs` for structure,
but simpler since bolster is an effect (not a triggered keyword ability with intervening-if).

**Tests to write:**

1. **`test_bolster_basic_single_creature`** -- CR 701.39a
   - P1 controls a single 2/3 creature on the battlefield.
   - Execute a spell/trigger that bolsters 2.
   - Assert: creature has 2 +1/+1 counters, layer-aware P/T is 4/5.
   - Validates basic counter placement.

2. **`test_bolster_chooses_least_toughness`** -- CR 701.39a
   - P1 controls three creatures: a 1/1, a 2/3, and a 3/5.
   - Execute bolster 2.
   - Assert: the 1/1 creature (least toughness) receives the 2 counters.
   - Assert: other creatures have no counters.

3. **`test_bolster_tied_toughness_deterministic`** -- CR 701.39a (tie-breaking)
   - P1 controls two creatures both with toughness 2 (a 1/2 and a 3/2).
   - Execute bolster 1.
   - Assert: the creature with the smaller ObjectId receives the counter.
   - Documents the deterministic fallback behavior.

4. **`test_bolster_no_creatures_does_nothing`** -- CR 701.39a (no creatures case)
   - P1 controls no creatures (only non-creature permanents or nothing).
   - Execute bolster 3.
   - Assert: no counters placed, no error, no panic.

5. **`test_bolster_uses_layer_aware_toughness`** -- CR 701.39a + ruling 2014-11-24
   - P1 controls a 1/1 creature with 2 +1/+1 counters (layer-aware T = 3) and a 2/2
     creature (layer-aware T = 2).
   - Execute bolster 1.
   - Assert: the 2/2 creature (lower layer-aware T) receives the counter, not the 1/1
     (which has higher effective T due to counters).

6. **`test_bolster_not_targeting_ignores_protection`** -- Ruling 2014-11-24
   - P1 controls only a creature with protection from white.
   - Execute a white-sourced bolster effect.
   - Assert: creature receives counters (bolster doesn't target, so protection
     doesn't prevent it).

7. **`test_bolster_can_target_source`** -- Edge case
   - P1 controls a 1/1 ETB creature that bolsters 2, and a 3/3 creature.
   - After the ETB trigger resolves, the 1/1 source (least toughness) receives counters.
   - Assert: source has 2 +1/+1 counters.

8. **`test_bolster_multiplayer_only_controllers_creatures`** -- Multiplayer
   - 4-player game. P1 controls a 5/5, P2 controls a 1/1.
   - P1's effect bolsters 2.
   - Assert: P1's 5/5 receives counters (it is the least-toughness creature P1 controls).
   - Assert: P2's 1/1 has no counters (bolster only considers controller's creatures).

**Test infrastructure:**
- Create test-only card definitions (sorcery with `Effect::Bolster`, creature with ETB
  bolster trigger) using the same pattern as `evolve.rs` helper functions.
- Use `ObjectSpec::creature(owner, name, power, toughness)` for creatures on the battlefield.
- Use `cast_and_resolve` helper pattern from evolve tests for casting spells.
- Verify counters via `obj.counters.get(&CounterType::PlusOnePlusOne)`.
- Verify layer-aware P/T via `calculate_characteristics`.

### Step 5: Card Definition (later phase)

**Suggested card**: Sandsteppe Mastodon
- Oracle text: "Reach / When this creature enters, bolster 5."
- Simple ETB bolster trigger on a large green creature.
- Tests both bolster and reach (already implemented as KeywordAbility).
- Card definition file: `crates/engine/src/cards/defs/sandsteppe_mastodon.rs`

**Alternative cards** (simpler):
- **Cached Defenses** (sorcery, {2}{G}, "Bolster 3") -- pure bolster sorcery, simplest test.
- **Elite Scaleguard** (creature, ETB bolster 2 + additional triggered ability) -- more complex.
- **Abzan Skycaptain** (creature, "When this creature dies, bolster 2") -- death trigger bolster.

**Recommendation**: Author **Cached Defenses** first (simplest sorcery-only card), then
**Sandsteppe Mastodon** (ETB creature). Use the `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Bolster selects creature with least toughness"
- P1 controls three creatures with different toughness values.
- P1 casts Cached Defenses (sorcery, bolster 3).
- Assert: the creature with the least toughness receives 3 +1/+1 counters.
- Assert: other creatures are unchanged.

**Subsystem directory**: `test-data/generated-scripts/baseline/` (bolster is a core
keyword action, not combat/stack/replacement-specific).

**Second scenario** (if needed): "ETB bolster on Sandsteppe Mastodon"
- P1 casts Sandsteppe Mastodon.
- ETB trigger fires: bolster 5.
- If Sandsteppe Mastodon has the least toughness (5/5) among P1's creatures, it gets the
  counters on itself. If P1 has a smaller creature, that creature gets them.

## Interactions to Watch

1. **Bolster + Doubling Season / Hardened Scales**: These modify counter placement. Bolster
   uses `AddCounter`-style logic (placing +1/+1 counters), so any future counter-doubling
   replacement effects will need to intercept bolster's counter placement. Currently no
   counter-doubling is implemented, so this is a future concern.

2. **Bolster + Humility**: Humility removes all abilities in Layer 6, which would prevent
   ETB triggers from firing in the first place. If an ETB bolster trigger somehow gets on
   the stack (e.g., Humility entered after the trigger was created), bolster still resolves
   normally because the effect itself is on the stack, not dependent on the permanent's
   abilities.

3. **Bolster + animated non-creatures**: If a non-creature permanent is animated (e.g.,
   Mutavault, man-lands), its layer-aware characteristics will show it as a creature. The
   implementation should use `calculate_characteristics` to check creature type, not raw
   `characteristics.card_types`. This ensures animated permanents are eligible for bolster.

4. **Bolster + toughness-modifying continuous effects**: Effects like Doran, the Siege Tower
   ("each creature assigns combat damage equal to its toughness rather than its power")
   don't change toughness itself. But effects like Elesh Norn (-2/-2 to opponents'
   creatures) DO change effective toughness. Using `calculate_characteristics` handles all
   of these correctly.

5. **Multiple bolster effects resolving sequentially**: After the first bolster places
   counters, the second bolster must recalculate which creature has the least toughness.
   The counters from the first bolster change the effective toughness of the first target.
   This is handled naturally because each `Effect::Bolster` execution reads the current
   state (with counters already placed).
