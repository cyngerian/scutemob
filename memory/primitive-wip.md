# Primitive WIP: PB-EF12 — granted `any_color` ManaAbility color choice (EF-W-PB2-3) — CLOSES THE EF QUEUE

batch: EF12
task: scutemob-114
branch: feat/pb-ef12-granted-anycolor-manaability-color-choice-ef-w-pb2-3
started: 2026-07-18
phase: implement

Plan: `memory/primitives/pb-plan-EF12.md`. Review: `memory/primitives/pb-review-EF12.md`.

Coordinator decision (recorded in `memory/decisions.md` 2026-07-18): the colour choice rides
`Command::TapForMana { chosen_color: Option<ManaColor> }`, validated against the offered set
(WUBRG; reject `Colorless`; reject `None` on an `any_color` ability — no silent Colorless default).
PROTOCOL bumps (Command is on the SR-8 wire); HASH does NOT (Command is not in the GameState hash
closure). Simulator emits a concrete legal colour (SR-38 precedent).

## Steps
- [x] 1. `Command::TapForMana` gains `chosen_color: Option<ManaColor>` (`#[serde(default)]`) — done
- [x] 2. `handle_tap_for_mana` signature + engine.rs dispatch thread `chosen_color` — done
- [x] 3. `handle_tap_for_mana` validates + produces chosen colour for `any_color` abilities — done (rejects `Some(Colorless)` and `None`; `!any_color` requires `None`)
- [x] 4. Backfill `chosen_color: None,` to all existing `Command::TapForMana { .. }` literals — done (106 real literal sites across 20 files; plan's ~227 estimate overcounted, per usual PB yield-calibration drift)
- [x] 5. elven_chorus grant wired + flipped Complete; grant-based any_color cards verified — done (cryptolith_rite.rs/paradise_mantle.rs already Complete, dispatch shared, proven end-to-end by a same-shape grant test)
- [x] 6. Restore demoted tap-cost `AddManaAnyColor` rocks/lands (oracle-verified) — done: 17 restored to Complete (16 rocks/lands/creatures + elven_chorus); 1 eyeballed restore (deathrite_shaman) reverted after the refined gate caught it (targeted ability, CR 605.1a disqualifies); 7 held back on real second blockers with notes rewritten (command_tower, arcane_signet, commanders_sphere, path_of_ancestry, mox_amber, forbidden_orchard, glistening_sphere)
- [x] 7. `effect_choose_gate` refined (registered_colors any_color→WUBRG; stub gate served-vs-unserved) — done, incl. 2 new non-vacuity tests + a documented-and-verified-absent "mixed" hole
- [x] 8. Simulator `legal_actions.rs` + `mana_solver.rs` emit concrete legal chosen_color — done + new simulator test proving engine-legality
- [x] 9. New tests `pb_ef12_any_color_choice.rs` (happy + decoys, non-vacuous) — done, 7 tests, decoys empirically proven non-vacuous
- [x] 10. PROTOCOL 17→18 + fingerprint re-pin + sentinels; HASH only if machine-forced — done, HASH stays 55 (confirmed, no hash_schema gate reaction)
- [x] 11. OOS seed for unserved AddManaAnyColor family; bookkeeping; authoring-report rerun — done, OOS-EF12-1 filed in w-pb2-engine-findings doc; roster updated; coverage 61.1%→62.1% (1,098/1,796 → 1,117/1,798)
- [x] 12. All gates green — build/test(3476 pass)/clippy/fmt/defs-fmt all green. /review NOT run (coordinator's job per runner brief).

phase: done  # /review (primitive-impl-reviewer, Opus): 0 findings. Gates green, 3476 tests. PROTOCOL 18, HASH 55. Committed a8eb45b5. EF QUEUE COMPLETE.
