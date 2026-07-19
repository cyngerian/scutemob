# Primitive WIP: PB-OS3 — WhenTappedForMana trigger target dispatch (OOS-EF6-1)

batch: OS3
task: scutemob-129
branch: feat/pb-os3-whentappedformana-trigger-target-dispatch-oos-ef6-1-u
started: 2026-07-19
phase: close-out

Plan: `memory/primitives/pb-plan-OS3.md`. Review: `memory/primitives/pb-review-OS3.md`.

## Brief (THE PLAN IS `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 PB-OS3; canonical finding `memory/primitives/ef-batch-plan-2026-07-17.md` §10 OOS-EF6-1)

CORRECTNESS. `WhenTappedForMana` triggers that declare a DSL target never forward it.
Root cause is a **PendingTriggerKind vs ability-index index-space mismatch**, the exact class
PB-EF3 (EF-W-MISS-10) fixed for *attack* triggers but did not sweep on the mana path:

- `rules/mana.rs::fire_mana_triggered_abilities` (~643-717) iterates **`def.abilities`** and, for a
  targeted / non-mana WhenTappedForMana ability, queues
  `PendingTrigger::blank(src, player, PendingTriggerKind::Normal)` with
  `trigger.ability_index = ability_idx` — the **raw `def.abilities` index** (mana.rs:711-714).
- `rules/abilities.rs::flush_pending_triggers` resolves targets for a `Normal`-kind trigger by
  reading `obj.characteristics.triggered_abilities.get(trigger.ability_index)` — the **runtime
  enriched vec** (abilities.rs:6990-7004, PB-EF3 A2). Those two vecs are **different index spaces**
  (keywords/other abilities aren't triggered-ability entries), AND `enrich_spec_from_def`
  (`testing/replay_harness.rs`) has **no `WhenTappedForMana` block** — so the runtime vec has no
  entry at all. Net: the declared `targets` are unreachable; `DeclaredTarget{0}` resolves to nothing.
  Proven empirically: wiring `TokenSpec.recipient` on forbidden_orchard produced **0 tokens**.

## Fix (recommended: Option B — reclassify, no schema bump)

Two options per §3; **Option B is minimal and behaviour-only**:

- **Option B (RECOMMENDED)**: in `fire_mana_triggered_abilities`, change the queued kind for the
  push-to-stack branch (mana.rs:712) from `PendingTriggerKind::Normal` →
  `PendingTriggerKind::CardDefETB`. That variant's flush lookup (`has_ability_targets` 6885-6899,
  target resolution 7006-7020) uses **`def.abilities.get(trigger.ability_index)`** — the raw def
  index the mana path already holds — so the declared `targets` resolve correctly. Stack-object
  build (abilities.rs:8107-8125) is identical for both kinds (`StackObjectKind::TriggeredAbility`);
  `CardDefETB` only sets `is_carddef_etb: true` (resolve effect via registry raw index — also what
  the mana path wants). `CardDefETB` is an **existing** enum variant (hash arm 46) → **NO
  HASH/PROTOCOL bump**. WhenTappedForMana is not an ETB event; `compute_trigger_doubling` keys on
  `triggering_event` (None here), so no spurious Panharmonicon doubling.
  - Verify: the immediate-mana-ability branch (mana.rs:698-707, `targets.is_empty()` &&
    `is_mana_producing_effect`) is **untouched** — only the stack-push branch changes.
- **Option A (fallback, more invasive)**: add a `WhenTappedForMana` enrich block in
  `enrich_spec_from_def` forwarding `targets.clone()` AND change the mana path to iterate the runtime
  `triggered_abilities` vec (so `ability_index` is the runtime index). Loses `source_filter`
  (ManaSourceFilter) unless also carried on the runtime `TriggeredAbilityDef`, and there is **no
  `TriggerEvent::SelfTappedForMana`** variant — adding one risks a schema bump. Prefer Option B;
  planner has final say but must justify any wire bump and STOP if forced.

**No new DSL/wire type → NO PROTOCOL/HASH bump.** If a bump is forced, STOP and re-scope.

## Chain to verify by EXECUTION (SR-34/36; AC 5034)
`Command::TapForMana`/activate → `handle_tap_for_mana` → `fire_mana_triggered_abilities` (queues) →
`flush_pending_triggers` (auto-picks the TargetOpponent) → stack object carries the target →
resolution creates the token for the **targeted opponent**. Assert the stack object's target, not
just the end effect. Chain proven by a passing test, not source-tracing.

## forbidden_orchard flip (AC 5035)
- Any-color half already fixed by PB-EF12 (mana ability resolves to a real chosen colour).
- After the engine fix, wire `TokenSpec.recipient: PlayerTarget::DeclaredTarget { index: 0 }` on the
  WhenTappedForMana `CreateToken` (recipient field exists, PB-EF2, `card_definition.rs:3868`),
  remove the approximation/TODO comment, flip `completeness: known_wrong → Complete`.
