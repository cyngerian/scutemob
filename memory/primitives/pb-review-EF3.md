# Primitive Batch Review: PB-EF3 — attack-trigger target fidelity + defending-player target

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 508.1m, 508.4, 506.4c, 601.2c, 603.2/603.3d, 113.7a, 306.8
**Engine files reviewed**: `crates/engine/src/testing/replay_harness.rs` (A1),
`crates/engine/src/rules/abilities.rs` (A2, guard, B1, flush), `crates/engine/src/rules/resolution.rs` (B2 threading, fizzle rule),
`crates/engine/src/effects/mod.rs` (B4 resolve arms, EffectContext), `crates/card-types/src/state/stack.rs` (B2 field),
`crates/card-types/src/cards/card_definition.rs` (B3 variants), `crates/engine/src/state/hash.rs` (B5, HASH 46),
`crates/engine/src/rules/protocol.rs` (PROTOCOL 8), `crates/engine/tests/core/bare_lookup_ratchet.rs`
**Card defs reviewed**: `ojutai_soul_of_winter.rs` (new), `hellrider.rs` (flip), `raid_bombardment.rs` (new) — 3 total
**Test file**: `crates/engine/tests/primitives/pb_ef3_attack_trigger_targets.rs`
**Golden script**: `test-data/generated-scripts/combat/192_mutate_gemrazer.json`

## Verdict: needs-fix

The core of PB-EF3 is correct and well-executed. A1 (forward `targets` in enrich), A2 (make runtime
`triggered_abilities` authoritative for `Normal`, def raw-index for `CardDefETB`), the 4-site
`Normal`→`CardDefETB` reclassification, the `has_ability_targets` guard, the B1 per-attacker
defending-player fan-out, the B2 threading, the wire double-bump, and the three shipped cards are all
verified sound against CR text and oracle text. The reclassification — the highest-risk change — is
genuinely a correctness fix, not a masking hack: all four sites raw-index `def.abilities`, and no
kind-sensitive downstream behavior (doubling, `private_to`, replacement, `once_per_turn`) diverges
under `CardDefETB`. **However, two MEDIUM edge-case correctness bugs surfaced in Part B's target
resolution**, both around what happens to a non-targeted attack-trigger effect when the defending
player / attacked planeswalker leaves before the trigger resolves. Neither is on the common path (all
tests are green and non-vacuous), but both produce wrong game state in reachable multiplayer
scenarios and both are contradicted by the code's own comments. Plus three LOW documentation/latent
items.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:6383-6388` | **`EffectTarget::AttackTarget` redirects to the planeswalker's controller instead of fizzling when the attacked planeswalker is removed (CR 506.4c).** The `from_combat`=None fallback to `ctx.defending_player` (= the removed planeswalker's controller) deals the damage to that player. Oracle + CR 506.4c: the attacker is attacking nothing → no damage. **Fix:** capture the LKI attack target (Player-vs-Planeswalker), and only fall back to `ctx.defending_player` when the original attack target was a **Player**. |
| 2 | **MEDIUM** | `abilities.rs:6731-6738` | **The `defending_player_id`→stack-target shortcut fires for every non-targeted `AnyCreatureYouControlAttacks` trigger, giving token/damage/lifegain effects a spurious `Target::Player(dp)` that wrongly fizzles the whole ability if the defending player leaves the game before it resolves (CR 608.2b vs a non-targeted effect).** B1 now tags *all* such triggers with `defending_player_id`; the shortcut (intended only for annihilator-family DeclaredTarget{0} effects) then sets a real stack target on Utvara Hellkite / Dromoka / Hellrider / Raid Bombardment etc. **Fix:** gate the shortcut so it only applies to triggers whose effect actually consumes `DeclaredTarget{0}` as the defending player (annihilator/dethrone/training), not to every `defending_player_id`-tagged trigger. |
| 3 | LOW | `effects/mod.rs:6560-6561` | **`PlayerTarget::DefendingPlayer` falls back to `ctx.controller` when the captured defender `has_lost`.** If the captured defending player has left, the arm returns the *controller* (line 6561), so a "defending player loses N life" effect would drain the controller. Latent (no shipped user; Brutal Hordechief blocked). Inconsistent with the `AttackTarget` arm, which returns empty on `has_lost`. **Fix:** return `vec![]` when `ctx.defending_player` is `Some` but not alive; keep the `controller` default only for the `None` (non-attack) context. |
| 4 | LOW | `abilities.rs:4703-4704` | **Stale comment.** After reclassifying the WhenDealsCombatDamageToPlayer carddef fallback to `CardDefETB`, the comment still says "The PendingTriggerKind::Normal path looks them up at resolution via the card registry fallback." **Fix:** update to reference `CardDefETB`. |
| 5 | LOW | `tests/core/bare_lookup_ratchet.rs:63-65` | **Ratchet justification comment describes behavior the code does not produce.** It claims the `state.objects.get(pw_id)` check makes the effect "correctly resolve to empty" per CR 506.4c — but the `ctx.defending_player` fallback redirects to the controller (Finding 1). The comment is otherwise a valid NONSWALLOW justification. **Fix:** correct the comment when Finding 1 is fixed. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | — | No card-def findings. All three shipped cards match oracle text exactly and use no gated-stub effect. |

### Finding Details

#### Finding 1: `EffectTarget::AttackTarget` does not fizzle on CR 506.4c (attacked planeswalker removed)

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:6363-6389` (fallback at 6383-6388)
**CR Rule**: 506.4c — "It continues to be an attacking creature, although it is not attacking any
player, planeswalker, or battle."
**Issue**: When the triggering attacker was attacking a planeswalker that is removed before the
trigger resolves, `from_combat` returns `None` (the `Planeswalker` arm's
`.filter(|o| o.zone == Battlefield)` rejects the gone planeswalker). The code then falls back to
`match ctx.defending_player { Some(dp) if alive => vec![Player(dp)], _ => vec![] }`. B1 captured
`ctx.defending_player` = the planeswalker's controller, who is alive, so the effect deals its damage
to the **controller** rather than fizzling. Hellrider attacking a planeswalker, in response to which
the planeswalker is destroyed/bounced, wrongly deals 1 to that player. The plan (§Part B, and the
`bare_lookup_ratchet` comment) explicitly promised "resolve to empty and the damage fizzles," which
the implementation cannot reach because `defending_player` is always populated for planeswalker
attacks. `ctx.defending_player` remains correct for `PlayerTarget::DefendingPlayer` (CR 508.4: the
defending player of a planeswalker attack *is* its controller) — only `AttackTarget`'s reuse of it
as a recipient fallback is wrong.
**Fix**: Thread the LKI attack target itself (e.g. capture `Option<AttackTarget>` at dispatch, or a
`bool attack_target_was_player`), and in the `AttackTarget` arm only fall back to
`ctx.defending_player` when the captured target was a Player; when it was a Planeswalker now gone,
return `vec![]`. Add a test: attacker attacks a planeswalker, remove it from combat/battlefield
before the trigger resolves, assert Hellrider deals 0 and the controller's life is unchanged.

