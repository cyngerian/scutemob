# PB-DX21 — execution notes (revert matrix, measurements, closure evidence)

**Task**: `scutemob-200` · **Branch**:
`feat/pb-dx21-cr-5081-attackers-may-be-declared-without-limit-oos-`
**Plan**: `memory/primitives/pb-plan-DX21.md` · **Stage 0**: `memory/primitives/pb-DX21-stage0.md`
**Review**: `memory/primitives/pb-review-DX21.md` (0 HIGH / 7 MEDIUM / 8 LOW, **all 15 taken**)

This file exists because the review's **L6** found that the revert evidence plan §9 mandates was
recorded in commit messages only, and `memory/primitive-wip.md` still belongs to PB-DX32. It is the
tracked artefact. Where a failure string is quoted below it is verbatim from the captured run;
where it is described rather than quoted, that is said so explicitly rather than implied.

## Commits

| SHA | Stage |
|---|---|
| `2bd2687b` | Stage 0 — baseline, plan, the CR refutation of the brief's preferred guard |
| `614f54ac` | Stage 1 — the CR 508.1 once-per-combat guard, the marker, the error variant, T1–T8 |
| `d198568f` | Stage 2 — `HASH_SCHEMA_VERSION` 72 → 73 |
| `ac21dd32` | Stage 3 — `legal_actions.rs` suppresses the offer after the declaration (SR-38) |
| `6f065bd6` | Stage 4 — both client-side mitigations deleted |
| `91e1f6b4` | Stage 5 — the comments and docs PB-DX21 falsified |
| `f573ef21` | Review fix cycle — M1–M5, M7, L1–L5, L7–L8 |

## Revert matrix

Every row was **executed**, not reasoned about. `Compiling mtg-engine` (or `mtg-simulator`) was
confirmed in the captured build output before the result was trusted — a stale binary is a silent
pass — then restored, with `git diff --stat` confirmed clean before the next.

| Probe | Revert applied | Result |
|---|---|---|
| T1 target overwrite | delete the guard block from `combat.rs` | RED — `expect_err` panicked on an `Ok(GameState{…})` |
| T2 trigger re-fire | same | RED, same shape |
| T3 raid-count clobber | same | RED, same shape |
| T4 empty declaration | same | RED — verbatim: `"CR 508.1a/508.8: the empty declaration already performed the once-per-combat action; a later non-empty declaration must be rejected: (GameState { … })"` |
| T4 **second** revert | guard condition → `!c.attackers.is_empty()` (the brief's preferred implementation) | **only T4 red**; T1/T2/T3/T5/T6/T7/T8 green. This is the discriminator between the two candidate guards, not merely between fixed and unfixed. |
| T5 per-combat scope | guard deletion leaves it **green by design** — it is a marker-*scope* probe, not a guard probe. Its real falsifier is a stale-`CombatState`-reuse or per-*turn* marker storage (documented in the test, review L5). | n/a |
| T6 success-path only | move the marker-set to just after the guard (set on entry) | RED at *"a rejected declaration must not set the marker"* — one step **earlier** than the plan predicted, which is a stronger catch |
| T2b CR 603.3d suspended trigger | move the marker-set below the `if state.pending_trigger_targets.is_some()` early return | RED, and only this test |
| T7 hash coverage | delete `self.attackers_declared.hash_into(hasher)` | RED with both digests equal (`left == right`, `[31, 129, 109, …]`) |
| T4 step 4 (CR 117.4), after the M5 repair | move the `players_passed` reset unconditionally above the guard | RED with a real `left: {} right: {PlayerId(2)}` mismatch |
| offer suppression | delete the `!combat.attackers_declared` clause at `legal_actions.rs:878` | RED (rebuild confirmed via `Compiling mtg-simulator`) |

## Two methodology findings the reverts produced

1. **`process_command` cannot observe a rejected command's mutations.** Its signature is
   `Result<(GameState, Vec<GameEvent>), GameStateError>`, so on `Err` Rust discards every mutation
   the callee made, wherever it happened. T6's first draft (via `process_command`) stayed **green**
   under the "set on entry" revert; it only discriminates when it calls
   `rules::combat::handle_declare_attackers(&mut state, …)` directly. T4's CR 117.4 assertion was
   vacuous for the same reason until repaired the same way. Filed tree-wide as **`OOS-DX21-7`**.
2. **`process_command`'s PB-DP7 admission gate intercepts before the handler.** A second
   `DeclareAttackers` issued while a trigger-target choice is pending is refused as
   `BlockedByPendingDecision` and never reaches `handle_declare_attackers`, so T2b asserts the
   marker on the suspended state **and** exercises the handler's own guard directly.

## Measurements

- **Baseline** (on-branch, before any edit): **4,388 / 0 / 5**, `--workspace --no-fail-fast` to a
  file, 43 `test result:` lines summed. Matches the criterion's stated number exactly.
- **Final**: **4,398 / 0 / 5** (+10), independently re-run by the coordinator; residual list empty.
  `clippy --workspace --all-targets -- -D warnings`, `fmt --check` and `tools/check-defs-fmt.sh`
  all clean.
- **HASH 72 → 73**, computed from the failing gate's own output, never predicted:
  `decl_fingerprint = 44f2c130…`, `stream_fingerprint = cf3e47e7…`,
  `FROZEN_HISTORY_PREFIX_DIGEST = e00c419b…`. History row **APPENDED**; no shipped row edited.
  **45** sentinel lines across **44** files re-pinned — the plan predicted 44, and the extra sites
  were two bare-`72` spellings (`hash_schema.rs`, `pb_dx6_unflattened_payment_sites.rs:1990`) plus
  two split across two lines, which a single-line grep cannot see and only a full run surfaced.
- **PROTOCOL 35**, gate-executed, unmoved.
- **Coverage 1,133/1,803 = 62.8%**, unmoved — proven by regenerating `tools/authoring-report.py`
  to a byte-identical body modulo the sha/date stamps, not by an empty card-defs diff (the batch
  mandates two comment edits there).
- **Benches** within noise, slightly faster: `priority_cycle_4p` 24.4–24.6 µs (pinned 25.5–26.0),
  `full_turn_4p` 216.0–218.0 µs (pinned 220–222).
- **Golden scripts**: exactly two files carry ≥2 `"action": "declare_attackers"` — `combat/069`
  and `combat/070` — and both repeats are **cross-turn** (Turn 4 / Turn 5), not same-combat. No
  script churn; SR-9b green.
- **Seeded-pin classification** (review M7): the gate-config rejection rate moved
  **31.081‰ → 6.909‰** and the wasted-tap share **89% → 92%**, both well inside their ratchets —
  **neither constant was changed**, both were documented with the mechanism.
  `pb_dx32_fuzz_output.rs` T4.1/T4.3 are **unmoved, proven by an executed ablation** (disabling the
  suppression clause reproduced byte-identical counters for that seed), not merely observed.
  Mechanism filed as **`OOS-DX21-6`**.

## Closure proof

`test_s8_scripted_human_playthrough_is_clean_on_five_seeds` is green **with no cap**. Its own
contract is *"the policy only ever submits an action the game offered it one instant earlier, so a
rejection means the offer was wrong"* — so with both client-side mitigations deleted **with their
mechanism**, that test can only stay green if the offer layer and the engine agree. That is the
closure of `OOS-M11-9`, and it is the reason the deletions were mandatory rather than cosmetic.
