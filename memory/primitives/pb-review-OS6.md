# Primitive Batch Review: PB-OS6 — DFC flip-condition sub-batch (OOS-EF5-4 a/b/g)

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 400.2, 400.7, 506.4/506.4b, 508.1, 508.4, 603.3, 614.1c, 701.28/712.18
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs` (Condition/Effect enums),
`crates/card-types/src/state/player.rs`, `crates/engine/src/effects/mod.rs`,
`crates/engine/src/rules/combat.rs`, `crates/engine/src/rules/replacement.rs`,
`crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/events.rs`,
`crates/engine/src/state/builder.rs`, `crates/engine/src/state/hash.rs`,
`crates/engine/src/rules/protocol.rs`
**Card defs reviewed**: `delver_of_secrets.rs`, `legions_landing.rs` (NEW), `thaumatic_compass.rs`,
`growing_rites_of_itlimoc.rs` (doc-only); `westvale_abbey` confirmed absent (3 authored + 1 doc-only)
**Test file**: `crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs` (12 tests, registered in
`tests/primitives/main.rs`)

## Verdict: clean bill

Engine changes match CR text; all three card defs match oracle text and produce correct game state;
both deviations flagged for scrutiny (ratchet bump, delver allowlist) are legitimate; the single
batched PROTOCOL 20→21 / HASH 57→58 bump is correct with append-only history and matching
fingerprints; SR-36 registration verified via `all_cards()`. No HIGH or MEDIUM findings. One LOW
observation on the delver allowlist wording (below) — LOW-only, no fix phase required.

## Engine Change Findings

None (HIGH/MEDIUM). See CR Coverage Check and the deviation analysis below.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | LOW | `delver_of_secrets.rs` | **"Strictly beneficial / no downside" overstates the faithfulness of the mandatory-if-true reveal.** Rare edge cases exist where declining the reveal is correct (dodge removal targeting fliers/power≥3, keep the Human Wizard subtype for tribal, hidden-info bluff). The mandatory model is nonetheless consistent with the engine's established peek-conditional convention (Herald's Horn pattern) and the marking is a deliberate reviewed decision, so this is not a wrong-game-state blocker. **Fix (optional):** soften the def-comment and allowlist wording from "no downside / strictly beneficial" to "beneficial in effectively all realistic board states," so a future reader doesn't over-trust the absolute claim. No behavioral change. |

## Deviation Analysis (the two runner deviations the task asked me to judge)

### Deviation 1 — `bare_lookup_ratchet.rs` ceiling for `effects/mod.rs` 107 → 109: LEGITIMATE

The two new bare lookups are genuine NONSWALLOW predicate reads, each an exact idiom-copy of a
sibling `Condition` arm in the same `check_condition` match:
- `Condition::TopCardIsInstantOrSorcery`: `state.zones.get(&lib_zone).and_then(|z| z.top())` — an
  empty library legitimately answers the peek `false` (mirrors `TopCardIsCreatureOfChosenType`). The
  actual object lookup uses `state.expect_object(id)` (the diagnostic/NONSWALLOW path, debug-asserts
  on a genuinely-absent object), and it is only reached when `z.top()` returned `Some`, so the
  empty-library case short-circuits before `expect_object` — no false debug_assert.
- `Condition::YouAttackedWithNOrMore(n)`: `state.players.get(&ctx.controller).map(..).unwrap_or(false)`
  — identical shape to `YouAttackedThisTurn` a few lines above; a missing controller answers the
  predicate `false`.

Neither swallows a lookup that should be an `expect_/lki_` diagnostic. The ratchet-comment
(bare_lookup_ratchet.rs:73-82) accurately describes both. Bump accepted.

### Deviation 2 — `delver_of_secrets` added to `completeness_deviation_scan.rs` ALLOWLIST: LEGITIMATE

The def-comment contains "modeled as" (a deviation needle), so a `Complete` def must be allowlisted.
The allowlist entry (completeness_deviation_scan.rs:139-147) correctly describes the *DSL shape*
(`Effect::Conditional` gated on `Condition::TopCardIsInstantOrSorcery`), not a papered-over behavioral
deviation, and draws the right contrast with `heralds_horn.rs` (`known_wrong`, because its "put into
hand" reveal decision is a genuine card-advantage tradeoff). The mandatory-if-true model produces
correct game state in effectively all realistic play. See LOW #1 for the only nuance (wording
overstates the edge-case-free claim); the allowlisting itself is a defensible, convention-consistent
call and does not violate Invariant #9.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 400.2 / 614.1c (peek printed types of top library card) | Yes | Yes | `TopCardIsInstantOrSorcery` reads `card.characteristics.card_types` (printed; library not layer-affected). `test_delver_flips_when_top_is_instant` + creature decoy |
| 508.1 / 508.4 (only *declared* attackers count; tokens entering attacking do not) | Yes | Yes | `attackers_declared_this_turn` set ONLY in `handle_declare_attackers` (combat.rs:630), the sole declare path; no token/put-onto-bf-attacking path sets it. `test_legions_landing_no_transform_on_two_attackers` pins the count gate |
| 508.1 ruling (captured count survives attacker leaving combat) | Yes | Yes | Count captured, not re-scanned; `test_legions_landing_transforms_even_if_attacker_leaves` |
| 506.4 (effect specifically removes from combat) | Yes | Yes | `Effect::RemoveFromCombat` + `remove_from_combat` helper clears attackers/blockers/blocked_attackers/damage_assignment_order (key + blocker lists). `test_thaumatic_spires_untaps_and_removes_from_combat`, `test_remove_from_combat_helper_clears_attacker_and_damage_order` |
| 506.4b (untap does not itself remove from combat → two-step Sequence) | Yes | Yes | Def uses `Effect::Sequence([UntapPermanent, RemoveFromCombat])`; `test_thaumatic_spires_decoy_untap_only_leaves_in_combat` pins that untap alone leaves it attacking |
| 603.3 (trigger-fires-then-Nothing self-gating) | Yes | Yes | Both delver and legions gate inside `Effect::Conditional`, not `intervening_if` (correctly avoids the WheneverYouAttack `intervening_if: None` lowering drop). `test_legions_landing_fires_once_not_per_creature` |
| 701.28/712.18 (Transform in place) | Yes | Yes | `TransformSelf` (PB-EF5); transform-count and target-name assertions in all three card tests |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| delver_of_secrets | Yes | 0 | Yes | {U} 1/1 Human Wizard front; Insectile Aberration 3/2 flying blue-via-indicator back; upkeep conditional TransformSelf. Complete. See LOW #1 (comment wording only) |
| legions_landing (NEW) | Yes | 0 | Yes | {W} Legendary Enchantment; ETB 1/1 WHITE Vampire lifelink token (recipient = controller); WheneverYouAttack → `YouAttackedWithNOrMore(3)` → TransformSelf. Back Adanto Legendary Land: `{T}:Add{W}` + `{1}{W},{T}`: same token. `WhenEntersBattlefield` correctly routes through `queue_carddef_etb_triggers` (self-ETB, not any-permanent). Registered in `all_cards()` (SR-36, `test_os6_cards_registered`). Complete |
| thaumatic_compass | Yes | 0 | Yes | Front (search basic land + end-step intervening-if TransformSelf) unchanged/already-Complete; Spires back gains two-step untap+RemoveFromCombat with is_attacking/Opponent target filter. Complete |
| growing_rites_of_itlimoc | Yes (stays partial, honest) | note reworded, not removed | N/A | Doc-only: `completeness::partial` note + inline comment re-pointed from "OOS-EF5-4(f)" to PB-OS8 / `Effect::LookAtTopThenPlace`. No behavioral change. Correct |
| westvale_abbey | absent (honest) | n/a | n/a | Confirmed no def file; OOS-OS6-1 seed recorded in `oos-retriage-plan-2026-07-18.md` |

## Wire / SR-8 Verification

- `PROTOCOL_VERSION` 20 → 21; `PROTOCOL_SCHEMA_FINGERPRINT` = `c617138c…dd2d6` matches the appended
  `version: 21` `PROTOCOL_HISTORY` row (protocol.rs:389); `version: 20` (PB-OS5) row preserved, not
  edited. Four closure-shape moves (Condition ×2, Effect ×1, GameEvent ×1) documented in the `- 21:`
  History line. Correct — no unnecessary bump, no missing sentinel.
- `HASH_SCHEMA_VERSION` 57 → 58; five new `HashInto` arms with distinct, appended discriminants:
  Condition 49 (`TopCardIsInstantOrSorcery`) / 50 (`YouAttackedWithNOrMore`, hashes the u32), Effect
  95 (`RemoveFromCombat`, hashes target), GameEvent 128 (`RemovedFromCombat`; prior max 127, no
  collision within the GameEvent match), plus `PlayerState.attackers_declared_this_turn` hashed right
  after `attacked_this_turn`. Appended `version: 58` `HASH_SCHEMA_HISTORY` row; `version: 39` baseline
  and all intermediate rows untouched. `PlayerState` field is GameState-only → correctly HASH-only,
  not PROTOCOL. Correct.
- Sentinel in the new test file: `PROTOCOL_VERSION == 21`, `HASH_SCHEMA_VERSION == 58u8`
  (`test_os6_version_sentinels`). WIP confirms all `…,20` / `…,57u8` sentinels across the suite bumped
  and both gate tests (`protocol_schema`, `hash_schema`) green with re-pinned FROZEN prefix digests.

## Behavior-Identical Refactor (apply_regeneration → remove_from_combat helper)

`apply_regeneration` step 3 (replacement.rs:2730) now calls the shared
`crate::rules::combat::remove_from_combat`, which clears exactly the same map set the plan attributes
to the original (attackers, blockers, blocked_attackers, and damage_assignment_order attacker-keys +
blocker-lists). Regression guarded by `test_regenerate_still_removes_from_combat` (this file) and the
pre-existing `regenerate.rs` suite; WIP reports all pass. `Effect::RemoveFromCombat` dispatch
(effects/mod.rs:2170) is a no-op when the target is not in combat (SR-4 LKI-fizzle) and only pushes
`GameEvent::RemovedFromCombat` when something was actually removed — correct per CR 506.4.

## Test Adequacy (SR-34/36 execution probing)

12 tests, all execution-driven (real `Command`s / `execute_effect`), each decoy failing on exactly
the field under test:
- delver: instant→flip vs creature-decoy→no flip (pins the condition).
- legions: 3→flip vs 2-decoy→no flip (pins the COUNT, not bare "you attacked"); fires-once (per
  declaration, not per creature); transforms-even-if-attacker-leaves (pins captured count).
- thaumatic: untap+remove vs untap-only-decoy→still in combat (pins removal comes from the new effect,
  CR 506.4b, not the untap); helper clears attacker-side and blocker-side maps; regenerate regression
  guard.
- registration smoke test (SR-36 via `all_cards()`); version sentinels.

All tests cite CR rules. Assertions check transformed status via `calculate_characteristics` face
name, combat-map membership, and event emission — the right observables.

## Previous Findings

N/A — first review of PB-OS6.
