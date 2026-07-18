# Primitive WIP: PB-EF3 — attack-trigger target fidelity + defending-player target

batch: PB-EF3
title: (A) EF-W-MISS-10 forward DSL targets in attack-trigger enrich + fix registry fallback; (B) EF-W-MISS-4 add a "defending player" PlayerTarget/EffectTarget (CR 508.1/506.2) correct in 4-player Commander
task: scutemob-103
branch: feat/pb-ef3-attack-trigger-target-fidelity-defending-player-targe
started: 2026-07-18
phase: implement
plan_file: memory/primitives/pb-plan-EF3.md

## Steps (from plan — unchecked)
- [x] A1 — forward `targets` in ALL `AbilityDefinition::Triggered` enrich blocks (replay_harness.rs); attack block ~3012 primary. Done: all 30 `if let AbilityDefinition::Triggered { ... }` blocks (lines 2293-3457, `crates/engine/src/testing/replay_harness.rs`) now destructure `targets` and forward `targets.clone()` instead of hardcoding `targets: vec![]`. 3 blocks (WhenDies, WhenDealsCombatDamageToPlayer, WhenBecomesTarget) already forwarded targets pre-PB; the other 27 fixed. Compiles clean (`cargo check -p mtg-engine --features test-util`).
- [x] A2 — guard registry fallback: Normal → runtime authoritative; CardDefETB → def raw-index (abilities.rs ~6686-6727). Done: rewrote the `ability_targets` block in `flush_pending_triggers` (abilities.rs) so `PendingTriggerKind::Normal` reads `obj.characteristics.triggered_abilities.get(trigger.ability_index).targets` unconditionally (no fallthrough), and `CardDefETB` reads `def.abilities.get(trigger.ability_index)` unconditionally. Regression sweep found 4 pre-existing sites that pushed `PendingTriggerKind::Normal` with a **raw `def.abilities` index** (never lowered via `enrich_spec_from_def`, by design — same pattern as the graveyard-trigger `CardDefETB` sites): WhenYouCastThisSpell (~3452), WhenExertedAsAttacks (~3739), the WhenDealsCombatDamageToPlayer carddef fallback (~4667, the Throat Slitter regression), and WheneverRingTemptsYou (~5545). Reclassified all 4 from `Normal` to `PendingTriggerKind::CardDefETB` — the kind whose established contract is exactly "ability_index indexes CardDef::abilities, always resolve via card registry" (mirrors Bloodghast's graveyard-trigger construction at ~6512). This is a correctness fix, not scope creep: A2's stated Normal/CardDefETB split was already violated by these 4 sites; they were "getting away with it" only because the pre-A2 fallback masked the mis-tagging. Regressions found and fixed: `test_mutate_gemrazer_trigger_queued_after_merge` (mutate.rs) — added an opponent artifact to the test setup so Gemrazer's "destroy target artifact/enchantment an opponent controls" trigger has a legal target (was passing vacuously before, since the target was silently dropped); `test_throat_slitter_end_to_end_precision_fix` (pbd_damaged_player_filter.rs) — fixed by the CardDefETB reclassification, no test changes needed; `ring_tempts_you::test_whenever_ring_tempts_you_trigger` — updated its `PendingTriggerKind::Normal` assertion to `CardDefETB` (the correct, now-accurate kind) with an explanatory comment; golden script `test-data/generated-scripts/combat/192_mutate_gemrazer.json` — rewrote to add P2's Arcane Signet as the legal target, updated the resolution steps to show it destroyed, and resolved both stale disputes (documenting the PB-EF3 fix). Full suite green, `cargo check -p mtg-engine --features test-util` clean.
- [ ] B1 — capture defending player at attack-trigger dispatch into PendingTrigger.defending_player_id (abilities.rs ~3873-3889)
- [ ] B2 — thread defending_player_id → StackObject.defending_player → EffectContext.defending_player (stack.rs, abilities.rs flush, resolution.rs, effects/mod.rs)
- [ ] B3 — add EffectTarget::AttackTarget + PlayerTarget::DefendingPlayer (card_definition.rs)
- [ ] B4 — resolve arms in resolve_player_target_list + resolve_effect_target_list_indexed (effects/mod.rs)
- [ ] B5 — hash StackObject.defending_player (hash.rs)
- [ ] C — exhaustive-match arms compiler names; PROTOCOL 7→8 + HASH 45→46 (machine-forced)
- [ ] Cards — ojutai_soul_of_winter.rs (new), hellrider.rs (flip), raid_bombardment.rs (new); OOS-EF3-1 filed for Silumgar; others documented-blocked
- [ ] Tests — crates/engine/tests/primitives/pb_ef3_attack_trigger_targets.rs (+ mod line in main.rs)

## Source findings
- memory/card-authoring/w-miss-engine-findings-2026-07-17.md — EF-W-MISS-4 (line 49), EF-W-MISS-10 (line 88)
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF3 section (line 217)

## Known facts (recon done by coordinator/worker before planning)
- **MISS-10 enrich sites**: `crates/engine/src/testing/replay_harness.rs` ~2991-3014 — the
  `WheneverCreatureYouControlAttacks` enrich loop hardcodes `targets: vec![]` (line ~3012),
  dropping the DSL `AbilityDefinition::Triggered { .. }` targets. (There may be a matching
  `add_triggered_ability`/`build.rs`-generated path; verify both enrich and the builder.rs path.)
- **MISS-10 fallback**: `crates/engine/src/rules/abilities.rs` ~6709-6723 — the registry
  fallback `def.abilities.get(trigger.ability_index)` raw-indexes `def.abilities` but
  `trigger.ability_index` indexes the runtime `triggered_abilities` vec, so it matches the
  wrong ability. The `from_runtime` path (~6689-6705) already returns `ab.targets` when
  non-empty — so forwarding targets in enrich makes `from_runtime` succeed; the fallback fix
  is defense-in-depth / for the non-enriched path.
- **AbilityDefinition::Triggered** has a `targets` field (card_definition.rs:315).
- **PlayerTarget** enum: card_definition.rs:2480 (has Controller/EachPlayer/EachOpponent/
  DeclaredTarget/ControllerOf/OwnerOf/TriggeringPlayer/DamagedPlayer/ControllerOfCounteredSpell/
  ControllerOfTriggeringObject). **EffectTarget** enum: card_definition.rs:2446.
- **Defender data**: `CombatState.attackers: OrdMap<ObjectId, AttackTarget>` (card-types/src/state/combat.rs:30);
  `AttackTarget::Player(pid)` | `Planeswalker(pw_id)`. Defending player = the Player, or the
  controller of the Planeswalker (CR 508.4/506.4b). Attack trigger dispatch: abilities.rs ~3881
  (per-attacker `collect_triggers_for_event(AnyCreatureYouControlAttacks)`).
- **Wire**: PROTOCOL currently 7, HASH 45. Adding a PlayerTarget/EffectTarget variant is a
  card-DSL change inside the SR-8 fingerprint closure → PROTOCOL bump forced by
  tests/protocol_schema.rs; HASH bump if the type is also in the GameState hash closure
  (PlayerTarget/EffectTarget likely reachable via Characteristics→Effect). Machine-forced.

## Candidates (9, discounted ~5-6)
- MISS-10 (re-author/flip): Ojutai (Dragonlord Ojutai), Soul of Winter
- MISS-4 (flip/author): hellrider, Brutal Hordechief, Raid Bombardment, Norn's Decree,
  Karazikar the Blind Jailer, Silumgar the Drifting Death, Cunning Rhetoric
- Verify EACH full chain vs oracle text (MCP authoritative) per feedback_verify_full_chain;
  demote honestly if a clause is inexpressible. MISS-10 and MISS-4 are separable if too large
  (ship MISS-10 first, file the rest) — but attempt both.
