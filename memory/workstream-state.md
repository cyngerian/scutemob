# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 12: Ability Words (Enrage, Alliance, Corrupted, Ravenous, Bloodrush) | available | — | Batch 11 complete; claim to start B12 |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | LOW remediation — T2/T3 items | ACTIVE | 2026-03-03 | Phase 0 complete; T2 done; working T2/T3 LOWs |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | 15 cards total authored; low yield until DSL gaps filled — see handoff |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-03-07
**Workstream**: W1: Abilities — Batch 11

Batch 11 complete. All 5 abilities implemented, reviewed, card+script+coverage done:
- Modal Choice (CR 700.2): modes_chosen Vec<usize> on CastSpell+StackObject; casting validation; resolution dispatch; copy propagation; allow_duplicate_modes. P2 now 17/17. Abzan Charm; script 169.
- Tribute (CR 702.104 — NOT 702.107): TributeNotPaid TriggerCondition; tribute_was_paid on GameObject; inline ETB; bot auto-declines. Fanatic of Xenagos; script 170. Review: clean.
- Fabricate (CR 702.123): KW 132 inline ETB; bot chooses counters; Servo token fallback (2016-09-20 ruling). 1 MEDIUM fix (token fallback test). Weaponcraft Enthusiast; script 171.
- Fuse (CR 702.102): KW 133, AbilDef::Fuse disc 51; was_fused on StackObject; hand-only; combined cost; left-then-right; copy propagates was_fused. 2 HIGH fixes (copy.rs was_fused propagation + doc). Wear//Tear; script 172.
- Spree (CR 702.172 — NOT 702.165): KW 134; mode_costs on ModeSelection; per-mode cost accumulation; modes_chosen.sort_unstable() for CR 700.2a. 1 MEDIUM fix (mode order test). Final Showdown; script 173.

1754 tests passing. 160 validated. P4 66/88. Scripts 169-173.
New infra: modes_chosen (Modal Choice); tribute_was_paid+TributeNotPaid (Tribute); Fabricate(u32) KW 131→ wait, Tribute=131, Fabricate=132, Fuse=133, Spree=134; AbilDef::Fuse disc 51; was_fused on StackObject; mode_costs on ModeSelection.
CR corrections: Tribute=702.104, Spree=702.172.

**Next**: Claim W1-B12. Check docs/ability-batch-plan.md for Batch 12 (Enrage, Alliance, Corrupted, Ravenous, Bloodrush — ability words, all Low effort, reuse existing trigger patterns).

**Commit**: f9df635 W1-B11: Batch 11 complete
**Commit prefix used**: W1-B11:

## Last Handoff

**Date**: 2026-03-07
**Workstream**: W1: Abilities — Batch 10

Batch 10 complete. All 7 abilities implemented, reviewed, card+script+coverage done:
- Devour (702.82): ETB sacrifice → +1/+1 counters, creatures_devoured field; KW 124; Predator Dragon; script 162; clean review
- Backup (702.165): ETB counter on target + grant abilities for turn; BackupTrigger SOK 46; KW 125; Backup Agent; script 163; 1 HIGH fix (PendingTrigger hash missing backup_abilities/backup_n + 4 B8 fields)
- Champion (702.72): ETB exile own creature or sacrifice self; LTB return via linked ability; ChampionETBTrigger/ChampionLTBTrigger SOK 47-48; ChampionFilter enum; KW 126; AbilDef 49; Changeling Hero; script 164; 1 MEDIUM fix (LTB detection used KW guard instead of champion_exiled_card.is_some() — CR 607.2a)
- Umbra Armor/Totem Armor (702.89): inline replacement — destroy Aura instead of enchanted permanent; KW 127; Hyena Umbra; script 165; 2 MEDIUM fixes (phased-out filter, non-deterministic selection)
- Living Metal (702.161): continuous effect Layer 4 adds Creature type on your turn; KW 128; Steel Guardian (synthetic DFC-free card); script 166; clean review
- Soulbond (702.95): pair creatures, share abilities while paired; paired_with on GameObject; EffectDuration::WhilePaired; SoulbondTrigger SOK 49; AbilDef 50; SoulbondGrant struct; KW 129; Silverblade Paladin; script 167; 1 MEDIUM fix (fizzle check: calculate_characteristics not base types)
- Fortify (702.67): Fortification activated ability attaches to land; Effect::AttachFortification; EffectFilter::AttachedLand; GameEvent::FortificationAttached; KW 130; Darksteel Garrison; script 168; 1 MEDIUM fix (CR 301.6 creature-Fortification guard) + 2 LOW fixes

