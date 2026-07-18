# Primitive WIP: PB-EF3 — attack-trigger target fidelity + defending-player target

batch: PB-EF3
title: (A) EF-W-MISS-10 forward DSL targets in attack-trigger enrich + fix registry fallback; (B) EF-W-MISS-4 add a "defending player" PlayerTarget/EffectTarget (CR 508.1/506.2) correct in 4-player Commander
task: scutemob-103
branch: feat/pb-ef3-attack-trigger-target-fidelity-defending-player-targe
started: 2026-07-18
phase: done
plan_file: memory/primitives/pb-plan-EF3.md

## Steps (from plan — unchecked)
- [x] A1 — forward `targets` in ALL `AbilityDefinition::Triggered` enrich blocks (replay_harness.rs); attack block ~3012 primary. Done: all 30 `if let AbilityDefinition::Triggered { ... }` blocks (lines 2293-3457, `crates/engine/src/testing/replay_harness.rs`) now destructure `targets` and forward `targets.clone()` instead of hardcoding `targets: vec![]`. 3 blocks (WhenDies, WhenDealsCombatDamageToPlayer, WhenBecomesTarget) already forwarded targets pre-PB; the other 27 fixed. Compiles clean (`cargo check -p mtg-engine --features test-util`).
- [x] A2 — guard registry fallback: Normal → runtime authoritative; CardDefETB → def raw-index (abilities.rs ~6686-6727). Done: rewrote the `ability_targets` block in `flush_pending_triggers` (abilities.rs) so `PendingTriggerKind::Normal` reads `obj.characteristics.triggered_abilities.get(trigger.ability_index).targets` unconditionally (no fallthrough), and `CardDefETB` reads `def.abilities.get(trigger.ability_index)` unconditionally. Regression sweep found 4 pre-existing sites that pushed `PendingTriggerKind::Normal` with a **raw `def.abilities` index** (never lowered via `enrich_spec_from_def`, by design — same pattern as the graveyard-trigger `CardDefETB` sites): WhenYouCastThisSpell (~3452), WhenExertedAsAttacks (~3739), the WhenDealsCombatDamageToPlayer carddef fallback (~4667, the Throat Slitter regression), and WheneverRingTemptsYou (~5545). Reclassified all 4 from `Normal` to `PendingTriggerKind::CardDefETB` — the kind whose established contract is exactly "ability_index indexes CardDef::abilities, always resolve via card registry" (mirrors Bloodghast's graveyard-trigger construction at ~6512). This is a correctness fix, not scope creep: A2's stated Normal/CardDefETB split was already violated by these 4 sites; they were "getting away with it" only because the pre-A2 fallback masked the mis-tagging. Regressions found and fixed: `test_mutate_gemrazer_trigger_queued_after_merge` (mutate.rs) — added an opponent artifact to the test setup so Gemrazer's "destroy target artifact/enchantment an opponent controls" trigger has a legal target (was passing vacuously before, since the target was silently dropped); `test_throat_slitter_end_to_end_precision_fix` (pbd_damaged_player_filter.rs) — fixed by the CardDefETB reclassification, no test changes needed; `ring_tempts_you::test_whenever_ring_tempts_you_trigger` — updated its `PendingTriggerKind::Normal` assertion to `CardDefETB` (the correct, now-accurate kind) with an explanatory comment; golden script `test-data/generated-scripts/combat/192_mutate_gemrazer.json` — rewrote to add P2's Arcane Signet as the legal target, updated the resolution steps to show it destroyed, and resolved both stale disputes (documenting the PB-EF3 fix). Full suite green, `cargo check -p mtg-engine --features test-util` clean.
- [x] B1 — capture defending player at attack-trigger dispatch into PendingTrigger.defending_player_id (abilities.rs ~3873-3889). Done: the `AnyCreatureYouControlAttacks` loop (abilities.rs, `GameEvent::AttackersDeclared` handler) now destructures `(attacker_id, attack_target)`, captures `pre_len` before `collect_triggers_for_event`, computes `defending_player` from `attack_target` (Player → that player; Planeswalker → its controller), and tags `triggers[pre_len..]` — identical pattern to the pre-existing `SelfAttacks` capture a few lines above. Compiles clean.
- [x] B2 — thread defending_player_id → StackObject.defending_player → EffectContext.defending_player (stack.rs, abilities.rs flush, resolution.rs, effects/mod.rs). Done exactly per plan: added `defending_player: Option<PlayerId>` to `StackObject` (`crates/card-types/src/state/stack.rs`, next to `lki_power`) — 12 full-literal constructor sites found via `cargo check`'s exhaustive-field errors and fixed (engine.rs×4, copy.rs×3, casting.rs×2 incl. a `#[cfg(test)]`-gated one `cargo check` doesn't see, `trigger_default`, plus 1 more surfaced only under `--all-targets` in casting.rs's own test module). `flush_pending_triggers` (abilities.rs, ~7847) sets `stack_obj.defending_player = trigger.defending_player_id` right after `triggering_creature_id`. `resolution.rs` sets `ctx.defending_player = stack_obj.defending_player` in BOTH the kicker path (~2109) and non-kicker path (~2203), mirroring `damaged_player` exactly. `EffectContext` gained `defending_player: Option<PlayerId>` with 6 constructor/literal sites fixed (`new`, `new_with_kicker`, 2 ForEach inner-context rebuilds, 2 Condition-delegation minimal contexts, plus abilities.rs's own inline activation-condition context and a test file `primitive_pb37.rs`). Compiles clean, full suite green.
- [x] B3 — add EffectTarget::AttackTarget + PlayerTarget::DefendingPlayer (card_definition.rs). Done: both variants added to `crates/card-types/src/cards/card_definition.rs` with CR 508.4/506.4c doc comments citing their resolution mechanism.
- [x] B4 — resolve arms in resolve_player_target_list + resolve_effect_target_list_indexed (effects/mod.rs). Done: `EffectTarget::AttackTarget` arm added to `resolve_effect_target_list_indexed` (looks up `ctx.triggering_creature_id` in `state.combat.attackers`, maps Player/Planeswalker to ResolvedTarget, falls back to `ctx.defending_player`, else empty); `PlayerTarget::DefendingPlayer` arm added to `resolve_player_target_list` (mirrors `DamagedPlayer`). Also fixed 2 pre-existing manual `PlayerTarget` matches in `Effect::Manifest`/`Effect::Cloak` that the plan's file:line pointer didn't mention (found via `cargo check`'s non-exhaustive-match errors) — both needed a `DefendingPlayer` arm too.
- [x] B5 — hash StackObject.defending_player (hash.rs). Done: `self.defending_player.hash_into(hasher);` added to `HashInto for StackObject` right after `lki_power`.
- [x] C — exhaustive-match arms compiler names; PROTOCOL 7→8 + HASH 45→46 (machine-forced). Done: `cargo build --workspace` found zero additional match arms needed in replay-viewer/tui/simulator (EffectTarget/PlayerTarget aren't displayed there, unlike StackObjectKind/KeywordAbility). PROTOCOL_VERSION 7→8, HASH_SCHEMA_VERSION 45→46 (actual bump differed from the plan's guessed final digests only in that the plan didn't know them in advance — both digests, all sentinels, and both FROZEN_HISTORY_PREFIX_DIGEST re-pins were driven entirely by the failing gates, not guessed). See completion report for exact digest values.
- [x] Cards — ojutai_soul_of_winter.rs (new), hellrider.rs (flip), raid_bombardment.rs (new); OOS-EF3-1 filed for Silumgar; others documented-blocked. Done: all 3 oracle texts verified against `cards.sqlite` (workspace-root DB — `mcp__mtg-rules__lookup_card` wasn't available as a callable tool in this session, used the same source data directly). `hellrider.rs` flipped from `partial` to `Complete` (default), TODO removed, uses `EffectTarget::AttackTarget`. `ojutai_soul_of_winter.rs` new, `Complete` (default), uses `TargetPermanentWithFilter{non_land:true, controller:Opponent}` + `Sequence[TapPermanent, PreventNextUntap]`. `raid_bombardment.rs` new, `Complete` (default), uses `max_power:Some(2)` filter + `EffectTarget::AttackTarget`. Blocked cards (Silumgar, Brutal Hordechief, Norn's Decree, Karazikar, Cunning Rhetoric) left untouched per instructions — OOS-EF3-1 filing is a coordinator closeout action, not done inline here. `tools/check-defs-fmt.sh --fix` applied (new defs needed reformatting).
- [x] Tests — crates/engine/tests/primitives/pb_ef3_attack_trigger_targets.rs (+ mod line in main.rs). Done: all 7 tests from the plan, each cites CR. Every decoy proven non-vacuous via temporary revert-and-restore (verified, then reverted before commit — final diff clean):
  - `test_attack_trigger_forwards_declared_target` / `test_attack_trigger_target_not_dropped_decoy`: fail when A1's `targets: targets.clone()` reverts to `vec![]`. **Deviation from plan**: these do NOT independently discriminate A2's guard for Ojutai specifically — Ojutai has exactly one runtime-registered triggered ability, so A2's raw-index fallback path is only reached when A1 is ALSO broken for that ability; verified empirically (reverting A2's guard alone, with A1 fixed, left both tests green). A2's necessity is instead proven by a separate, real regression: reverting the Normal→CardDefETB kind reclassification for the Throat Slitter site (with A1/A2 both left in their fixed state) fails the pre-existing `pbd_damaged_player_filter::test_throat_slitter_end_to_end_precision_fix` — documented in the decoy test's docstring.
  - `test_hellrider_damages_defending_player_4p` / `test_hellrider_damages_attacked_planeswalker` / `test_raid_bombardment_power_filter`: fail when `EffectTarget::AttackTarget`'s resolve arm is stubbed to return empty.
  - `test_defending_player_target_multiplayer` / `test_defending_player_captured_survives_attacker_removal`: fail when `PlayerTarget::DefendingPlayer`'s resolve arm is stubbed to always return `ctx.controller`.
  Full suite green (`cargo test --all`), clippy clean, fmt clean, `cargo build --workspace` clean.

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

## Fixes applied (scutemob-103 review, all 5 findings closed)

- **Finding 1 (MEDIUM, `effects/mod.rs` `EffectTarget::AttackTarget`)**: restructured the
  resolve arm to consult the attacker's *live* `combat.attackers` entry FIRST. While the
  attacker is still present there, its `AttackTarget` variant is authoritative — `Player`
  resolves normally, `Planeswalker` resolves to the object if present or else **fizzles
  (empty), full stop, no fallback**. The `ctx.defending_player` fallback (captured at
  dispatch, CR 113.7a) is now reserved strictly for the case where the attacker itself is
  no longer in the live `combat.attackers` map at all. Chosen approach: **lazy, no new
  captured field** — exactly the "cleanest" option the review offered, since the fix is
  purely about which existing signal (`combat.attackers` live lookup vs. the
  already-captured `ctx.defending_player`) takes priority, not about adding new state.
  Zero wire/hash impact. New test `test_hellrider_fizzles_when_attacked_planeswalker_removed`
  (declares Hellrider attacking a planeswalker, removes the planeswalker from
  `state.objects` before the trigger resolves via `state.objects_mut().remove(&pw_id)`,
  asserts the planeswalker's former controller's life is unchanged). Proven non-vacuous:
  reverting to the old fallback-on-any-None shape makes it fail (39 instead of 40).
- **Finding 2 (MEDIUM, `abilities.rs` `defending_player_id` shortcut)**: gated the
  shortcut with `matches!(trigger.triggering_event, Some(SelfAttacks) |
  Some(SelfAttacksPlayerWithMostLife) | Some(SelfAttacksWithGreaterPowerAlly) |
  Some(SelfBecomesBlocked))` — the annihilator / dethrone / training / **afflict**
  keyword-family events (afflict was NOT in the review's literal list but its
  `LoseLife{DeclaredTarget{0}}` effect depends on this exact shortcut via
  `TriggerEvent::SelfBecomesBlocked`; the first cut of this fix regressed all 5
  `afflict::*` tests, caught by the full suite run — restored by adding
  `SelfBecomesBlocked` to the whitelist). `AnyCreatureYouControlAttacks`-triggered
  effects (Utvara-style token/lifegain, `EffectTarget::AttackTarget` damage) no longer
  receive the spurious `Target::Player(dp)`. New test
  `test_untargeted_attack_trigger_survives_defending_player_leaving` asserts BOTH (1)
  directly that the flushed stack object's `targets` is empty for such a trigger, and
  (2) the life-gain effect still executes after the defending player is marked
  `has_lost = true`. Note on (2): traced the resolution path (`resolution.rs` ~1930-2225)
  and found that a `Normal`-kind trigger whose `ability_index` IS found in the runtime
  `characteristics.triggered_abilities` (true for essentially all enriched card-def
  triggers, including annihilator/dethrone/training/afflict themselves) resolves via the
  "characteristics path" (~2132), which never re-checks `stack_obj.targets` legality —
  only the CardDefETB / registry-fallback path (~2058) does. So assertion (2) alone is
  **vacuous** for this specific fix in the current engine (confirmed empirically: reverted
  the gate, effect still executed); assertion (1) is what actually discriminates the fix
  and is what's pinned as load-bearing. Both proven via revert-and-restore.
- **Finding 3 (LOW, `effects/mod.rs` `PlayerTarget::DefendingPlayer`)**: changed the
  `Some(dp)` branch to return `vec![]` (not `vec![ctx.controller]`) when the captured
  defending player has lost; the `ctx.controller` fallback is now reserved for the `None`
  case (no attack context at all), matching the `AttackTarget` arm's has-lost handling.
  No dedicated new test required by the fix-phase brief (Brutal Hordechief, the only
  would-be user, remains unauthored/blocked on a separate primitive); full suite
  (including `test_defending_player_target_multiplayer` /
  `test_defending_player_captured_survives_attacker_removal`, both still using
  `PlayerTarget::DefendingPlayer` in the non-has_lost case) stays green.
- **Finding 4 (LOW, `abilities.rs:4703-4704` stale comment)**: updated to say
  "PendingTriggerKind::CardDefETB" and note the sites were reclassified from `Normal` by
  PB-EF3's A2 fix.
- **Finding 5 (LOW, `bare_lookup_ratchet.rs` justification comment)**: verified the
  comment is now accurate post-Finding-1-fix (the `AttackTarget` arm genuinely fizzles
  rather than redirecting); appended a confirmation note rather than rewriting, since the
  original wording turned out to be correct once Finding 1 was fixed.

**Wire/hash**: no bump. All five fixes are pure control-flow/gating changes; no new
fields, variants, or serialized shapes were introduced.

**Gates (post-fix)**: `cargo build --workspace` clean; `cargo test --all` 3364 passed, 0
failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean;
`tools/check-defs-fmt.sh` clean (1785 defs). No remaining TODOs in hellrider.rs,
ojutai_soul_of_winter.rs, raid_bombardment.rs.
