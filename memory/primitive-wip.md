# Primitive WIP: PB-EF6 — TargetRequirement::TargetOpponent (EF-W-PB2-2)

batch: PB-EF6
title: Add `TargetRequirement::TargetOpponent` — an opponent-restricted player target (CR 102.4 / 115.x) so "target opponent …" oracle text can be authored without permitting an illegal self-target. Threads the source's controller into player-target validation.
task: scutemob-107
branch: feat/pb-ef6-targetrequirementtargetopponent-ef-w-pb2-2
started: 2026-07-18
phase: review
## Card def / test / wire-bump steps (2026-07-18)
- [x] 3 clean flips → Complete: shaman_of_the_pack (ETB authored: LoseLife + EffectAmount::PermanentCount{Elf,You} + TargetOpponent), raiders_wake (Raid ability authored: AtBeginningOfYourEndStep + Condition::YouAttackedThisTurn + DiscardCards + TargetOpponent), vengeful_bloodwitch (TargetPlayer→TargetOpponent, known_wrong marker removed)
- [x] fell_specter corrected (ETB TargetPlayer→TargetOpponent), stays Complete
- [x] blood_tribute target-fixed (TargetOpponent), stays partial (HalfLife blocker)
- [x] blessed_alliance mode-2 idx3 target-fixed (TargetOpponent), idx0 kept TargetPlayer, stays partial (Escalate/mode_targets blocker)
- [x] forbidden_orchard: targets fixed to TargetOpponent; TokenSpec.recipient wiring ATTEMPTED then REVERTED (new engine finding — WhenTappedForMana Normal-kind triggers don't populate characteristics.triggered_abilities, so targets/recipient are dead on this dispatch path; broke mana_triggers::test_mana_trigger_forbidden_orchard, caught + fixed). Marker rewritten to cite both blockers precisely. Stays known_wrong.
- [x] ajani_sleeper_agent emblem target-fixed (TargetOpponent), stale TODO comments removed, marker's "targets any player" clause dropped. Stays known_wrong.
- [x] flare_of_malice left untouched
- [x] No remaining TargetOpponent-blocker TODO/ENGINE-BLOCKED comments in the 4 Complete defs (shaman/raiders_wake/vengeful_bloodwitch/fell_specter); the 5 non-Complete defs correctly still carry their real (non-TargetOpponent) blocker TODOs
- [x] New test module `pb_ef6_target_opponent.rs` (8 tests) registered in primitives/main.rs — all pass
- [x] Decoys proven non-vacuous by temporary breakage + revert: (1) casting.rs self-rejection arm forced to `Ok` unconditionally → reddened test_target_opponent_spell_accepts_opponent_rejects_self_4p AND test_target_opponent_decoy_self_must_be_rejected; (2) abilities.rs outer picker given a `.or(Some(trigger.controller))` self-fallback → reddened test_target_opponent_trigger_no_opponent_removed_from_stack. Both reverted, suite re-verified green.
- [x] PROTOCOL 10→11 (fingerprint 07e51466..., frozen-prefix re-pinned to 971517e8...), HASH 48→49 (decl 0f8e380b..., stream d3f8ecb0..., frozen-prefix re-pinned to ae92dcee...) — all 4 digests read from the FAILING gate output, never hand-guessed. History rows appended (never edited). 31 scattered HASH_SCHEMA_VERSION sentinels bulk-bumped 48u8→49u8.
- [x] `cargo test --all` green (all groups, 0 failures)
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] `cargo build --workspace` clean
- [x] `cargo fmt --check` clean; `tools/check-defs-fmt.sh` clean (had to fix a card_defs_fmt::completeness_deviation_scan hit on vengeful_bloodwitch's "approximat" wording)
- [ ] /review — coordinator's job, not run here
## Engine steps (2026-07-18)
- [x] Step 1: `TargetRequirement::TargetOpponent` unit variant added, card_definition.rs (after UpToN)
- [x] Step 2: hash.rs discriminant 18 arm added (exhaustive match)
- [x] Step 3: casting.rs `validate_player_satisfies_requirement` threads `caster`; TargetOpponent arm added; UpToN delegates caster; both call sites (~5829, ~6016) updated
- [x] Step 4: casting.rs `validate_object_satisfies_requirement` valid-match rejects TargetOpponent (combined with TargetPlayer arm)
- [x] Step 5: abilities.rs outer picker (~6873) + UpToN-inner picker (~6982) both got TargetOpponent arms with NO self-fallback; also added TargetOpponent=>false to the object-scan closure's exhaustive match (compile-forced, not in plan's explicit list but required)
- [x] Step 6: resolution.rs `is_target_legal` left UNCHANGED per DECISION 4
- [x] `cargo check -p mtg-engine` clean; `cargo build --workspace` clean (no simulator/TUI/replay-viewer arms needed, confirmed)
plan_file: memory/primitives/pb-plan-EF6.md
plan_complete: true  # 2026-07-18 — pb-plan-EF6.md written & verified. Teams-absence CONFIRMED (no team field on PlayerState; opponent = id != caster, mirroring EachOpponent idiom). Roster-recall/TODO sweep found 2 cards NOT in the brief: vengeful_bloodwitch (known_wrong→Complete CLEAN FLIP — marker's sole blocker was this variant) + fell_specter (Complete, latent TargetPlayer-for-"target opponent" self-target bug → correctness fix). Net: 3 clean flips (shaman, raiders_wake, vengeful_bloodwitch) + fell_specter fix + 3 honest target-fixes staying non-Complete (blood_tribute, blessed_alliance idx3, forbidden_orchard) + ajani minimal emblem fix; flare_of_malice left untouched. Wire: PROTOCOL 10→11, HASH 48→49 (both machine-forced; current consts verified in source). is_target_legal needs NO change (DECISION 4 confirmed). No TargetRequirement matches in simulator/TUI/replay-viewer (exhaustive-match sweep clean).

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF6 section (line ~371); §1 table (EF-W-PB2-2, line 188)
- memory/card-authoring/w-pb2-engine-findings-2026-07-17.md — EF-W-PB2-2 (line 35)

