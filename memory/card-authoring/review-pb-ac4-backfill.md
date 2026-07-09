# PB-AC4 Backfill Card Review

**Reviewed**: 2026-07-08
**Reviewer**: card-batch-reviewer (Opus)
**Scope**: `ModeSelection.mode_targets` migration (CR 700.2c/601.2c/700.2f) + UpToN cleanup +
deliberately-not-migrated cards + `pb_ac4_card_integration.rs`.
**Oracle source**: `lookup_card` MCP (authoritative).

**Findings**: 2 HIGH, 0 MEDIUM, 5 LOW.

---

## Migration-contract audit (all 11 migrated cards)

Verified per card: `mode_targets.len() == modes.len()`; `Spell.targets == vec![]`; targetless modes
carry an empty inner `vec![]`; every `DeclaredTarget(n)` / `PlayerTarget::DeclaredTarget(n)` index is
LOCAL (0-based within its mode) and in-bounds for that mode's `mode_targets[m]`; no nested `UpToN`
inside `mode_targets`; mode ORDER matches oracle order.

| Card | modes==targets | Spell.targets empty | local-index OK | mode order OK |
|------|:---:|:---:|:---:|:---:|
| casualties_of_war | 5==5 | yes | yes | yes |
| cryptic_command | 4==4 | yes | yes | yes |
| izzet_charm | 3==3 | yes | yes | yes |
| abzan_charm | 3==3 | yes | yes | yes |
| boros_charm | 3==3 | yes | yes | yes |
| golgari_charm | 3==3 | yes | yes | yes |
| rakdos_charm | 3==3 | yes | yes | yes |
| evolution_charm | 3==3 | yes | yes | yes |
| archdruids_charm | 3==3 | yes | yes (2-target mode) | yes |
| archmages_charm | 3==3 | yes | yes | yes |
| incendiary_command | 4==4 | yes | yes | yes |

