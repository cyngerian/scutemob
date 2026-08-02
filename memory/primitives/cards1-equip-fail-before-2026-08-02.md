# CARDS-1 (OOS-M11-10) — fail-before evidence, 2026-08-02

Captured on branch `feat/cards-1-equip-target-repair-batch---close-oos-m11-10-defs`
(worktree `scutemob-179`), against the PRE-FIX card corpus (no edits under
`crates/card-defs/`, `crates/engine/src/`, or `crates/card-types/src/` — this file is
proof the two new test files, on their own, discriminate the defect described in
OOS-M11-10 before any fix lands).

## Commands run

```
~/.cargo/bin/cargo test --test primitives cards1_equip_target_repair -- --nocapture
~/.cargo/bin/cargo test --test core cards1_equip_target_roster -- --nocapture
~/.cargo/bin/cargo test --test mechanics_e_l equip -- --nocapture
~/.cargo/bin/cargo test --test mechanics_m_z reconfigure -- --nocapture
~/.cargo/bin/cargo test --test mechanics_e_l fortify -- --nocapture
```

## `primitives cards1_equip_target_repair` — pre-fix result: 5 passed, 3 failed

```
test cards1_equip_target_repair::t1_zero_target_ability_accepted_paid_and_silently_fizzles ... ok
t7b measured fortify_roster = {"Darksteel Garrison"}
t7b measured reconfigure_roster = {"Lizard Blades"}
test cards1_equip_target_repair::t7a_cryptic_coat_triggered_attach_untouched ... ok
test cards1_equip_target_repair::t7b_fortify_and_reconfigure_rosters_pinned_and_unperturbed ... ok
test cards1_equip_target_repair::t2_skullclamp_zero_targets_rejected_post_fix ... FAILED
test cards1_equip_target_repair::t3_skullclamp_legal_target_attaches_and_applies_bonus ... ok
test cards1_equip_target_repair::t4_skullclamp_opponent_creature_rejected_post_fix ... ok
test cards1_equip_target_repair::t5_engine_query_reports_slot_and_candidates_scoped_to_controller ... FAILED
test cards1_equip_target_repair::t6_non_vacuity_floors ... FAILED

test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 1009 filtered out
```

### Pass/fail table vs. the plan's prediction

| Test | Plan predicted (pre-fix) | Actual (pre-fix) | Notes |
|---|---|---|---|
| T1 | PASS | PASS | permanent defect-shape record, as designed |
| T2 | FAIL | **FAIL** | matches prediction |
| T3 | FAIL | **PASS** (unpredicted) | see deviation note below |
| T4 | FAIL | **PASS** (unpredicted) | see deviation note below |
| T5 | FAIL | **FAIL** | matches prediction |
| T6 | FAIL | **FAIL** | matches prediction |
| T7a | (not scoped by brief's pre/post table) | PASS | no defect scope here |
| T7b | (not scoped by brief's pre/post table) | PASS | pins measured, see below |

**Deviation from the brief's prediction (T3/T4 pass pre-fix, not fail)**: the brief's
"T2/T3/T4/T5/T6 and R2 FAIL" pre-fix expectation undercounted the legacy special-case
at `rules/abilities.rs:539-582`. That code validates a *volunteered* target (creature,
on battlefield, controller == activating player) whenever the command actually
declares one, even though the ability's own `target_requirements` list is empty (so a
missing-target activation slips past it, which is exactly OOS-M11-10). T3 and T4 both
declare a target in the `Command::ActivateAbility.targets` field, so the legacy check
already validates legality correctly for those two cases pre-fix — T3's target is a
legal creature-you-control (attach succeeds), T4's target is an opponent's creature
(rejected by the legacy check's `on_battlefield_and_controlled` test, independent of
the `TargetRequirement` list). This is real, accurate information about the existing
code, not a weakened test: T2 (zero declared targets) and T5/T6 (the query surface,
which reads `target_requirements` directly and sees nothing) still fail exactly as
predicted, because they exercise the actual gap (no `TargetRequirement` declared) that
the legacy fallback cannot cover.

### T2 exact failure (verbatim, trimmed of the full `GameState`/`CardRegistry` Debug
dump the first draft's panic message included — the assertion below replaces that with
a compact message; the underlying `Result` observed was `Ok(...)`, confirming the
zero-target activation is currently ACCEPTED)

```
thread 'cards1_equip_target_repair::t2_skullclamp_zero_targets_rejected_post_fix' panicked at crates/engine/tests/primitives/cards1_equip_target_repair.rs:346:18:
expected Err(GameStateError::InvalidTarget(_)) once Skullclamp declares its TargetRequirement (CR 601.2c: 0 declared targets against a 1-target-mandatory requirement is illegal); got Ok(_) instead -- pre-fix, Skullclamp's equip ability still declares targets: vec![] so a zero-target activation is silently ACCEPTED (this IS the bug this batch closes)
```

The raw pre-edit panic (before the message was tightened) showed the full `Ok((GameState
{ .. }, [ManaCostPaid { player: PlayerId(1), cost: ManaCost { generic: 1, .. } },
AbilityActivated { player: PlayerId(1), source_object_id: ObjectId(1), stack_object_id:
ObjectId(8) }, PriorityGiven { player: PlayerId(1) }]))` — i.e., the activation
succeeded, paid 1 generic mana, and went on the stack, exactly the "accepted despite
zero targets" defect.

### T5 exact failure (verbatim)

```
thread 'cards1_equip_target_repair::t5_engine_query_reports_slot_and_candidates_scoped_to_controller' panicked at crates/engine/tests/primitives/cards1_equip_target_repair.rs:466:5:
assertion `left == right` failed: Skullclamp's equip ability must report exactly one TargetRequirement post-fix
  left: 0
 right: 1
```

