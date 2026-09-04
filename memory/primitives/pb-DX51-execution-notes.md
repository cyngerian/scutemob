# PB-DX51 — test-suite execution notes

Task `scutemob-226`; engine changes shipped at `ee6b1e18` (see that commit's message and
`memory/primitives/pb-plan-DX51.md`). This note covers the TEST-WRITING task only: three new
files, all probes watched RED by an executed revert, and the corrections this task made to the
dispatching prompt's own claims.

## §1 — Pre-edit baseline

`~/.cargo/bin/cargo test --workspace --no-fail-fast`, captured to a file, taken on this branch
BEFORE any test file was added: **5,044 / 0 / 5**, **60** result-producing targets. Matches the
dispatching prompt's stated baseline exactly.

## §2 — Files added

* `crates/engine/tests/primitives/pb_dx51_cr_508_8_skip.rs` — t1, t2, t3, t4, t5, t6, x1 (7 tests)
* `crates/engine/tests/core/pb_dx51_attacker_entry_roster.rs` — `r1`, `r1b`, `t_census_report` (3 tests)
* `crates/simulator/tests/pb_dx51_blocker_offer.rs` — `b1` (1 test)
* `mod` lines added to `crates/engine/tests/primitives/main.rs` and `crates/engine/tests/core/main.rs`

Post-edit: `~/.cargo/bin/cargo test --workspace --no-fail-fast` → **5,055 / 0 / 5**, **61** targets
(60 → 61: the new `pb_dx51_blocker_offer` simulator binary). `+11` tests, 0 removals, 0 renames.
`cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --check` clean.

## §3 — Revert matrix (all rows executed against the FINAL committed production tree, then restored)

Restoration method for every row: the pre-edit file was copied to the scratchpad before mutation,
then `cp` restored byte-for-byte, verified with `diff` against the saved original (`diff` printed
nothing / "restored clean" for every row).

