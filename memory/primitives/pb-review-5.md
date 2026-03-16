# Primitive Batch Review: PB-5 -- Targeted Activated/Triggered Abilities

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 115.1 (targeting), CR 601.2c (target declaration), CR 602.2 (activated abilities), CR 603.1/603.3d (triggered abilities), CR 608.2b (fizzle rule)
**Engine files reviewed**: `card_definition.rs` (TargetRequirement enum, targets field on Activated/Triggered/Spell/LoyaltyAbility/SagaChapter), `abilities.rs` (flush_pending_triggers, handle_activate_ability target validation), `resolution.rs` (TriggeredAbility resolution path, fizzle check), `casting.rs` (validate_targets), `mod.rs` (validate_target_protection), `hash.rs` (HashInto for TargetRequirement), `effects/mod.rs` (DeclaredTarget resolution), `targeting.rs` (Target/SpellTarget)
**Card defs reviewed**: 25 card defs with targeted Activated or Triggered abilities

## Verdict: needs-fix

The engine infrastructure for PB-5 is solid: `targets: Vec<TargetRequirement>` on Activated and Triggered ability definitions, target validation via `validate_targets()` at activation time, hash support, and good test coverage for activated ability target validation. However, there are two significant issues: (1) triggered abilities with targets from CardDef do not get their targets populated when pushed onto the stack -- the trigger_targets logic in flush_pending_triggers only handles hardcoded cases (Ward, Provoke, Exalted, etc.), not general CardDef targets; and (2) the TriggeredAbility resolution path does not perform CR 608.2b fizzle checks on declared targets. Several card defs named in the batch spec (mother_of_runes, skrelv_defector_mite, yavimaya_hollow, zealous_conscripts, gilded_drake, fell_stinger) still have their abilities completely absent with TODO comments, which is expected since they require primitives beyond just targeting (color choice, control exchange, regenerate, exploit triggers). Multiple card defs have over-permissive target filters documented as TODOs.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `abilities.rs:6190-6228` | **Triggered ability targets not populated from CardDef.** `flush_pending_triggers` only populates targets from hardcoded trigger metadata (ward, provoke, exalted, annihilator, opponent-cast). CardDef-based triggered abilities with `targets: vec![TargetRequirement::...]` (e.g. Nullpriest, Bladewing, Emeria) get pushed onto the stack with empty targets, making DeclaredTarget{index:0} resolve to nothing. **Fix:** When building `trigger_targets` for `PendingTriggerKind::Normal`, look up the CardDef's Triggered ability and if it has non-empty `targets`, auto-select a legal target (MVP deterministic fallback -- first legal object matching the requirement). This parallels how the replay harness auto-targets for spells. |
| 2 | **MEDIUM** | `resolution.rs:2008-2048` | **No CR 608.2b fizzle check for CardDef triggered abilities.** The TriggeredAbility CardDef resolution path executes the effect without checking `is_target_legal` on `stack_obj.targets`. Per CR 608.2b, if all targets of a triggered ability are illegal at resolution, the ability should be removed without effect. Currently the effect silently no-ops (DeclaredTarget returns empty vec), which produces the correct game state but does not emit a fizzle event. **Fix:** Before executing the CardDef effect, check if `stack_obj.targets` is non-empty and all targets are illegal via `is_target_legal`; if so, skip execution and emit an appropriate event. |
| 3 | **MEDIUM** | `abilities.rs:6190-6228` | **No target validation for triggered abilities at trigger time.** CR 603.3d requires targets to be chosen as the triggered ability is put on the stack. The current code does not call `validate_targets` for triggered abilities -- it just pushes whatever targets are in the hardcoded logic. When Finding 1 is fixed to auto-select targets from CardDef requirements, the selection must also validate via `validate_targets` (checking protection, hexproof, etc.). **Fix:** After selecting targets for CardDef triggered abilities, pass them through `casting::validate_targets` before pushing onto the stack. If no legal target exists, the triggered ability should not go onto the stack at all (CR 603.3d). |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | `forerunner_of_slaughter.rs:30` | **Over-permissive target.** Oracle says "Target colorless creature" but def uses `TargetRequirement::TargetCreature` (any creature). TODO documented. **Fix:** Add `is_colorless: bool` to TargetFilter, or use `exclude_colors` with all 5 colors set. Deferred to a future PB if no TargetFilter field exists. |
| 5 | MEDIUM | `blinkmoth_nexus.rs:85` | **Over-permissive target.** Oracle says "Target Blinkmoth creature" but def uses `TargetRequirement::TargetCreature`. TODO documented. **Fix:** Use `TargetCreatureWithFilter(TargetFilter { has_subtype: Some(SubType("Blinkmoth")), .. })`. |
| 6 | MEDIUM | `boseiju_who_endures.rs:63` | **Over-permissive target.** Oracle says "target artifact, enchantment, or nonbasic land an opponent controls" but def uses `TargetRequirement::TargetPermanent` (any permanent including creatures and basic lands, and no controller restriction). TODO documented. **Fix:** Add `TargetPermanentWithFilter` with appropriate has_card_types OR logic and `controller: TargetController::Opponent`. Requires TargetFilter to support OR-semantics on card types (has_card_types field exists). |
| 7 | MEDIUM | `otawara_soaring_city.rs:36` | **Over-permissive target.** Oracle says "target artifact, creature, enchantment, or planeswalker" (excludes lands) but def uses `TargetRequirement::TargetPermanent`. TODO documented. **Fix:** Use `TargetPermanentWithFilter(TargetFilter { non_land: true, .. })`. |
| 8 | MEDIUM | `eiganjo_seat_of_the_empire.rs:34` | **Over-permissive target.** Oracle says "target attacking or blocking creature" but def uses `TargetRequirement::TargetCreature` (any creature). TODO documented. **Fix:** Requires an attacking/blocking filter on TargetFilter (DSL gap). |
| 9 | MEDIUM | `boseiju_who_endures.rs:48-51` | **Search filter too restrictive.** Oracle says opponent "may search for a land card with a basic land type" but def uses `basic: true` which means only Basic lands. Shock lands, triomes, etc. with basic land types would be excluded. **Fix:** Add `has_basic_land_type: bool` to TargetFilter (DSL gap) or document as known inaccuracy. |
| 10 | LOW | `ghost_quarter.rs:52` | **Unnecessarily complex target.** Oracle says "Destroy target land" -- uses `TargetPermanentWithFilter(TargetFilter { has_card_type: Some(CardType::Land) })` when `TargetLand` variant exists and is simpler. **Fix:** Replace with `TargetRequirement::TargetLand`. |
| 11 | LOW | `bladewing_the_risen.rs:41` | **Missing activated ability.** Oracle says "{B}{R}: Dragon creatures get +1/+1 until end of turn" -- this pump ability is absent. TODO documented. Requires creature-type-filtered temporary pump (separate DSL gap). |
| 12 | LOW | `emeria_the_sky_ruin.rs:35` | **Missing intervening-if condition.** Oracle says "if you control seven or more Plains" -- this condition is absent (trigger fires unconditionally). TODO documented. Requires count-threshold condition (separate DSL gap). |
| 13 | LOW | `teneb_the_harvester.rs:23` | **Missing optional mana payment.** Oracle says "you may pay {2}{B}" -- trigger fires unconditionally without mana payment. TODO documented. Requires triggered-ability cost or PayManaOrElse effect (separate DSL gap). |
| 14 | LOW | `reanimate.rs:17` | **Missing life loss.** Oracle says "You lose life equal to its mana value" -- only the MoveZone effect is implemented. TODO documented. Requires EffectAmount::ManaValueOfTarget (separate DSL gap). |
| 15 | LOW | `haven_of_the_spirit_dragon.rs:40,57` | **Incomplete target filter.** Oracle says "Dragon creature card or Ugin planeswalker card" but only Dragon creature cards are targeted. TODO documented. Requires OR-union of name+type filters (DSL gap). |

