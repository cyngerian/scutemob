# Primitive Batch Plan: PB-AC6 — Phase & opponent-action conditions

**Generated**: 2026-07-09
**Primitive**: 3 new `TriggerCondition` variants (`AtBeginningOfFirstMainPhase`,
`AtBeginningOfPostcombatMain`, `WhenBecomesTarget`) + 5 new `Condition` variants
(`YouAttackedThisTurn`, `CreatedATokenThisTurn`, `OpponentCastNSpells`,
`SpellMastery`, `OpponentControlsMoreLandsThanYou`).
**CR Rules (verified via MCP)**: 505.1/505.1a/505.1b/505.2/505.6 (main phases),
500.2 (phase-end), 601.2c (spell target announcement), 602.2b (ability target
announcement), 603.2/603.2b/603.2c (trigger timing), 207.2c (ability words /
spell mastery), 702.21a (Ward becomes-target precedent).
**Cards affected**: ~7 confirmed-clean + ~4 strong-partial (backfill) + 11
stays-blocked. Discounted-yield brief said ~18; **honest clean yield is 7**
(planner-overcount calibration applied — see roster table).
**Dependencies**: none (all prerequisite infra — generic sweeps, PermanentTargeted
event, per-turn PlayerState counters, check_condition — already exists).
**Deferred items from prior PBs**: none targeted; this batch closes the
`YouAttackedThisTurn` / `CreatedATokenThisTurn` / `SpellMastery` /
`OpponentControlsMoreLandsThanYou` / first-&-postcombat-main / becomes-target
DSL gaps flagged across ~18 card TODOs.

**Discriminant chain — NO NEW KeywordAbility / AbilityDefinition / StackObjectKind
variants.** `TriggerCondition` and `Condition` are independent enums with their own
hash discriminants. Therefore **no changes** to
`tools/tui/src/play/panels/stack_view.rs` (`StackObjectKind` match) or
`tools/replay-viewer/src/view_model.rs` (`StackObjectKind` + `KeywordAbility`
matches). One new `TriggerEvent` variant is added (`PermanentBecomesTarget`), but
`TriggerEvent` is not matched in the TUI/replay-viewer. Still run
`cargo build --workspace` after implement/fix (the runner misses cross-crate
breaks ~50% of the time).

---

## MANDATORY pre-existing TODO sweep (roster-recall gate) — RESULT

