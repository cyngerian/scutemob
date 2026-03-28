# Primitive Batch Plan: PB-35 -- Modal Triggers + Graveyard Conditions + Planeswalker Abilities

**Generated**: 2026-03-28
**Primitive**: Three gap closures: (1) modal triggered abilities, (2) graveyard-zone activated/triggered abilities, (3) planeswalker loyalty ability fixes
**CR Rules**: 700.2, 700.2b, 603.3c, 602.2, 606
**Cards affected**: ~45 (25 modal trigger fixes + 12 graveyard recursion fixes + 8 PW fixes)
**Dependencies**: PB-14 (planeswalker framework), PB-26 (trigger variants), PB-31 (Cost::RemoveCounter)
**Deferred items from prior PBs**: G-24 (Nykthos, Three Tree City -- requires Command::ChooseColor, M10); G-25 partial (Springleaf Drum, Cryptolith Rite, Faeburrow Elder, Arena of Glory -- PB-37)

## Primitive Specification

### G-27: Modal Triggered Abilities

**Problem**: `AbilityDefinition::Triggered` has no `modes` field. Cards with "whenever X, choose one" cannot express mode selection. Currently, some cards use `Effect::Choose` inline (which auto-picks mode 0 -- wrong game state for cards where modes have different effects/targets), and others have empty abilities or partial implementations.

**Solution**: Add `modes: Option<ModeSelection>` field to `AbilityDefinition::Triggered`. When a modal triggered ability fires and is put on the stack (via `flush_pending_triggers`), the engine selects modes deterministically (bot fallback: mode 0). At resolution time, the chosen modes' effects execute instead of the main `effect` field.

**Key CR difference from modal spells**: CR 700.2b says modal triggered abilities choose modes when put on the stack (not at trigger fire time). CR 603.3c: if no mode can be chosen (no legal targets for any mode), the ability is removed from the stack.

### G-29: Graveyard-Zone Activated/Triggered Abilities

**Problem**: `handle_activate_ability` requires the source to be on the battlefield (or in hand for Channel). Cards like Reassembling Skeleton, Cult Conscript, Earthquake Dragon, Bloodghast, and Nether Traitor have abilities that activate or trigger from the graveyard zone. The DSL has no way to express "this ability works from the graveyard."

**Solution**: Add `activation_zone: Option<ActivationZone>` field to `AbilityDefinition::Activated`. Default `None` means battlefield (current behavior). `Some(ActivationZone::Graveyard)` means the ability can only be activated when the source is in the owner's graveyard. Extend `handle_activate_ability` to check this field and allow activation from the graveyard when set.

For triggered abilities from the graveyard (Bloodghast, Nether Traitor), add `trigger_zone: Option<TriggerZone>` to `AbilityDefinition::Triggered`. Default `None` means battlefield. `Some(TriggerZone::Graveyard)` means the trigger monitors events while the source is in the graveyard.

### G-30: Planeswalker Loyalty Ability Fixes

**Problem**: Many planeswalker card defs have TODOs on individual loyalty abilities. Most are NOT PW-framework issues -- they are general DSL gaps (emblems, delayed triggers, "any color" mana, etc.) that are out of scope for PB-35. Only PW-specific issues (loyalty ability structure, modes on loyalty abilities) are in scope.

