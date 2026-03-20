# Primitive Batch Plan: PB-21 — Fight & Bite

**Generated**: 2026-03-19
**Primitive**: `Effect::Fight` (mutual power-based damage between two creatures) and `Effect::Bite` (one-sided power-based damage from one creature to another)
**CR Rules**: 701.14 (Fight), 701.14a-d
**Cards affected**: 5 existing fixes + 0 new (some existing cards have other blocking DSL gaps beyond Fight/Bite)
**Dependencies**: PB-5 (targeted abilities) — confirmed present
**Deferred items from prior PBs**: none

## Primitive Specification

Fight and Bite are keyword actions that deal non-combat damage between creatures using their power values.

- **Fight** (CR 701.14a): Two creatures each deal damage equal to their power to each other simultaneously. This is the literal "fights" keyword action on cards.
- **Bite**: An informal term for one-sided power-based damage — "target creature you control deals damage equal to its power to target creature you don't control." Not a CR keyword action, but a common card pattern that is mechanically distinct from Fight (only one creature deals damage).

The engine already has `Effect::DealDamage { target, amount: EffectAmount::PowerOf(source) }` which can express Bite in isolation (Eomer, King of Rohan uses this pattern at line 44 of its card def). However, **Fight** requires a new Effect variant because it involves two simultaneous damage events that both require creature validation per CR 701.14b.

Key design decision: `Effect::Bite` is added as a dedicated variant (not just `DealDamage + PowerOf`) because:
1. CR 701.14b validation applies to bite-like spells too (Ram Through ruling: "If either creature is an illegal target... the creature you control won't deal damage to any creature or player")
2. The source of damage for a bite is the creature, not the spell (matters for deathtouch, lifelink, infect, protection — see Warstorm Surge ruling)
3. Having a dedicated variant makes card defs more readable and self-documenting

## CR Rule Text

### CR 701.14 — Fight

> **701.14a** A spell or ability may instruct a creature to fight another creature or it may instruct two creatures to fight each other. Each of those creatures deals damage equal to its power to the other creature.
>
> **701.14b** If one or both creatures instructed to fight are no longer on the battlefield or are no longer creatures, neither of them fights or deals damage. If one or both creatures are illegal targets for a resolving spell or ability that instructs them to fight, neither of them fights or deals damage.
>
> **701.14c** If a creature fights itself, it deals damage to itself equal to twice its power.
>
> **701.14d** The damage dealt when a creature fights isn't combat damage.

### Key Implementation Notes

- **CR 701.14b**: Both creatures must be on the battlefield AND still be creatures at resolution. If either fails, NEITHER deals damage. This is an all-or-nothing check.
- **CR 701.14c**: Self-fight = 2x power damage to self. Edge case, but must be handled.
- **CR 701.14d**: Fight damage is non-combat. This means it does NOT count as commander damage, but deathtouch/lifelink/infect DO apply (they apply to all damage, not just combat damage).
- **Damage source**: The creature is the source of the damage, not the spell. This matters for deathtouch, lifelink, infect, and protection checks.

## Engine Changes

### Change 1: Add `Effect::Fight` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `Meld` (line ~1435), before the closing brace of `enum Effect`
**Pattern**: Follow `DealDamage` pattern but with two creature targets

```
/// CR 701.14a: Two creatures fight each other. Each deals damage equal to
/// its power to the other creature.
///
/// CR 701.14b: If either creature is no longer on the battlefield or is no
/// longer a creature at resolution, neither deals damage.
/// CR 701.14c: If a creature fights itself, it deals damage to itself equal
/// to twice its power.
/// CR 701.14d: Fight damage is non-combat damage.
Fight {
    /// The first creature (typically "target creature you control").
    attacker: EffectTarget,
    /// The second creature (typically "target creature you don't control").
    defender: EffectTarget,
},
```

### Change 2: Add `Effect::Bite` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new variant after `Fight`, before the closing brace of `enum Effect`