### Finding Details

#### Finding 1: Triggered ability targets not populated from CardDef

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:6190-6228`
**CR Rule**: CR 603.3d -- "If the triggered ability has any targets, its controller announces the legal target(s)."
**Issue**: When `flush_pending_triggers` pushes a `PendingTriggerKind::Normal` triggered ability onto the stack, the `trigger_targets` vec is built only from hardcoded cases (Ward targeting_stack_id, Provoke provoke_target_creature, Exalted exalted_attacker_id, etc.). For CardDef-defined triggered abilities like Nullpriest of Oblivion's ETB ("return target creature card from your graveyard to the battlefield"), the targets vec ends up empty. The effect then uses `EffectTarget::DeclaredTarget { index: 0 }` which resolves to nothing, making the ability a no-op instead of returning a creature.
**Fix**: In the `else` branch at line 6227 (the fallback that returns `vec![]`), add logic to: (1) look up the PendingTrigger's source object's CardDef, (2) find the matching Triggered ability by index, (3) if it has non-empty `targets: Vec<TargetRequirement>`, auto-select legal targets using the same deterministic fallback used elsewhere (first legal match), (4) validate via `casting::validate_targets`. If no legal target exists, skip pushing the trigger onto the stack.

#### Finding 2: No fizzle check for CardDef triggered abilities at resolution

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:2008-2048`
**CR Rule**: CR 608.2b -- "If the spell or ability specifies targets, it checks whether the targets are still legal."
**Issue**: The CardDef triggered ability resolution path (lines 2008-2048) checks the intervening-if condition but does not check whether declared targets are still legal. If the target is removed in response (e.g., an opponent exiles the creature card from graveyard before Nullpriest's trigger resolves), the ability should fizzle. Currently the DeclaredTarget resolution silently returns an empty vec, producing the correct game state (no effect) but without proper fizzle event emission.
**Fix**: Before the `if condition_holds {` block at line 2026, add: if `!stack_obj.targets.is_empty()` and all targets are illegal per `is_target_legal()`, skip execution and emit `AbilityResolved` (or a new `AbilityFizzled` event).

#### Finding 3: No target validation for triggered abilities at trigger time

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6190-6228`
**CR Rule**: CR 603.3d -- "As the ability is put on the stack, its controller [...] chooses targets."
**Issue**: Even when Finding 1 is fixed to auto-select targets, those targets must be validated against protection, hexproof, shroud (CR 702.16b, 702.11a, 702.18a) at the time the trigger goes on the stack. The current code has no such validation for triggered abilities.
**Fix**: After auto-selecting targets, call `casting::validate_targets()` with the source object's characteristics. If validation fails (no legal target), do not push the trigger onto the stack.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 115.1 | Yes | Yes | TargetRequirement enum covers all target phrases |
| CR 115.1c | Yes | Yes | Activated ability targets validated via validate_targets |
| CR 115.1d | Partial | No | Triggered ability targets NOT populated from CardDef (Finding 1) |
| CR 601.2c | Yes | Yes | validate_targets called for activated abilities |
| CR 602.2b | Yes | Yes | Activated abilities follow spell-like target validation |
| CR 603.3d | No | No | Triggered ability target selection missing (Finding 1, 3) |
| CR 608.2b | Partial | No | Fizzle check for triggered abilities missing (Finding 2); works for spells |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| mother_of_runes | No | 2 | Wrong -- no ability | Color choice DSL gap |
| skrelv_defector_mite | No | 2 | Wrong -- no ability, no CantBlock | Color choice + complex grant DSL gap |
| yavimaya_hollow | No | 1 | Wrong -- no regenerate ability | Regenerate DSL gap |
| zealous_conscripts | No | 2 | Wrong -- no ETB trigger | Control change + untap DSL gap |
| gilded_drake | No | 1 | Wrong -- no ETB trigger | Exchange control DSL gap |
| fell_stinger | No | 1 | Wrong -- no exploit trigger | Targeted exploit trigger DSL gap |
| access_tunnel | Yes | 0 | Yes | Fully implemented |
| reanimate | Partial | 1 | Partial -- move correct, life loss missing | EffectAmount::ManaValueOfTarget DSL gap |
| goblin_motivator | Yes | 0 | Yes | Fully implemented |
| slayers_stronghold | Yes | 0 | Yes | Fully implemented |
| flamekin_village | Yes | 0 | Yes | Fully implemented |
| rogues_passage | Yes | 0 | Yes | Fully implemented |
| hanweir_battlements | Yes | 0 | Yes | Targeting correct; meld separate |
| blinkmoth_nexus | Partial | 1 | Partial -- target too permissive | Missing Blinkmoth subtype filter |
| ghost_quarter | Yes | 0 | Yes | Works correctly (TargetLand preferred) |
| forerunner_of_slaughter | Partial | 1 | Partial -- target too permissive | Missing colorless filter |
| haven_of_the_spirit_dragon | Partial | 1 | Partial -- missing Ugin targeting | Name+type union DSL gap |
| buried_ruin | Yes | 0 | Yes | Fully implemented |
| hall_of_heliods_generosity | Yes | 0 | Yes | Fully implemented |
| boseiju_who_endures | Partial | 2 | Partial -- target+search too permissive/restrictive | Multiple DSL gaps |
| otawara_soaring_city | Partial | 2 | Partial -- target too permissive | Missing non_land filter + cost reduction |
| eiganjo_seat_of_the_empire | Partial | 2 | Partial -- target too permissive | Missing attacking/blocking filter + cost reduction |
| nullpriest_of_oblivion | Yes* | 0 | Partial* | Target correct but not populated at trigger time (Finding 1) |
| bladewing_the_risen | Partial | 1 | Partial -- target correct, pump missing | Dragon pump DSL gap |
| emeria_the_sky_ruin | Partial | 2 | Partial -- no Plains count check | Count-threshold condition DSL gap |
| bloodline_necromancer | Yes* | 0 | Partial* | Target correct but not populated at trigger time (Finding 1) |
| den_protector | Partial | 1 | Partial* | Target correct, evasion missing; trigger targets not populated |
| briarblade_adept | Yes* | 0 | Partial* | Target correct but not populated at trigger time (Finding 1) |
| teneb_the_harvester | Partial | 1 | Partial -- no mana payment | Optional payment DSL gap |

*Cards marked with asterisk have correctly declared targets on triggered abilities but those targets are not actually populated when the trigger goes on the stack (Finding 1).

## Previous Findings (re-review only)

N/A -- first review.
