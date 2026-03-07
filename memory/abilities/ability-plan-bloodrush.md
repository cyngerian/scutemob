# Ability Plan: Bloodrush

**Generated**: 2026-03-07
**CR**: 207.2c (ability word -- no individual CR entry; underlying mechanics are activated ability rules CR 602)
**Priority**: P4
**Batch**: 12
**Similar abilities studied**: Cycling (CR 702.29 -- discard-from-hand activated ability), Forecast (CR 702.57 -- hand-activated ability with targets and stack object)

## CR Rule Text

CR 207.2c: "An ability word appears in italics at the beginning of some abilities. Ability words are similar to keywords in that they tie together cards that have similar functionality, but they have no special rules meaning and no individual entries in the Comprehensive Rules."

Bloodrush is listed among the ability words. It has NO special rules meaning -- it is simply a label for a pattern of activated abilities that share the template:

> "{cost}, Discard this card: Target attacking creature gets +N/+N [and gains {keyword}] until end of turn."

The underlying rules are:
- **CR 602.2**: Activating an ability puts it on the stack and pays its costs.
- **CR 602.2a**: If activated from a hidden zone (hand), the card is revealed.
- **CR 602.2b**: The remainder follows CR 601.2b-i (same as casting a spell).
- **CR 115**: Target validation -- "target attacking creature" restricts to creatures currently registered as attackers in combat state.

## Key Edge Cases