## Candidates (4) — recon done 2026-07-18, chain-verified vs MCP oracle
| Card | current | oracle clause needing TargetOpponent | other surviving blockers | verdict |
| --- | --- | --- | --- | --- |
| shaman_of_the_pack | `inert` | ETB "target opponent loses life = Elves you control" | NONE — amount is `EffectAmount::PermanentCount{has_subtype:Elf, controller:You}` (already expressible) | **CLEAN FLIP → Complete** |
| raiders_wake | `partial` | Raid "target opponent discards a card" at your end step | NONE — `TriggerCondition::AtBeginningOfYourEndStep` ✓, `Condition::YouAttackedThisTurn` ✓ (intervening_if), `Effect::DiscardCards{player:DeclaredTarget{0}}` ✓ (fell_specter.rs precedent) | **CLEAN FLIP → Complete** |
| forbidden_orchard | `known_wrong` | trigger "target opponent creates a 1/1 Spirit" | **`Effect::AddManaAnyColor` stub** (EF-W-PB2-3, STILL OPEN — barred from Complete; adds Colorless = wrong game state) | STAYS non-Complete; plan recommends fixing BOTH target-side defects (targets→TargetOpponent + TokenSpec.recipient=DeclaredTarget{0} via PB-EF2) and rewriting marker to cite ONLY EF-W-PB2-3 |
| ajani_sleeper_agent | `known_wrong` | -6 emblem trigger "target opponent gets two poison counters" | +1 & -3 are `Effect::Sequence(vec![])` no-ops; emblem lacks creature/pw spell-type filter; Compleated unimplemented | STAYS known_wrong; minimal emblem target fix (TargetPlayer→TargetOpponent) only |

## Roster-recall / TODO-sweep additions (NOT in original brief — forced adds)
| Card | current | why added | verdict |
| --- | --- | --- | --- |
| vengeful_bloodwitch | `known_wrong` | marker's SOLE cited blocker is the missing opponent-only variant; death trigger + LoseLife/GainLife all expressible | **CLEAN FLIP → Complete** (3rd coverage-mover the brief missed) |
| fell_specter | **Complete** | ships `TargetPlayer` for oracle "target opponent discards a card" — latent legal-but-wrong self-target on a shipped-Complete def | correctness fix, stays Complete |
| blood_tribute | `partial` | ships `TargetPlayer` for "target opponent loses half life" | target-fix only; STAYS partial (real blocker `EffectAmount::HalfLife`) |
| blessed_alliance | `partial` | mode-2 idx3 ships `TargetPlayer` for "target opponent" (idx0 correctly stays TargetPlayer) | idx3 target-fix only; STAYS partial (Escalate/mode_targets conflict) |
| flare_of_malice | `known_wrong` | ships `TargetPlayer` but authored against WRONG oracle | LEFT UNTOUCHED (full re-author needed, out of scope) |