**No local-index bugs found.** This was the flagged highest-risk class; every `DeclaredTarget`
index re-baselined to 0 (or 0/1 for `archdruids_charm` mode 1) matches its mode's slice length.
`archdruids_charm` mode 1 (`mode_targets[1]` has two entries: [creature you control, creature you
don't control]) correctly uses `DeclaredTarget{0}` (AddCounter + Bite source) and `DeclaredTarget{1}`
(Bite target) — no cross-mode contamination.

Multiplayer-sensitive routings verified correct: `cryptic_command` mode 1 uses
`PlayerTarget::OwnerOf(...)` for "owner's hand" (CR 108.3, not Controller); `archmages_charm` mode 1
"target player draws" uses `PlayerTarget::DeclaredTarget{0}` (not Controller).

---

## HIGH

### H1 — golgari_charm.rs:44-48 — stale ENGINE-BLOCKED marker; mode 2 is a silent no-op but IS expressible (KI-3) — **FIXED**

**Status: FIXED.** Mode 2 now uses `Effect::Regenerate { target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter { has_card_type: Some(CardType::Creature), controller: TargetController::You, ..Default::default() })) }`,
exactly as this finding prescribed. Stale marker removed. Regression test:
`test_golgari_charm_mode_2_regenerates_only_callers_creatures` in
`crates/engine/tests/pb_ac4_card_integration.rs` — casts mode 2, then applies a
`DestroyPermanent { AllPermanentsMatching(Creature) }` board wipe and asserts the
caster's creature survives (shield intercepts) while the opponent's unshielded
creature dies.

Oracle mode 2: "Regenerate each creature you control."

The card leaves mode 2 as `Effect::Sequence(vec![])` (a no-op) with the marker:
> "ENGINE-BLOCKED: no bulk-regenerate Effect variant exists (a single `Effect::Regenerate` only
> targets one object)."

**This marker is false.** `Effect::Regenerate` resolves a target *list* and applies a regeneration
shield to every resolved object — `crates/engine/src/effects/mod.rs:3184` does
`let targets = resolve_effect_target_list(state, target, ctx);` then loops `for resolved in &targets`.
`EffectTarget::AllPermanentsMatching(filter)` with `controller: TargetController::You` resolves to all
creatures you control (`effects/mod.rs:5965-5980`, `TargetController::You => obj.controller ==
ctx.controller`). So the mode is fully expressible today as:

```rust
Effect::Regenerate {
    target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
        has_card_type: Some(CardType::Creature),
        controller: TargetController::You,
        ..Default::default()
    })),
}
```

**Wrong-game-state scenario**: opponent casts a `Destroy all creatures` board wipe; the Golgari
player responds with Golgari Charm choosing "Regenerate each creature you control." Per oracle their
creatures survive. In the engine the mode does nothing and every creature is destroyed. Silent,
game-losing divergence, and the fix uses only existing primitives. HIGH.

(The file-level header comment lines 44-47 carries the same false claim.)

### H2 — abzan_charm.rs:68-71 — mode 2 target filter adds a `controller: You` restriction absent from oracle (KI-1) — **FIXED**

**Status: FIXED.** `mode_targets[2]` now uses `TargetRequirement::TargetCreature`
(no controller filter) — any creature is a legal target, matching oracle. Header
and inline comments corrected to no longer claim "you control." The distribute
residual (L1 below) remains open and is unaffected by this fix. Regression test:
`test_abzan_charm_mode_2_can_target_opponents_creature` in
`crates/engine/tests/pb_ac4_card_integration.rs` — casts mode 2 targeting an
opponent-controlled creature and asserts the 2 +1/+1 counters legally resolve.

Oracle mode 2: "Distribute two +1/+1 counters among **one or two target creatures**." (No controller
restriction — any creature is a legal target.)

`mode_targets[2]` is authored as:
```rust
TargetRequirement::TargetCreatureWithFilter(TargetFilter {
    controller: TargetController::You,
    ..Default::default()
})
```
This makes only creatures **you control** legal targets. The header (line 2, 6) and the inline
comment (lines 50-55) also misstate the oracle as "…target creatures you control," so the deviation
is baked in as if it were correct rather than flagged.

**Wrong-game-state scenarios**:
1. You control no creatures but an opponent controls one. Per oracle you may still choose this mode
   and target the opponent's creature (a legal, if unusual, play — e.g., to grow a Monarch defender
   or feed a later effect). The engine reports the mode has no legal target, incorrectly narrowing
   your legal choices / potentially the whole spell's castability in that mode.
2. Targeting an opponent's creature to place counters (politically or to enable an "enters/attacks
   with counters" synergy you benefit from) is oracle-legal and is silently rejected.

Wrong target-legality is HIGH per the project's own KI-1 scheme. Note this restriction is separate
from — and unmarked by — the (honest) distribute-primitive ENGINE-BLOCKED marker on the same mode.

---

## MEDIUM

**Zero MEDIUM findings.**

---

## LOW

### L1 — abzan_charm.rs:56-60 — distribute approximation (documented, legal subset)

Mode 2 applies both +1/+1 counters to a single declared target (`count: 2`) instead of "distribute…
among one or two." The ENGINE-BLOCKED marker is honest: no distribute-N-among-M primitive exists
(confirmed — no `DistributeCounters` variant anywhere). Putting both counters on one creature is a
legal outcome of the oracle text, so this loses the *split-across-two* option only (reduced choice,
not an illegal result). LOW. (The controller-restriction half of this mode is H2 above.)

### L2 — skullsnatcher.rs:39-60 — triggered UpToN auto-selects 0; "that player's graveyard" approximated