Also fixed: MR-B9-01 generic upkeep trigger gap (upkeep_actions() now has CardDef sweep).

1706 tests passing. 155 validated. P4 64/88. Scripts 162-168.
New infra: KW 124-130; AbilDef 49-50; SOK 46-49; Effect::AttachFortification (disc 42); EffectFilter::AttachedLand (disc 13); GameEvent::FortificationAttached (disc 100); EffectDuration::WhilePaired; creatures_devoured+champion_exiled_card+paired_with on GameObject; ChampionFilter enum; SoulbondGrant struct; cast_spell_devour harness action.

**Next**: Claim W1-B11. Check docs/ability-batch-plan.md for Batch 11 contents (Modal Choice system, Tribute, Fabricate, Fuse, Spree).

**Commit prefix used**: W1-B10:

## Last Handoff

**Date**: 2026-03-06
**Workstream**: W1: Abilities — Batch 9

Batch 9 complete. All 6 abilities implemented, reviewed, card+script+coverage done:
- Graft (702.58): ETB counters, enter-trigger counter move, optional intervening-if; GraftTrigger SOK 44; Simic Initiate; script 156; 1 HIGH fix (view_model.rs missing GraftTrigger arm)
- Scavenge (702.97): graveyard activated ability, power snapshot before exile, sorcery speed; ScavengeAbility SOK 45; Deadbridge Goliath; script 157; clean review
- Outlast (702.107): tap+mana activated, sorcery speed, AddCounter self; KW 121, AbilDef 48; Ainok Bond-Kin; script 158; clean review. Post-review: view_model.rs missing Outlast display arm (fixed inline)
- Amplify (702.38): ETB replacement, hand reveal, creature type matching; KW 122; Canopy Crawler; script 159; clean review
- Bloodthirst (702.54): ETB replacement, damage_received_this_turn tracking on PlayerState; KW 123; Stormblood Berserker; script 160; clean review
- Amass (701.47): Effect::Amass keyword action, Army token create/find, subtype add; GameEvent::Amassed; Dreadhorde Invasion; script 161; 1 MEDIUM fix (emit Amassed on token failure per CR 701.47b)

1641 tests passing. 148 validated. P4 57/88. Scripts 156-161.
New infra: GraftTrigger (SOK 44); ScavengeAbility (SOK 45); AbilDef 47 (Scavenge), 48 (Outlast); KW 119-123; damage_received_this_turn on PlayerState; Effect::Amass; GameEvent::Amassed.
Discriminants used: KW 119-123; AbilDef 47-48; SOK 44-45; Effect disc 41; GameEvent disc 98.

**HAZARD — Generic trigger gap**: `upkeep_actions()` in `turn_actions.rs` only fires hardcoded keyword triggers (Suspend, Vanishing, Fading, Echo, Cumulative Upkeep, Forecast). Cards with `TriggerCondition::AtBeginningOfYourUpkeep` in their CardDefinition (e.g., Dreadhorde Invasion) silently never fire their upkeep triggers. Recommend adding a general CardDef trigger sweep in `upkeep_actions()` before Batch 10 cards are scripted — this will affect Champion, Soulbond, and other ETB/LTB pattern cards if they use generic trigger conditions.

**Next**: Claim W1-B10. Check docs/ability-batch-plan.md for Batch 10 contents (Devour, Backup, Champion, Totem Armor, Living Metal, Soulbond, Fortify — ETB/dies patterns). Fix generic trigger gap first if it affects B10 scripts.

**Commit prefix used**: W1-B9:

## Last Handoff

**Date**: 2026-03-06
**Workstream**: W1: Abilities — Batch 8

Batch 8 complete. All 7 abilities implemented, reviewed, card+script+coverage done:
- Vanishing (702.63): ETB time counters, upkeep removal, sacrifice trigger; Aven Riftwatcher; script 149; MEDIUM fix (find_map→filter_map sum for multi-instance)
- Fading (702.32): ETB fade counters, single upkeep trigger (remove or sacrifice); Blastoderm; script 150; clean review
- Echo (702.30): echo_pending flag on GameObject, upkeep trigger, PayEcho command; Avalanche Riders; script 151; clean review. CR corrected: 702.30 not 702.31
- Cumulative Upkeep (702.24): age counters, escalating cost, PayCumulativeUpkeep; Mystic Remora; script 152; HIGH fix (expect→unwrap_or) + MEDIUM (life_lost_this_turn)
- Recover (702.59): GY creature-death trigger, PayRecover command, pay_recover harness action; Grim Harvest; script 153; clean review. Also fixed grim_harvest.rs modes: vec![]→None
- Forecast (702.57): hand activated ability, upkeep-only restriction, forecast_used_this_turn tracking, activate_forecast harness action; Sky Hussar; script 154; clean review
- Phasing (702.26): simultaneous phase-in/out (snapshot before mutate, CR 502.1), indirect phasing (CR 702.26h), is_phased_in() filters on 30+ battlefield sites; Teferi's Isle; script 155; HIGH+MEDIUM fixes for simultaneous phasing + filter gaps