`rules::queries::ability_target_requirements` returns Skullclamp's real (pre-fix)
`targets: vec![]` — confirming the browser-path half of OOS-M11-10: the query the play
server calls to populate `ActionOptionView.target_slots` reports zero slots, so the
picker never asks for a target at all.

### T6 exact failure (verbatim)

```
thread 'cards1_equip_target_repair::t6_non_vacuity_floors' panicked at crates/engine/tests/primitives/cards1_equip_target_repair.rs:514:5:
T5's requirement list must be non-empty
```

Downstream of T5's empty requirement list — expected, since T6 asserts the same query
result is non-empty.

### T7b measured rosters (println output, `--nocapture`)

```
t7b measured fortify_roster = {"Darksteel Garrison"}
t7b measured reconfigure_roster = {"Lizard Blades"}
```

Both match the pinned expected sets exactly (`{"Darksteel Garrison"}` and
`{"Lizard Blades"}`), so `t7b_fortify_and_reconfigure_rosters_pinned_and_unperturbed`
passes pre-fix — these two neighbouring mechanisms are untouched by this batch and the
pins record that Darksteel Garrison (Fortify) and Lizard Blades (Reconfigure) both
still carry the identical `targets: vec![]` defect shape, out of scope for this batch.

## `core cards1_equip_target_roster` — pre-fix result: 2 passed, 1 failed

```
test cards1_equip_target_roster::r1_equip_activated_attach_equipment_roster_is_pinned ... ok
test cards1_equip_target_roster::r3_walk_is_not_vacuous ... ok
test cards1_equip_target_roster::r2_every_roster_member_has_exactly_the_expected_target_requirement ... FAILED

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 458 filtered out
```

R1 confirms the roster is exactly the 17 expected names (16 hand-authored equip defs +
Helm of the Host). R3 confirms the walk itself is not vacuous. R2 — the actual fix
gate — fails for all 17 of 17 roster members pre-fix, verbatim:

```
thread 'cards1_equip_target_roster::r2_every_roster_member_has_exactly_the_expected_target_requirement' panicked at crates/engine/tests/core/cards1_equip_target_roster.rs:193:5:
R2 (CARDS-1 / OOS-M11-10 fix gate) failed for 17 of 17 roster members:
Accorder's Shield: expected exactly 1 TargetRequirement, found 0 ([])
Argentum Armor: expected exactly 1 TargetRequirement, found 0 ([])
Basilisk Collar: expected exactly 1 TargetRequirement, found 0 ([])
Batterskull: expected exactly 1 TargetRequirement, found 0 ([])
Cathar's Shield: expected exactly 1 TargetRequirement, found 0 ([])
Diamond Pick-Axe: expected exactly 1 TargetRequirement, found 0 ([])
Hammer of Nazahn: expected exactly 1 TargetRequirement, found 0 ([])
Helm of the Host: expected TargetRequirement::TargetCreatureWithFilter, found TargetCreature (this is exactly Helm of the Host's pre-fix under-restrictive shape if it is a bare TargetCreature, or the original empty-vec defect if this list should not be reached with zero targets)
Lightning Greaves: expected exactly 1 TargetRequirement, found 0 ([])
Shadowspear: expected exactly 1 TargetRequirement, found 0 ([])
Skullclamp: expected exactly 1 TargetRequirement, found 0 ([])
Spidersilk Net: expected exactly 1 TargetRequirement, found 0 ([])
Swiftfoot Boots: expected exactly 1 TargetRequirement, found 0 ([])
Sword of Fire and Ice: expected exactly 1 TargetRequirement, found 0 ([])
Sword of Vengeance: expected exactly 1 TargetRequirement, found 0 ([])
Thornbite Staff: expected exactly 1 TargetRequirement, found 0 ([])
Whispersilk Cloak: expected exactly 1 TargetRequirement, found 0 ([])
```

16 of 17 fail with "found 0 ([])" (the original empty-`targets`-list defect). Helm of
the Host fails differently: it already declares one requirement, but it's the
under-restrictive `TargetRequirement::TargetCreature` (no "you control" scoping),
confirming the plan's note that Helm of the Host needed a different repair (tightening
an existing requirement, not adding a missing one) from the other 16 (adding a missing
requirement).

## Baseline: neighbouring mechanism test suites unaffected pre-fix

All pass, unchanged, confirming this batch's two new test files do not perturb Equip,
Reconfigure, or Fortify's existing coverage:

```
$ cargo test --test mechanics_e_l equip
running 19 tests
... (equip::* — 15 tests, living_weapon::* — 4 tests)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 479 filtered out

$ cargo test --test mechanics_m_z reconfigure
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 631 filtered out

$ cargo test --test mechanics_e_l fortify
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 491 filtered out
```

## Scope confirmation

`git diff --stat` at capture time (this task's only changes):

```
 crates/engine/tests/core/main.rs       | 1 +
 crates/engine/tests/primitives/main.rs | 1 +
 2 files changed, 2 insertions(+)
```

plus two new untracked files:
`crates/engine/tests/core/cards1_equip_target_roster.rs`,
`crates/engine/tests/primitives/cards1_equip_target_repair.rs`.

Zero lines touched under `crates/card-defs/`, `crates/engine/src/`,
`crates/card-types/src/`, `crates/simulator/src/`, or `tools/`.

## Post-fix expectation (for the follow-on fix task)

Once all 17 roster members' equip abilities declare
`targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter { controller:
TargetController::You, ..Default::default() })]` (tightening Helm of the Host's
existing `TargetCreature` requirement, adding the missing requirement to the other
16), all 8 `primitives::cards1_equip_target_repair` tests and all 3
`core::cards1_equip_target_roster` tests are expected to pass with zero further edits
to either test file.
