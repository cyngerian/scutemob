# PB-DX39 — task list (scutemob-230)

Status legend: `[ ]` pending · `[~]` in_progress · `[x]` completed

## Stage 0 — measure, decide, predict (before any production line)
- [x] S0.1 Pre-edit full-workspace baseline to a file (reproduce PB-DX52's 5,156 / 0 / 5 on 65 targets or report the discrepancy)
- [x] S0.2 Enumerate EVERY `state.objects.get(&source_id)` / `.get(&effect.source)` site in `effect_applies_to` (+ `is_effect_active`) — the brief's 17 is a FLOOR
- [x] S0.3 MCP: verify Umezawa's Jitte ruling 2005-02-01 text; verify CR 611.2a, 611.2c, 608.2h, 113.7a, 109.5, 400.7
- [x] S0.4 Measure whether the EXISTING LKI channels carry the source: `lki_objects` (SR-24 keyword gate), `StackObject.{lki_counters,lki_power,sacrificed_creature_lki}` — by EXECUTION, not by reading
- [x] S0.5 Reproduce both defects on a real drive BEFORE any fix (Jitte +2/+2 lost; Mardu +0/+3 lost)
- [x] S0.6 Wire prediction PER OPTION in writing, committed before any production line: (a) LKI read — predicted NONE; (b) stored StackObject snapshot — predicted HASH bump. Record the chosen option + counterfactual.
- [x] S0.7 State + pin the moment the snapshot represents (set determined at RESOLUTION, CR 611.2c; source characteristics from LKI of the moment it last existed, CR 608.2h/113.7a)

## Census (AC 7361)
- [x] C1 `all_cards()` census PRINTED by a test (SR-36, never grepped): (i) source-relative filter on a `SacrificeSelf`-cost ability, (ii) equipment/aura ability reading `AttachedCreature`/`AttachedLand`/`AttachedPermanent`, (iii) instant/sorcery mass effect (the PB-DX5 CONTROL class)
- [x] C2 Inverse ORACLE axis: "until end of turn" pumps printed on a sacrifice- or equip-cost ability
- [x] C3 Classify every member live-wrong / covered / still-blocked; each repaired or its exact missing identifier NAMED
- [x] C4 Flips predicted and NAMED before regeneration (0 expected); `mardu_ascendancy`'s marker note rewritten to name BOTH blockers; not promoted

## Implementation (AC 7359)
- [x] I1 ONE shared arithmetic (`source_view`-shape, PB-DX49 idiom) answering source controller / attached_to / chosen_* for every arm
- [x] I2 Make the LKI actually carry the source at the two moments it can leave with its ability pending (activation-cost self-move; departure while its ability is on the stack / pending)
- [x] I3 Rewire every enumerated arm of `effect_applies_to` onto the shared arithmetic — no per-arm LKI fallbacks
- [x] I4 Mechanism-keyed gate: a new arm that re-reads `state.objects` directly inside `effect_applies_to` is RED
- [x] I5 PB-DX5 T12 control (snapshot_affected_set runs DURING resolution) stays green and byte-identical

## Probes (AC 7359)
- [x] P1 Jitte drive on a real LocalGame/HumanChoice: equip → combat damage → activate → destroy the Jitte in response → resolve → layer-resolved P/T shows +2/+2 on the formerly equipped creature
- [x] P2 Mardu drive: sacrifice-activate → resolve → every creature its controller controls has +0/+3, asserted by layer-resolved P/T
- [x] P3 Primitive-level probes for each repaired arm class + the CONTROL class (must stay green)
- [x] P4 Roster gates (census printed, mechanism gate, SR-24 coupling gate)

## Wire + gates (AC 7360, 7362)
- [x] W1 Fingerprints gate-computed AFTER the change; if moved: sentinels re-pinned BY SYMBOL then survivor-scanned on BOTH axes (shape AND value spelling, `OOS-DX36-8`), history rows appended never edited, frozen-prefix digests re-pinned, `history_is_append_only` + `frozen_prefix_is_pinned` green. If unmoved: gate-executed and the counterfactual stated.
- [x] G1 Post-edit full run; delta by test NAME via BYTE-EXACT python set difference (`OOS-DX20b-5`), count-vs-name reconciliation, duplicate-name scan (`OOS-DX35-8`), leavers/renames disclosed
- [x] G2 `clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`, `cargo build --workspace` — ALL against the FINAL tree
- [x] G3 `npm run build`: run it or state N/A with the measured reason
- [x] G4 Benches: merge-base A/B, same-code band measured FIRST, published honestly (`effect_applies_to` IS on the layer walk)
- [x] G5 Coverage regenerated, self-dating churn reverted, flips confirmed against the prediction
- [x] G6 Revert matrix executed by the coordinator (not accepted from delegated reports); UNDISCRIMINATED rows disclosed in the test itself; matrix in `memory/primitives/pb-DX39-execution-notes.md`

## Bookkeeping (AC 7362)
- [x] B1 Registry: grep the IDs first (dispatch hygiene 5); close `OOS-DX5-3` + `OOS-DX5-7`'s residual with pipes ESCAPED; file `OOS-DX39-N`
- [x] B2 v4 memo §4 row 15 struck; banner repointed to rank 16 (PB-DX53)
- [x] B3 `memory/primitives/pb-DX39-execution-notes.md` written
- [x] B4 CLAUDE.md Current State + `memory/workstream-state.md` handoff
- [x] B5 `/review` run; all findings taken or declined with reasons
- [x] B6 Headline surfaces re-checked against the registry AFTER the fix cycle (dispatch hygiene 8)
- [x] B7 All four acceptance criteria satisfied via `esm task satisfy` as each is met
