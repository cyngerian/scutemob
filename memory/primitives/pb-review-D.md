# Primitive Batch Review: PB-D — TargetController::DamagedPlayer

> **Note**: this file supersedes the older `pb-review-D.md` written for the stillborn
> PB-D "Chosen Creature Type" (2026-04-02). The `PB-D` letter was reused in the
> post-PB-N queue per `docs/primitive-card-plan.md` Phase 1.8.

**Date**: 2026-04-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 510.1, 510.3a, 601.2c, 603.2, 603.3d
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (enum + doc comment)
- `crates/engine/src/state/hash.rs` (sentinel bump + HashInto arm)
- `crates/engine/src/rules/casting.rs` (2 sites: TargetCreatureWithFilter / TargetPermanentWithFilter validation)
- `crates/engine/src/rules/abilities.rs` (4 sites: auto-target TCWF/TPWF + sacrifice-trigger defensive + runtime-targets fix)
- `crates/engine/src/effects/mod.rs` (5 sites: DestroyAll x2, ExileAll, match-filter-controller, EachPermanentMatching)
- `crates/engine/src/testing/replay_harness.rs` (non-exhaustive `matches!` and wildcard `_ =>` paths — no change required, fall-through safe)

**Card defs reviewed** (6):
- `throat_slitter.rs` — precision fix
- `sigil_of_sleep.rs` — precision fix
- `mistblade_shinobi.rs` — newly authored (mandatory, per R7)
- `alela_cunning_conqueror.rs` — goad trigger authored
- `natures_will.rs` — Effect::Sequence with two ForEach blocks
- `balefire_dragon.rs` — newly authored (ForEach + CombatDamageDealt)

**Tests reviewed**: `crates/engine/tests/pbd_damaged_player_filter.rs` (7 MANDATORY tests M1–M7).

---

## Verdict: PASS-WITH-NOTES

No HIGH findings. No MEDIUM findings that require blocking remediation. The implementation precisely matches the plan. Every dispatch site from the Change 7 table is handled. The hash sentinel bump and variant discriminant are correct. All 6 card defs faithfully encode their oracle text. The runner's one stop-and-flag (runtime-target honoring in `flush_pending_triggers`) is a legitimate and necessary fix: without it, the M1/M2/M4/M5 tests that attach triggers via `ObjectSpec::with_triggered_ability` would not see their runtime `targets` vector consulted, and the primitive would appear to work in card-registry tests but silently fail everywhere else. The fix is scoped narrowly (PendingTriggerKind::Normal only) and preserves the registry fallback path.

One LOW finding: stale deferral comment in Marisi pointing at "PB-37" as the blocker for DamagedPlayer support, now that PB-D has landed the primitive. Marisi stays deferred on CantCast, but the specific note is misleading. Not a blocker.

Task is ready to signal-ready.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `crates/engine/src/cards/defs/marisi_breaker_of_the_coil.rs:22-23` | **Stale TODO reference to PB-37.** After PB-D ships the primitive, the comment "goad each creature that player controls ... requires TargetController::DamagedPlayer support. Deferred to PB-37" is misleading — the support now exists. Marisi remains blocked on the CantCast TODO at line 21, but this specific deferral reason is stale. **Fix:** update the TODO to say "Deferred on CantCast (phase-scoped) primitive above; DamagedPlayer ForEach support landed in PB-D." Or collapse the two TODOs into one. Non-blocking; Marisi is a deferred card. |

## Card Definition Findings

