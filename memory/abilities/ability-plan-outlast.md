# Ability Plan: Outlast

**Generated**: 2026-03-06
**CR**: 702.107
**Priority**: P4
**Similar abilities studied**: Adapt (KeywordAbility::Adapt in `types.rs`, card def in `defs/sharktocrab.rs` using `AbilityDefinition::Activated`), Scavenge (full dedicated Command/SOK pipeline in `abilities.rs`, `engine.rs`, `command.rs`, `resolution.rs` -- overkill for Outlast)

## CR Rule Text

702.107. Outlast

702.107a Outlast is an activated ability. "Outlast [cost]" means "[Cost], {T}: Put a +1/+1 counter on this creature. Activate only as a sorcery."

## Key Edge Cases

- **Summoning sickness (ruling 2014-09-20)**: The cost includes {T}, so a creature with summoning sickness cannot use Outlast unless it has haste. The existing `handle_activate_ability` already checks this (line ~192 of `abilities.rs`).
- **Sorcery speed**: "Activate only as a sorcery" means active player only, main phase, stack empty. The existing `sorcery_speed: true` on `ActivatedAbility` already enforces this (line ~100 of `abilities.rs`).
- **Self-targeting**: The counter goes on "this creature" -- no targeting another creature. `EffectTarget::Source` handles this.
- **+1/+1 counter interaction**: Outlast creatures in Khans of Tarkir typically also have a static ability that grants something to creatures with +1/+1 counters. The static ability is a separate `AbilityDefinition::Static` -- NOT part of Outlast itself. The Outlast implementation only handles the "+1/+1 counter on self" part.
- **Multiplayer**: No special multiplayer considerations. Standard sorcery-speed restriction applies.
- **No targeting means no fizzle**: Since the effect targets Source (self), there is no target validation at resolution. The creature just gets the counter. If the creature left the battlefield between activation and resolution, the ability fizzles naturally because its source is gone (handled by existing ActivatedAbility resolution in `resolution.rs`).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Outlast is a pure activated ability, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Outlast` variant after `Scavenge` (line ~1112).
**Discriminant**: KW 121.
**Doc comment**: `/// CR 702.107: Outlast [cost] -- activated ability on the battlefield. / "[Cost], {T}: Put a +1/+1 counter on this creature. Activate only as a sorcery." / Discriminant 121.`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Outlast { cost: ManaCost }` variant after `Scavenge` (line ~525).
**Discriminant**: AbilDef 48.
**Doc comment**: `/// CR 702.107: Outlast [cost]. A convenience variant that enrich_spec_from_def / expands into an ActivatedAbility with: requires_tap=true, mana_cost=cost, / sorcery_speed=true, effect=AddCounter(Source, PlusOnePlusOne, 1). / Cards should also include AbilityDefinition::Keyword(KeywordAbility::Outlast) / for quick presence-checking. Discriminant 48.`

**No new StackObjectKind needed** -- Outlast uses the existing `Command::ActivateAbility` and `StackObjectKind::ActivatedAbility` infrastructure because it activates from the battlefield as a standard activated ability.

### Step 2: Hash Updates

**File**: `crates/engine/src/state/hash.rs`

**Action 1**: Add `KeywordAbility::Outlast` arm in the `HashInto for KeywordAbility` impl (after line ~611):
```rust
// Outlast (discriminant 121) -- CR 702.107
KeywordAbility::Outlast => 121u8.hash_into(hasher),
```

