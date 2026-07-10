# Primitive Batch Review: PB-AC8 — Static restrictions, win-cons, no-max-hand

**Date**: 2026-07-09
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: `b65bf630`
**CR Rules (MCP-verified)**: 104.1, 104.2b, 104.3e/f/h, 508.1c/d, 701.21a, 704.5, 801.14
**Engine files reviewed**: `state/stubs.rs`, `state/hash.rs`, `effects/mod.rs`, `rules/combat.rs`,
`rules/casting.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `rules/turn_actions.rs`,
`rules/engine.rs`, `rules/events.rs`, `cards/card_definition.rs`, `state/mod.rs`
**Card defs reviewed**: 0 modified this commit (backfill deferred to post-review step per wip);
stale-comment audit performed on 5 PARTIAL cards
**Tests reviewed**: `crates/engine/tests/pb_ac8_restrictions_and_wingame.rs` (20 tests)

## Verdict: needs-fix

**No HIGH findings.** The three built primitives (`CantAttackOwner`, `CantBeSacrificed`,
`Effect::WinGame`), the cleanup layer-correctness bug fix, the hash arms, and the unplanned
`handle_all_passed` game-over poll are all correct against MCP-verified CR text and do not
allow illegal game states or produce wrong game state for any authored card. The mandatory
4-player WinGame test genuinely exercises the multiplayer path and the new poll. However, the
graded item — the **CantBeSacrificed full dispatch chain** — is **incomplete**: four delayed
self-sacrifice sites (Blitz, Encore, decayed-EOC, Mobilize) are unguarded while the
structurally-identical Evoke path *was* guarded, and the helper's own docstring falsely claims
"every sacrifice dispatch site" routes through it (E1, MEDIUM). A parallel CR 508.1d gap exists
on the goad requirement path (E2, MEDIUM). Both have zero current card impact but are the exact
half-wiring failure mode tracked by `feedback_verify_full_chain`. Plus minor LOWs.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **MEDIUM** | `resolution.rs:3308`, `resolution.rs:6814`, `turn_actions.rs:2144`, `turn_actions.rs:1087` | **Incomplete CantBeSacrificed dispatch chain — 4 delayed self-sacrifice sites unguarded.** Blitz, Encore, decayed-EOC, and Mobilize (`sacrifice_at_end_step`) sacrifices do NOT call `object_cant_be_sacrificed`, while Evoke (`resolution.rs:2953`) does. Helper docstring (`effects/mod.rs:7113-7116`) claims "Every sacrifice dispatch site ... must route through this helper" — false. **Fix:** guard all four sites mirroring Evoke, OR narrow the docstring to enumerate actual coverage + file an OOS seed. |
| E2 | **MEDIUM** | `combat.rs:284-312` | **Goad "must attack if able" does not yield to CantAttackOwner.** The MustAttackEachCombat path (`combat.rs:394-403`) got the owner-exclusion; the goad requirement path did not. A goaded creature with CantAttackOwner whose only opponent is its owner would be forced to declare an attack that is simultaneously illegal — no legal declaration exists. CR 508.1d violation. **Fix:** add the same `no_legal_target` owner-exclusion to the goad "must attack" scan. |
| E3 | LOW | `effects/mod.rs:3112` + `engine.rs:1577` | **WinGame immediate-win vs SBA ordering.** The game-over poll runs *after* `resolve_top_of_stack`'s SBA pass (`resolution.rs:7629`). A winner at ≤0 life not yet SBA-marked would be dragged into a draw instead of winning immediately (CR 104.1 "ends immediately"). The 104.3f guard only checks already-`has_lost`. Marginal; no roster card. **Fix:** none required now; note as a documented edge / OOS seed. |
| E4 | LOW | `combat.rs:394-403` | **Must-attack owner-exclusion ignores planeswalkers.** `no_legal_target` only scans players; a must-attack + CantAttackOwner creature able to attack its owner's planeswalker is wrongly treated as having no legal target and not forced. Consistent with pre-existing must-attack behavior (which never considered target availability). Marginal; no roster card. **Fix:** none required now; note as edge. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `alexios_deimos_of_kosmos.rs:9,28,29`; `thassas_oracle.rs:10,28`; also `hellkite_tyrant.rs`, `simic_ascendancy.rs`, `call_the_spirit_dragons.rs` | **Stale "not in DSL" comments now inaccurate.** These PARTIAL cards' TODOs assert primitives that this batch just built ("no `Effect::WinGame` variant", "sacrifice restriction not in DSL", "can't attack its owner ... not in DSL"). No card def was modified this commit (backfill deferred), so this is a forward-looking flag. **Fix:** in the backfill phase, UPDATE (do not delete — cards stay PARTIAL) each comment to reflect that the restriction/WinGame primitive now exists and name the *remaining* out-of-scope gap. |

### Finding Details

#### Finding E1: Incomplete CantBeSacrificed dispatch chain

**Severity**: MEDIUM
**CR Rule**: 701.21a — "can't be sacrificed" is an ordinary continuous restriction enforced at
every sacrifice site.
**Independent enumeration of ALL sacrifice sites** (not the runner's table):

| Site | Location | Guarded? |
|------|----------|----------|
| `eligible_sacrifice_targets` (→ `Effect::SacrificePermanents`, PB-AC2 optional-cost sac) | `effects/mod.rs:7153` | ✅ |
| Board-wipe "sacrifice all creatures" (Living Death) | `effects/mod.rs:5535` | ✅ |
| Activation cost — sacrifice self | `abilities.rs:622` | ✅ |
| Activation cost — sacrifice filter | `abilities.rs:762` | ✅ |
| Forage — sacrifice a Food | `abilities.rs:894` | ✅ |
| Cast cost — Emerge | `casting.rs:1898` | ✅ |
| Cast cost — Bargain | `casting.rs:3224` | ✅ |
| Cast cost — Casualty | `casting.rs:3283` | ✅ |
| Cast cost — SpellAdditionalCost (mandatory) | `casting.rs:3354` | ✅ |
| Cast cost — Devour | `casting.rs:4473` | ✅ |
| Evoke self-sac ETB | `resolution.rs:2953` | ✅ |
| Exploit "you may sacrifice" | `resolution.rs:3847` | N/A — engine unconditionally declines (VERIFIED at source; no sacrifice dispatch to guard) |
| **Blitz delayed self-sac** | `resolution.rs:3308-3333` | ❌ **MISSING** |
| **Encore delayed self-sac** | `resolution.rs:6814-6825` | ❌ **MISSING** |
| **decayed_sacrifice_at_eoc** | `turn_actions.rs:2144-2151` | ❌ **MISSING** |
| **sacrifice_at_end_step (Mobilize)** | `turn_actions.rs:1087` | ❌ **MISSING** |

`object_cant_be_sacrificed` appears in exactly 4 files (effects/mod.rs, casting.rs×5,
abilities.rs×3, resolution.rs×1) and **never** in `turn_actions.rs`. The cost-payment and
effect-driven chains (the sites that matter for real edicts/board-wipes) are complete and
correct. The gap is confined to keyword/mechanic delayed self-sacrifice triggers.

**Impact**: 0 authored cards. `CantBeSacrificed` is registered only via
`AbilityDefinition::StaticRestriction`; the sole roster user is Alexios (self-referential, not
Blitz/Encore/Mobilize/decayed), and no card grants a blanket "creatures you control can't be
sacrificed." But the Evoke path was explicitly wired "for dispatch-chain completeness" while
these structurally-identical siblings were not — this is precisely the half-wiring pattern
`feedback_verify_full_chain` exists to catch, and the helper docstring's universal claim is a
correctness-hazard comment (per conventions "aspirationally-wrong comments").
**Fix**: guard Blitz/Encore/decayed/Mobilize with `object_cant_be_sacrificed` mirroring Evoke,
OR downgrade the docstring to enumerate the actual covered set and record an OOS seed for the
delayed-sacrifice sites.

#### Finding E2: Goad requirement does not yield to CantAttackOwner

**Severity**: MEDIUM
**CR Rule**: 508.1d — "the number of requirements being obeyed [must equal] the maximum possible
... without disobeying any restrictions." A goaded creature (requirement, CR 701.15b) whose only
possible defender is forbidden by CantAttackOwner (restriction, CR 508.1c) can legally obey zero
requirements.
**Issue**: `combat.rs:355-414` (MustAttackEachCombat) correctly computes `no_legal_target` and
skips forcing. The goad "must attack if able" block at `combat.rs:284-312` does NOT — it forces
the attack whenever the creature is untapped/unsick/non-defender, ignoring CantAttackOwner. A
goaded Alexios-style creature in a 2-player game where the only opponent is its owner would have
no legal declaration (goad forces attack; CantAttackOwner forbids the only target).
**Impact**: 0 authored cards (requires goad co-applied with CantAttackOwner). But it is the same
CR 508.1d requirement the must-attack fix satisfied, left un-uniform.
**Fix**: apply the `no_legal_target` owner-exclusion to the goad scan as well.

## System-Interaction Verification (per review scope)

**B. Effect::WinGame correctness** — CORRECT.
- CR 104.3f: `effects/mod.rs:3114-3118` early-outs when the controller is already `has_lost`
  (`unwrap_or(true)` is a safe default). Verified by `test_wingame_controller_already_lost_is_noop`.
- CR 704.5: `sba.rs` was NOT modified (no WinGame/win reference; grep-confirmed). Winning is
  handled as an effect, not an SBA. ✅
- 4-player atomicity: marks ALL `!has_lost && !has_conceded` opponents in one pass
  (`effects/mod.rs:3120-3134`); does not remove players from `state.players`. `check_player_sbas`
  (`sba.rs:242-243`) skips already-lost players — no mid-batch player removal. ✅
- Event stream: `PlayerLost { reason: OpponentWonGame }` per opponent, then `GameOver { winner }`
  computed from `active_players()` (`state/mod.rs:1062-1072`, filters has_lost/has_conceded).
  Verified sane by the 4p test. ✅

**C. Unplanned `handle_all_passed` game-over poll** (`engine.rs:1577-1580`) — SAFE.
- In the stack-non-empty branch, `resolve_top_of_stack` has ALREADY run SBAs, flushed pending
  triggers, and granted priority (`resolution.rs:7622-7638`) before control returns. The early
  `return Ok(events)` only pre-empts the identical trailing `Ok(events)` at line 1608 and adds
  `check_game_over` — it skips NO priority hand-off, trigger flush, or cleanup. ✅
- Blast radius beyond WinGame: `is_game_over` can now also be true here for **draw-from-empty-
  library during resolution** (`effects/mod.rs:7530` sets `has_lost` directly, NOT via SBA). This
  is a latent *improvement* — previously such a `GameOver` was never emitted on that command
  (process_command has no final poll; the next command errored `GameAlreadyOver`). `cargo test`
  (3055 pass) confirms no regression. Skipping `flush_pending_triggers` on the early return is
  acceptable: the game has ended (CR 104.1) and triggers were already flushed inside
  `resolve_top_of_stack`. ✅

**D. CantAttackOwner keys on owner** — CORRECT.
- Declaration check `combat.rs:127` reads `obj.owner` (not controller). ✅ Verified by
  `test_cant_attack_owner_illegal_declaration` (owner p1, controller p2).
- CR 508.1d must-attack yield: `combat.rs:394-403` correctly excludes owner from legal targets.
  Verified by `test_cant_attack_owner_yields_mustattack_requirement`.
- Goad directional logic (`combat.rs:314-353`) UNCHANGED and not broken — but see E2 (goad
  "must attack" block does not get the owner-exclusion).

**E. hash.rs** — CORRECT.
- `HASH_SCHEMA_VERSION` 34→35 (`hash.rs:296`) with changelog entry; live sentinel test uses
  `assert_eq!(HASH_SCHEMA_VERSION, 35u8)` (strict-equality per convention).
- GameRestriction arms 9/10 added; match is **exhaustive with no wildcard** (`hash.rs:1890-1915`,
  arms 0–10 for all 11 variants) — a removed/added variant is a compile error.
- `Effect::WinGame => 90u8` (`hash.rs:6200`) sequential after CounterUnlessPays(89), no collision
  within the Effect impl. `LossReason::OpponentWonGame => 5u8` (`hash.rs:1105`).
- **No new struct fields** anywhere (WinGame reuses `has_lost`; restrictions ride the existing
  hashed `ActiveRestriction`). This cleanly avoids the PB-AC1/AC5 new-field hash HIGH.
- Mutation-verified tests genuinely distinguish variants (`test_hash_distinguishes_*`): they
  assert two *different* variants hash differently, and arm removal is a compile error — they
  cannot trivially pass against a removed arm.
- Bulk 34u8→35u8 sentinel bump across ~26 test files: **could not diff without git access**;
  build + `cargo test --all` (3055) passing is consistent with a mechanical change. Flagged as a
  verification limitation, not a finding.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 104.1 (game ends immediately when a player wins) | Yes | Yes | 4p test asserts exactly one active player + GameOver |
| 104.2b (effect states a player wins) | Yes | Yes | `test_wingame_1v1_*` |
| 104.3f (win+lose simultaneously → lose) | Yes | Yes | `test_wingame_controller_already_lost_is_noop` |
| 104.3h/801.14 (limited-range) | Correctly NOT applied | n/a | Commander doesn't use rule 801; plan correction verified via MCP |
| 508.1c (CantAttackOwner restriction) | Yes | Yes | `test_cant_attack_owner_illegal_declaration` / `_can_attack_other_player` |
| 508.1d (requirement yields to restriction) | Partial | Yes (must-attack) | must-attack path ✅; **goad path ✗ (E2)** |
| 701.21a (can't be sacrificed) | Partial | Partial | effect + cost sites ✅; **4 delayed sites ✗ (E1)**; cast-cost sites untested (T1) |
| 704.5 (win-by-effect is not an SBA) | Yes | n/a | sba.rs unmodified |
| 402.2/514.1 (cleanup discard skip) bug fix | Yes | Yes | `test_cleanup_layer_granted_no_max_hand_size_skips_discard` |

## Test Quality Assessment

- `test_wingame_4player_all_three_opponents_lose`: genuinely 4 players via `four_player()`,
  casts a real spell, resolves through `process_command`, asserts all 3 opponents `has_lost`,
  `active_players() == [p1]`, and `GameOver { winner: Some(p1) }`. Exercises architecture
  invariant #5 AND the new `handle_all_passed` poll (the only emitter of GameOver here). Strong.
- CantBeSacrificed: has a **negative control**
  (`test_cant_be_sacrificed_negative_normal_permanent_is_sacrificed`) — good. Covers edict,
  choice-exclusion, board-wipe, activation-cost-self, activation-cost-filter. Not tautological
  (assertions are on engine zone-change behavior, not test-set state).
- Hash tests distinguish variants (mutation-verified) and use a live `assert_eq!` sentinel.
- **T1 (LOW test gap)**: no test covers CantBeSacrificed at the cast-time cost sites
  (Emerge/Bargain/Casualty/Devour/SpellAdditionalCost) though guards were added there; and none
  cover the E1 delayed-sacrifice sites. The "full chain" claim is only partially tested. **Fix:**
  add at least one cast-cost negative test in the fix phase.

## Card Def Summary

| Card | Modified this commit | Stale comment now inaccurate | Notes |
|------|----------------------|------------------------------|-------|
| (none) | 0 | — | Backfill (Nezahal, Toski + PARTIALs) is the next, unchecked wip step |
| alexios_deimos_of_kosmos | No | Yes (C1) | "sacrifice restriction / attack restriction not in DSL" now false |
| thassas_oracle | No | Yes (C1) | "no Effect::WinGame variant" now false; card stays PARTIAL (LookAtTopN + devotion-vs-library gaps) |
| hellkite_tyrant / simic_ascendancy / call_the_spirit_dragons | No | Partial | comments partially updated; verify during backfill |

## Summary

- **0 HIGH.** No wrong game state for any authored card; no illegal states; no missing hash arm;
  no new unhashed struct field; no `.unwrap()` in library code introduced.
- **2 MEDIUM** (E1 incomplete CantBeSacrificed delayed-sac chain; E2 goad path CR 508.1d gap) —
  both are within-primitive completeness/consistency gaps with 0 current card impact but match
  the tracked half-wiring failure mode; both should be fixed for correctness completeness.
- **4 LOW** (E3 win-vs-SBA ordering edge; E4 planeswalker target edge; T1 cast-cost test gap;
  C1 stale PARTIAL-card comments to correct during backfill).