- **Compose test (mandatory, one test)**: 4-player game; tap orchard for a **chosen colour** →
  mana produced AND a Spirit token created for the **targeted opponent** (a **decoy** opponent
  proves the recipient is the declared target, not the controller / a random / a wrong opponent).
- Flip to Complete ONLY if both halves compose; else honest marker recording the real remaining
  blocker.

## Roster sweep (all_cards(), AC 5036 — authoritative, not grep)
Indicative grep (7 defs): badgermole_cub, crypt_ghast, forbidden_orchard, leyline_of_abundance,
miraris_wake, wild_growth, zendikar_resurgent. **Only forbidden_orchard declares a target**; the
other 6 are inline mana-doublers/immediate abilities (`targets: vec![]`) and resolve via the
untouched immediate branch — verify each from `all_cards()` and confirm none regress. Report the
full set in close-out.

## Mandatory tests
- **End-to-end target dispatch** (AC 5034): the 4-player decoy compose test above, asserting the
  stack object / resolved recipient is the declared opponent.
- **No-regression**: an existing mana-doubler (e.g. wild_growth / miraris_wake — empty-target inline
  path) still fires and is unaffected by the kind change.
- **Existing `mana_triggers::test_mana_trigger_forbidden_orchard`** updated to the correct
  (recipient) behaviour with a CR 605.5a citation.
- No-target fizzle (CR 603.3d): planner judges — in practice all opponents always exist in a real
  4-player game; the auto-picker skips if no legal opponent.

## Close-out
Close **OOS-EF6-1** in `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 (SHIPPED banner +
table strike) and `memory/primitives/ef-batch-plan-2026-07-17.md` §10 (CLOSED banner). Update the
forbidden_orchard header comment. Update PB-OS3 status.

## Steps
- [x] 1. Plan — primitive-impl-planner → pb-plan-OS3.md (already present; Option B endorsed end-to-end)
- [x] 2. Implement engine change (mana.rs kind reclassify per Option B) — `crates/engine/src/rules/mana.rs` `fire_mana_triggered_abilities` else-branch: `PendingTriggerKind::Normal` → `PendingTriggerKind::CardDefETB`; `ability_idx` (raw def.abilities index) unchanged; comment added citing OOS-EF6-1/CR 605.5a
- [x] 3. Flip forbidden_orchard (recipient wiring + Complete) — both halves compose (proven by 4p decoy test); `crates/card-defs/src/defs/forbidden_orchard.rs` `recipient: PlayerTarget::DeclaredTarget{index:0}` added, TODO/approximation comments removed, `completeness: Completeness::Complete`
- [x] 4. Tests: `crates/engine/tests/rules/mana_triggers.rs` — updated `test_mana_trigger_forbidden_orchard` (CardDefETB kind + p2-recipient assert); NEW `test_forbidden_orchard_token_goes_to_declared_opponent_4player` (4p decoy compose, AC 5034/5035); NEW `test_mana_doubler_when_tapped_for_mana_no_regression` (wild_growth, immediate branch untouched); NEW `test_when_tapped_for_mana_roster_sweep` (SR-34/36, all_cards()-derived, AC 5036). Non-vacuity proven: reverted Change 1 to `Normal`, confirmed both forbidden_orchard tests FAIL, restored fix, confirmed all 16 pass again.
- [x] 5. Confirm no PROTOCOL/HASH bump (sentinel tests) — `pb_ef7_modal_activated::test_ef7_hash_and_protocol_versions` and `pb_ef12_any_color_choice::test_ef12_protocol_version_sentinel` both green: PROTOCOL_VERSION==18, HASH_SCHEMA_VERSION==55, unchanged.
- [x] 6. Review — primitive-impl-reviewer → pb-review-OS3.md; CLEAN BILL (0 HIGH/MED/LOW, 1 INFO forward-looking no-action re: mana_produced not propagated on CardDefETB path — not needed by forbidden_orchard). No fix phase.
- [x] 7. Green gates: `cargo build --workspace` clean; `cargo test --all` all-green (incl. `card_defs_fmt::card_defs_are_rustfmt_clean`); `cargo clippy --workspace --all-targets -- -D warnings` zero warnings; `cargo fmt --check` clean (after reformatting mana_triggers.rs); `tools/check-defs-fmt.sh` clean (1798 defs)
- [x] 8a. Close OOS-EF6-1 in source docs + plan — DONE: SHIPPED banner + table strike in oos-retriage-plan §3; CLOSED banner in ef-batch-plan §10 (canonical finding). forbidden_orchard header comment already rewritten by runner. (workstream-state §38 PB-EF6 line is a historical filing record, left as-is.)
- [ ] 8b. /review; Completion Sequence