| row | what was reverted | tests run | RED | GREEN (stated controls) | verdict |
|---|---|---|---|---|---|
| **R1** | `turn_structure.rs`'s skip predicate reverted to `c.attackers.is_empty()` (dropped `!c.had_attackers &&`) | `primitives pb_dx51` (7) | **t1**, **t4** | t2, t3 (control), t5, t6, x1 | Matches the prompt's prediction for t1/t4/t3 but **REFUTES it for t2** — t2 stays GREEN under R1 (see §4). |
| **R2** | `CombatState::add_attacker` no longer sets `had_attackers` | `primitives pb_dx51` (7) | **t1**, **t4**, **t5**, **t6** (its companion-flip half) | t2, t3 (control), x1 | Matches the prompt's prediction. |
| **R3** | One CR 508.4 entrant site (`effects/mod.rs`'s `CreateToken` `enters_attacking` arm) reverted to a raw `combat.attackers.insert(id, target.clone())`, bypassing `add_attacker` | `core pb_dx51` (3) + `primitives pb_dx51` (7) | **r1**, **r1b** | all 7 behavioural probes in `pb_dx51_cr_508_8_skip.rs` | **No behavioural probe reddens** — none of t1-t6/x1 drives `Effect::CreateToken`'s `enters_attacking` path, so this defect would have been invisible to every behavioural probe in this batch. This is exactly what `r1`/`r1b` exist for, and the prompt's own framing ("note whether any behavioural probe reddens — if none does, say so") is confirmed true for this site. |
| **R4** | `legal_actions.rs`'s `DeclareBlockers` offer conjunct dropped back to `!combat.attackers.is_empty()` alone | `pb_dx51_blocker_offer` (1) | **b1** | — | Matches the prompt's prediction. |
| **R5** | `combat.rs`: `CombatState::new` init moved back above the per-attacker validation loop (two-site edit: the guard-adjacent init restored, the later infallible-tail init removed) | `primitives pb_dx51` (7) | **x1** | t1, t2, t3, t4, t5, t6 | Matches the prompt's prediction. Panic: `OOS-DX21-5: a REFUSED declaration must leave state.combat exactly as it found it -- CR 732, "the game returns to the moment before the declaration"`. |
| **R5b** | On top of R5: a throwaway scratch test (`r5b_scratch_process_command_version_is_vacuous_under_r5`, added to the test file, run, then fully removed and the file restored to its pre-R5b byte-identical state before committing) rewrote x1's precondition half to go through `process_command` (clone-before-call idiom) instead of the direct handler | `primitives pb_dx51 r5b_scratch` (1, scratch only) | — | **the scratch test itself, GREEN under R5** | Demonstrates `OOS-DX21-7` by execution: the `process_command`-shaped assertion is structurally vacuous under R5 (stays green even though the fix is reverted), which is exactly why x1 is required to call `rules::combat::handle_declare_attackers` directly. |

**0 rows honestly UNDISCRIMINATED.** Every row reddened at least one test in this batch's own
suite (R3 reddens the roster gate rather than a behavioural probe, which is disclosed above and
in the roster file's own module doc rather than treated as a gap).

### §3.1 — Exact panic text (R1, R2, R5, as required, plus R3/R4 for completeness)

**R1** (`t1`):
```
assertion `left == right` failed: CR 508.8: a creature WAS declared this combat -- declare-blockers
must not be skipped even though combat.attackers is now empty (the pre-PB-DX51 defect read
`attackers.is_empty()` at step end and would have jumped straight to EndOfCombat here)
  left: EndOfCombat
 right: DeclareBlockers
```
**R1** (`t4`):
```
assertion `left == right` failed: CR 508.4/508.8: a creature was put onto the battlefield attacking
with NO CR 508.1 declaration at all -- even after THAT entrant is itself removed from combat, the
skip must not fire
  left: EndOfCombat
 right: DeclareBlockers
```

**R2** (`t1`): `precondition: had_attackers must be set by the declaration`
**R2** (`t4`): `precondition: add_attacker must mark had_attackers even with no declaration`
**R2** (`t5`): `precondition: had_attackers is set by the declaration`
**R2** (`t6`): `the SAME field must flip true once a real declaration happens in THIS combat`

**R3** (`r1`):
```
PB-DX51 r1: found the raw mutation `.attackers.insert(` outside CombatState::add_attacker's own
implementation in: [("crates/engine/src/effects/mod.rs", 1)]. Every CR 508.1/508.4 entry site
must route through CombatState::add_attacker so `had_attackers` (CR 508.8) cannot be silently
forgotten at a new site -- see plan §1.2.
```
**R3** (`r1b`):
```
assertion `left == right` failed: PB-DX51 r1b: expected exactly 5 production call sites of
CombatState::add_attacker (...), found 4: [effects/mod.rs:7003, combat.rs:791,
resolution.rs:6175, resolution.rs:6654]
  left: 4
 right: 5
```

**R4** (`b1`):
```
assertion `left == right` failed: an action the engine will refuse (GameStateError::
AlreadyDeclaredBlockers) must not be offered (found 1 in [PassPriority, DeclareBlockers {
eligible: [ObjectId(2)], attackers: [ObjectId(1)] }])
  left: 1
 right: 0
```

**R5** (`x1`):
```
OOS-DX21-5: a REFUSED declaration must leave state.combat exactly as it found it -- CR 732,
"the game returns to the moment before the declaration"
```

## §4 — Corrections to the dispatching prompt's own claims (the most valuable part of this report)

1. **t2 does NOT redden under R1 (or R2), contrary to the prompt's stated prediction, and the
   reason is structural rather than a fixture mistake.** t2's scenario ("declare TWO attackers,
   remove only ONE") leaves the surviving attacker (B) in `combat.attackers`, so
   `attackers.is_empty()` is `false` at the moment `advance_step` decides the skip — under BOTH
   the shipped predicate (`!c.had_attackers && c.attackers.is_empty()`) and the R1-reverted one
   (`c.attackers.is_empty()` alone). The AND-with-`is_empty()` structure means `had_attackers`
   (and therefore R2's revert of it) is **only** load-bearing when the WHOLE map becomes empty —
   never in a "some survive" scenario. This was verified empirically before being accepted: t2
   stayed GREEN under both R1 and R2 on the first run, not assumed.

   t2's *actual* value is real and disclosed in the test's own doc: it proves the DOWNSTREAM
   consequence (a real `Command::DeclareBlockers` block registers, and `combat_damage_step`
   actually marks damage) after a partial mid-step CR 506.4 removal — a claim t1's fixture
   structurally cannot make, because t1's only attacker IS the one removed, leaving nothing left
   to block or damage. "The step occurred" and "the blocks and damage happened" are different
   claims, and t2 is the one that can make the second claim; it just isn't the row that
   discriminates R1/R2, and the prompt's revert-matrix table was wrong to predict that it would.

2. **t4, as the prompt literally specified it ("a CR 508.4 entrant is added ... assert the steps
   are NOT skipped", nothing about removing it), does NOT discriminate R1 either, for the exact
   same structural reason as (1).** An entrant that is added and never removed leaves
   `combat.attackers` non-empty at step end, which the PRE-PB-DX51 predicate already handled
   correctly on its own. Verified empirically: an early draft of t4 (add the entrant via
   `CombatState::add_attacker`, never remove it, assert not-skipped) passed cleanly under R1.
   **This task did not ship that draft** — t4 as committed adds the CR 508.4 entrant with NO
   declaration and then removes IT TOO (via Reconnaissance, the same real production route as
   t1/t2/t5), which is the only way to build a fixture where `attackers_declared` stays `false`
   for the WHOLE combat, `attackers.is_empty()` becomes `true` at step end, and only
   `had_attackers` is left standing between the skip firing and not firing. That version reddens
   under both R1 and R2, confirmed by execution.

3. **`ObjectStatus` has no `damage_marked` field** — the prompt's/plan's implicit assumption that
   combat damage would be readable at `obj.status.damage_marked` is wrong; it is
   `GameObject::damage_marked` directly (`crates/card-types/src/state/game_object.rs:1028`), a
   sibling of `status`, not a member of it. One-line fix, caught by `cargo check` immediately.

4. **A pre-existing, PB-DX51-unrelated offer-layer gap was found while building `b1`, and is
   reported rather than silently worked around**: `legal_actions.rs`'s `DeclareBlockers` offer
   does not itself check `player == combat.attacking_player` (only `combat::handle_declare_blockers`
   does, at the engine level). An UNTAPPED creature controlled by the ATTACKING player is counted
   as an "eligible" blocker by the offer layer's loop, so if the attacking player has any
   untapped creature that is not itself the (tapped, non-Vigilance) attacker, `StubProvider` would
   offer them `LegalAction::DeclareBlockers`, which the engine would then refuse. This was caught
   because `b1`'s first draft built its attacker UNTAPPED (a synthetic `CombatState`, not routed
   through the real declare-attackers tap step) and observed p1 being offered `DeclareBlockers`
   for its own untapped attacker. Worked around in the fixture (the attacker is now built
   `.tapped()`, matching CR 508.1f's real post-declaration state for a non-Vigilance attacker) —
   **not fixed in production**, since it is outside this task's scope (tests only) and outside
   PB-DX51's diff (the `player == combat.attacking_player` check was already absent before
   `ee6b1e18`; PB-DX51 only added the `defenders_declared` conjunct beside it). Recommend filing
   as a new seed: **the `DeclareBlockers` offer can suggest an action to the ATTACKING player that
   the engine will always refuse (SR-38)**, reachable whenever the attacker has any other
   untapped creature.

5. **The coordinator's mid-task correction (Reconnaissance as the CR 506.4 reproduction route,
   the three-site emptying census, and `remove_from_combat` being `pub(crate)`) was already the
   design this task had independently arrived at before the correction message landed** — verified
   by rereading this file's own module doc, which states the same three-site census
   (`effects/mod.rs:3427`, `replacement.rs:3643`, and the raw `abilities.rs:2361` Ninjutsu-bounce
   site) and the same Reconnaissance-based route for t1/t2/t5. No design change was needed in
   response to the correction; its content is folded into the module doc's "Correction recorded,
   not silently worked around" paragraph, citing the coordinator's finding rather than duplicating
   the derivation.

## §5 — What this task did NOT do (explicitly out of scope)

* No production source under `crates/*/src` or `tools/*/src` was modified in the final tree (all
  six revert rows were applied and then restored byte-identical, verified by `diff`).
* The offer-layer gap in §4.4 was not fixed — reported only.
* The four-of-six-cause CR 506.4 cleanup gap the coordinator asked about (§0 of the module doc)
  was not fixed — reported only, per the coordinator's own instruction not to work around it
  silently.