**Action 2**: Add `AbilityDefinition::Outlast` arm in the `HashInto for AbilityDefinition` impl (after line ~3624):
```rust
// Outlast (discriminant 48) -- CR 702.107
AbilityDefinition::Outlast { cost } => {
    48u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

### Step 3: Enrichment (Rule Enforcement via Existing Infrastructure)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, add an arm for `AbilityDefinition::Outlast { cost }` that expands it into an `ActivatedAbility`. Add this after the `AbilityDefinition::Activated` expansion loop (after line ~1708).

**Pattern**: Similar to how Sharktocrab's Adapt is expanded via `AbilityDefinition::Activated`, but using the dedicated `Outlast` variant for cleaner DSL.

**Expansion logic**:
```rust
// CR 702.107a: Expand Outlast into an ActivatedAbility.
// "Outlast [cost]" means "[Cost], {T}: Put a +1/+1 counter on this creature.
// Activate only as a sorcery."
if let AbilityDefinition::Outlast { cost } = ability {
    let ab = ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: Some(cost.clone()),
            sacrifice_self: false,
        },
        description: format!("Outlast (CR 702.107a)"),
        effect: Some(Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: EffectAmount::Fixed(1),
        }),
        sorcery_speed: true,
    };
    spec = spec.with_activated_ability(ab);
}
```

**CR**: 702.107a -- the expansion faithfully translates the CR text: cost includes tap + mana, effect is "+1/+1 counter on this creature", sorcery speed.

**No new Command handler needed**: The existing `Command::ActivateAbility` handler in `engine.rs` (line ~142) dispatches to `handle_activate_ability` in `abilities.rs`, which:
1. Checks priority (CR 602.2)
2. Checks split second (CR 702.61a)
3. Checks source on battlefield (CR 602.2)
4. Checks sorcery speed (CR 602.5d) -- because `sorcery_speed: true`
5. Pays tap cost with summoning sickness check (CR 302.6 / 702.10)
6. Pays mana cost (CR 602.2a)
7. Pushes ActivatedAbility onto the stack
8. At resolution, executes `Effect::AddCounter` on the source

All of these are exactly what Outlast needs. No custom handler required.

### Step 4: Match Arm Updates

**Check for exhaustive matches on `KeywordAbility`**: Grep for `match.*keyword` or `KeywordAbility::` in resolution.rs, layers.rs, combat.rs, etc. The `KeywordAbility` enum likely has a wildcard arm in most match expressions, but verify. If any match is exhaustive, add the `Outlast` arm.

**Check for exhaustive matches on `AbilityDefinition`**: Grep for match expressions on `AbilityDefinition`. The hash.rs match is already covered in Step 2. Check `enrich_spec_from_def` and any other sites.

### Step 5: Unit Tests

**File**: `crates/engine/tests/outlast.rs`
**Tests to write**:

1. `test_outlast_basic_adds_counter` -- CR 702.107a: Creature with Outlast on the battlefield, player pays mana + taps, ability resolves, creature gets 1 +1/+1 counter. Uses `Command::ActivateAbility` with `ability_index: 0`.

2. `test_outlast_sorcery_speed_restriction` -- CR 702.107a: Cannot activate during non-main phase, with non-empty stack, or as non-active player.

3. `test_outlast_summoning_sickness` -- Ruling 2014-09-20 / CR 302.6: Creature with summoning sickness cannot activate Outlast (requires {T}). Creature with haste CAN activate it.

4. `test_outlast_requires_mana` -- CR 602.2b: Insufficient mana returns error.

5. `test_outlast_already_tapped` -- CR 602.2: Cannot activate if source is already tapped.

6. `test_outlast_stacks_counters` -- CR 702.107a: Activate Outlast twice (across turns), creature gets 2 cumulative +1/+1 counters.

7. `test_outlast_not_a_cast` -- Outlast is an activated ability, not a spell. `spells_cast_this_turn` unchanged, no `SpellCast` event.

**Pattern**: Follow `crates/engine/tests/scavenge.rs` for test structure (helper functions, builder setup, assertions). Key difference: source is on the battlefield (not graveyard), command is `ActivateAbility` (not `ScavengeCard`).

**Test setup pattern**:
- Card definition with `AbilityDefinition::Keyword(KeywordAbility::Outlast)` + `AbilityDefinition::Outlast { cost }`.
- `ObjectSpec::creature(owner, name, P, T).with_card_id(card_id).with_keyword(KeywordAbility::Outlast)` on battlefield.
- Must call `enrich_spec_from_def()` to populate `activated_abilities` from the `Outlast` AbilDef variant, OR manually add the `ActivatedAbility` via `.with_activated_ability(...)`.
- Give player mana, set priority, activate at `ability_index: 0`.

### Step 6: Card Definition (later phase)

**Suggested card**: Ainok Bond-Kin ({1}{W}, 2/1, Outlast {1}{W}, grants first strike to creatures with +1/+1 counters)
**Alternative simpler card**: Abzan Falconer ({2}{W}, 2/3, Outlast {W}, grants flying to creatures with +1/+1 counters)
**Card lookup**: use `card-definition-author` agent
**Note**: The static ability granting keywords to creatures with +1/+1 counters requires a `Condition::TargetHasCounters` or similar -- may need DSL support. If not available, author the Outlast part and add a TODO for the static grant.

### Step 7: Game Script (later phase)

**Suggested scenario**: Player casts a creature with Outlast, waits a turn (no summoning sickness), activates Outlast to add a +1/+1 counter, verifies counter is present.
**Subsystem directory**: `test-data/generated-scripts/abilities/`
**Script name**: `1XX_outlast_basic.json` (next available sequence number)

## Interactions to Watch

- **Summoning sickness**: Already handled by `handle_activate_ability` (checks `has_summoning_sickness` when `requires_tap: true`). No additional code needed.
- **Humility / ability removal**: If Outlast is removed from the creature (e.g., by Humility in Layer 6), the `activated_abilities` list on the `GameObject` will be cleared by the layer system. The creature cannot activate Outlast. This is correct behavior and handled by existing infrastructure.
- **Stifle / counter**: If the activated ability is countered (e.g., by Stifle), the creature is already tapped (tap was paid as cost) and mana is spent, but no counter is placed. This is correct -- the existing `StackObjectKind::ActivatedAbility` counter-handling path in `resolution.rs` simply removes the ability from the stack.
- **Proliferate**: After Outlast adds a +1/+1 counter, Proliferate can add more. No special interaction -- standard counter mechanics.
- **Hardened Scales / Doubling Season**: These are replacement effects on "putting counters" and apply to the `AddCounter` effect. Already handled by existing infrastructure if those replacement effects are implemented.

## Key Design Decision: No New Command/SOK

Unlike Scavenge (which needed `Command::ScavengeCard` + `StackObjectKind::ScavengeAbility` because it activates from the graveyard with exile-as-cost and power-snapshot targeting), Outlast is a vanilla activated ability that:
1. Activates from the battlefield (standard `ActivateAbility`)
2. Pays mana + tap (standard `ActivationCost`)
3. Has sorcery-speed restriction (standard `sorcery_speed: true`)
4. Puts a +1/+1 counter on self (standard `Effect::AddCounter`)

All infrastructure exists. The implementation is:
- `KeywordAbility::Outlast` for keyword presence/display
- `AbilityDefinition::Outlast { cost }` for clean DSL in card definitions
- Expansion in `enrich_spec_from_def` to wire into the existing activated ability pipeline
- Hash entries for both new variants
- Unit tests verifying the integration
