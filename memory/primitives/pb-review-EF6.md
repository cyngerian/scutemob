# Primitive Batch Review: PB-EF6 — `TargetRequirement::TargetOpponent`

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 102.2 / 102.3 (opponent definition), 115.1 / 115.3 (targeting), 601.2c
(declaration-time restriction), 603.3d (trigger removed if no legal choice), 608.2b
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs`,
`crates/engine/src/state/hash.rs`, `crates/engine/src/rules/casting.rs`,
`crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/protocol.rs`,
`crates/engine/tests/core/{protocol_schema,hash_schema}.rs`
**Card defs reviewed** (8): shaman_of_the_pack, raiders_wake, vengeful_bloodwitch, fell_specter
(all verified against MCP oracle), blood_tribute, blessed_alliance, forbidden_orchard,
ajani_sleeper_agent (flare_of_malice confirmed untouched)
**Test file**: `crates/engine/tests/primitives/pb_ef6_target_opponent.rs` (8 tests)

## Verdict: needs-fix (LOW only)

**Zero HIGH, zero MEDIUM findings.** The primitive is correct end-to-end: the enum variant,
hash discriminant 18, the `caster`-threaded declaration-time validation (`id != caster`), the
object-side rejection, and both auto-target pickers (outer + UpToN-inner) with **no**
self-fallback all match CR 102.3 / 601.2c / 603.3d. Both call sites of
`validate_player_satisfies_requirement` were updated (grep-confirmed: the only two callers,
casting.rs:5830 and 6018, plus the recursive UpToN delegation at 6088 — no hidden caller at old
arity). Wire bump PROTOCOL 10→11 / HASH 48→49 is machine-forced, history rows are appended (not
edited), fingerprints in the const match the appended tail row and the frozen-prefix digests were
re-pinned, and all 31+ `HASH_SCHEMA_VERSION` sentinels plus `protocol_version_sentinel` are at
49/11 with no stray 48/10. All four flipped/corrected defs match oracle text exactly; the four
non-Complete defs carry truthful markers citing their real (non-TargetOpponent) surviving blockers.
Tests are non-vacuous (the two decoys were proven to redden under injected breakage per the WIP
attestation, and my read of the setups confirms each discriminates on the opponent-ness / no-self-
fallback property). Only three LOW test/doc-hygiene items and one informational note remain — none
block the batch.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `card_definition.rs:2877` (+ card markers) | **CR 102.4 mis-cited for the opponent definition.** Doc-comment says "CR 102.3/102.4: opponent = any player not on your team". CR 102.4 is the *"your team" shorthand* rule, not the opponent definition; the FFA opponent definition is CR 102.2 (two-player) + 102.3 (teams). **Fix:** change the citation to "CR 102.2/102.3" in the enum doc-comment and in the vengeful_bloodwitch/forbidden_orchard markers that echo "102.4". Cosmetic; no behavior impact. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | LOW | `pb_ef6_target_opponent.rs` (shaman test) | **`TargetController::You` on the count filter is untested.** `test_shaman_of_the_pack_etb_targets_opponent_loses_life` never puts an Elf on an opponent's battlefield, so the 3-Elf case cannot distinguish `controller: You` from `controller: Any`. The filter field is set correctly and the machinery is shared with eomer, so this is a decoy gap, not a bug. **Fix (opportunistic):** add a P2-controlled Elf to the 2-Elf case and assert P2 still loses exactly 3 (proving opponent Elves are excluded). |
| 3 | LOW | `pb_ef6_target_opponent.rs` | **No negative test that an object target is rejected for `TargetOpponent`.** casting.rs:6367 (`TargetPlayer | TargetOpponent => false`) is build-enforced but unexercised. **Fix (opportunistic):** cast the test spell at a `Target::Object(<a creature>)` and assert `Err(InvalidTarget)`. |

### Finding Details

#### Finding 1: CR 102.4 mis-cited for opponent definition
**Severity**: LOW
**File**: `crates/card-types/src/cards/card_definition.rs:2877` (also echoed in
`vengeful_bloodwitch.rs:25`, `forbidden_orchard.rs` marker)
**CR**: 102.2 "In a two-player game, a player's opponent is the other player." / 102.3 (teams).
102.4 is unrelated ("your team" shorthand).
**Issue**: The variant doc and two card markers attribute the opponent definition to CR 102.4,
which actually defines "your team". Substantively harmless (102.3 is also cited and is correct),
but a future reader chasing the citation lands on the wrong rule.
**Fix**: Replace "102.4" with "102.2" in the opponent-definition citations.

#### Finding 2: You-restriction on the Elf count is not decoy-tested
**Severity**: LOW
**Oracle**: "target opponent loses life equal to the number of **Elves you control**."
**Issue**: The integration test only ever creates P1-controlled Elves, so it cannot catch a
regression that widened `controller: You` to `Any`. Not a mislabeled test (the name promises
opponent-targeting + life loss, both of which it validates), so this is a genuine LOW gap rather
than a fix-phase-HIGH test-validity defect.
**Fix**: Add an opponent-controlled Elf to the 2-Elf case and assert the loss stays 3.

#### Finding 3: Missing object-side rejection test
**Severity**: LOW
**CR**: 601.2c — a player requirement cannot be satisfied by an object target.
**Issue**: `TargetOpponent` on the object-side `valid` match returns `false`; only compile
exhaustiveness guards it, no runtime test.
**Fix**: Add a case casting the TargetOpponent test spell at a `Target::Object` and assert
`Err(InvalidTarget)`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 102.2/102.3 (opponent = non-caster, no teams) | Yes (`id != caster`) | Yes | test 1/2 (4p accept-opp/reject-self), test 8 |
| 115.1 / 601.2c (declaration-time restriction) | Yes (casting.rs:6091) | Yes | test 1 rejects self, accepts P2/P3/P4 |
| 603.3d (no legal target → trigger removed, no self-fallback) | Yes (abilities.rs:6908, 7035; NO `.or_else` fallback) | Yes | test 8 (has_lost opponent → stack empty, P1 life untouched) — decoy proven non-vacuous |
| Object-side player-req rejection (601.2c) | Yes (casting.rs:6367) | No | build-enforced only (Finding 3) |
| UpToN delegation carries `caster` | Yes (casting.rs:6088) + inner picker (abilities.rs:7035) | Indirect | hash-distinctness test uses UpToN; no `UpToN{TargetOpponent}` card exists yet |
| 608.2b (no resolution re-check needed) | N/A — unchanged (DECISION 4) | N/A | opponent-ness cannot change; correct to leave `is_target_legal` untouched |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| shaman_of_the_pack | Yes | 0 | Yes | inert→Complete; ETB LoseLife + PermanentCount{Elf,You}; subtype-only (correct) |
| raiders_wake | Yes | 0 | Yes | partial→Complete; both triggers present; Raid uses AtBeginningOfYourEndStep + YouAttackedThisTurn intervening-if |
| vengeful_bloodwitch | Yes | 0 | Yes | known_wrong→Complete; WheneverCreatureDies{You, exclude_self:false} (own death triggers, correct); LoseLife(opp)/GainLife(you) |
| fell_specter | Yes | 0 | Yes | stays Complete; Flying + ETB TargetPlayer→TargetOpponent; second discard trigger unchanged |
| blood_tribute | Yes | 3 (HalfLife/non-mana Kicker/if-kicked) | N/A (partial) | TargetOpponent applied; marker truthfully cites HalfLife |
| blessed_alliance | Yes | 2 (Escalate+mode_targets, up-to-two) | N/A (partial) | idx3→TargetOpponent, idx0 kept TargetPlayer (mode 0 = any player, correct); marker cites Escalate blocker |
| forbidden_orchard | Yes | documented | N/A (known_wrong) | TargetOpponent applied but marker correctly notes the target is *dead* on the WhenTappedForMana Normal-kind dispatch path; recipient-wiring reverted per default-to-defer; marker cites the WhenTappedForMana auto-target gap **and** EF-W-PB2-3 |
| ajani_sleeper_agent | Yes | many (no-op +1/-3, spell filter, Compleated) | N/A (known_wrong) | emblem TargetPlayer→TargetOpponent; stale "targets any player" clause dropped from marker; TODO comments removed |
| flare_of_malice | — | — | — | untouched (correct — wrong-oracle, full re-author out of scope) |

## Notes / Non-findings (verified, not defects)

- **Auto-picker selects the first active opponent deterministically** rather than offering the
  controller a choice among opponents (CR 601.2c/603.3d say the controller chooses). This is a
  **pre-existing engine limitation** shared identically with the `TargetPlayer` family picker
  (abilities.rs:6876–6903) — not introduced by PB-EF6 and out of scope. Consistent with engine
  convention; no finding filed.
- **`blessed_alliance` mode-0 kept `TargetPlayer`** (index 0) — confirmed only index 3 changed;
  mode 0 "target player gains 4 life" genuinely targets any player. No legal-but-wrong regression.
- **forbidden_orchard declares a target it never resolves** — harmless: the trigger stays
  known_wrong and creates the Spirit for the controller exactly as before (the WhenTappedForMana
  dispatch gap means `targets` is unread). No new wrong game state vs. the pre-batch known_wrong def.
- **Wire bump**: PROTOCOL_HISTORY row 11 (`07e51466…`) matches PROTOCOL_SCHEMA_FINGERPRINT;
  HASH_SCHEMA_HISTORY row 49 (decl `0f8e380b…`, stream `d3f8ecb0…`) appended; frozen-prefix digests
  re-pinned (protocol `971517e8…`, hash `ae92dcee…`). Rows appended, not edited. Correct.
- **No `.unwrap()`/`.expect()`** introduced in engine library code (validation returns typed
  `Err`; pickers use `unwrap_or(false)`, non-panicking).
- **Exhaustive-match completeness**: only three engine files match on `TargetRequirement::`
  variants (hash.rs, abilities.rs, casting.rs) — all updated. The object-scan closure's added
  `TargetPlayer | TargetOpponent => false` (abilities.rs:7174) is correct. Other files reference the
  type but never enumerate variants; `cargo build --workspace` is the seal.