None. All 6 cards correctly encode oracle text, use the new primitive as designed, and preserve/document the correct residual TODOs (Mistblade MayEffect; Alela first-spell-per-turn).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| CR 510.1 (combat damage assignment) | Pre-existing | Exercised via M1/M2/M4/M5/M7 | PendingTrigger.damaged_player plumbing already in place |
| CR 510.3a (damage-dealt triggers) | Yes | M1, M4, M5, M7 | new variant reads `trigger.damaged_player` / `ctx.damaged_player` |
| CR 601.2c (target selection at trigger-stack-entry) | Yes | M1, M2, M3, M7 | auto-target-selection arm in `abilities.rs:6484-6487` and `6506-6509` |
| CR 603.2 / 603.2g (trigger event matching) | Pre-existing | — | unchanged |
| CR 603.3d (no legal target -> skip) | Yes | M2 | exercises filter exclusion then `all_satisfied = false` skip |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|--------------------|-------|
| throat_slitter | Yes | 0 | Yes | Combat-damage trigger + nonblack (exclude_colors=[Black]) + DamagedPlayer |
| sigil_of_sleep | Yes | 0 | Yes (combat path) | Uses `combat_only: false`; noncombat damage path has a pre-existing PB-37 limitation in abilities.rs (documented TODO at line ~4926, out of PB-D scope). Combat damage still sets damaged_player correctly. |
| mistblade_shinobi | Yes (mandatory form) | 1 (MayEffect, by design) | Yes | Authored as mandatory per plan R7 and Sigil precedent; TODO rephrased to point at future MayEffect primitive (not stripped, per plan directive) |
| alela_cunning_conqueror | Yes | 1 (first-spell-per-turn, pre-existing, preserved) | Yes | Goad trigger uses Faerie subtype filter on the triggering creatures + DamagedPlayer target controller |
| natures_will | Yes | 0 | Yes | Effect::Sequence with two ForEach blocks (DamagedPlayer lands tap, You lands untap); inner DeclaredTarget{index:0} semantics verified against `effects/mod.rs:2420` which propagates `damaged_player` into the inner_ctx |
| balefire_dragon | Yes | 0 | Yes | ForEach+DealDamage with CombatDamageDealt; `effects/mod.rs:2419-2420` propagates `combat_damage_amount` AND `damaged_player` into inner_ctx per iteration — R3 verified safe |

---

## Finding Details

### Finding E1: Stale PB-37 deferral comment in Marisi

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/marisi_breaker_of_the_coil.rs:22-23`
**Issue**: Marisi's goad sub-effect TODO reads "requires TargetController::DamagedPlayer support. Deferred to PB-37." After PB-D ships the primitive, the deferral reason is misleading — the support now exists. Marisi is still correctly deferred (the CantCast TODO at line 21 is a hard blocker), so the card-level disposition doesn't change; only the reason comment is stale.
**Fix**: Update the TODO text to: `TODO: "goad each creature that player controls" needs ForEach+DamagedPlayer (added in PB-D). Card still deferred on CantCast primitive above.` Or collapse both TODOs into one combined deferral note.
**Note on neighbor**: Similar residual at `skullsnatcher.rs:31` ("`that player's graveyard` -> TargetController::DamagedPlayer not in DSL") is subtly different — it's about a graveyard-zone filter scoped to the damaged player, which is NOT what PB-D provides (PB-D scopes battlefield permanents by controller, not graveyards by owner). The Skullsnatcher comment remains accurate as-is.

---

## Plan Adherence Checklist