1592 tests passing. 141 validated. P4 51/88. Scripts 149-155.
New infra: CounterType::Fade+Age; SOK 37-43 (VanishingCounter, VanishingSacrifice, Fading, Echo, CumulativeUpkeep, Recover, ForecastAbility); echo_pending+phased_out_indirectly+phased_out_controller on GameObject; forecast_used_this_turn on GameState; CumulativeUpkeepCost enum; PermanentsPhasedOut/PermanentsPhasedIn events; pay_recover+activate_forecast harness actions.
Discriminants used: KW 112-118; AbilDef 41-46.

**HAZARD**: Ability planner sometimes generates wrong KW/AbilDef discriminants (used already-taken values). Always verify the discriminant chain from the previous batch before running the runner. The runner itself was given correct values manually each time.

**Next**: Claim W1-B9. Check docs/ability-batch-plan.md for Batch 9 contents (Graft, Scavenge, Outlast, Amplify, Bloodthirst, Amass — counter/growth mechanics).

**Commit prefix used**: W1-B8:

## Last Handoff

**Date**: 2026-03-06
**Workstream**: W1: Abilities — Batch 7

Batch 7 complete. All 6 abilities implemented, reviewed, card+script+coverage done:
- Replicate (702.56): Train of Thought, script 143, cast_spell_replicate harness
- Gravestorm (702.69): Follow the Bodies, script 144, permanents_put_into_graveyard_this_turn
- Cleave (702.148): Path of Peril, script 145, WasCleaved condition, cast_spell_cleave harness
- Splice (702.47): Glacial Ray + Reach Through Mists, script 146, splice_cards on CastSpell
- Entwine (702.42): Promise of Power, script 147, entwine_paid+was_entwined, ModeSelection→helpers.rs
- Escalate (702.120): Blessed Alliance, script 148, escalate_modes u32, CR corrected from 702.121

1526 tests passing. 134 validated. P4 44/88. Scripts 143-148.
New harness actions: cast_spell_replicate, cast_spell_cleave, cast_spell_splice, cast_spell_entwine, cast_spell_escalate.
Discriminants used: KW 107-111, AbilDef 36-40, SOK 36 (GravestormTrigger).

**Next**: Claim W1-B8. Check docs/ability-batch-plan.md for Batch 8 contents (Vanishing, Fading, Echo, Cumulative Upkeep, Recover, Forecast, Phasing — upkeep/time abilities).

**Commit prefix used**: W1-B7:

## Last Handoff

**Date**: 2026-03-02 (session end)
**Workstream**: W1: Abilities — Batch 5
**Task**: Implement Batch 5: Alt-cast hand/exile (Dash, Blitz, Plot, Prototype, Impending)
**Completed**:
- W3 structural refactor: CastSpell 13 booleans → `alt_cost: Option<AltCostKind>`, PendingTrigger 21 booleans → `kind: PendingTriggerKind`, GameObject `was_evoked/was_escaped/was_dashed` → `cast_alt_cost: Option<AltCostKind>` — commit 201bc48
- Dash (CR 702.109): ETB haste, EOT return trigger, 7 tests, Zurgo Bellstriker, script 132 — commit 54f6ea9
- Blitz (CR 702.152): ETB haste + EOT sacrifice + inline draw-on-death, 9 tests (SBA lethal path), Riveteers Requisitioner, script 133 — commit 4499bda
- Plot (CR 702.170): new Command::PlotCard special action + free cast (AltCostKind::Plot), 20 tests, Slickshot Show-Off, script 134 — commit 9750a51
- Prototype (CR 702.160/718): NOT an AltCost — separate `prototype: bool` on CastSpell; zone-change revert (CR 718.4), copy propagation (CR 718.3c); 2 HIGH fixes; 10 tests, Blitz Automaton, script 135 — commit aa46447
- Impending (CR 702.176): AltCostKind::Impending, Layer 4 type-removal inline, time counter ETB + end-step removal; clean review (4 LOW test gaps); 11 tests, Overlord of the Hauntwoods, script 136 — commit c2d30fd
- helpers.rs: added ManaColor + ManaAbility to DSL prelude (enables Everywhere token mana_abilities)
- replay_harness.rs: cast_spell_impending action type + "time" in parse_counter_type
- 1421 tests passing; 122 validated total; P4 30/88
**Next**: Claim W1-B6. Check docs/ability-batch-plan.md for Batch 6 contents.
**Hazards**: Discriminant chain: KeywordAbility 95-99, AbilityDefinition 28-32, StackObjectKind 31-33. StackObject still has per-ability was_X fields (was_dashed, was_blitzed etc.) — not consolidated, deferred. Prototype's `prototype: bool` on CastSpell still causes ~85-file update when new Prototype cards added — could use Default+struct-update eventually.
**Commit prefix used**: `W1-B5:`, `W3:` (structural refactor)

