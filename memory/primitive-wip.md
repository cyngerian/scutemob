# Primitive WIP: PB-EF3b — granted keyword-triggers fire (Melee / Battle Cry / Annihilator)

batch: PB-EF3b
title: Synthesize the keyword-derived triggered ability when a trigger-keyword (Melee, Battle Cry, Annihilator) is granted by a continuous effect (LayerModification::AddKeyword), not only from printed keywords. Closes EF-W-MISS-3.
task: scutemob-104
branch: feat/pb-ef3b-granted-keyword-triggers-fire-meleebattle-cryannihil
started: 2026-07-18
phase: implement
plan_file: memory/primitives/pb-plan-EF3b.md

## Source findings
- memory/card-authoring/w-miss-roster-2026-07-17.md — EF-W-MISS-3 (line 174: "Granted keyword-trigger is a silent no-op")
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF3b section (line 267)

## Recon (done by coordinator/worker before planning) — VERIFY before implementing

### The bug
- crates/engine/src/state/builder.rs synthesizes a derived TriggeredAbilityDef for each
  PRINTED keyword: Annihilator (~478-505), Battle Cry (~506-539), Melee (~602-627). These land
  in the object's base triggered_abilities.
- crates/engine/src/rules/layers.rs:1165 LayerModification::AddKeyword(kw) does
  chars.keywords.insert(kw) and NOTHING else. A granted trigger-keyword never gets a derived
  TriggeredAbilityDef, so the trigger silently never fires. Static/evasion keywords (flying,
  haste) work because they need no derived trigger.

### Keyword model = SET (crucial for no-double-fire)
- Characteristics.keywords is OrdSet<KeywordAbility> (game_object.rs:768, card_definition.rs:3736).
  Redundant instances collapse to ONE entry. So printed Melee + granted Melee = one Melee in
  the set. CR 702.121b / 702.91b / 702.86b ("each instance triggers separately") is NOT
  representable under this model -- reconcile to EXACTLY ONE derived trigger per trigger-keyword.
  Consistent with how the engine already models keyword redundancy. Document as a known modeling
  limitation; the no-double-fire decoy proves single fire.

### How triggers are collected (why appending to RESOLVED chars is the right layer)
- collect_triggers_for_event (abilities.rs ~6112) reads expect_characteristics(state, obj_id)
  (post-layers), iterates resolved_chars.triggered_abilities, pushes a PendingTrigger with
  embedded_effect: trigger_def.effect.clone() and ability_index = idx-into-resolved.
  => Appending the synthesized def to RESOLVED characteristics makes Battle Cry (ForEach) and
     Annihilator (SacrificePermanents) resolve normally via their embedded effect. Melee has
     effect: None and needs kind-tagging (below).

### The Melee tagging hazard (raw vs resolved ability_index)
- In GameEvent::AttackersDeclared (abilities.rs ~3652-3668) the Melee kind-tag reads RAW
  obj.characteristics.triggered_abilities.get(t.ability_index) (via state.fizzle_object), but
  t.ability_index indexes RESOLVED chars. For a granted-only Melee the synthesized def exists
  only in resolved, so the raw lookup returns None -> tag skipped -> Melee fires as a plain
  effect:None trigger and does nothing. FIX: switch the Melee tag (and audit the Myriad ~3590
  and Provoke ~3613 tags for the same raw-read) to expect_characteristics(state, t.source).
  For PRINTED keywords raw==resolved at that index, so no regression.

### Proposed fix shape
1. Extract shared helper derived_attack_trigger_for_keyword(kw: &KeywordAbility) ->
   Option<TriggeredAbilityDef> returning the exact def builder.rs builds (Melee / BattleCry /
   Annihilator(n); None otherwise). Call from builder.rs (printed) AND from layers reconciliation
   (granted) so shapes never drift.
2. Post-layer reconciliation in calculate_characteristics (layers.rs): after all layers applied,
   for each trigger-keyword in final chars.keywords, if chars.triggered_abilities has no derived
   def for it (identify by description prefix + effect signature), append
   derived_attack_trigger_for_keyword(kw). Handles printed (present -> skip), granted-only
   (absent -> append), printed+granted (present -> skip, no double). Humility/RemoveAllAbilities
   clear keywords -> nothing appended -> correct.
3. Fix the Melee (and audit Myriad/Provoke) tag reads to resolved chars.
4. NO new DSL type, NO schema bump expected. PROVE it: if PROTOCOL_SCHEMA_FINGERPRINT / HASH
   gates stay green with no version edit, no bump. Justify in writing if a bump proves necessary.

### Candidate cards (verify chain vs oracle via MCP -- feedback_verify_full_chain)
- Neither adriana*.rs nor skyhunter*.rs exists yet -> author fresh, not flip.
- Adriana, Captain of the Guard {3}{R}{W} Legendary Creature -- Human Knight, 4/4.
  "Melee. Other creatures you control have melee." Printed Melee on self + unconditional
  continuous anthem granting Melee to other creatures you control.
- Skyhunter Strike Force {2}{W} Creature -- Cat Knight, 2/2. "Flying. Melee. Lieutenant --
  As long as you control your commander, other creatures you control have melee." CHECK whether
  a "control your commander" (Lieutenant) conditional-grant condition primitive exists. If not,
  Skyhunter is BLOCKED on that condition -> truthfully mark, don't ship wrong.
  (Olivia Crimson Bride out of scope -- bespoke trigger, not a keyword grant.)

## Steps (from plan pb-plan-EF3b.md — unchecked)
- [x] Change 1: add shared helper `derived_attack_trigger_for_keyword` in builder.rs (verbatim defs)
- [x] Change 2: route builder's 3 inline printed-keyword blocks through the helper
- [x] Change 3: post-layer reconciliation in calculate_characteristics (L435→436), description-dedup
- [x] Change 4: fix Melee + Myriad + Provoke tag reads raw→resolved (calculate_characteristics, None-tolerant)
- [x] Change 5: confirm NO exhaustive-match sites (cargo build --workspace)
- [x] Card: adriana_captain_of_the_guard.rs NEW → Complete (printed Melee + OtherCreaturesYouControl anthem)
- [x] Card: skyhunter_strike_force.rs NEW → partial (Flying+Melee; Lieutenant omitted; OOS-EF3b-1)
- [x] Tests: crates/engine/tests/primitives/pb_ef3b_granted_keyword_triggers.rs (+ mod line), 8 tests w/ CR + non-vacuity
- [x] Prove NO schema bump (PROTOCOL/HASH untouched, gates green)
- [x] File OOS-EF3b-1, OOS-EF3b-2