- [x] Enum variant added with CR 510.3a doc comment, discriminant 3 preserved by ordering (`card_definition.rs:2398`)
- [x] Hash schema sentinel bumped 4 -> 5 with history comment (`hash.rs:31-32`)
- [x] Hash HashInto arm returns `3u8` (`hash.rs:4144`)
- [x] casting.rs x2: `DamagedPlayer => false` with CR 601.2c explanation (5525-5528, 5542-5545)
- [x] abilities.rs 6461-6468 (TargetCreatureWithFilter): reads `trigger.damaged_player.is_some_and(|dp| obj.controller == dp)` (6484-6487)
- [x] abilities.rs 6474-6482 (TargetPermanentWithFilter): same pattern (6506-6509)
- [x] abilities.rs 5629-5645 (sacrifice trigger): `DamagedPlayer => return false` defensive (5649-5651)
- [x] effects/mod.rs 869-873 (DestroyAll path 1): `ctx.damaged_player.unwrap_or(controller)` fallback (873-877)
- [x] effects/mod.rs 1050-1052 (ExileAll): same pattern (1058-1062)
- [x] effects/mod.rs 1155-1157 (DestroyAll path 2): same pattern (1168-1172)
- [x] effects/mod.rs 5410-5412 (match-filter-controller path): same pattern (5428-5432)
- [x] effects/mod.rs 7203-7206 (EachPermanentMatching, load-bearing): same pattern with explicit "load-bearing for Nature's Will + Balefire Dragon" comment (7227-7233)
- [x] replay_harness.rs 2342: `matches!` non-exhaustive, falls through to false — correct (no change required)
- [x] replay_harness.rs 2445: `matches!` non-exhaustive, falls through to false — correct (no change required)
- [x] replay_harness.rs 2776-2782: wildcard `_ =>` arm handles DamagedPlayer (falls through to AnyPlayerDrawsCard) — correct (no change required)
- [x] HASH_SCHEMA_VERSION history comment (used 2026-04-19 rather than plan date 2026-04-13 — cosmetic, reflects commit date)
- [x] 6 card defs updated: all match oracle text; Mistblade TODO rephrased, not stripped (per R7); Alela first-spell TODO preserved (per plan); throat_slitter/sigil/balefire/nature TODOs stripped
- [x] 7 MANDATORY tests M1-M7 all present, cite CR 510.3a (M1/M2/M4/M5/M7) or CR 601.2c (M3), or note hash infra (M6)
- [x] M6 asserts `HASH_SCHEMA_VERSION == 5u8`
- [x] M4 asserts the 4/4 land split (P2's 4 Forests tapped, P3's 4 Plains untapped) — the specific discriminator the plan called for
- [x] M5 asserts multiplayer isolation for DestroyAll (P3's creatures destroyed, P2's unaffected)

## Stop-and-Flag Review

**Runner disclosure**: `flush_pending_triggers` (`abilities.rs:6307-6326`) fix to honor `ab.targets` when non-empty.

**Reviewer assessment**:
- **Correctness**: The `TriggeredAbilityDef.targets: Vec<TargetRequirement>` field exists at `game_object.rs:608`. Prior behavior only consulted the card registry for auto-target selection, which skipped runtime triggers added via `ObjectSpec::with_triggered_ability` (the entire path used by M1/M2/M4/M5). Without this fix, tests that build triggers in-test rather than via CardDefinition would silently collapse to "no targets required" and resolve on a null target — a false pass. The fix reads runtime `ab.targets` first; on empty, falls through to registry. This is strictly more correct and cannot regress any scenario where the runtime targets were intentionally left empty (both paths now agree in that case).
- **Scope**: Gated on `PendingTriggerKind::Normal` only, so CardDefETB (which does not consult runtime triggered_abilities) is unaffected.
- **Plan alignment**: The plan's Change 5 prescribes "add the arm at `abilities.rs:6461-6468`" — the auto-target-selection site. But that site is only reached when `ability_targets` is non-empty, which requires the runtime-targets fix at 6307. The fix is a necessary precondition for the plan's Change 5 to have observable effect in tests. Not a scope expansion; a prerequisite.

**Verdict**: accept the fix. It is correct, necessary, narrowly scoped, and does not expand PB-D's surface.

**Other runner disclosures**:
- **Hash-version comment date (2026-04-19 vs plan's 2026-04-13)**: cosmetic, reflects commit date. Accept.
- **Mistblade "you may" TODO rephrased, not stripped**: confirmed at `mistblade_shinobi.rs:24-25`. Matches plan R7 exactly. Accept.

---

## Previous Findings

N/A for this review cycle (reusing filename that previously held a review for a different, stillborn PB-D; the older "Chosen Creature Type" review is not re-relevant to this PB).
