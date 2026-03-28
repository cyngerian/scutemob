# Primitive WIP: PB-35 -- Modal triggers + graveyard conditions + planeswalker abilities

batch: PB-35
title: Modal triggers + graveyard conditions + planeswalker abilities
cards_affected: ~60
started: 2026-03-28
phase: closed
plan_file: memory/primitives/pb-plan-35.md

## Gap Groups
- G-27: Modal triggered abilities (~26 cards) — modal choice on trigger resolution, extend B11 modal support to triggered abilities
- G-29: Graveyard recursion conditions (~23 cards) — MoveZone + conditions, some need ActivationCondition extensions
- G-30: Planeswalker remaining (~11 cards) — individual PW loyalty abilities, framework exists (PB-14)

## Deferred from Prior PBs
- G-25 partial (Springleaf Drum, Cryptolith Rite, Faeburrow Elder, Arena of Glory) deferred to PB-37
- G-24 (Nykthos, Three Tree City) deferred — requires Command::ChooseColor (M10)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - ActivationZone enum + activation_zone on AbilityDefinition::Activated + ActivatedAbility
  - TriggerZone enum + trigger_zone + modes on AbilityDefinition::Triggered
  - hash.rs updated (ActivationZone, TriggerZone, ActivatedAbility.activation_zone, Triggered modes/trigger_zone)
  - handle_activate_ability extended for graveyard activation
  - collect_graveyard_carddef_triggers() added + called from check_triggers for PermanentEnteredBattlefield
  - flush_pending_triggers: modes_chosen = [0] set for modal triggered abilities (CR 700.2b)
  - resolution.rs: modal dispatch uses modes_chosen to select chosen mode effects
  - replay_harness.rs: WhenDies/WhenAttacks/WhenDealsCombatDamageToPlayer conversions updated for modal (mode 0 fallback)
  - helpers.rs: exports ActivationZone, TriggerZone
- [x] 2. Card definition fixes
  - G-29: reassembling_skeleton.rs — ActivationZone::Graveyard, return to BF tapped
  - G-29: bloodghast.rs — TriggerZone::Graveyard landfall trigger, return to BF
  - G-29: earthquake_dragon.rs — ActivationZone::Graveyard, sacrifice land cost
  - G-29: cult_conscript.rs — ActivationZone::Graveyard (without condition, deferred)
  - G-27: retreat_to_kazandu.rs — modal landfall trigger (modes: counter or gainlife)
  - G-27: retreat_to_coralhelm.rs — modal landfall trigger (modes: untap or scry)
  - G-27: felidar_retreat.rs — modal landfall trigger (modes: token or mass counter+vigilance)
  - G-27: junji_the_midnight_sky.rs — modal WhenDies trigger (modes: discard+life or reanimate)
  - G-27: shambling_ghast.rs — modal ETB trigger (modes: treasure or -1/-1 counter)
  - G-27: tectonic_giant.rs — modal WhenAttacks trigger (modes: damage or Nothing)
  - G-27: hullbreaker_horror.rs — modal WheneverYouCastSpell trigger (min_modes: 0)
  - G-27: glissa_sunslayer.rs — modal combat damage trigger (3 modes)
  - G-27: goblin_cratermaker.rs — modal activated ability using Effect::Choose
  - G-27: umezawas_jitte.rs — full modal activated ability using Effect::Choose
- [x] 3. New card definitions (if any) — none needed
- [x] 4. Unit tests
  - modal_triggers.rs: 6 tests (structure checks for Retreat to Kazandu, Felidar Retreat, Junji, Shambling Ghast, Glissa Sunslayer, Hullbreaker Horror)
  - graveyard_abilities.rs: 5 tests (Reassembling Skeleton structure/activation, zone check, Bloodghast structure, Earthquake Dragon sacrifice cost)
  - Total new tests: 11 (2419 total, 0 failures)
- [x] 5. Workspace build verification — clean (0 errors, 0 warnings, 0 clippy, fmt OK)

## Review
findings: 9 (HIGH: 1, MEDIUM: 3, LOW: 5)
verdict: needs-fix
review_file: memory/primitives/pb-review-35.md

## Fix Phase (2026-03-28)
- [x] HIGH-1: shambling_ghast.rs — WhenEntersBattlefield → WhenDies; header comment updated
- [x] MEDIUM-2: shambling_ghast.rs — TargetCreature → TargetCreatureWithFilter(controller: Opponent)
- [x] MEDIUM-3: bloodghast.rs — Added TODO comment about "you may return" being non-optional
- [x] MEDIUM-4: umezawas_jitte.rs — TODO(PB-37): prefix already present; no change needed
- [x] Test fix: modal_triggers.rs — test_modal_etb_trigger_structure renamed to test_modal_death_trigger_structure_shambling_ghast, checks WhenDies
- [x] All tests pass (0 failed); 0 clippy warnings