The UpToN cleanup itself is structurally correct for the DSL (`UpToN{count:2, inner:
TargetCardInGraveyard}` replaces the prior two-mandatory-targets approximation). Two documented
residuals remain, both honestly marked: (a) `abilities.rs` routes a non-player-inner `UpToN` on a
*triggered* ability to `None`, so the ability currently exiles nothing until player-declared
triggered targeting exists; (b) "that player's graveyard" is approximated as
`TargetController::Opponent`, which in multiplayer would permit exiling a *non-damaged* opponent's
graveyard. Both are pre-existing, out-of-AC4-scope, and currently moot (auto-0). LOW.

### L3 — honestly-marked no-op modes (verified genuine gaps)

The following modes are left as `Effect::Nothing` / empty `Sequence` with ENGINE-BLOCKED markers that
I verified name **real** residual gaps (not AC4-scope): 
- `boros_charm.rs:31-37` mode 1 "permanents you control gain indestructible" — `EffectFilter` has no
  source-controller-relative all-permanents variant (`AllPermanents` is unscoped; `ControlledBy`
  needs a concrete `PlayerId`; only type-scoped `CreaturesYouControl`/`ArtifactsYouControl`/… exist).
  Honest.
- `rakdos_charm.rs:28-32` mode 0 "exile target player's graveyard" — no effect exiles a whole
  graveyard zone (`AllPermanentsMatching` is battlefield-only; `EachCardInAllGraveyards` hits all
  graveyards, not a target player's). Honest.
- `rakdos_charm.rs:38-43` mode 2 "each creature deals 1 damage to its controller" — `EffectTarget`
  has no `ControllerOf`/per-iteration variant (`PlayerTarget::ControllerOf` exists but `DealDamage`
  takes an `EffectTarget`, and a `ForEach` iteration binds the creature only as `DeclaredTarget{0}`,
  with no path to that creature's controller as the damage recipient). Honest.
- `incendiary_command.rs:55-57` mode 3 wheel "each player discards their hand, then draws that many"
  — needs a per-player snapshot-before-discard count inside a `ForEach`; `EffectAmount::HandSize`
  reads live (would read 0 after discard) and there is no per-player discard-count capture. Honest.

These produce wrong game state *only if the blocked mode is chosen* and cannot be `vec![]`'d
individually (they are single modes of otherwise-working modal spells), which matches the project's
accepted pattern. Flagged LOW/informational for tracking, not as correctness defects of the AC4 work.

### L4 — blessed_alliance & collective_resistance — non-migration decision is CORRECT, but pre-AC4 flat-union bug persists

Escalate + `mode_targets` is hard-rejected at cast: `crates/engine/src/rules/casting.rs:3526-3531`
(`if mode_targets_active.is_some() && escalate_modes > 0 { return Err(... "Escalate combined with
ModeSelection.mode_targets is not supported") }`). Migrating either card would make any multi-mode
Escalate cast fail. **The decision not to migrate is sound, not laziness — verified against the
engine.** Both retain their flat mandatory-target lists, which means the pre-AC4 "choosing one mode
still forces declaring targets for the other modes" wrong-state persists (e.g. Blessed Alliance mode-0
"gain 4 life" still demands two creature targets + a second player). This is genuinely blocked on the
unimplemented Escalate+per-mode-targets combination and is documented in both files. LOW/informational.

### L5 — pb_ac4_card_integration.rs:402-453 — archmage gain-control test does not probe the MV filter boundary

The four tests are valid and each proves its stated claim:
- `test_casualties_of_war_castable_choosing_creature_subset` casts choosing ONLY mode 1 in a board
  with no artifact/enchantment/land/planeswalker anywhere and asserts the creature is destroyed —
  this is exactly the pre-AC4 flat-union "must declare all five targets" uncastable bug; it is a
  valid forward regression guard proving the subset-castable fix (it cannot literally execute the old
  code path, which is expected).
- `test_izzet_charm_damage_mode_needs_no_spell_target` proves an unchosen counter mode's spell-target
  requirement is not enforced.
- `test_cryptic_command_counter_and_bounce_sliced_independently` proves CR 700.2f independent slicing
  (countered spell → owner's graveyard; bounced permanent → owner's hand).
- `test_archmages_charm_gain_control_of_low_mv_permanent` proves `Effect::GainControl` +
  `non_land`/`max_cmc` filter work end-to-end (control of the MV-1 artifact transfers to p1).

Gap: the archmage test only asserts an MV-1 permanent is a legal, working target; it does not assert
that an MV-2 permanent is *rejected*. The `max_cmc: Some(1)` boundary is therefore unproven by the
suite. Not a defect — incomplete coverage only. LOW.

---

## Verification notes (claims checked against engine source)

- **archmages_charm mode 2 (the flagged "author claims GainControl exists" case)**: CONFIRMED not an
  approximation. `Effect::GainControl { target, duration }` exists (`card_definition.rs:2059-2062`);
  `EffectDuration::Indefinite` exists (`continuous_effect.rs:52`); `TargetFilter.max_cmc`
  (`card_definition.rs:2671`) and `.non_land` (`:2647`) both exist; the integration test
  (L5) proves it end-to-end. Oracle "no stated duration" → `Indefinite` is correct (CR 613.1b). No
  finding.
- **has_card_types = OR semantics** (archdruids mode 2 "artifact or enchantment"): CONFIRMED — used by
  dozens of destroy-artifact-or-enchantment cards with explicit "OR semantics" comments; matches
  oracle. No finding.
- **`archdruids_charm` mode 1 "creature you don't control"** → `TargetController::Opponent`: correct
  in a Commander FFA (a creature you don't control is controlled by an opponent). No finding.
- **cryptic_command mode 2** `TapPermanent { AllPermanentsMatching(Creature, Opponent) }` taps ALL
  matches (`effects/mod.rs:1863-1875` iterates the resolved list). Correct. No finding.
- **bridgeworks_battle** UpToN cleanup: it is a *spell* (not a triggered ability), so the
  skullsnatcher auto-0 residual does not apply; optional Fight target handled by
  `Effect::Fight`/CR 701.14b. `Spell.targets` (flat, non-modal) correctly retains index 0
  (mandatory, creature you control) + index 1 (`UpToN{1}`, creature you don't control). No finding.
- **Mode min/max counts**: casualties 1..5 ("one or more"), cryptic/incendiary 2..2 ("choose two"),
  all charms 1..1 ("choose one"); `allow_duplicate_modes: false` throughout. All match oracle.


---

## Fix-Phase Resolution (2026-07-08)

| Finding | Severity | Status | Resolution |
|---------|----------|--------|------------|
| H1 golgari_charm mode 2 stubbed no-op behind false marker | HIGH | **FIXED** | Implemented `Effect::Regenerate` over the controller's creatures; stale ENGINE-BLOCKED marker deleted. Regression test added. |
| H2 abzan_charm mode 2 wrong `controller: You` target filter | HIGH | **FIXED** | Removed the controller restriction (oracle: "one or two target creatures", any controller); corrected misquoted oracle in header + inline comments. Regression test added. The *distribute* variable-count residual remains, honestly marked. |
| L1 abzan distribute approximation | LOW | note-only | Honestly marked; real residual primitive. |
| L2 skullsnatcher triggered-UpToN auto-0 | LOW | note-only | Honestly marked; real residual (trigger-time player-declared targeting). |
| L3 honestly-marked no-op modes (boros/rakdos/incendiary) | LOW | note-only | Verified as genuine engine gaps. |
| L4 blessed_alliance / collective_resistance retain pre-AC4 flat-union behavior | LOW | note-only | Correct: blocked on the Escalate + `mode_targets` hard-reject. |
| L5 archmage test lacks MV<=1 boundary probe | LOW | note-only | Coverage gap, not a defect. |

Gates after fixes: build --workspace clean, `cargo test --all` 2957 passed / 0 failed,
`clippy --all-targets -- -D warnings` clean, `fmt --check` clean.
