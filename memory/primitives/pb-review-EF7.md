# Primitive Batch Review: PB-EF7 — modal `AbilityDefinition::Activated { modes }`

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-ef7-modal-abilitydefinitionactivated-ef-w-pb2-4` (96cbbd12..bd43762b vs 359c824d)
**CR Rules**: 601.2b, 602.2 / 602.2b, 700.2 / 700.2a / 700.2c / 700.2d / 700.2f, 400.7
**Engine files reviewed**: `rules/abilities.rs` (mode validation + effect bake), `rules/resolution.rs`
(ActivatedAbility arm, confirmed unchanged), `rules/casting.rs` (`validate_targets_positional`,
`matches_filter` routing), `rules/protocol.rs` (PROTOCOL 12), `state/hash.rs` (HASH 50 + two arms),
`testing/replay_harness.rs` (enrich propagation), `effects/mod.rs` (`matches_filter`, `DealDamage`)
**Card defs reviewed**: `goblin_cratermaker.rs`, `cankerbloom.rs`, `umezawas_jitte.rs` (3)
**Tests reviewed**: `tests/primitives/pb_ef7_modal_activated.rs` (11 tests)

## Verdict: needs-fix

**No HIGH findings.** The engine change is CR-correct and carefully built: mode validation
(range / min-max / duplicate-per-`allow_duplicate_modes` / ascending-sort) and the per-mode
target announce/validate split both run **before any cost payment** (CR 602.2 rewind is
satisfied — proven by `test_700_2a_invalid_mode_index_rejected`), the chosen mode is baked into
`embedded_effect` at activation (approach (a)), and resolution.rs is genuinely untouched so the
sacrificed-source case (CR 400.7) works through the existing `embedded_effect` path. Both flipped
cards match oracle text, the colorless/nonland filter is honored by `matches_filter` and routed
through `validate_targets_positional`, Jitte stays `known_wrong` with a truthful note, OOS-EF7-1 is
filed, and PROTOCOL/HASH bumps are machine-derived with history rows and matching digests. The
`needs-fix` verdict rests on **two MEDIUM test-quality gaps** and a few LOW cosmetic/comment items —
no wrong game state was found.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine correctness findings. Mode logic, ordering, effect bake, hash/protocol all verified correct. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | LOW | `goblin_cratermaker.rs` | **oracle_text string mismatch.** Def stores "Goblin Cratermaker deals 2 damage…"; current Oracle is "This creature deals 2 damage…". Cosmetic only; game state correct. **Fix:** change the string to "This creature deals 2 damage to target creature." |
| 4 | LOW | `umezawas_jitte.rs` | **Stale/aspirational comments.** Header (L9-10) says "only the +2/+2 mode is implemented … deferred to PB-37" and L35 `TODO(PB-37)`, both contradicting the accurate PB-EF7 completeness note (the modal primitive now exists; real blocker is the trigger → OOS-EF7-1). **Fix:** update L9-10 and L35 to reference PB-EF7 / OOS-EF7-1 and state the surviving blocker is the combat-damage-to-any-recipient trigger. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| 1 | MEDIUM | `test_601_2b_modal_choice_survives_intervening_change` | **LKI test does not discriminate the property its name promises.** The intervening change (`p2.life_total += 5`) is orthogonal to mode/target resolution; the test passes identically under approach (a) *and* a hypothetical re-resolution (approach b). **Fix:** make the change load-bearing — after activation, add a *second* colorless nonland permanent to the board and assert only the frozen target (GC Colorless Rock) is destroyed, proving the target/mode are frozen by id at activation, not re-selected at resolution. |
| 2 | MEDIUM | (missing) | **Validation branches have zero coverage.** No test exercises: duplicate-mode rejection (CR 700.2d, `allow_duplicate_modes: false`), `min_modes`/`max_modes` bounds (choosing 2 modes on a `max_modes: 1` ability), the multi-mode+`mode_targets` hard-reject, or ascending-sort. Both flip cards are choose-exactly-one so cannot reach these arms; the file already builds a synthetic non-modal ability (`test_700_2a_modes_chosen_on_nonmodal_rejected`), so a synthetic `max_modes: 2` modal ability is feasible. **Fix:** add ≥3 tests over a synthetic modal ability: (a) duplicate index rejected when `allow_duplicate_modes:false`; (b) count > `max_modes` rejected; (c) multi-mode + `mode_targets: Some` hard-rejected. |
| 5 | LOW | (missing) | **No direct CR 700.2a "unchoosable mode" test.** Choosing mode 1 (destroy colorless) with no legal colorless permanent is only covered indirectly (colored-rock Part A rejects on target type, not absence). **Fix (optional):** add a test activating mode 1 with an empty target slice and no legal target → `InvalidTarget`/`InvalidCommand`. |
| 6 | LOW (informational) | — | **Bots cannot activate these cards.** `random_bot`/harness send empty `modes_chosen` → auto-select mode 0, which needs a target the bot does not supply → activation fails; Cankerbloom's no-target Proliferate mode is unreachable by bots. Already noted in the plan's Risks. Simulator follow-up, not a PB-EF7 defect. |

### Finding Details

#### Finding 1 (MEDIUM): LKI test is not a discriminator
**Test**: `pb_ef7_modal_activated.rs:365`
**CR**: 400.7 / 601.2b
**Issue**: Per `memory/conventions.md` ("Test-validity MEDIUMs are fix-phase HIGHs"), a test whose
setup cannot discriminate the behavior in its title is a validity gap. Here the mutation is a
life-total bump on a non-target player — it cannot change which mode/target resolves under any
implementation, so the test only proves "activation→resolution completes." Mitigating: approach (a)
makes mode re-selection structurally impossible (the source is sacrificed and gone; nothing re-reads
`.modes` at resolution), so there is no *live* bug this fails to catch — hence MEDIUM, not HIGH.
**Fix**: add a second colorless nonland permanent between activation and resolution and assert it
survives while the frozen target is destroyed, exercising "target frozen by ObjectId, not re-derived."

#### Finding 2 (MEDIUM): untested validation arms
**File**: `rules/abilities.rs:342-427`
**CR**: 700.2a / 700.2d
**Issue**: The duplicate-mode guard (352-363), min/max bounds (364-377), ascending sort (379), and
the multi-mode+`mode_targets` hard-reject (423-427) are all live but unexercised. A regression that
deleted any of them would keep the suite green. Both flip cards (`max_modes:1`, single choice) cannot
reach these arms.
**Fix**: synthetic-ability tests as listed above.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2b / 602.2b (mode announced at activation) | Yes | Yes | `test_602_2b_…`, reverse decoy |
| 602.2 (illegal activation rewinds; validate before payment) | Yes | Yes | `test_700_2a_invalid_mode_index_rejected` (no cost paid) |
| 700.2a (range / min-max / illegal-mode) | Yes | Partial | range + nonmodal tested; min/max **untested** (F2); unchoosable-mode indirect (F5) |
| 700.2c (per-mode targets; no target for unchosen mode) | Yes | Yes | `test_cankerbloom_mode2_proliferate_needs_no_target` (headline fix) |
| 700.2d (duplicate modes) | Yes | **No** (F2) | `allow_duplicate_modes` honored in code + hash, no test |
| 700.2f (per-mode target reqs; positional) | Yes | Yes | `validate_targets_positional`, mode-1 exclude_colors decoy |
| 400.7 (sacrificed source at resolution) | Yes | Yes | approach (a); `in_graveyard` assertions + LKI test |

## Card Def Summary

| Card | Oracle Match | TODOs / stale | Game State Correct | Notes |
|------|-------------|----------------|--------------------|-------|
| goblin_cratermaker.rs | Yes (behavior) | oracle_text wording (F3, LOW) | Yes | `exclude_colors:{WUBRG}`+`non_land` correctly expresses "colorless nonland permanent"; both `DeclaredTarget` local index 0; `Complete`; no `Effect::Choose` in compiled tree |
| cankerbloom.rs | Yes | 0 | Yes | 3 modes; mode-2 Proliferate has empty target slice (CR 700.2c); `Complete`; only a comment mentions `Effect::Choose` (gate walks `all_cards()`, not text) |
| umezawas_jitte.rs | note truthful | stale header/L35 (F4, LOW) | n/a (known_wrong) | surviving blocker correctly = combat-damage-to-any-recipient trigger, not the modal primitive; OOS-EF7-1 filed |

## Wire Bumps (verified)

- **PROTOCOL 11→12**: `PROTOCOL_VERSION = 12`, `PROTOCOL_SCHEMA_FINGERPRINT = 05eaa04b…`
  (protocol.rs:148) == history row v12 (protocol.rs:268). History `- 12:` line present. Machine-derived
  (protocol_schema.rs recompute passes in the green suite).
- **HASH 49→50**: `HASH_SCHEMA_VERSION = 50` (hash.rs:448); history row v50 appended (hash.rs:620);
  both hash arms present — runtime `ActivatedAbility` (hash.rs:2852) and DSL `Activated` (hash.rs:6657);
  `ModeSelection::hash_into` covers `mode_targets` (hash.rs:5818), so per-mode targets do not collide.
- Sentinel `test_ef7_hash_and_protocol_versions` pins both to 50/12 (strict-equality form).

## Verification of key claims

- **Effect bake / approach (a)**: `abilities.rs:487-501` overrides `embedded_effect` with the chosen
  mode(s) at activation; resolution arm (`resolution.rs:1841-1876`) reads `embedded_effect` +
  `stack_obj.targets` unchanged; for `SacrificeSelf` the live-lookup fallback is dead but harmless.
- **Local target indexing**: both cards choose exactly one mode → `stack_obj.targets` is that single
  slice → `DeclaredTarget{index:0}` reads `targets[0]`. Correct, no offset math.
- **Filter routed through matches_filter**: `casting.rs:6254` for `TargetPermanentWithFilter`;
  `matches_filter` honors both `exclude_colors` (effects/mod.rs:8249) and `non_land` (8257).
  Non-vacuity of the exclude_colors decoy confirmed by worker canary (delete field → Part A reddens).
- **Ordering before payment**: mode/target validation at 329-478 precedes tap (641), life (686),
  sacrifice payment.
- **Corpus sweep soundness**: sweep enumerated `all_cards()` for `Activated`+`Effect::Choose` (3 hits);
  `effect_choose_gate` (walking compiled `all_cards()`) plus a green 3416-test suite confirm no
  `Complete` def retains `Effect::Choose`, so no modal-activated card was missed.

## Recommendation

Merge-blocking only on the two MEDIUM test gaps (F1, F2) per the "test-validity is fix-phase HIGH"
convention — the engine and card behavior are correct. Fix F1/F2, optionally F3/F4/F5 (all cheap),
then collect.