Ran `grep -i '(TODO|ENGINE-BLOCKED).*(<primitive keyword>)'` over
`crates/engine/src/cards/defs/` for every primitive keyword (first main /
postcombat / becomes-target / spell mastery / attacked / created-a-token /
opponent-cast / more-lands / Raid / Valiant / Addendum). **Sweep is non-empty**;
see roster table. Two cards surfaced ONLY by the broadened Raid/Valiant sweep and
are **forced adds** (not in the PB brief's implicit list):
`bloodsoaked_champion.rs` (clean `YouAttackedThisTurn` activation-condition) and
`raiders_wake.rs` (Raid end-step, `YouAttackedThisTurn`). Both are recorded in the
roster with the note *"added via pre-existing TODO sweep."*

`land_tax.rs` was ALSO nearly missed by keyword grep (its TODO line names the
`OpponentControlsMoreLandsThanYou` type in CamelCase, not the spaced oracle
phrase). It is the sole clean card for that condition — recorded.

---

## Verified CR text

**CR 505.1** — "There are two main phases in a turn. In each turn, the first main
phase (also known as the precombat main phase) and the second main phase (also
known as the postcombat main phase) are separated by the combat phase."
**CR 505.1a** — "Only the first main phase of the turn is a precombat main phase.
All other main phases are postcombat main phases. This includes the second main
phase of a turn in which the combat phase has been skipped. It is also true of a
turn in which an effect has caused an additional combat phase and an additional
main phase to be created."
→ **Engine mapping is exact**: `Step::PreCombatMain` occurs exactly once per turn
(present once in `STEP_ORDER`, `rules/turn_structure.rs:10`). Every *additional*
main phase created by effects is inserted as `Step::PostCombatMain`
(`turn_structure.rs:64` — `Phase::PostCombatMain => Step::PostCombatMain`). So
"first main" ⇔ `Step::PreCombatMain` and "each postcombat main" ⇔
`Step::PostCombatMain`. **No new state field needed to distinguish them.**

**CR 505.2 / 500.2** — main phase has no steps; it ends when all players pass in
succession with the stack empty. There are no turn-based actions in the main phase
except Saga lore counters (505.4), Attractions (505.5), Archenemy scheme (505.3).
"At the beginning of your [first/postcombat] main phase" abilities are **triggered
abilities** that fire on phase begin (CR 603.2b) and go on the stack when the
active player would receive priority (505.6).
→ **Engine mapping**: they must be queued by a generic CardDef sweep in the
turn-based-action entry for that step (mirroring the upkeep / end-step sweeps),
then flushed by `enter_step` before priority is granted.

**CR 601.2c** (spell targets) — "The chosen objects and/or players each become a
target of that spell. (Any abilities that trigger when those objects and/or
players become the target of a spell trigger at this point; they'll wait to be put
on the stack until the spell has finished being cast.)"
**CR 602.2b** (ability targets) — "The remainder of the process for activating an
ability is identical to the process for casting a spell listed in rules 601.2b–i."
→ **Confirmed**: `WhenBecomesTarget` fires at **announcement** (target choice),
NOT resolution. The engine already emits `GameEvent::PermanentTargeted` at exactly
this point: `rules/casting.rs:4696` (after `SpellCast`, for each battlefield
target) and `rules/abilities.rs:1096` / `abilities.rs:1675` (after
`AbilityActivated`). This is the same event that drives Ward (CR 702.21a). We reuse
it. The trigger goes on the stack above the spell/ability (CR 601.2c "wait to be
put on the stack") — the engine's queue→flush ordering already does this.

**CR 207.2c** — "An ability word appears in italics… they have no special rules
meaning and no individual entries in the Comprehensive Rules. The ability words
are … **spell mastery** …"
→ **Confirmed**: `SpellMastery` is an ability word, NOT a keyword ability. It gets
a **`Condition`** variant and **must NOT** receive a `KeywordAbility` discriminant.
The condition it stands for is the card text "two or more instant and/or sorcery
cards in your graveyard."

**CR 603.2b/603.2c** — phase-begin triggers fire when the phase begins; an ability
triggers once per occurrence of its event.

---

## Existing-infra survey (what we reuse, verified line-by-line)

- **Main-phase turn-based-action dispatch**: `rules/turn_actions.rs:19`
  `execute_turn_based_actions` matches `Step::PreCombatMain =>
  precombat_main_actions(state)` (line 30). `Step::PostCombatMain` currently falls
  to `_ => Ok(Vec::new())` (line 31) — **no postcombat hook exists yet**.
- **Generic CardDef upkeep sweep** (the B9/MR-B9-01 pattern to copy):
  `turn_actions.rs:358-443` — scans battlefield permanents, reads
  `state.card_registry`, matches `AbilityDefinition::Triggered { trigger_condition }`
  for `AtBeginningOfYourUpkeep`/`AtBeginningOfEachUpkeep`, pushes
  `PendingTrigger { kind: PendingTriggerKind::CardDefETB, .. }`. **This is the exact
  template** for both new main-phase sweeps.
- **`precombat_main_actions`** (`turn_actions.rs:486-542`): currently only does
  Saga lore counters. Must gain the generic first-main sweep.
- **`enter_step`** (`rules/engine.rs:1606`): runs turn-based actions on step entry,
  pushes their triggers, then flushes before granting priority. No change needed —
  it already handles whatever the sweeps queue.
- **Per-turn PlayerState counters + reset**: `state/player.rs` has
  `cards_drawn_this_turn` (327), `spells_cast_this_turn` (335),
  `life_gained_this_turn` (366), `damage_received_this_turn` (375).
  `reset_turn_state` (`turn_actions.rs:1539`) resets `spells_cast_this_turn`
  **only for the incoming active player** (line 1555, storm semantics), and resets
  `cards_drawn_this_turn`/`life_gained_this_turn`/`damage_received_this_turn`
  **for ALL players** in the loop at lines 1572-1585. New per-turn bools go in that
  all-players loop.
- **`previous_turn_spells_cast`** (reset pattern reference): `state/mod.rs:236`
  (GameState field); captured & reset in `reset_turn_state` at
  `turn_actions.rs:1543-1548` (snapshot the outgoing player's `spells_cast_this_turn`
  into `previous_turn_spells_cast`, then zero the per-player counter). Studied per
  brief — confirms the established "capture-then-reset-in-reset_turn_state" pattern.
- **`check_condition`** (`effects/mod.rs:7731`): exhaustive match, no `_` arm →
  **new Condition variants REQUIRE new arms here or it won't compile.**
- **`check_static_condition`** (`effects/mod.rs:8090`): has a `_ =>
  check_condition(state, condition, &ctx)` fallback (line 8157-8185) → **no arms
  needed there**; the new conditions delegate automatically.
- **`GameEvent::PermanentTargeted`** (`rules/events.rs:745`) + Ward dispatch
  (`rules/abilities.rs:4152-4181`): the announcement-time targeting hook. We add
  our becomes-target dispatch immediately after the Ward block.
- **Runtime trigger model**: `AbilityDefinition::Triggered { trigger_condition }`
  is converted to a runtime `TriggeredAbilityDef { trigger_on: TriggerEvent, .. }`
  in `enrich_spec_from_def` (`testing/replay_harness.rs:1919`; the `if let` chain at
  2164-3192 maps each `TriggerCondition` → `TriggerEvent`).
  `collect_triggers_for_event` (`rules/abilities.rs:6163`) scans
  `calculate_characteristics(...).triggered_abilities` (layer-resolved) for
  `trigger_on == event_type`. `calculate_characteristics` (`rules/layers.rs`) starts
  from `obj.characteristics.clone()` (base) and does NOT re-enrich from the registry
  — so becomes-target triggers must be present in `obj.characteristics.triggered_abilities`,
  which `enrich_spec_from_def` populates at object construction. (Same mechanism as
  Ward, Prowess, `WheneverCreatureYouControlAttacks`.)
- **Ward precedent** (`state/builder.rs:380-415`): keyword→`TriggeredAbilityDef`
  with `trigger_on: TriggerEvent::SelfBecomesTargetByOpponent`; effect references the
  targeting spell via `EffectTarget::DeclaredTarget { index: 0 }`, populated from
  `PendingTrigger.targeting_stack_id` set at `abilities.rs:4177`. We copy this
  targeting-stack-id tagging for becomes-target effects that reference "that spell's
  controller" (Bonecrusher Giant).
- **Token flag chokepoint**: `state/mod.rs:349` `add_object` already special-cases
  `zone_id == ZoneId::Battlefield && object.is_token` (line 361) to set
  `entered_turn`. ALL 13 `GameEvent::TokenCreated` emission sites funnel a token
  `GameObject` (`is_token = true`) through `add_object` before emitting (verified:
  `effects/mod.rs:591-596`, `resolution.rs:4706-4722`, etc.). **This is a single
  unmissable chokepoint** for `created_token_this_turn` — no need to touch 13 sites.
- **`activation_condition`** on `AbilityDefinition::Activated` is `Option<Condition>`,
  propagated by `enrich_spec_from_def` (~line 2038) and enforced at activation via
  `check_condition` (CR 602.5b). Confirms `YouAttackedThisTurn` /
  `CreatedATokenThisTurn` work as activation gates (Idol, Bloodsoaked Champion).
- **HASH_SCHEMA_VERSION** = **32** (verified `state/hash.rs:254`). Bumps to **33**.

---

## Engine Changes

### Change 1 — Two new mutable PlayerState tracking fields

**File**: `crates/engine/src/state/player.rs`
**Action**: add two `bool` fields to `PlayerState` (place near
`damage_received_this_turn`, ~line 375):
```rust
/// CR 508.1 / Raid (CR 702, ability word): true once the player has declared
/// one or more attackers this turn. Set in handle_declare_attackers. Reset for
/// ALL players at each turn boundary. Used by Condition::YouAttackedThisTurn.
/// Creatures put onto the battlefield attacking (CR 508.4) do NOT set this —
/// only a declare-attackers action counts as "you attacked" (Bloodsoaked
/// Champion ruling).
pub attacked_this_turn: bool,
/// CR 111.10: true once the player has created one or more tokens this turn.
/// Set in GameState::add_object (single chokepoint). Reset for ALL players at
/// each turn boundary. Used by Condition::CreatedATokenThisTurn.
pub created_token_this_turn: bool,
```

**Field: `attacked_this_turn`**
- **Type**: `bool`, on `PlayerState`.
- **Set at**: `rules/combat.rs::handle_declare_attackers` (fn starts line 32) —
  AFTER the validation loop, once attackers are committed to `state.combat`, gate
  on `!attackers.is_empty()`:
  `if let Some(ps) = state.players.get_mut(&player) { ps.attacked_this_turn = true; }`.
  (Do NOT set it in the token-enters-attacking path in `effects/mod.rs` — CR 508.4
  creatures were not "declared"; Bloodsoaked Champion ruling.)
- **Reset at**: `turn_actions.rs::reset_turn_state`, in the all-players loop
  (lines 1572-1585): `p.attacked_this_turn = false;`. Cite the same rationale as
  `cards_drawn_this_turn` (multiplayer "this turn" is the current game turn).
- **Hash**: add `self.attacked_this_turn.hash_into(hasher);` to the `PlayerState`
  HashInto impl (`state/hash.rs`, alongside line ~1398 `damage_received_this_turn`).
- **Init**: `state/builder.rs` PlayerState literal (~lines 258-265):
  `attacked_this_turn: false,`.

**Field: `created_token_this_turn`**
- **Type**: `bool`, on `PlayerState`.
- **Set at**: `state/mod.rs::add_object`, inside the existing
  `if zone_id == ZoneId::Battlefield && object.is_token { … }` block (line 361):
  `if let Some(ps) = self.players.get_mut(&object.controller) { ps.created_token_this_turn = true; }`.
  Single chokepoint — every token creation passes through here.
- **Reset at**: `reset_turn_state` all-players loop:
  `p.created_token_this_turn = false;`.
- **Hash**: add `self.created_token_this_turn.hash_into(hasher);` next to the field
  above in the `PlayerState` HashInto impl.
- **Init**: `state/builder.rs` PlayerState literal: `created_token_this_turn: false,`.
- **Edge case (documented, not blocking)**: when a *token* component splits off a
  merged (mutate/meld) permanent leaving the battlefield, it re-enters via
  `move_object_to_zone`, not `add_object`, so it does NOT re-set the flag. This is
  correct (splitting is not "creating a token") and irrelevant to the roster.

**HASH mutation-verified tests (REQUIRED — this omission was a review HIGH in both
PB-AC1 and PB-AC5)**: one test per field that flips the field on an otherwise-fixed
state and asserts the public hash changes:
- `test_hash_sensitive_attacked_this_turn`
- `test_hash_sensitive_created_token_this_turn`
Pattern: build a state, `let h0 = state.hash()`, mutate
`state.players.get_mut(&p1).unwrap().attacked_this_turn = true`, assert
`state.hash() != h0`.

### Change 2 — `HASH_SCHEMA_VERSION` bump

**File**: `crates/engine/src/state/hash.rs:254`
**Action**: `pub const HASH_SCHEMA_VERSION: u8 = 33;` (was 32). Add a changelog
entry (the file has a `///` changelog around lines 15-250): "33: PB-AC6 —
`PlayerState::attacked_this_turn` + `created_token_this_turn`; new
`TriggerCondition` variants (FirstMain/PostcombatMain/WhenBecomesTarget), new
`Condition` variants (5), new `TriggerEvent::PermanentBecomesTarget`."
Update the `assert_eq!(HASH_SCHEMA_VERSION, 33)` version-guard test.

### Change 3 — Three new `TriggerCondition` variants

**File**: `crates/engine/src/cards/card_definition.rs` (enum at line 2874)
**Action**: add after `WhenExertedAsAttacks` (line 3219):
```rust
/// CR 505.1a / 603.2b: "At the beginning of your first main phase." Fires ONCE
/// per turn, on Step::PreCombatMain (which occurs exactly once per turn).
/// Queued by the generic CardDef sweep in precombat_main_actions.
AtBeginningOfFirstMainPhase,
/// CR 505.1a / 603.2b: "At the beginning of [each of] your postcombat main
/// phase[s]." Fires on every Step::PostCombatMain, including extra main phases
/// created by effects (CR 505.1a). Queued by postcombat_main_actions.
AtBeginningOfPostcombatMain,
/// CR 601.2c / 602.2b / 603.2: "Whenever [this permanent / a <filter> you
/// control] becomes the target of a spell [or ability] [an opponent controls]."
/// Fires at target-ANNOUNCEMENT (not resolution), driven by
/// GameEvent::PermanentTargeted. Distinct from WhenBecomesTargetByOpponent
/// (Ward-only: self + spell-or-ability + opponent).
WhenBecomesTarget {
    /// None = the trigger source itself must be the target ("Whenever this
    ///   creature becomes the target..."). Some(filter) = any permanent the
    ///   source's controller controls that matches `filter` becoming a target
    ///   ("a creature/Dragon you control").
    #[serde(default)]
    scope: Option<TargetFilter>,
    /// If true, only fires when the targeting spell/ability is controlled by an
    ///   opponent of the source's controller. If false, any controller.
    #[serde(default)]
    by_opponent: bool,
    /// If true, fires on a spell OR ability. If false, spell only.
    #[serde(default)]
    include_abilities: bool,
},
```
**Hash** (`state/hash.rs`, `TriggerCondition` HashInto ~line 4903, last discriminant
`WhenExertedAsAttacks => 44`):
```rust
TriggerCondition::AtBeginningOfFirstMainPhase => 45u8.hash_into(hasher),
TriggerCondition::AtBeginningOfPostcombatMain => 46u8.hash_into(hasher),
TriggerCondition::WhenBecomesTarget { scope, by_opponent, include_abilities } => {
    47u8.hash_into(hasher);
    scope.hash_into(hasher);
    by_opponent.hash_into(hasher);
    include_abilities.hash_into(hasher);
}
```
(`Option<TargetFilter>` and `bool` already impl `HashInto`.)

### Change 4 — Generic first-main sweep

**File**: `crates/engine/src/rules/turn_actions.rs`, `precombat_main_actions`
(line 486)
**Action**: add a generic CardDef sweep for
`TriggerCondition::AtBeginningOfFirstMainPhase`, copied verbatim from the upkeep
sweep at 358-443 (registry scan → `PendingTrigger { kind:
PendingTriggerKind::CardDefETB, source, ability_index, controller: active, .. }`),
BUT matching `AtBeginningOfFirstMainPhase` and only for permanents controlled by
the active player (`controller == active`). Filter battlefield + phased-in
(`!obj.status.phased_out`). Push before returning `events`.
**CR**: 505.1a / 603.2b / 603.3 — first main is `Step::PreCombatMain`, entered once
per turn.

### Change 5 — New `postcombat_main_actions` + dispatch wiring

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**:
(a) In `execute_turn_based_actions` (line 19), add arm:
`Step::PostCombatMain => Ok(postcombat_main_actions(state)),` (currently falls to
`_ => Ok(Vec::new())`).
(b) Add `fn postcombat_main_actions(state: &mut GameState) -> Vec<GameEvent>` that
runs the same generic CardDef sweep matching
`TriggerCondition::AtBeginningOfPostcombatMain` for the active player's battlefield
permanents. Return `Vec::new()` for direct events (triggers flushed by
`enter_step`).
**CR**: 505.1a — every postcombat main (including extra mains, which are all
`Step::PostCombatMain`) fires these. No per-turn dedup: "each of your postcombat
main phases" fires once per such phase (correct by construction, since the sweep
runs on each `Step::PostCombatMain` entry).

### Change 6 — New `TriggerEvent::PermanentBecomesTarget`

**File**: `crates/engine/src/state/game_object.rs`, `TriggerEvent` enum (line 329)
**Action**: add after `SelfBecomesTargetByOpponent` (or at end):
```rust
/// CR 601.2c / 602.2b / 603.2: A permanent became the target of a spell (or
/// ability). Global becomes-target event carrying the per-card scope/opponent/
/// spell-vs-ability parameters. Dispatched INLINE in the PermanentTargeted
/// handler (rules/abilities.rs), NOT via collect_triggers_for_event equality
/// (the params are read directly from this variant). Distinct from the Ward-only
/// SelfBecomesTargetByOpponent.
PermanentBecomesTarget {
    scope: Option<crate::cards::card_definition::TargetFilter>,
    by_opponent: bool,
    include_abilities: bool,
},
```
(Precedent for state→cards path: `TriggeredAbilityDef` already holds
`Option<crate::cards::card_definition::TargetFilter>` at game_object.rs:640/648, so
this does not introduce a new module cycle.)
**Hash** (`state/hash.rs`, `TriggerEvent` HashInto ~line 2434, last discriminant
`CounterPlaced => 46`):
```rust
TriggerEvent::PermanentBecomesTarget { scope, by_opponent, include_abilities } => {
    47u8.hash_into(hasher);
    scope.hash_into(hasher);
    by_opponent.hash_into(hasher);
    include_abilities.hash_into(hasher);
}
```

### Change 7 — `enrich_spec_from_def` conversion for `WhenBecomesTarget`

**File**: `crates/engine/src/testing/replay_harness.rs`, `enrich_spec_from_def`
(fn at 1919; add to the `if let` conversion chain near the other becomes-target /
attack conversions ~2200-2820)
**Action**: convert `AbilityDefinition::Triggered { trigger_condition:
TriggerCondition::WhenBecomesTarget { scope, by_opponent, include_abilities },
effect, targets, intervening_if, .. }` into a `TriggeredAbilityDef` with
`trigger_on: TriggerEvent::PermanentBecomesTarget { scope: scope.clone(),
by_opponent, include_abilities }`, carrying `effect`, `targets`, `intervening_if`
through, and all other `TriggeredAbilityDef` fields set to their existing defaults
(`None`/`false`/`vec![]`) exactly as the sibling conversions do. **Only `trigger_on`
carries the params** — no new `TriggeredAbilityDef` struct fields (avoids touching
~45 existing literals).
**Note**: `AtBeginningOfFirstMainPhase` / `AtBeginningOfPostcombatMain` do NOT get
an enrich block — they fire via the registry-scan sweeps (Changes 4/5), like
upkeep/end-step triggers. The `if let` chain silently skips them (correct).

### Change 8 — Inline becomes-target dispatch in the PermanentTargeted handler

**File**: `crates/engine/src/rules/abilities.rs`, `check_triggers`, the
`GameEvent::PermanentTargeted` arm (line 4152), immediately AFTER the existing Ward
block (which ends ~4180).
**Action**: add a global becomes-target scan. Pseudocode:
```
// Determine whether the targeting stack object is a spell (CR 601.2c) vs an
// ability (CR 602.2b). Spell-only triggers ignore ability targeting.
let targeting_is_spell = state.stack_objects.iter()
    .find(|so| so.id == *targeting_stack_id)
    .map(|so| matches!(so.kind, StackObjectKind::Spell { .. }))
    .unwrap_or(false);
let target_controller = state.objects.get(target_id).map(|o| o.controller);
for src in state.objects.values().filter(|o| o.zone == Battlefield && o.is_phased_in()) {
    let chars = calculate_characteristics(state, src.id).unwrap_or(base);
    for (idx, tdef) in chars.triggered_abilities.iter().enumerate() {
        let TriggerEvent::PermanentBecomesTarget { scope, by_opponent, include_abilities }
            = &tdef.trigger_on else { continue };
        // spell-vs-ability gate
        if !include_abilities && !targeting_is_spell { continue; }
        // opponent gate (CR 702.21a-style)
        if *by_opponent && *targeting_controller == src.controller { continue; }
        // scope gate
        match scope {
            None => { if src.id != *target_id { continue; } }          // "this creature"
            Some(f) => {
                // "a <filter> you control": the TARGET must match filter and be
                // controlled by the trigger source's controller.
                if target_controller != Some(src.controller) { continue; }
                let tchars = calculate_characteristics(state, *target_id).unwrap_or(base);
                if !matches_filter(&tchars, f) { continue; }
            }
        }
        // queue trigger; tag targeting_stack_id so effects can reference the
        // targeting spell/ability via DeclaredTarget{0} (Bonecrusher).
        push PendingTrigger { source: src.id, ability_index: idx,
            kind: CardDefETB or the standard runtime-triggered path, ..};
        set t.targeting_stack_id = Some(*targeting_stack_id);
    }
}
```
**Implementation notes for the runner**:
- Mirror the existing Ward block's `PendingTrigger` construction and
  `targeting_stack_id` tagging (abilities.rs:4176-4178). Reuse whatever
  `PendingTriggerKind` the runtime-`triggered_abilities` resolution path expects
  (the same one Ward uses so `flush_pending_triggers` maps `DeclaredTarget{0}` to
  the targeting stack object).
- `matches_filter` is `crate::effects::matches_filter`. For `scope: Some(filter)`
  that references counters, also apply `check_has_counter_type` if the filter uses
  counters (roster filters here are subtype/type only — `creature`, `Dragon`).
- **Multi-target edge (CR 603.2c)**: the engine emits one `PermanentTargeted` per
  entry in `battlefield_targets`; a spell that lists the same permanent in two
  target slots yields two events → two triggers. This matches the existing Ward
  behavior; we deliberately keep it consistent. Documented as an accepted edge, not
  fixed here.
**CR**: 601.2c / 602.2b (announcement timing), 603.2 (once per occurrence),
702.21a (Ward precedent for opponent gate + targeting-stack tagging).

### Change 9 — Five new `Condition` variants

**File**: `crates/engine/src/cards/card_definition.rs`, `Condition` enum (line 3241)
**Action**: add after `ControllerGainedLifeThisTurn` (line 3438):
```rust
/// Raid (ability word) / CR 508.1: "if you attacked this turn." True when the
/// effect controller's PlayerState.attacked_this_turn is set. (Declaring one or
/// more attackers; CR 508.4 tokens-attacking do not count.)
YouAttackedThisTurn,
/// CR 111.10: "if you created a token this turn." True when the controller's
/// PlayerState.created_token_this_turn is set.
CreatedATokenThisTurn,
/// "if an opponent cast N or more spells this turn." True if ANY living opponent
/// of the controller has spells_cast_this_turn >= n. See LIMITATION note.
OpponentCastNSpells(u32),
/// Spell mastery (ability word, CR 207.2c): "if there are two or more instant
/// and/or sorcery cards in your graveyard."
SpellMastery,
/// "if an opponent controls more lands than you." True if ANY living opponent
/// controls strictly more lands than the controller. Land counts use
/// layer-resolved types and exclude phased-out permanents.
OpponentControlsMoreLandsThanYou,
```
**Hash** (`state/hash.rs`, `Condition` HashInto, last discriminant
`ControllerGainedLifeThisTurn => 42` at line 5241):
```rust
Condition::YouAttackedThisTurn => 43u8.hash_into(hasher),
Condition::CreatedATokenThisTurn => 44u8.hash_into(hasher),
Condition::OpponentCastNSpells(n) => { 45u8.hash_into(hasher); n.hash_into(hasher); }
Condition::SpellMastery => 46u8.hash_into(hasher),
Condition::OpponentControlsMoreLandsThanYou => 47u8.hash_into(hasher),
```

### Change 10 — `check_condition` arms

**File**: `crates/engine/src/effects/mod.rs`, `check_condition` (line 7731,
exhaustive match — arms REQUIRED for compile). `check_static_condition` needs NO
change (has `_ => check_condition(...)` fallback at 8157).
**Action**:
```rust
Condition::YouAttackedThisTurn => state.players.get(&ctx.controller)
    .map(|p| p.attacked_this_turn).unwrap_or(false),
Condition::CreatedATokenThisTurn => state.players.get(&ctx.controller)
    .map(|p| p.created_token_this_turn).unwrap_or(false),
// LIMITATION (documented): reuses the existing per-player spells_cast_this_turn,
// which is reset only for the active player each turn (storm semantics). This is
// exact for the realistic case (Mindbreak Trap evaluated against the active
// comboing opponent) but over-counts for a NON-active opponent (their counter is
// last reset at the start of their own turn and accrues across intervening turns).
// No new tracker is added because (a) the brief forbids duplicate trackers when a
// field exists, and (b) 0 roster cards are actually unblocked by this condition
// (Mindbreak Trap stays blocked on the Trap alt-cost). Flag for reviewer.
Condition::OpponentCastNSpells(n) => state.players.iter().any(|(pid, ps)|
    *pid != ctx.controller && !ps.has_lost && ps.spells_cast_this_turn >= *n),
// CR 207.2c / 400.2: graveyard cards use printed characteristics (no layer calc),
// mirroring CardTypesInGraveyardAtLeast.
Condition::SpellMastery => {
    let gy = ZoneId::Graveyard(ctx.controller);
    state.objects.values().filter(|o| o.zone == gy
        && (o.characteristics.card_types.contains(&CardType::Instant)
            || o.characteristics.card_types.contains(&CardType::Sorcery)))
        .count() >= 2
}
// Multiplayer: "an opponent controls more lands than you" (Land Tax / Weathered
// Wayfarer oracle wording) → ANY living opponent with strictly more lands.
Condition::OpponentControlsMoreLandsThanYou => {
    let count_lands = |pid: PlayerId| state.objects.values().filter(|o| {
        o.zone == ZoneId::Battlefield && o.is_phased_in() && o.controller == pid && {
            let chars = crate::rules::layers::calculate_characteristics(state, o.id)
                .unwrap_or_else(|| o.characteristics.clone());
            chars.card_types.contains(&CardType::Land)   // W3-LC: layer-resolved type
        }
    }).count();
    let mine = count_lands(ctx.controller);
    state.players.iter().any(|(pid, ps)| *pid != ctx.controller && !ps.has_lost
        && count_lands(*pid) > mine)
}
```
**CR / discipline**: `OpponentControlsMoreLandsThanYou` uses
`calculate_characteristics` (not raw `obj.characteristics.card_types`) per the W3-LC
audit — a permanent may be a land only via a type-changing effect, and phased-out
permanents (CR 702.26b) are excluded via `is_phased_in()`. "ANY opponent" is
correct per oracle text and the multiplayer-first invariant.

### Change 11 — Exhaustive-match site table (compile-critical)

| File | Match / literal | Action |
|------|-----------------|--------|
| `state/hash.rs` `TriggerCondition` impl (~4903) | exhaustive | +3 arms (disc 45/46/47) |
| `state/hash.rs` `Condition` impl (~4990-5242) | exhaustive | +5 arms (disc 43-47) |
| `state/hash.rs` `TriggerEvent` impl (~2434) | exhaustive | +1 arm (disc 47) |
| `state/hash.rs` `PlayerState` impl (~1387) | field list | +2 field hashes |
| `state/hash.rs:254` `HASH_SCHEMA_VERSION` | const | 32 → 33 + changelog + version-guard test |
| `effects/mod.rs` `check_condition` (7731) | exhaustive | +5 arms |
| `effects/mod.rs` `check_static_condition` (8090) | has `_` fallback | **no change** |
| `testing/replay_harness.rs` `enrich_spec_from_def` (1919) | `if let` chain | +1 `WhenBecomesTarget` block |
| `rules/turn_actions.rs` `execute_turn_based_actions` (19) | match on Step | +`PostCombatMain` arm |
| `rules/turn_actions.rs` `precombat_main_actions` (486) | — | +first-main sweep |
| `rules/turn_actions.rs` new `postcombat_main_actions` | — | new fn + sweep |
| `rules/turn_actions.rs` `reset_turn_state` (1572-1585) | all-players loop | +2 bool resets |
| `rules/combat.rs` `handle_declare_attackers` (32) | — | set `attacked_this_turn` |
| `rules/abilities.rs` `check_triggers` PermanentTargeted arm (4152) | — | +inline becomes-target dispatch |
| `state/mod.rs` `add_object` (361) | — | set `created_token_this_turn` |
| `state/player.rs` `PlayerState` struct (~375) | — | +2 fields |
| `state/builder.rs` PlayerState literal (258-265) | field list | +2 inits |

**No changes** to `tools/tui/*` or `tools/replay-viewer/*` (no new
`StackObjectKind`/`KeywordAbility`). Still run `cargo build --workspace`.

**Harness (`script_schema.rs` / `translate_player_action`)**: **no changes.** No
new cast-cost mechanic or `PlayerAction` field. `WhenBecomesTarget` fires on ordinary
`CastSpell`/`ActivateAbility`. The main-phase triggers fire on phase advancement,
which the JSON harness does not drive (gotcha: "turn-based triggers don't auto-fire
in the harness") — they are covered by unit tests that advance the turn, not by
scripts.

---

## Card Definition Fixes (backfill — runner authors these AFTER engine + review)

**TODO sweep result: 18 candidate cards.** Confidence legend: **CLEAN** = fully
authorable with this batch; **PARTIAL** = trigger/condition unblocked but another
clause still gapped (author what's expressible, keep a narrowed marker);
**BLOCKED** = stays blocked on an out-of-scope primitive.

| Card | File | Marker / oracle | Unblocked by | Confidence |
|------|------|-----------------|--------------|-----------|
| Searslicer Goblin | `searslicer_goblin.rs` | Raid end-step → make 1/1 Goblin | `YouAttackedThisTurn` (intervening-if on `AtBeginningOfYourEndStep`) | **CLEAN** |
| Chart a Course | `chart_a_course.rs` | draw 2, discard unless attacked | `YouAttackedThisTurn` (Conditional + `Not`) | **CLEAN** |
| Bloodsoaked Champion | `bloodsoaked_champion.rs` | {1}{B} return from GY, only if attacked | `YouAttackedThisTurn` (activation_condition, activation_zone GY) | **CLEAN** — *TODO-sweep add* |
| Idol of Oblivion | `idol_of_oblivion.rs` | {T}: draw, only if created a token | `CreatedATokenThisTurn` (activation_condition) | **CLEAN** |
| Dark Petition | `dark_petition.rs` | spell mastery → add {B}{B}{B} | `SpellMastery` (Conditional + AddMana) | **CLEAN** |
| Land Tax | `land_tax.rs` | upkeep, if opp more lands → tutor 3 basics | `OpponentControlsMoreLandsThanYou` (intervening-if) | **CLEAN** |
| Venerated Rotpriest | `venerated_rotpriest.rs` | creature you control targeted by spell → opp poison | `WhenBecomesTarget{scope:Some(creature), by_opponent:false, include_abilities:false}` | **CLEAN** (Toxic already impl) |
| Alesha, Who Laughs at Fate | `alesha_who_laughs_at_fate.rs` | Raid end-step reanimate MV≤power | `YouAttackedThisTurn` (intervening-if); reanimate-with-MV≤source-power target constraint may be partial | **PARTIAL** |
| Raider's Wake | `raiders_wake.rs` | Raid end-step | `YouAttackedThisTurn` | **PARTIAL** — *TODO-sweep add*; verify full effect |
| Goldspan Dragon | `goldspan_dragon.rs` | attacks OR targeted-by-spell → Treasure; + Treasure-buff static | `WhenBecomesTarget{scope:None,false,false}` (dual with `WhenAttacks`); Treasure-granting static (`static_grant_filter`) may be gapped | **PARTIAL** |
| Bonecrusher Giant | `bonecrusher_giant.rs` | targeted-by-spell → 2 dmg to that spell's controller | `WhenBecomesTarget{scope:None,false,false}` + `ControllerOf(DeclaredTarget{0})`; verify Adventure (`// Stomp`) status | **PARTIAL** (fix wrong `WhenBecomesTargetByOpponent`) |
| Black Market | `black_market.rs` | first main: add {B} per charge counter | `AtBeginningOfFirstMainPhase` + AddMana-per-counter | **PARTIAL** |
| Black Market Connections | `black_market_connections.rs` | first main, choose one or more (modal) | `AtBeginningOfFirstMainPhase`; **modal-on-trigger gap** | **BLOCKED** (modal trigger) |
| Ripples of Undeath | `ripples_of_undeath.rs` | first main: mill 3, may pay 1 life to return one milled | `AtBeginningOfFirstMainPhase`; mill-tracking conditional return gapped | **BLOCKED** |
| Florian, Voldaren Scion | `florian_voldaren_scion.rs` | postcombat main: impulse X = opps' life lost | `AtBeginningOfPostcombatMain`; needs opponents'-life-lost X + impulse-play | **BLOCKED** |
| Tymna the Weaver | `tymna_the_weaver.rs` | postcombat main: pay X life = opps dealt combat dmg, draw X | `AtBeginningOfPostcombatMain`; needs "opponents dealt combat damage this turn" tracker | **BLOCKED** |
| Scalelord Reckoner | `scalelord_reckoner.rs` | Dragon you control targeted by opp → destroy that player's nonland | `WhenBecomesTarget{Some(Dragon),true,true}`; "that player's permanent" target restriction gapped | **BLOCKED** |
| Tectonic Giant | `tectonic_giant.rs` | attacks OR targeted-by-opp-spell → modal | `WhenBecomesTarget{None,true,false}`; **modal-on-trigger gap** | **BLOCKED** |
| Flowerfoot Swordmaster | `flowerfoot_swordmaster.rs` | Valiant: self targeted by spell/ability you control, first time each turn | needs you-control + first-time-each-turn (out of scope) | **BLOCKED** |
| Kaito Shizuki | `kaito_shizuki.rs` | +1 draw/discard-unless-attacked | `YouAttackedThisTurn` would help but card is a **planeswalker** (known gap) | **BLOCKED** (planeswalker) |
| Minas Tirith | `minas_tirith.rs` | activate only if attacked with **2+** creatures | needs count-based condition (bool insufficient) | **BLOCKED** |
| Battle Cry Goblin | `battle_cry_goblin.rs` | Pack tactics (attacked with total power 2+) | needs power/count-based condition | **BLOCKED** |
| Mindbreak Trap | `mindbreak_trap.rs` | Trap: if opp cast 3+, pay {0} | `OpponentCastNSpells(3)` exists but needs **Trap alt-cost** | **BLOCKED** (Trap) |

**Confirmed CLEAN yield: 7.** (Brief's discounted-yield ~18 is optimistic; per the
"planners overcount 2-3×" calibration, most first-main/postcombat/becomes-target
cards carry a second gapped clause.) The runner should author the 7 CLEAN cards,
attempt the 5 PARTIAL cards (authoring the expressible clauses and leaving a
**narrowed** marker for the residual gap), and leave BLOCKED cards with corrected
markers naming the true remaining blocker. Delete stale markers on fully-cleaned
cards.

---

## Unit Tests

**File**: `crates/engine/tests/pb_ac6_phase_action_conditions.rs` (new)

Trigger-condition tests (drive real phase advancement — the harness/script layer
cannot, per the turn-based-trigger gotcha):
- `test_first_main_phase_trigger_fires_once` — place a permanent with an
  `AtBeginningOfFirstMainPhase` CardDef trigger; advance to `Step::PreCombatMain`;
  assert the trigger queued exactly once (CR 505.1a / 603.2b).
- `test_postcombat_main_trigger_fires` — assert an `AtBeginningOfPostcombatMain`
  trigger fires on `Step::PostCombatMain` and NOT on `Step::PreCombatMain`
  (CR 505.1a).
- `test_first_main_trigger_only_active_player` — multiplayer: a non-active player's
  first-main trigger does NOT fire on the active player's first main (CR 505.1 —
  "your").
- `test_becomes_target_self_by_spell` — Goldspan-shape (`scope:None`,
  `by_opponent:false`, `include_abilities:false`): cast a spell targeting the
  creature → trigger fires; targeting by an ability → does NOT fire (CR 601.2c vs
  602.2b spell-only).
- `test_becomes_target_scope_you_control` — Rotpriest-shape (`scope:Some(creature)`):
  a spell targeting ANOTHER creature you control fires the trigger on Rotpriest; a
  spell targeting an opponent's creature does not (CR 603.2).
- `test_becomes_target_by_opponent_gate` — `by_opponent:true`: your own spell
  targeting the creature does NOT fire; an opponent's does (CR 702.21a analog).
- `test_becomes_target_fires_at_announcement_not_resolution` — assert the trigger is
  on the stack ABOVE the targeting spell before the spell resolves (CR 601.2c).

Condition tests (use `check_condition` directly + integration):
- `test_you_attacked_this_turn` — false pre-combat; declare attackers; true; reset
  at turn boundary → false (Raid, CR 508.1). Negative: token entering attacking
  (CR 508.4) does NOT set it.
- `test_created_a_token_this_turn` — false; create a token via `add_object`; true;
  reset → false (CR 111.10). Verify it's set regardless of which emission path.
- `test_spell_mastery_two_instants_or_sorceries` — 0/1 in GY → false; 2 → true;
  mix of 1 instant + 1 sorcery → true; 2 creatures → false (CR 207.2c).
- `test_opponent_controls_more_lands` — equal lands → false; opponent +1 → true;
  phased-out opponent land excluded (CR 702.26b); type-changed nonland-→-land via a
  continuous effect counts (W3-LC layer discipline).
- `test_opponent_cast_n_spells` — active opponent casts 3 spells this turn → true at
  N=3, false at N=4. (Document the non-active-opponent limitation in a comment.)

Card integration tests (name the CR they cite):
- `test_searslicer_goblin_raid` — attack, pass to end step, assert Goblin token
  created; no attack → no token (Raid, CR 508.1).
- `test_bloodsoaked_champion_return` — attacked → activation legal from GY; not
  attacked → illegal (CR 602.5b activation condition).
- `test_idol_of_oblivion_token_gate` — no token → draw ability illegal; after
  creating a token → legal (CR 111.10).
- `test_dark_petition_spell_mastery` — 2 instants in GY → {B}{B}{B} added; else not
  (CR 207.2c).
- `test_land_tax_intervening_if` — opponent has more lands → upkeep trigger fires &
  re-checks at resolution (CR 603.4); equalize lands in response → fizzle.
- `test_venerated_rotpriest_becomes_target` — target your own creature with a spell
  → target opponent gets a poison counter (CR 601.2c).

**Hash tests** (in the same file or `tests/hash_*`):
- `test_hash_sensitive_attacked_this_turn`, `test_hash_sensitive_created_token_this_turn`
  (mutation-verified, per Change 1).
- Update the `HASH_SCHEMA_VERSION == 33` guard.

**Pattern references**: main-phase trigger tests → mirror upkeep/end-step trigger
tests (search `tests/` for `AtBeginningOfYourUpkeep` / B9/B14 sweep tests).
Becomes-target tests → mirror Ward tests (`SelfBecomesTargetByOpponent`) and PB-N
`WheneverCreatureYouControlAttacks` tests for the enrich-then-place setup.

---

## Verification Checklist

- [ ] `cargo check -p mtg-engine` (engine primitive compiles)
- [ ] `HASH_SCHEMA_VERSION == 33`; version-guard test updated
- [ ] Both hash mutation-tests pass (flip field → hash changes)
- [ ] All 7 CLEAN card TODOs resolved; markers deleted
- [ ] 5 PARTIAL cards authored where expressible with narrowed markers
- [ ] BLOCKED cards' markers corrected to name the true remaining blocker
- [ ] `cargo test --all`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo build --workspace` (confirms tui/replay-viewer still build — no SOK/KW
      change expected, but verify)
- [ ] No remaining stale TODOs on cleaned card defs

---

## Risks & Edge Cases

- **HIGH (recurring)**: forgetting `attacked_this_turn` / `created_token_this_turn`
  in `PlayerState` HashInto AND in `reset_turn_state`. This exact omission was a
  review HIGH in PB-AC1 and PB-AC5. Both mutation-tests + the reset are mandatory.
- **`OpponentCastNSpells` — PLAN CORRECTION (worker, 2026-07-09). OVERRIDES the
  design in Change 10.** The plan proposed reusing `PlayerState::spells_cast_this_turn`
  and accepting an over-count for non-active opponents. **Rejected.** Verified in code:
  `turn_actions.rs:1555` resets that field only for the incoming active player, and the
  comment at `turn_actions.rs:1568` states the exclusion is deliberate (storm scoping).
  So for a non-active opponent the counter accumulates across intervening turns and
  `OpponentCastNSpells(n)` would return **wrong game state** — which violates the W6
  policy ("no partial implementations, no wrong game state") and fails acceptance
  criterion 4336 verbatim ("this-turn trackers reset at **correct** turn boundaries").
  Low roster yield is not a licence to ship an incorrect primitive.

  **Required instead**: add a dedicated `PlayerState::spells_cast_this_game_turn: u32`,
  incremented at the same four sites as `spells_cast_this_turn`
  (`resolution.rs:5133`, `resolution.rs:5787`, `copy.rs:462`, `copy.rs:688`), hashed in
  `PlayerState::hash_into`, and **reset for ALL players** in the existing all-players
  loop of `reset_turn_state` (alongside `cards_drawn_this_turn` / `life_lost_this_turn`,
  which already have exactly this multiplayer-correct shape). `OpponentCastNSpells(n)`
  reads the new field. Do NOT change `spells_cast_this_turn`, `storm_count`, or
  `previous_turn_spells_cast` — see OOS-AC6-1.
- **`WhenBecomesTarget` production-enrich dependency**: becomes-target triggers must
  live in `obj.characteristics.triggered_abilities` (populated by
  `enrich_spec_from_def`); `calculate_characteristics` does not re-enrich from the
  registry. This is the SAME model as Ward / `WheneverCreatureYouControlAttacks`, so
  tests must place the creature on the battlefield pre-enriched (standard builder
  pattern). Cast-then-target coverage is a pre-existing baseline shared by all
  event-driven CardDef triggers, not introduced here.
- **Multi-slot targeting** (CR 603.2c): one trigger per `PermanentTargeted` event =
  one per target slot; a spell targeting the same permanent twice fires twice. Kept
  consistent with existing Ward behavior; documented, not changed.
- **Spell-vs-ability detection** relies on looking up `targeting_stack_id` in
  `state.stack_objects` and matching `StackObjectKind::Spell`. If the stack object
  is not found (already resolved/removed — shouldn't happen at announcement), the
  code conservatively treats it as not-a-spell; ensure the lookup happens while the
  targeting object is still on the stack (it is, at `PermanentTargeted` emission).
- **Modal-on-trigger gap** blocks Black Market Connections and Tectonic Giant even
  with the new triggers — do NOT force-author these; leave narrowed markers.
- **Adventure** (Bonecrusher Giant) and **planeswalker** (Kaito) are independent
  gaps; verify before claiming Bonecrusher clean.
- **`is_phased_in()` + `calculate_characteristics`** discipline is mandatory in
  `OpponentControlsMoreLandsThanYou` (W3-LC) — raw `obj.characteristics.card_types`
  would miss type-changed lands and count phased-out permanents.
- **`add_object` token chokepoint** assumes every token creation sets `is_token` and
  enters via `add_object`. Verified across all 13 `GameEvent::TokenCreated` sites;
  the merged-permanent split path (`move_object_to_zone`) is intentionally excluded
  (not "creating" a token).

---

## Out-of-scope seeds surfaced while verifying the plan (worker, 2026-07-09)

- **OOS-AC6-1 — `storm_count` is multiplayer-incorrect (pre-existing, NOT fixed here).**
  CR 702.40a: storm copies the spell "for each spell cast before it this turn" — by
  **any** player. `copy.rs:291-297` reads only `caster.spells_cast_this_turn`, so storm
  under-counts opponents' spells. Compounding: because that field is reset only for the
  incoming active player (`turn_actions.rs:1555`), a storm spell cast at instant speed on
  an opponent's turn (e.g. **Brain Freeze**, an Instant with Storm) also **over-counts**
  the caster's own stale spells from their previous turn. Two bugs that partially mask
  each other. Fixing requires deciding the reset semantics jointly with CR 730.2
  (`previous_turn_spells_cast`, Daybound), which reads the same field — hence out of
  scope for PB-AC6. PB-AC6 sidesteps both by adding a separate, all-players-reset
  `spells_cast_this_game_turn` rather than mutating the storm field.