#### Finding 2: spurious defending-player stack target fizzles non-targeted attack triggers

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6731-6738` (the
`trigger.defending_player_id.filter(|_| !has_ability_targets)` shortcut), interacting with the
resolution fizzle rule at `resolution.rs:2058-2059`.
**CR Rule**: 508.4 / 608.2b — a non-targeted attack-trigger effect is not a targeted ability and must
not be countered for "all targets illegal."
**Issue**: The `defending_player_id`→`Target::Player` shortcut exists for annihilator/dethrone/
training, whose effects read `DeclaredTarget{0}`. B1 now tags **every**
`AnyCreatureYouControlAttacks` trigger with `defending_player_id`. Any such trigger with
`targets: vec![]` and a non-`DeclaredTarget` effect (Utvara Hellkite's token creation, Dromoka's
life gain, and the newly-shipped Hellrider / Raid Bombardment, which use `EffectTarget::AttackTarget`)
therefore has a real `Target::Player(dp)` placed on its stack object. If `dp` leaves the game (loses/
concedes) while the trigger is on the stack, `resolution.rs:2058` sees a non-empty targets vec with
all targets illegal and **fizzles the whole ability** — so Utvara creates no Dragon token, etc. This
is a widening of a pre-existing latent issue (SelfAttacks already tagged `defending_player_id`) to
the much larger `AnyCreatureYouControlAttacks` cohort. The `has_ability_targets` guard correctly
prevents the *opposite* bug (a real declared target being overwritten — Ojutai), but does not prevent
the spurious target on genuinely target-less effects. Note: for the two shipped damage cards the
observable harm is small (if the defending player left, the damage is moot); the real regression is on
token/lifegain effects like Utvara.
**Fix**: Restrict the `defending_player_id` shortcut to triggers whose effect actually consumes the
defending player via `DeclaredTarget{0}` (i.e. keyword annihilator/dethrone/training-derived
triggers), rather than firing for any `defending_player_id`-tagged trigger. A clean approach: gate the
shortcut on `trigger.kind`/effect shape, or migrate the annihilator family to
`PlayerTarget::DefendingPlayer` (now available) and drop the stack-target shortcut. Add a decoy test:
a non-targeted `AnyCreatureYouControlAttacks` token/lifegain trigger where the defending player
concedes before it resolves — assert the effect still happens.

## Highest-risk-area verdicts (per task focus)

**1. Reclassification (Normal→CardDefETB, 4 sites) — CORRECT, not a masking hack.**
- `WhenYouCastThisSpell` (`abilities.rs:3453`), `WhenExertedAsAttacks` (`abilities.rs:3748-3760`),
  `WhenDealsCombatDamageToPlayer` carddef fallback (`abilities.rs:4716-4728`), `WheneverRingTemptsYou`
  (`abilities.rs:5587-5605`) — each derives `ability_index` from `def.abilities.iter().enumerate()`, a
  raw def index. `CardDefETB` is exactly the kind whose contract is "index into `CardDef::abilities`,
  always resolve via the registry" (`resolution.rs:1977-1990`, `abilities.rs:6791-6805`). Correct.
- Downstream kind-sensitivity checked: `doubler_applies_to_trigger` (`abilities.rs:8015`) branches on
  `triggering_event`/`controller`/`entering_object_id`, **not** `kind` — doubling unchanged.
  `GameEvent::AbilityTriggered` is emitted identically → `private_to` unchanged. Replacement uses
  `CardDefETB` only for saga/ETB (`replacement.rs`), untouched by these check_triggers sites.
  `once_per_turn_flag` (`abilities.rs:6621`) is *improved* (registry path now correct for def indices).
  None of the 4 sites set `embedded_effect`, so `flush`'s `CardDefETB → embedded_effect: None`
  (`abilities.rs:7859-7866`) loses nothing. The auto-target/`has_ability_targets` reads match the
  index namespace per kind.

**2. `has_ability_targets` guard — CORRECT.** Reads runtime `triggered_abilities[idx].targets` for
`Normal` and `def.abilities[idx]` `Triggered.targets` for `CardDefETB` — the same namespaces A2 uses.
Every existing shortcut user (annihilator/exalted/dethrone/training) declares `targets: vec![]`, so
`has_ability_targets` is false for them and the shortcut still fires. Ojutai (real targets) → guard
true → shortcut skipped → auto-select reads the real `TargetPermanentWithFilter`. Verified sound.

**3. B1 per-attacker fan-out — CORRECT.** `abilities.rs:3903-3921`: `pre_len` captured per attacker,
`triggers[pre_len..]` tagged with that attacker's own `defending_player`; Planeswalker→controller.
Identical to the SelfAttacks pattern. `test_raid_bombardment_power_filter` proves no cross-wiring
between two attackers attacking different players.

**4. B4 resolution + B2 threading — CORRECT (modulo Findings 1/2).** `AttackTarget` maps
Player→`ResolvedTarget::Player`, Planeswalker→`ResolvedTarget::Object`; `DefendingPlayer` mirrors
`DamagedPlayer`. `ctx.defending_player` set in both kicker (`resolution.rs:2110-2112`) and non-kicker
(`resolution.rs:2203-2205`) paths, and in `flush` (`abilities.rs:7885-7888`).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 508.4 (defending player) | Yes | Yes | `test_hellrider_damages_defending_player_4p`, `test_defending_player_target_multiplayer` |
| 508.1m/601.2c (attack trigger targets) | Yes (A1/A2) | Yes | `test_attack_trigger_forwards_declared_target` + decoy; golden 192 |
| 603.3d (auto-select) | Yes | Yes | decoy adds a land (illegal target) to pin identity |
| 506.4c (pw removed → attacker attacks nothing) | **Partial** | **No** | Finding 1: pw-present case tested; pw-removed redirects to controller |
| 113.7a (LKI capture) | Yes | Yes | `test_defending_player_captured_survives_attacker_removal` (player case) |
| 306.8 (pw damage ≠ controller life) | Yes | Yes | `test_hellrider_damages_attacked_planeswalker` |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ojutai_soul_of_winter.rs | Yes | 0 | Yes | 5/6, {5}{W}{U}, Legendary Dragon; Flying+Vigilance; Dragon-filter trigger; TapPermanent+PreventNextUntap on TargetPermanentWithFilter{non_land, Opponent}. Correct. |
| hellrider.rs | Yes | 0 (TODO removed) | Yes* | 3/3 Devil, {2}{R}{R}, Haste; DealDamage{AttackTarget, 1}. *Correct except Findings 1/2 edge cases. |
| raid_bombardment.rs | Yes | 0 | Yes* | Enchantment {2}{R}; max_power:2 filter; DealDamage{AttackTarget, 1}. Per-attacker filter verified. *Findings 1/2 edge cases. |

No gated-stub effects (Choose/MayPayOrElse/AddManaChoice/AddManaAnyColor) present. Blocked cards
spot-checked against oracle via MCP and genuinely inexpressible: **Silumgar** (defending-player-scoped
continuous `-1/-1` — needs a locked `EffectFilter::CreaturesControlledBy`, OOS-EF3-1);
**Brutal Hordechief** (ability 1 expressible now, ability 2 "opponents block if able + you choose
blocks" inexpressible); **Cunning Rhetoric** (defender-side "opponent attacks you" trigger + play-from-
exile any-color — different primitive). Norn's Decree / Karazikar likewise multi-gap. All correctly
left unauthored.

## Wire / gate verdicts

- **PROTOCOL 7→8**: correct; `EffectTarget`/`PlayerTarget` variants are in the SR-8 fingerprint
  closure. `- 8:` History line + `PROTOCOL_HISTORY` epoch row appended (`protocol.rs:99, 216`).
- **HASH 45→46**: correct; `StackObject.defending_player` is in the GameState hash closure and is
  hashed (`hash.rs:3652`). `- 46:` History line + `HASH_HISTORY` row (`hash.rs:414, 557`).
- **bare_lookup_ratchet**: raises 105→107 (effects/mod.rs) and 72→74 (abilities.rs) are justified
  NONSWALLOW predicate reads matching existing residue shape (see Finding 5 re: the wording of the
  CR 506.4c justification).

## Test verdict

All seven tests are real and were proven non-vacuous by the runner (revert-and-restore). The decoys
(4-player C/D-unaffected, land-not-tapped, planeswalker-loyalty-vs-life, per-attacker power filter,
capture-survives-attacker-removal) genuinely discriminate. The runner is honest that the Ojutai A2
decoy does not independently discriminate A2 (single runtime ability), and A2/reclassification is
instead pinned by the pre-existing `pbd_damaged_player_filter::test_throat_slitter_end_to_end_precision_fix`
— acceptable. **Coverage gaps:** no test for Finding 1 (planeswalker removed before resolution) or
Finding 2 (non-targeted attack trigger + defending player leaves) — both would currently fail.

## Golden script (192_mutate_gemrazer.json) verdict

Correct MTG. Gemrazer oracle ("Whenever this creature mutates, destroy target artifact or enchantment
an opponent controls") verified via MCP; the rewrite gives the trigger a legal target (P2's Arcane
Signet) with P1's own Signet as a controller-filter decoy, and asserts the correct one is destroyed.
This exercises the A1 (targets forwarded through enrich for `SelfMutates`) + A2 (auto-select) fix in
the script harness. Both dispute resolutions are honest — they document the PB-EF3 mechanism and are
resolved by `scutemob-103`, not merely flipped green.

## Recommendation

Ship-blocking? No HIGH findings; the three cards are correct on the common path and all gates are
green. But Findings 1 and 2 are genuine CR-correctness bugs that ship with the very cards this PB
adds (Hellrider/Raid Bombardment) and affect a pre-existing cohort (Utvara/Dromoka). Recommend fixing
both before collect, or filing them as tracked OOS/EF findings with the two missing tests added.