**Scope reduction**: After analysis, very few PW TODOs are actually PW-framework-specific. Most are waiting on other DSL gaps. The PW cards fixable in this batch are those where the TODO is only about wiring existing DSL primitives that have since been implemented (e.g., Effect::Choose now exists for modal activated abilities on Goblin Cratermaker, Umezawa's Jitte). This is a card-fix sweep, not an engine change.

## CR Rule Text

### CR 700.2 (Modal spells and abilities)
> 700.2. A spell or ability is modal if it has two or more options in a bulleted list preceded by instructions for a player to choose a number of those options, such as "Choose one --." Each of those options is a mode.

### CR 700.2b (Modal triggered abilities)
> 700.2b. The controller of a modal triggered ability chooses the mode(s) as part of putting that ability on the stack. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. If no mode is chosen, the ability is removed from the stack. (See rule 603.3c.)

### CR 603.3c (Modal trigger stack placement)
> 603.3c. If a triggered ability is modal, its controller announces the mode choice when putting the ability on the stack. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. If no mode is chosen, the ability is removed from the stack. (See rule 700.2.)

### CR 602.2 (Activated ability activation)
> 602.2. To activate an ability is to put it onto the stack and pay its costs...

## Engine Changes

### Change 1: Add `modes` field to `AbilityDefinition::Triggered`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `modes: Option<ModeSelection>` field to `AbilityDefinition::Triggered` variant (after `targets`).
**Pattern**: Follow `AbilityDefinition::Spell { modes: Option<ModeSelection>, .. }` at line ~221.
**CR**: 700.2b -- modal triggered abilities.

```
Triggered {
    trigger_condition: TriggerCondition,
    effect: Effect,
    intervening_if: Option<Condition>,
    #[serde(default)]
    targets: Vec<TargetRequirement>,
    /// CR 700.2b: Modal triggered ability. When Some, the controller chooses modes
    /// when the trigger is put on the stack. The chosen modes' effects replace the
    /// main `effect` field at resolution.
    #[serde(default)]
    modes: Option<ModeSelection>,
},
```

### Change 2: Add `ActivationZone` enum and field on `AbilityDefinition::Activated`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a new `ActivationZone` enum (Battlefield, Graveyard) and add `activation_zone: Option<ActivationZone>` to `AbilityDefinition::Activated`.
**CR**: 602.2 -- abilities from non-battlefield zones.

```
/// Zone from which an activated ability can be activated.
/// Default (None) = battlefield only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationZone {
    /// Ability can only be activated while the source is in its owner's graveyard.
    Graveyard,
}
```

Add to `Activated` variant:
```
/// If Some, the ability can be activated from this zone instead of the battlefield.
/// Default None = battlefield only (standard CR 602.2 behavior).
#[serde(default)]
activation_zone: Option<ActivationZone>,
```

### Change 3: Extend `handle_activate_ability` for graveyard activation

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: At line ~156-187, extend the zone check to read `activation_zone` from the ability definition. If `activation_zone == Some(ActivationZone::Graveyard)`, allow activation from `ZoneId::Graveyard(owner)` instead of requiring `ZoneId::Battlefield`.
**CR**: 602.2 -- activation from non-battlefield zones.

The check should:
1. Look up the ability def from `characteristics.activated_abilities[ability_index]`
2. Read its `activation_zone` field
3. If `Some(ActivationZone::Graveyard)`: require `obj.zone == ZoneId::Graveyard(obj.owner)` and `obj.owner == player`
4. If `None`: existing battlefield check (unchanged)

### Change 4: Add `trigger_zone` field to `AbilityDefinition::Triggered`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `trigger_zone: Option<TriggerZone>` to `AbilityDefinition::Triggered`.

```
/// Zone where this triggered ability monitors for events.
/// Default (None) = battlefield. Some(Graveyard) = graveyard zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerZone {
    Graveyard,
}
```

**Note**: This is simpler than it seems. `check_triggers` currently only fires triggers for objects on the battlefield. With `trigger_zone: Some(TriggerZone::Graveyard)`, the trigger dispatch in `abilities.rs` must also scan graveyard objects. This requires a change to `collect_triggers_for_event` (or wherever CardDef triggers are checked).

### Change 5: Modal trigger mode selection in `flush_pending_triggers`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers` (line ~5818), when creating `StackObjectKind::TriggeredAbility`, propagate mode selection. The mode choice happens at stack-put time (CR 700.2b).

For deterministic bot behavior: auto-select mode 0 (consistent with `Effect::Choose` fallback). Store the chosen mode index in `StackObject.modes_chosen`.

At resolution time (`resolution.rs` line ~1802), when resolving a `TriggeredAbility`, if the CardDef's `Triggered.modes` is `Some`, use `stack_obj.modes_chosen` to select which mode effects to execute (same pattern as modal spell resolution at line ~312).

### Change 6: Graveyard trigger dispatch in `check_triggers` / `collect_triggers_for_event`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the trigger checking functions, when scanning for CardDef `AbilityDefinition::Triggered` entries, also scan objects in graveyard zones whose `trigger_zone == Some(TriggerZone::Graveyard)`.

Currently, `queue_carddef_etb_triggers` and the generic CardDef trigger sweep in `end_step_actions`/`upkeep_actions` only look at battlefield objects. The `check_triggers` function scans `state.objects` for matching `TriggerEvent`s, but only considers objects on the battlefield (gated by `obj.zone == ZoneId::Battlefield`).

For graveyard triggers: after the main battlefield scan, do a second pass over graveyard objects, checking only triggers with `trigger_zone: Some(TriggerZone::Graveyard)`.

### Change 7: Exhaustive match updates

Files requiring new match arms or field additions for the new `modes` / `activation_zone` / `trigger_zone` fields:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/state/hash.rs` | `AbilityDefinition::Triggered` hash | Hash `modes`, `trigger_zone` fields |
| `crates/engine/src/state/hash.rs` | `AbilityDefinition::Activated` hash | Hash `activation_zone` field |
| `crates/engine/src/state/hash.rs` | `ActivationZone` enum | New HashInto impl |
| `crates/engine/src/state/hash.rs` | `TriggerZone` enum | New HashInto impl |
| `crates/engine/src/rules/resolution.rs` | `TriggeredAbility` resolution (~L1802) | Check modes and use `stack_obj.modes_chosen` for modal dispatch |
| `crates/engine/src/rules/abilities.rs` | `flush_pending_triggers` (~L5818) | Set `modes_chosen` on StackObject for modal triggers |
| `crates/engine/src/rules/abilities.rs` | `handle_activate_ability` (~L156) | Check `activation_zone` for graveyard abilities |
| `crates/engine/src/rules/abilities.rs` | trigger scanning functions | Scan graveyard objects for `trigger_zone::Graveyard` |
| `crates/engine/src/rules/turn_actions.rs` | `end_step_actions` / `upkeep_actions` CardDef sweeps | Also scan graveyard objects with `trigger_zone::Graveyard` |
| `crates/engine/src/testing/replay_harness.rs` | `AbilityDefinition::Triggered` destructuring (if any) | Add `modes`, `trigger_zone` fields |
| `tools/replay-viewer/src/view_model.rs` | `AbilityDefinition` serialization | Ensure new fields serialize (serde default handles this) |
| `crates/engine/src/cards/helpers.rs` | exports | Export `ActivationZone`, `TriggerZone` |

**Note**: Since `modes`, `activation_zone`, and `trigger_zone` are all `Option` with `#[serde(default)]`, existing card defs compile without changes. Only card defs that need these features must add the new fields.

## Card Definition Fixes

### G-27: Modal Triggered Ability Cards

These cards have triggered abilities with "choose one" that are currently TODO or wrong. After adding `modes` to `Triggered`, they can use `ModeSelection` on the trigger:

#### retreat_to_kazandu.rs
**Oracle text**: "Landfall -- Whenever a land you control enters, choose one -- Put a +1/+1 counter on target creature. / You gain 2 life."
**Current state**: Empty abilities (TODO)
**Fix**: Add `AbilityDefinition::Triggered` with landfall trigger condition + `modes: Some(ModeSelection { min_modes: 1, max_modes: 1, modes: [counter, gain_life], ... })`.

#### retreat_to_coralhelm.rs
**Oracle text**: "Landfall -- Whenever a land you control enters, choose one -- You may tap or untap target creature. / Scry 1."
**Current state**: Empty abilities (TODO)
**Fix**: Add Triggered with landfall + modes. Mode 0: tap/untap target (approximation: untap target creature). Mode 1: Scry 1.

#### felidar_retreat.rs
**Oracle text**: "Landfall -- Whenever a land you control enters, choose one -- Create a 2/2 white Cat Beast creature token. / Put a +1/+1 counter on each creature you control + vigilance."
**Current state**: Partial (mode 0 only, token creation)
**Fix**: Replace with full modal trigger. Mode 0: CreateToken. Mode 1: Sequence(ForEach counters, grant vigilance).

#### junji_the_midnight_sky.rs
**Oracle text**: "When Junji dies, choose one -- Each opponent discards two cards and loses 2 life. / Put target non-Dragon creature card from a graveyard onto battlefield."
**Current state**: Empty abilities (TODO -- "modal death trigger")
**Fix**: Add Triggered with WhenDies + modes.

#### shambling_ghast.rs
**Oracle text**: "When Shambling Ghast enters, create a Treasure token or put a -1/-1 counter on target creature."
**Current state**: Decayed only (TODO -- "ETB choose one")
**Fix**: Add Triggered with WhenEntersBattlefield + modes.

#### tectonic_giant.rs
**Oracle text**: "Whenever this creature attacks or becomes the target..., choose one -- 3 damage to each opponent / exile top 2 impulse draw."
**Current state**: Partial (attack trigger with damage mode only)
**Fix**: Replace with modal trigger. Mode 1 (impulse draw) is a DSL gap -- use Effect::Nothing placeholder. Mode 0 (damage) works.

#### hullbreaker_horror.rs
**Oracle text**: "Whenever you cast a spell, choose up to one -- Return target spell you don't control to hand / Return target nonland permanent to hand."
**Current state**: Empty (TODO -- "choose up to one")
**Fix**: Add Triggered with WheneverYouCastSpell + modes (min_modes: 0, max_modes: 1 for "choose up to one"). Bounce effect uses existing MoveZone.

#### black_market_connections.rs
**Oracle text**: "At the beginning of your first main phase, choose one or more -- [treasure/draw/token]"
**Current state**: Empty (TODO -- new trigger condition + modal)
**Fix**: Approximate with `AtBeginningOfYourUpkeep` (no first-main-phase trigger yet). Use `modes: Some(ModeSelection { min_modes: 1, max_modes: 3, modes: [...], allow_duplicate_modes: false, ... })`.

#### caesar_legions_emperor.rs
**Oracle text**: "Whenever you attack, may sacrifice another creature. When you do, choose two -- [tokens/draw/damage]"
**Current state**: Empty (TODO -- reflexive trigger + modal)
**Fix**: DEFERRED -- reflexive trigger ("when you do") not in DSL. Too complex for PB-35.

#### frontier_siege.rs
**Oracle text**: "As this enters, choose Khans or Dragons. [different triggers per mode]"
**Current state**: Empty (TODO -- ETB modal choice)
**Fix**: DEFERRED -- "as enters" modal choice is a permanent-state modal, not a triggered ability modal. Different mechanism needed.

#### windcrag_siege.rs
**Oracle text**: "As this enters, choose Mardu or Jeskai."
**Current state**: Empty (TODO)
**Fix**: DEFERRED -- same as frontier_siege.

#### parapet_thrasher.rs
**Oracle text**: "Whenever Dragons deal combat damage to opponent, choose one that hasn't been chosen this turn"
**Current state**: Empty (TODO -- multiple gaps)
**Fix**: DEFERRED -- "hasn't been chosen this turn" tracking not in DSL.

#### glissa_sunslayer.rs
**Oracle text**: "Whenever deals combat damage to player, choose one -- draw+lose 1 / destroy enchantment / remove counters"
**Current state**: Already uses Effect::Choose (auto-picks mode 0)
**Fix**: Convert to use `modes` on Triggered. Mode 2 (remove counters) remains TODO (no generic counter removal).

#### goblin_cratermaker.rs
**Oracle text**: "{1}, Sacrifice: Choose one -- deal 2 to creature / destroy colorless nonland permanent"
**Current state**: TODO -- "modal activated abilities not expressible"
**Fix**: Use `Effect::Choose` in the Activated ability's effect. This is a modal ACTIVATED ability, not triggered -- `Effect::Choose` already works here.

#### umezawas_jitte.rs
**Oracle text**: "Remove a charge counter: Choose one -- +2/+2 / -1/-1 / gain 2 life"
**Current state**: Partial (charge counter trigger works, modal activated TODO)
**Fix**: Use `Effect::Choose` in the activated ability's effect. Already has the activated ability structure.

### G-29: Graveyard Recursion Cards

Cards that need `activation_zone: Some(ActivationZone::Graveyard)` or `trigger_zone: Some(TriggerZone::Graveyard)`:

#### reassembling_skeleton.rs
**Oracle text**: "{1}{B}: Return this card from your graveyard to the battlefield tapped."
**Current state**: Empty abilities (TODO)
**Fix**: Add `Activated` with `activation_zone: Some(ActivationZone::Graveyard)`, cost: {1}{B}, effect: MoveZone to battlefield tapped.

#### cult_conscript.rs
**Oracle text**: "{1}{B}: Return this card from your graveyard to the battlefield. Activate only if a non-Skeleton creature died under your control this turn."
**Current state**: ETB replacement only (TODO for activated)
**Fix**: Add Activated with `activation_zone: Some(ActivationZone::Graveyard)`. The "activate only if non-Skeleton creature died this turn" condition needs `Condition::CreatureDiedThisTurn` -- NEW condition variant needed. DEFERRED condition if too complex; implement basic graveyard activation without the condition first.

#### earthquake_dragon.rs
**Oracle text**: "{2}{G}, Sacrifice a land: Return this card from your graveyard to your hand."
**Current state**: Has flying/trample + cost reduction (TODO for graveyard activated)
**Fix**: Add Activated with `activation_zone: Some(ActivationZone::Graveyard)`, cost includes sacrifice a land (Cost::SacrificePermanent with land filter), effect: MoveZone(Source, Hand).

#### bloodghast.rs
**Oracle text**: "Landfall -- Whenever a land you control enters, you may return this card from your graveyard to the battlefield."
**Current state**: Conditional static works (TODO for landfall graveyard trigger + can't block)
**Fix**: Add Triggered with `trigger_zone: Some(TriggerZone::Graveyard)`, trigger_condition: landfall, effect: MoveZone(Source, Battlefield). Keep the conditional static for haste.

#### nether_traitor.rs
**Oracle text**: "Whenever another creature is put into your graveyard from the battlefield, you may pay {B}. If you do, return this card from your graveyard to the battlefield."
**Current state**: Haste + Shadow only (TODO for graveyard trigger)
**Fix**: DEFERRED -- "may pay {B}" reflexive cost on trigger resolution not in DSL. Would need Effect::MayPay or similar.

#### scavenging_ooze.rs
**Oracle text**: "{G}: Exile target card from a graveyard. If creature card, +1/+1 counter and gain 1 life."
**Current state**: Empty abilities (TODO -- conditional resolution effect)
**Fix**: This is a BATTLEFIELD activated ability targeting a graveyard card (not a graveyard-zone ability). The gap is "conditional effect based on exiled card's type at resolution." Can be approximated with Effect::Sequence + Conditional. Uses existing TargetRequirement::TargetCardInGraveyard.

#### bojuka_bog.rs
**Oracle text**: "When this land enters, exile target player's graveyard."
**Current state**: Has ETB tapped (TODO for graveyard exile trigger)
**Fix**: Add Triggered ETB with Effect::ExileGraveyard or ForEach exile. The DSL gap is the lack of "exile all cards in target player's graveyard" effect.

### G-30: Planeswalker Card Fixes

These PW defs have TODOs that can be fixed with existing DSL primitives (not PW-framework issues):

#### sarkhan_fireblood.rs
**TODO**: "Optional discard-then-draw not in DSL" / "Any combination of colors + Dragon restriction"
**Fix**: Mode 0 (+1): Use Effect::Choose with looting (discard-then-draw exists). Mode 1 (+2): DEFERRED -- any-color mana with Dragon restriction.

#### chandra_flamecaller.rs
**TODO**: "Discard all cards then draw that many plus one"
**Fix**: Use Effect::Sequence with DiscardHand + DrawCards(EffectAmount::HandSizeBeforeDiscard). DEFERRED -- `HandSizeBeforeDiscard` variant not in EffectAmount.

#### sorin_lord_of_innistrad.rs
**TODO**: "Emblem with static P/T modification" / "Destroy up to three targets"
**Fix**: DEFERRED -- emblems not in DSL.

#### xenagos_the_reveler.rs
**TODO**: "+1 count-based mana (X = creatures you control)" / "-6 exile top 7 put creatures/lands"
**Fix**: DEFERRED -- count-based mana production + complex library exile.

**Net PW fixable in PB-35**: Very few. Most PW TODOs depend on emblems, delayed triggers, or complex effects not yet in DSL. Recommend deferring G-30 PW fixes to PB-37 (residual cleanup) and focusing PB-35 on G-27 + G-29.

## New Card Definitions

None -- all affected cards already have card def files with TODOs.

## Unit Tests

**File**: `crates/engine/tests/modal_triggers.rs` (NEW)
**Tests to write**:
- `test_modal_triggered_ability_basic` -- Retreat to Kazandu: landfall fires, mode 0 auto-selected (counter placed). CR 700.2b.
- `test_modal_triggered_ability_mode_selection` -- Verify modes_chosen propagates through StackObject for triggered abilities. CR 603.3c.
- `test_modal_death_trigger` -- Junji: dies trigger fires with modal choice. CR 700.2b.
- `test_modal_etb_trigger` -- Shambling Ghast: ETB modal fires. CR 700.2b.

**File**: `crates/engine/tests/graveyard_abilities.rs` (NEW)
**Tests to write**:
- `test_graveyard_activated_ability_basic` -- Reassembling Skeleton: activate from graveyard, returns to battlefield tapped. CR 602.2.
- `test_graveyard_activated_ability_zone_check` -- Cannot activate Reassembling Skeleton from battlefield (it's a graveyard-only ability).
- `test_graveyard_triggered_ability` -- Bloodghast: landfall while in graveyard triggers return to battlefield.
- `test_graveyard_activated_sacrifice_cost` -- Earthquake Dragon: sacrifice a land + mana cost from graveyard to return to hand.

**Pattern**: Follow tests in `crates/engine/tests/combat_damage_triggers.rs` for triggered ability testing patterns, and `crates/engine/tests/abilities.rs` for activated ability testing.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (non-deferred ones)
- [ ] New card defs authored (if any)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except explicitly deferred)

## Risks & Edge Cases

- **Modal trigger target legality**: CR 700.2b says if no mode can be chosen (all modes have illegal targets), the trigger is removed from the stack. The deterministic bot always picks mode 0; if mode 0's targets are illegal but mode 1's are legal, the bot makes a suboptimal choice. Acceptable for pre-alpha.
- **"Choose up to one" (0 modes allowed)**: Hullbreaker Horror says "choose up to one" -- `min_modes: 0`. If 0 modes are chosen, the trigger resolves with no effect. The bot should auto-pick mode 0 when a legal target exists, or 0 modes when no target exists.
- **Graveyard activation zone validation**: The activated ability's cost (mana, sacrifice, etc.) must still be payable. A creature in the graveyard has no mana abilities -- the player pays from their mana pool as normal. Tap costs on graveyard abilities are meaningless (can't tap a graveyard card) -- none of the target cards have tap costs on their graveyard abilities.
- **Graveyard trigger zone scanning performance**: Adding a second pass over graveyard objects in `check_triggers` adds O(graveyard_size) per trigger event. Acceptable for Commander (graveyards are typically <40 cards, events fire <100/turn).
- **"As this enters, choose" (Siege cycle)**: Frontier Siege, Windcrag Siege, etc. use "as this permanent enters" modal choice, which is a DIFFERENT mechanism than "whenever X, choose one." The Siege pattern creates a permanent state (stored choice) that determines which triggered ability is active. This is NOT covered by PB-35's modal trigger support. Deferred.
- **Reflexive triggers** ("when you do, choose"): Caesar, Nether Traitor's "may pay {B}, if you do" are reflexive triggered abilities (CR 603.12). Not in scope for PB-35.
- **AbilityDefinition::Activated field addition**: `activation_zone: Option<ActivationZone>` must be `#[serde(default)]` to avoid breaking existing card defs. The `Activated` variant already has many fields; this adds one more.
- **StackObject.modes_chosen reuse**: The `modes_chosen` field already exists on StackObject (used for modal spells). Reusing it for modal triggers is clean -- triggered abilities put on the stack as StackObjectKind::TriggeredAbility already flow through the same resolution path.

## Session Estimate

**2 sessions**:
- Session 1: Engine changes (Changes 1-6), exhaustive match updates (Change 7), basic tests
- Session 2: Card def fixes (G-27 modal triggers + G-29 graveyard abilities), remaining tests, workspace build verification

## Deferred to PB-37 / Future

- **G-30 PW fixes**: Most PW TODOs depend on emblems, delayed triggers, complex effects -- not PW framework. Defer to PB-37 residual.
- **"As enters, choose" (Siege cycle)**: Needs permanent-state modal mechanism, not triggered ability modal.
- **Reflexive triggers** ("when you do"): CR 603.12, separate engine feature.
- **"Choose one that hasn't been chosen this turn"** (Parapet Thrasher): Per-turn mode tracking.
- **Cult Conscript condition** ("if a non-Skeleton creature died this turn"): New Condition variant needed.
- **Nether Traitor** ("may pay {B}"): Reflexive cost on trigger resolution.
- **Black Market Connections trigger**: "At the beginning of your first main phase" is not an existing TriggerCondition. Approximate with upkeep or defer.