1. **No "activate only as an instant" restriction.** The oracle text does NOT include any timing restriction. Bloodrush is activated at instant speed (any time the player has priority). The practical limitation is the targeting requirement -- "target attacking creature" means there must be a creature in the `combat.attackers` map.
2. **Discard is part of the cost (CR 602.2b).** The card is discarded immediately when the ability is activated, before the ability goes on the stack. If the ability is countered (Stifle), the card remains in the graveyard -- it was already consumed as cost.
3. **Targeting restriction: attacking creature.** The target must be a creature currently registered as an attacker. This check must happen at activation time (target validation) AND at resolution time (CR 608.2b -- target legality recheck). If the creature is no longer attacking at resolution (e.g., combat ended, creature removed from combat), the ability fizzles.
4. **Pump + keyword grant until end of turn.** The effect registers continuous effects with `UntilEndOfTurn` duration: (a) `PtModify` layer with `ModifyBoth(N)`, and (b) `Ability` layer with `GainKeyword(keyword)` if the bloodrush grants a keyword.
5. **Can target any attacking creature, not just your own.** The oracle text says "target attacking creature" with no controller restriction. In multiplayer, you could bloodrush an opponent's attacking creature (unusual but legal).
6. **Madness interaction.** If the bloodrush card has madness, the discard-as-cost should route through the madness exile path (same pattern as Cycling -- check `KeywordAbility::Madness` on the object before moving to graveyard).
7. **Split second blocks bloodrush.** Bloodrush is an activated ability, not a mana ability. CR 702.61a prevents activation while split second is on the stack.
8. **Multiplayer: no special considerations.** Any player can activate bloodrush from their hand during any combat phase when they have priority, targeting any attacking creature on the battlefield.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (KeywordAbility::Bloodrush, AbilityDefinition::Bloodrush)
- [ ] Step 2: Rule enforcement (Command, handler, resolution)
- [ ] Step 3: Trigger wiring (n/a -- no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: KeywordAbility Variant + AbilityDefinition Variant

**1a. KeywordAbility enum variant**

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Bloodrush` variant after `Spree` (discriminant 134).
**Discriminant**: 135
**Pattern**: Follow `KeywordAbility::Forecast` at line ~1078 (static marker for quick presence-checking; cost/effect stored in AbilityDefinition).

```
/// CR 207.2c: Bloodrush -- ability word. Activated ability from hand:
/// "{cost}, Discard this card: Target attacking creature gets +N/+N
/// [and gains keyword] until end of turn."
/// Static marker for quick presence-checking (`keywords.contains`).
/// The bloodrush cost and effect are stored in `AbilityDefinition::Bloodrush`.
///
/// Discriminant 135.
Bloodrush,
```

**1b. AbilityDefinition variant**

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Bloodrush` variant after `Fuse` (discriminant 51).
**Discriminant**: 52
**Pattern**: Follow `AbilityDefinition::Forecast { cost, effect }` at card_definition.rs line ~516.

```
/// CR 207.2c: Bloodrush -- ability word. Activated ability from hand.
/// "{cost}, Discard this card: Target attacking creature gets +N/+N
/// [and gains {keyword}] until end of turn."
///
/// `cost`: mana cost of the bloodrush activation.
/// `power_boost`: the +N to power.
/// `toughness_boost`: the +N to toughness.
/// `grants_keyword`: optional keyword granted until end of turn.
///
/// Discriminant 52.
Bloodrush {
    cost: ManaCost,
    power_boost: i32,
    toughness_boost: i32,
    grants_keyword: Option<KeywordAbility>,
},
```

**Design rationale**: Using dedicated fields (`power_boost`, `toughness_boost`, `grants_keyword`) rather than a generic `Effect` because every bloodrush card follows the exact same template. This avoids the complexity of building `Effect::Sequence` with `ApplyContinuousEffect` at definition time and makes the handler simpler. The handler constructs the continuous effects internally.

**1c. TargetFilter: must_be_attacking field**

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `pub must_be_attacking: bool` to `TargetFilter` struct (line ~1030).
**Default**: `false` (via `Default` derive -- must verify the derive or add manual default).

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In the `TargetCreatureWithFilter` arm of `validate_target_object` (~line 4457), after `passes_filter && passes_controller`, add a check:
```
if filter.must_be_attacking {
    let is_attacking = state.combat.as_ref()
        .map(|c| c.attackers.contains_key(id))
        .unwrap_or(false);
    if !is_attacking { return false; }
}
```
Note: `validate_target_object` currently does NOT take `state` as a parameter (it only takes the object). This check needs `state` access. Two approaches:
- **(A) Add `state` parameter to `validate_target_object`** -- larger refactor, affects all callers.
- **(B) Check `must_be_attacking` in the bloodrush handler at activation time, before calling `validate_targets`** -- keeps validation self-contained in the handler, similar to how Forecast checks upkeep timing.

**Recommendation: Approach (B).** The bloodrush handler validates the target is an attacking creature before pushing to the stack. At resolution time, the handler also re-validates. This keeps the generic target validation infrastructure unchanged. The `must_be_attacking` field on `TargetFilter` is NOT needed -- the handler itself enforces this constraint.

**Revised 1c**: No changes to `TargetFilter`. The "target attacking creature" restriction is enforced directly in the `handle_activate_bloodrush` function, not via `TargetFilter`.

**1d. StackObjectKind variant**

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `BloodrushAbility` variant after `RavenousDrawTrigger` (discriminant 50).
**Discriminant**: 51

```
/// CR 207.2c: Bloodrush activated ability on the stack.
///
/// The source card has been discarded (moved to graveyard as cost).
/// When this resolves, apply +N/+N and optionally grant a keyword
/// to the target attacking creature until end of turn.
///
/// Discriminant 51.
BloodrushAbility {
    source_object: ObjectId,
    target_creature: ObjectId,
    power_boost: i32,
    toughness_boost: i32,
    grants_keyword: Option<KeywordAbility>,
},
```

**1e. Hash updates**

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for:
- `KeywordAbility::Bloodrush` (discriminant 135u8)
- `AbilityDefinition::Bloodrush { cost, power_boost, toughness_boost, grants_keyword }` (discriminant 52u8)
- `StackObjectKind::BloodrushAbility { .. }` (discriminant 51u8)

**1f. Match arm additions**

Files that have exhaustive matches on `KeywordAbility` and/or `StackObjectKind`:
- `tools/replay-viewer/src/view_model.rs` -- add arms for BOTH `KeywordAbility::Bloodrush` and `StackObjectKind::BloodrushAbility`
- `tools/tui/src/play/panels/stack_view.rs` -- add arm for `StackObjectKind::BloodrushAbility`
- `crates/engine/src/state/builder.rs` -- add `Bloodrush` to `enrich_spec_from_def` if needed (add `KeywordAbility::Bloodrush` to keywords when `AbilityDefinition::Bloodrush` is present)
- `crates/engine/src/rules/resolution.rs` -- add resolution arm + countered arm for `BloodrushAbility`
- `crates/engine/src/lib.rs` -- export `Bloodrush` if needed

### Step 2: Command + Handler

**2a. Command variant**

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `ActivateBloodrush` command variant after `ActivateForecast` (~line 438).

```
/// CR 207.2c: Activate a bloodrush ability from hand.
///
/// Bloodrush is an activated ability that functions only while the card is in
/// the player's hand. The activation cost is the mana cost plus discarding the
/// card itself. The effect pumps a target attacking creature until end of turn.
///
/// Unlike `ActivateAbility` (which requires the source on the battlefield),
/// this command works from the hand zone. The card is discarded as cost.
ActivateBloodrush {
    player: PlayerId,
    card: ObjectId,
    target: ObjectId,
},
```

**2b. Engine dispatch**

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::ActivateBloodrush` arm in the main match, following the Cycling/Forecast pattern (~line 270):

```
Command::ActivateBloodrush { player, card, target } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_activate_bloodrush(
        &mut state, player, card, target,
    )?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

**2c. Handler function**

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `pub fn handle_activate_bloodrush(...)` after `handle_activate_forecast` (~line 900).

**Pattern**: Follow `handle_cycle_card` (lines 488-690) for the discard-as-cost pattern, and `handle_activate_forecast` (lines 771-900) for the targets + stack push pattern.

Handler logic:
1. Priority check (CR 602.2)
2. Split second check (CR 702.61a)
3. Zone check: card must be in `Hand(player)`
4. Keyword check: card must have `KeywordAbility::Bloodrush`
5. Target validation: `target` must be on battlefield, must be a creature, must be in `state.combat.attackers`
6. Look up bloodrush cost from `AbilityDefinition::Bloodrush` in `CardRegistry`
7. Pay mana cost
8. Discard self as cost (move card from hand to graveyard; handle Madness if present)
9. Emit `CardDiscarded` event
10. Push `StackObjectKind::BloodrushAbility` onto stack with target info
11. Emit `PermanentTargeted` for Ward trigger checks
12. Reset `players_passed`, emit `PriorityGiven` to active player

**2d. Resolution**

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::BloodrushAbility` arm in the main resolution match.

Resolution logic:
1. Validate target creature is still a legal target (on battlefield, still a creature, still attacking -- CR 608.2b). Note: "still attacking" is questionable -- by the time a stack item resolves, combat may have progressed. However, the target just needs to be a legal creature on the battlefield at resolution; the "attacking" part was the targeting restriction at activation time. At resolution, we only check that the target is still a creature on the battlefield (standard target legality recheck).
2. Register `ContinuousEffect` with `EffectFilter::SingleObject(target_creature)`, `EffectLayer::PtModify`, `LayerModification::ModifyBoth(power_boost)` (or separate Power/Toughness if different), `EffectDuration::UntilEndOfTurn`.
3. If `grants_keyword.is_some()`, register a second `ContinuousEffect` with `EffectLayer::Ability`, `LayerModification::GainKeyword(keyword)`, same filter and duration.
4. Emit `AbilityResolved` event.

**Important CR clarification on resolution target check**: At resolution (CR 608.2b), the game only checks that the target is still legal according to the targeting requirement. The targeting requirement is "target attacking creature." If the creature is no longer attacking (combat ended while bloodrush was on the stack), the target is no longer legal and the ability fizzles. This is the correct interpretation -- "attacking" is part of the target restriction, checked both at activation and resolution.

**2e. Countered arm**

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::BloodrushAbility { .. }` to the countered/fizzle match arm list (~line 5351+). Comment: "If countered (e.g. by Stifle), the bloodrush card is already in the graveyard (discarded as cost). No pump or keyword is applied."

### Step 3: Trigger Wiring

**N/A.** Bloodrush has no triggered abilities. It is a purely activated ability. No `PendingTriggerKind` variant needed.

### Step 4: Replay Harness Action

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"activate_bloodrush"` action type in `translate_player_action()`.

```
"activate_bloodrush" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    let target_id = target_list.first()
        .and_then(|t| match t { Target::Object(id) => Some(*id), _ => None })
        .ok_or_else(|| /* error */)?;
    Some(Command::ActivateBloodrush {
        player,
        card: card_id,
        target: target_id,
    })
}
```

### Step 5: Unit Tests

**File**: `crates/engine/tests/bloodrush.rs`
**Tests to write**:

1. `test_bloodrush_basic_pump` -- Activate bloodrush from hand during combat, target an attacking creature. Verify: card discarded from hand to graveyard, ability on stack, after resolution creature has +N/+N until end of turn. **CR 207.2c, CR 602.2**
2. `test_bloodrush_grants_keyword` -- Activate bloodrush that also grants trample (Ghor-Clan Rampager pattern). Verify creature gains trample until end of turn. **CR 207.2c**
3. `test_bloodrush_target_must_be_attacking` -- Attempt to activate bloodrush targeting a non-attacking creature. Verify error returned. **CR 115**
4. `test_bloodrush_no_combat_fails` -- Attempt to activate bloodrush when there is no combat (no attackers declared). Verify error returned. **CR 115**
5. `test_bloodrush_not_in_hand_fails` -- Attempt to activate bloodrush on a card on the battlefield. Verify error returned. **CR 602.2a (hand zone requirement)**
6. `test_bloodrush_card_discarded_as_cost` -- Verify the card goes to graveyard immediately (before ability resolves). **CR 602.2b**
7. `test_bloodrush_countered_no_pump` -- Activate bloodrush, then counter it (Stifle). Verify: card remains in graveyard, target creature does NOT get pump. **CR 602.2, abilities on stack can be countered**
8. `test_bloodrush_pump_expires_end_of_turn` -- Verify the +N/+N expires at end of turn cleanup. **CR 514.2**
9. `test_bloodrush_insufficient_mana_fails` -- Attempt bloodrush without enough mana. Verify error. **CR 602.2b**
10. `test_bloodrush_split_second_blocks` -- Split second on stack, attempt bloodrush. Verify error. **CR 702.61a**

**Pattern**: Follow `crates/engine/tests/forecast.rs` structure (helpers, find_object, pass_all, card definition setup).

### Step 6: Card Definition

**Suggested card**: Ghor-Clan Rampager
**Oracle**: `Trample / Bloodrush -- {R}{G}, Discard this card: Target attacking creature gets +4/+4 and gains trample until end of turn.`
**File**: `crates/engine/src/cards/defs/ghor_clan_rampager.rs`
**Use**: `card-definition-author` agent

Key definition fields:
- `mana_cost: ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }`
- `card_types: vec![CardType::Creature]`
- `subtypes: vec![SubType("Beast")]`
- `power: Some(4), toughness: Some(4)`
- `keywords: ordset![KeywordAbility::Trample, KeywordAbility::Bloodrush]`
- `abilities: vec![AbilityDefinition::Bloodrush { cost: ManaCost { red: 1, green: 1, ..Default::default() }, power_boost: 4, toughness_boost: 4, grants_keyword: Some(KeywordAbility::Trample) }]`

### Step 7: Game Script

**Suggested scenario**: Player casts creature, enters combat, declares it as attacker, activates Ghor-Clan Rampager's bloodrush from hand targeting the attacker. Stack resolves, attacker gets +4/+4 and trample. Then damage step to verify pump applied.

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Script name**: `174_bloodrush_ghor_clan_rampager.json` (next after script 173)

### Step 8: Coverage Doc Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Bloodrush row from `none` to `validated`. Add file references.

## Interactions to Watch

- **Madness**: If a bloodrush card also has madness (no current cards do, but could via continuous effects), the discard-as-cost should trigger the madness exile-instead-of-graveyard path. Follow the Cycling madness handler pattern (abilities.rs ~line 562-643).
- **Ward**: The bloodrush ability targets a creature on the battlefield. If that creature has Ward, the `PermanentTargeted` event should trigger Ward. The handler must emit `PermanentTargeted` event (same as `handle_activate_ability`).
- **Combat state at resolution**: At resolution time, if combat has ended (creature is no longer in `combat.attackers`), the target is no longer legal and the ability fizzles. The resolution arm must check this.
- **Stifle/counterspell on abilities**: The card is already discarded (in graveyard) when the ability is countered. No recovery.
- **Protection from red/green**: If the target creature gains protection from the bloodrush card's colors after activation but before resolution, the ability fizzles (standard targeting legality recheck).

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Bloodrush | 135 |
| AbilityDefinition | Bloodrush | 52 |
| StackObjectKind | BloodrushAbility | 51 |