**Discounted ship: 3 clean flips** (shaman_of_the_pack, raiders_wake, vengeful_bloodwitch) + 1 latent-bug correctness fix (fell_specter). Beats the ef-batch-plan "~3" estimate via roster recall.

## COORDINATOR SCOPING DECISIONS (constraints for planner/runner)

### DECISION 1 — `TargetRequirement::TargetOpponent` is a UNIT variant
New unit variant on `TargetRequirement` (card_definition.rs:2875, after UpToN). Doc: `"target opponent"` — an
opponent of the source's controller (CR 102.3 opponent def; CR 115.1 targeting). New hash
discriminant **18** (current max: UpToN=17, hash.rs:5054). It is a **player** requirement (same
family as TargetPlayer), NOT an object requirement. VERIFIED.

### DECISION 2 — validation restricts candidates to opponents; thread the source controller
`validate_player_satisfies_requirement(id, req)` (casting.rs:6074) currently takes NO caster and
returns `Ok(())` for the player-target family. Add a `caster: PlayerId` param and a `TargetOpponent`
arm: `Ok(())` iff `id != caster`. **No teams model exists** — CONFIRMED (PlayerState has no team
field; opponent idiom `p != controller` used at effects/mod.rs:3769/6327/6507). Also add the
`TargetOpponent` arm to the `UpToN{inner}` delegation. Both call sites (casting.rs:5829 closure,
6016) have `caster` in scope. VERIFIED.

### DECISION 3 — auto-target picker must NOT fall back to self
The trigger auto-target picker (`flush_pending_triggers`, abilities.rs:6873 outer / 6982 UpToN-inner)
picks "first opponent, falling back to controller" for the TargetPlayer family. `TargetOpponent`
needs its OWN arm that picks the first active opponent and, if NONE exists, contributes no candidate →
the trigger is skipped (CR 603.3d). NEVER fall back to `trigger.controller`. Both matches have `_`
catch-alls routing to a battlefield scan, so a missing arm is a SILENT mis-route (not a compile
error) — arms are mandatory. VERIFIED.

### DECISION 4 — resolution re-check needs NO change
`is_target_legal` (resolution.rs:7783) only re-checks the target player is still active. Opponent-ness
is a DECLARATION-TIME restriction (CR 115.3/601.2c); once legally chosen it can't "become" a
self-target, and a departed opponent is caught by `has_lost`/`has_conceded`. CONFIRMED at plan time
(fn stores no caster/requirement by design).

### DECISION 5 — check ALL requirement-listing sites — SWEEP COMPLETE
- casting.rs:6078 (validate_player_satisfies_requirement) — arm added (has `_`, MUST add)
- casting.rs:6174/6351 (validate_object_satisfies_requirement `valid` match) — EXHAUSTIVE, add `=> false`
- casting.rs:5829/6016 (call sites) — thread `caster`
- abilities.rs:6873, 6982 (auto-target pickers) — arms added (decision 3)
- hash.rs:5017 — EXHAUSTIVE, add discriminant 18 (COMPILE-ERROR gate)
- simulator LegalActionProvider — `grep TargetRequirement` → 0 matches; no change, no SG-1 hazard. CONFIRMED
- replay_harness translate_player_action/resolve_targets — resolves explicit Target::Player from JSON;
  validation downstream via validate_targets; NO harness change. CONFIRMED
- TUI stack_view / replay-viewer view_model — `grep TargetRequirement` → 0 matches; those match on
  StackObjectKind/KeywordAbility, not TargetRequirement. NO display arm. CONFIRMED
- `cargo build --workspace` is the seal gate.

### DECISION 6 — wire bump machine-forced
New `TargetRequirement` variant is in the SR-8 protocol fingerprint closure (reaches the card DSL)
AND the GameState hash closure (TargetRequirement is hashed). **PROTOCOL 10→11 and HASH 48→49**
(current consts verified: protocol.rs:118 = 10, hash.rs:435 = 48), both machine-forced by
protocol_schema + hash_schema + sentinel tests. Re-pin fingerprints from the FAILING gate output
(never hand-guess), append history rows (PROTOCOL_HISTORY protocol.rs:188; HASH_SCHEMA_HISTORY;
+ `- 11:`/`- 49:` History lines; bump HASH_SCHEMA_VERSION sentinels). If only one gate moves, STOP
and investigate.

### DECISION 7 — no gated-stub effects in backfill authoring
shaman/raiders_wake/vengeful_bloodwitch author to Complete with real primitives only.
forbidden_orchard's `AddManaAnyColor` and ajani's no-ops are barred from Complete — keep those defs
truthfully marked; do NOT paper over with a stub to force a flip.