## Last Handoff

**Date**: 2026-03-05 (session end)
**Workstream**: W1: Abilities — Batch 6
**Task**: Implement Batch 6: Cost modification (Bargain, Emerge, Spectacle, Surge, Casualty, Assist)
**Completed**:
- Bargain (CR 702.166): optional additional cost, bargain_sacrifice+was_bargained chain; Torch the Tower; script 137 — clean review
- Emerge (CR 702.119): alt cost, sacrifice creature reduces MV, get_emerge_cost()/reduce_cost_by_mv(); Elder Deep-Fiend; script 138 — clean review
- Spectacle (CR 702.137): alt cost if opponent lost life; new life_lost_this_turn on PlayerState; Skewer the Critics; script 139 — needs-fix (2 MEDIUM fixed: test name, commander tax test)
- Surge (CR 702.117): alt cost if you cast another spell this turn; Reckless Bushwhacker; script 140 — clean review; cast_spell_surge harness arm added
- Casualty (CR 702.153): additional cost, StackObjectKind::CasualtyTrigger+copy; Make Disappear; script 141 — clean review
- Assist (CR 702.132): another player pays generic; assist_player+assist_amount on CastSpell; Huddle Up; script 142 — clean review
- Batch 6 checkbox checked in workstream-coordination.md
- LegalActionProvider doc comment updated (full bot behavior = W2 TUI task)
- Commit: 322bfae W1-B6: Batch 6 complete
**Next**: Claim W1-B7. Check docs/ability-batch-plan.md for Batch 7 contents (Replicate, Gravestorm, Overload, Cleave, Splice, Entwine*, Escalate*). Note Entwine+Escalate depend on Modal choice (Batch 11).
**Discriminant chain**: KeywordAbility 100-105 used; AbilityDefinition disc 33-35 used; StackObjectKind 34 used. Next: KW 106, AbilDef 36, SOK 35.
**Hazards**: Casualty CR number is 702.153 (not 702.154 — that's Enlist); plan files carry correct 702.153.

## Handoff History

### 2026-03-01 (session end) — W1: Abilities — Batch 4
- Retrace, Jump-Start, Aftermath, Embalm, Eternalize, Encore; 1336 tests; 117 validated; P4 25/88; scripts 126-131; cards: Flame Jab, Radical Idea, Cut//Ribbons, Sacred Cat, Proven Combatant, Briarblade Adept; commits cada8d5–3991065

### 2026-03-01 (session end) — W1: Abilities — Batch 3
- Melee, Poisonous, Toxic, Enlist, Ninjutsu/CommanderNinjutsu; 1295 tests; P4 19/88; scripts 121-125; cards: Wings of the Guard, Poisonous Viper, Pestilent Syphoner, Coalition Skyknight, Ninja of the Deep Hours; commits 3e695b4–17e19fd

### 2026-03-01 (session end) — W1: Abilities — Batch 2
- Flanking, Bushido, Rampage, Provoke, Afflict, Renown, Training; 1254 tests; P4 13/88; scripts 114-120; cards: Suq'Ata Lancer, Devoted Retainer, Wolverine Pack, Goblin Grappler, Khenra Eternal, Topan Freeblade, Gryff Rider; commit 92f1265

### 2026-03-01 (session end) — W1: Abilities — Batch 1
- Horsemanship, Skulk, Devoid, Decayed, Ingest; 1177 tests; P4 6/88; scripts 109-113; cards: Shu Cavalry, Furtive Homunculus, Forerunner of Slaughter, Shambling Ghast, Mist Intruder; commit 9cc5672

### 2026-02-28 (session end) — W1: Abilities — Batch 0
- Bolster, Adapt, Shadow, Partner With, Overload; 1166 tests; scripts 104-108; P3 36/40, P4 1/88; commit 2729c3d