```
/// One-sided power-based damage: the source creature deals damage equal to
/// its power to the target creature. Only the source deals damage; the
/// target does not deal damage back.
///
/// CR 701.14b (by analogy): If the source creature is no longer on the
/// battlefield or is no longer a creature, no damage is dealt.
/// CR 701.14d: This damage is non-combat damage.
///
/// The source creature is the damage source (relevant for deathtouch,
/// lifelink, infect — see Warstorm Surge ruling 2011-09-22).
Bite {
    /// The creature dealing damage (its power determines the amount).
    source: EffectTarget,
    /// The creature receiving damage.
    target: EffectTarget,
},
```

### Change 3: Dispatch `Effect::Fight` in `execute_effect_inner`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm before the closing `}` of `execute_effect_inner` (line ~3452)
**CR**: 701.14a-d

Implementation logic:
1. Resolve both `attacker` and `defender` to ObjectIds
2. Check both are on the battlefield and are creatures (CR 701.14b) — if either fails, return (no damage)
3. Get power of each creature via `calculate_characteristics()` (layer-aware)
4. Handle self-fight (CR 701.14c): if same ObjectId, deal 2x power to self
5. Deal damage from attacker to defender (attacker's power) — use same damage pipeline as DealDamage (doubling, prevention, infect, deathtouch, lifelink) but with the CREATURE as source
6. Deal damage from defender to attacker (defender's power) — same pipeline
7. Emit `GameEvent::DamageDealt` for each direction with the creature as source

**Critical**: The damage source must be the creature, NOT `ctx.source` (which is the spell). Create a temporary `EffectContext` or pass the creature ObjectId directly to the damage helpers. Look at how `DealDamage` resolves at line 184 — it uses `ctx.source` as the damage source. For Fight/Bite, override this with the creature's ObjectId.

### Change 4: Dispatch `Effect::Bite` in `execute_effect_inner`

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm after `Effect::Fight`
**CR**: 701.14b, 701.14d (by analogy)

Implementation logic:
1. Resolve `source` and `target` to ObjectIds
2. Check source is on battlefield and is a creature (CR 701.14b analog) — if not, return
3. Check target is on battlefield — if not, return
4. Get power of source creature via `calculate_characteristics()` (layer-aware)
5. Clamp power to 0 if negative
6. Deal damage from source creature to target — creature is the damage source (not the spell)
7. Emit `GameEvent::DamageDealt`

### Change 5: Hash discriminants for `Effect::Fight` and `Effect::Bite`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms in `impl HashInto for Effect` (after discriminant 57 at line ~4478)

| Variant | Discriminant | Fields to hash |
|---------|-------------|----------------|
| `Effect::Fight` | 58 | `attacker`, `defender` |
| `Effect::Bite` | 59 | `source`, `target` |

**Pattern**: Follow `Effect::AttachEquipment` hash at line ~4398 (two EffectTarget fields)

### Change 6: No other exhaustive match sites

The `Effect` enum is only exhaustively matched in two files:
- `effects/mod.rs` — dispatch (Change 3 & 4)
- `state/hash.rs` — hashing (Change 5)

No changes needed in:
- `tools/replay-viewer/src/view_model.rs` — does not match on Effect
- `tools/tui/src/play/panels/stack_view.rs` — does not match on Effect
- `crates/engine/src/rules/layers.rs` — does not match on Effect
- `crates/engine/src/rules/copy.rs` — does not match on Effect

## Fight/Bite Damage Helper

To avoid code duplication between Fight and Bite, extract a helper function:

```rust
/// Deal non-combat damage from one creature to another, using the creature as the damage source.
/// CR 701.14d: Fight damage is non-combat damage.
/// The creature_source ObjectId is used as the damage source for deathtouch/lifelink/infect checks.
fn deal_creature_power_damage(
    state: &mut GameState,
    creature_source: ObjectId,
    target_creature: ObjectId,
    power: i32,
    events: &mut Vec<GameEvent>,
)
```

This function:
1. Clamps power to 0 (negative power deals 0 damage)
2. Applies damage doubling (`apply_damage_doubling` with creature as source)
3. Applies damage prevention (`apply_damage_prevention` with creature as source)
4. Checks infect on the creature source (not the spell)
5. Checks deathtouch/lifelink on the creature source
6. Applies damage to the target creature (toughness damage marking)
7. Emits `GameEvent::DamageDealt { source: creature_source, target: Object(target_creature), amount }`

Place this helper near the existing damage code in `effects/mod.rs` (after `execute_effect_inner`, before `resolve_amount`).

## Card Definition Fixes

### brash_taunter.rs

**Oracle text**: "Indestructible\nWhenever this creature is dealt damage, it deals that much damage to target opponent.\n{2}{R}, {T}: This creature fights another target creature."
**Current state**: Two TODOs — damage reflection trigger and fight activated ability
**Fix**: Add the fight activated ability using `Effect::Fight`. The damage-reflection trigger ("whenever dealt damage, deal that much to target opponent") remains a DSL gap (requires a dynamic damage amount from the triggering event) — keep that TODO but note it is NOT a Fight/Bite gap.

```rust
AbilityDefinition::Activated {
    cost: Cost::Mana(ManaCost { generic: 2, red: 1, ..Default::default() }),
    // {T} cost enforced by tap_required on the card (tap_self: true)
    effect: Effect::Fight {
        attacker: EffectTarget::Source,
        defender: EffectTarget::DeclaredTarget { index: 0 },
    },
    timing_restriction: None,
    targets: vec![TargetRequirement::TargetCreature],
},
```

Note: Brash Taunter says "another target creature" — the target must not be the source. This is enforced by standard target legality (you can't target the same object that is the source of an activated ability as "another"). No special filter needed.

Also note: The activated ability has a tap cost. Check whether the existing card def or the `AbilityDefinition::Activated` has a mechanism for tap cost. If not, document as TODO. The runner should check `Cost::Tap` or similar.

### bridgeworks_battle.rs

**Oracle text**: "Target creature you control gets +2/+2 until end of turn. It fights up to one target creature you don't control."
**Current state**: Empty abilities vec
**Fix**: This is a sorcery with two effects — pump then fight. The "up to one" targeting means the second target is optional (you can cast with only the first target). This is expressible:

```rust
abilities: vec![
    AbilityDefinition::Spell {
        effect: Effect::Sequence(vec![
            Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    modifications: vec![ContinuousModification::PowerToughnessBoost(2, 2)],
                    filter: EffectFilter::DeclaredTarget { index: 0 },
                    duration: EffectDuration::UntilEndOfTurn,
                }),
            },
            // "fights up to one target creature you don't control"
            // If no second target, fight does nothing (CR 701.14b)
            Effect::Fight {
                attacker: EffectTarget::DeclaredTarget { index: 0 },
                defender: EffectTarget::DeclaredTarget { index: 1 },
            },
        ]),
        targets: vec![
            TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::You,
                ..Default::default()
            }),
            // "up to one target creature you don't control" — TODO: "up to one" is optional
            // targeting, which may not be fully supported. Use standard targeting for now.
            TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::Opponent,
                ..Default::default()
            }),
        ],
    },
],
```

Note: "up to one" means the second target is optional. The DSL may not support optional targets. If `TargetRequirement` has no optional variant, the runner should document this as a minor limitation but proceed with mandatory targeting (the card can still be cast if there's a legal target for both).

Also note: Bridgeworks Battle is a MDFC (Modal Double-Faced Card) with a land back face. The runner should check if the card def handles the back face correctly. The existing card def only has the sorcery front face.

### ram_through.rs

**Oracle text**: "Target creature you control deals damage equal to its power to target creature you don't control. If the creature you control has trample, excess damage is dealt to that creature's controller instead."
**Current state**: Empty abilities, TODO about targeting and trample overflow
**Fix**: Use `Effect::Bite` for the base effect. The trample overflow clause is a conditional bonus that requires checking the source for trample and computing excess damage — this is an edge case that can be noted as TODO but the core Bite is implementable:

```rust
abilities: vec![
    AbilityDefinition::Spell {
        effect: Effect::Bite {
            source: EffectTarget::DeclaredTarget { index: 0 },
            target: EffectTarget::DeclaredTarget { index: 1 },
        },
        targets: vec![
            TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::You,
                ..Default::default()
            }),
            TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::Opponent,
                ..Default::default()
            }),
        ],
    },
],
```

Note: The trample excess-damage clause ("If the creature you control has trample, excess damage is dealt to that creature's controller instead") is NOT part of the Fight/Bite primitive. It's a unique Ram Through-specific behavior. Keep a TODO for it.

### frontier_siege.rs

**Oracle text**: "As this enchantment enters, choose Khans or Dragons. Khans — At the beginning of each of your main phases, add {G}{G}. Dragons — Whenever a creature you control with flying enters, you may have it fight target creature you don't control."
**Current state**: Empty abilities, TODO about modal ETB choice
**Fix**: The modal ETB choice ("choose Khans or Dragons") is a separate DSL gap that blocks full implementation. However, the Dragons mode's fight ability can now reference `Effect::Fight`. The runner should update the TODO to note that Fight is now available but the blocking gap is the modal ETB choice, not the fight effect.

### eomer_king_of_rohan.rs

**Oracle text**: "...Eomer deals damage equal to its power to any target."
**Current state**: Already implemented using `Effect::DealDamage { target, amount: EffectAmount::PowerOf(EffectTarget::Source) }`
**Fix**: None needed. This card uses "deals damage equal to its power to any target" which is a bite-to-any-target pattern. The existing `DealDamage + PowerOf` works correctly because the damage source is the spell (Eomer's ETB trigger), and `ctx.source` correctly points to Eomer. No change needed.

## Cards NOT Fixed by This Primitive

These cards mention fight/bite-like patterns but have OTHER blocking DSL gaps:

| Card | Pattern | Blocking Gap |
|------|---------|-------------|
| terror_of_the_peaks.rs | "deals damage equal to that creature's power" | Entering creature's power as EffectAmount (not self power) |
| jagged_scar_archers.rs | "deals damage equal to its power to target creature with flying" | CDA (*/\* = Elf count), TargetFilter for flying creatures |
| legolasquick_reflexes.rs | "deals damage equal to its power to up to one target creature" | Temporary triggered ability grant |
| legolass_quick_reflexes.rs | (duplicate card def) | Same as above |

These are authoring-phase issues, not primitive gaps. The Fight/Bite primitive does not unblock them.

## New Card Definitions

No new card definitions are required for PB-21. The sample cards mentioned in the PB spec (Infectious Bite, Warstorm Surge, Archdruid's Charm) do not have existing card defs. They will be authored during Phase 2 bulk authoring when all their prerequisites are met.

## Unit Tests

**File**: `crates/engine/tests/fight_bite.rs` (new file)
**Tests to write**:

- `test_fight_basic` — Two creatures fight. Each takes damage equal to the other's power. CR 701.14a.
- `test_fight_one_dies` — A 5/5 fights a 2/2. The 2/2 dies, the 5/5 survives with 2 damage. CR 701.14a + SBA 704.5g.
- `test_fight_both_die` — A 3/3 fights a 3/3. Both die. CR 701.14a + SBA 704.5g.
- `test_fight_creature_left_battlefield` — One creature is bounced before fight resolves. Neither deals damage. CR 701.14b.
- `test_fight_target_not_creature` — One target stops being a creature before fight resolves (e.g., was animated land, animation ended). Neither deals damage. CR 701.14b.
- `test_fight_self` — A creature fights itself. Takes 2x its power in damage. CR 701.14c.
- `test_fight_not_combat_damage` — Fight damage does not count as combat damage (does not trigger "whenever this creature deals combat damage" abilities). CR 701.14d.
- `test_fight_deathtouch` — A 1/1 deathtouch fights a 5/5. The 5/5 dies (deathtouch applies to non-combat damage). The 1/1 takes 5 damage and dies.
- `test_fight_lifelink` — A creature with lifelink fights. The lifelink creature's controller gains life equal to the damage dealt.
- `test_bite_basic` — Creature A bites creature B. Only A's power as damage is dealt to B. B does NOT deal damage to A. CR 701.14d (non-combat).
- `test_bite_source_not_on_battlefield` — Source creature is gone at resolution. No damage dealt. CR 701.14b analog.
- `test_bite_zero_power` — Source creature has 0 power. No damage dealt.
- `test_bite_negative_power` — Source creature has negative power (e.g., after -X/-0 effect). No damage dealt (clamped to 0).

**Pattern**: Follow tests in `crates/engine/tests/card_def_fixes.rs` for card-based testing patterns. Use `GameStateBuilder` with two creatures on the battlefield, cast a spell that triggers Fight/Bite.

## Implementation Order

1. Add `Effect::Fight` and `Effect::Bite` variants to `enum Effect` in `card_definition.rs`
2. Add `deal_creature_power_damage` helper in `effects/mod.rs`
3. Add `Effect::Fight` dispatch arm in `execute_effect_inner`
4. Add `Effect::Bite` dispatch arm in `execute_effect_inner`
5. Add hash discriminants (58, 59) in `state/hash.rs`
6. `cargo check` — verify compilation
7. Fix card defs: `brash_taunter.rs`, `bridgeworks_battle.rs`, `ram_through.rs`, update `frontier_siege.rs` TODO
8. Write unit tests in `tests/fight_bite.rs`
9. `cargo test --all` — verify all tests pass
10. `cargo clippy -- -D warnings` — verify no warnings
11. `cargo build --workspace` — verify full workspace builds (replay-viewer, TUI, etc.)

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (brash_taunter fight ability, bridgeworks_battle, ram_through)
- [ ] frontier_siege TODO updated (fight is available, modal ETB remains blocking gap)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining fight/bite-specific TODOs in affected card defs

## Risks & Edge Cases

- **Damage source identity**: The creature must be the damage source for Fight/Bite, not `ctx.source` (the spell). This is critical for deathtouch, lifelink, infect, and protection. The helper function must create appropriate damage parameters with the creature's ObjectId.
- **Infect interaction**: If the fight source has infect and the target is a creature, damage is dealt as -1/-1 counters (CR 702.90a). If the target is a player (not applicable to Fight but relevant if Bite is later extended), infect gives poison counters instead.
- **Damage doubling**: Damage-doubling effects (e.g., Fiery Emancipation) apply per-source, so Fight generates two separate damage events that each go through the doubling pipeline independently.
- **Simultaneous damage**: CR 701.14a says creatures deal damage to each other. This is simultaneous (both creatures see each other's power BEFORE any damage is applied). Implementation must read both powers before applying either damage.
- **Last Known Information**: If a creature is removed in response, CR 701.14b says no damage at all. LKI is not needed here because we simply check if the creature exists at resolution time.
- **"Up to one" targeting**: Bridgeworks Battle uses "up to one target creature you don't control." The DSL may not have an optional target mechanism. If the second target is missing at resolution, the fight simply does nothing (CR 701.14b handles this naturally).
- **Tap cost on activated fight (Brash Taunter)**: The runner needs to verify that `Cost::TapSelf` or equivalent exists and is wired into the activated ability. Check `Cost` enum for tap-self support.
- **Ram Through trample overflow**: This is an edge case specific to Ram Through and is NOT part of the Fight/Bite primitive. It should remain as a TODO.
