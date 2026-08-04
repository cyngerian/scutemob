# Primitive Batch Review: PB-DX32 — make the fuzzer's *output* mean something

**Date**: 2026-08-03
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 514.1, 704.3, 704.5m/n, 500.4, 601.2c, 603.3d, 608.2d, 701.9b, 701.22a, 701.23a,
701.25a (all *observed*, none implemented — this batch adds no rules behaviour)
**Engine files reviewed**: `crates/engine/tests/core/decision_gate.rs` (appended T17 only),
`crates/engine/tests/core/decision_site_walk.rs` (read-only, unmodified)
**Simulator files reviewed**: `crates/simulator/src/{report,local_game,driver,invariants,
decision_coverage,lib,bin/fuzzer}.rs`, `crates/simulator/tests/{pb_dx32_fuzz_output,
sim5_bot_cast_discipline,local_game_playthrough}.rs`
**Other**: `tools/play-server/src/{main,view}.rs`, `crates/simulator/src/deck.rs`,
the nine committed `memory/primitives/pb-dx32-*.txt` evidence files
**Card defs reviewed**: **0 — by design.** `crates/card-defs/` is out of scope for this batch and
the coordinator confirmed the diff over it is empty.

---

## READ THIS FIRST — what I verified, and how

**I had no shell tool in this session. I executed nothing.** Not one test, not one revert, not one
`cargo` invocation. `OOS-DX22-11` exists precisely to stop a read-only inspection being reported as
a proof, so the ledger is explicit:

| claim | how I checked it | strength |
|---|---|---|
| Both terminal `GameResult` paths run `check_no_leaked_tokens` | read `local_game.rs:697` + `driver.rs:149` → both call `result_snapshot`, which calls it at `:492` | **source-proved** (there is exactly one call site and both paths reach it) |
| Nothing but `no_orphaned_tokens` was reclassified | read `record_violations` (`local_game.rs:541-549`) — a single string equality, everything else falls to `violations` | **source-proved** |
| `player_consistency` / `attachment_validity` stay HARD | as above, plus the committed A/B (`pb-dx32-stage4-fuzz-after.txt:19-21`) shows both in the HARD block | **source-proved + artefact** |
| `--stop-on-error` keys on the HARD bucket only | `fuzzer.rs:207` reads `result.violations` | **source-proved** |
| `OBSERVABLE_ROW_IDS` == the `Served` rows of `ROWS` | enumerated all 22 ids and their classes out of `decision_site_walk.rs:287-514` by grep and compared by hand | **source-proved** |
| Both `row_id_for` matches are exhaustive, no wildcard | read `decision_coverage.rs:256-276` | **source-proved** |
| `CleanupDiscard` has no `ROWS` row (CR 514.1) | enumerated `ROWS` ids; confirmed CR 514.1 is a turn-based action via MCP | **source-proved + CR-checked** |
| `COMMANDER_POOL`'s mirror is clause-for-clause | diffed `pb_dx32_fuzz_output.rs:681-685` against `deck.rs:42-46` by eye — identical three clauses in the same order | **source-proved** |
| Invariant 7: no new field in `GameOverView` | read `view.rs:2487-2505` — unchanged; and `violation_summary` (`:2454`) drops `description` entirely, so no name can leak | **source-proved** |
| `tools/` footprint is one line | grepped every `..Default::default()` in `main.rs`; only `:3339` is a `GameResult` site | **source-proved** |
| T3.2's controlled fixture really leaves the run open | read `submit()` (`:966-1010`) — it applies the sequence and returns, it does **not** call `advance()`, so `waste_run` is still `Some` at `.waste()` | **source-proved** |
| The **15+ executed reverts** and their quoted failure messages | **NOT verified.** Taken on trust. For T6.1(b), T4.2 and T2.3 I reconstructed the failure path from source and found each quoted message *consistent* with what the code would emit — that is corroboration, not proof | **trust + consistency check** |
| The A/B numbers (426 → 125 hard + 301 transient, 16/20 → 6/20) | read out of the two committed artefacts and cross-summed: 301+114+11 = 426, 114+11 = 125 | **artefact-verified** |
| Test counts / clippy / fmt / PROTOCOL 35 / HASH 72 / coverage 62.8% | taken from the coordinator's own executed runs, per the dispatch | **given** |

On the specific question *"can a comment satisfy the T6.1 gate?"*: `strip_line_comments` **is**
applied, to the whole file, at `decision_gate.rs:1503`, before `extract_const_array_block` ever
runs. A `//`-commented tuple is therefore genuinely invisible to the gate, and the T6.1(b) revert
would redden as reported. **But the strip handles line comments only** — see Finding M8.

---

## Verdict: needs-fix

No HIGH findings. The batch is unusually well built: `result_snapshot` is a real single-source
collapse, the transient/hard split is answered by a strictly stronger end-state property that is
genuinely asserted on both `GameResult` paths, the `_AT_GATE_CONFIG` threshold duplication is
**forced rather than evasive** (both gate-config populations measure *higher* than the binary's, so
reusing the binary constants would have been red on arrival — the honest direction), the
`decision_coverage.rs` roster is a faithful id-only mirror of `ROWS`, and the `COMMANDER_POOL`
filter is a true clause-for-clause mirror of `deck.rs`. Architecture Invariant 7 is intact and,
if anything, the seat payload got *less* leaky (the token-naming `no_orphaned_tokens` class left
the HARD bucket). What is wrong is smaller and mostly of one shape: **the batch's own
comment-and-doc hygiene has holes in exactly the places the plan told it to look**, and one of
them silently zeroed a diagnostic in the very file the batch cites as its precedent. Eight MEDIUM
and nine LOW findings below. Stage 7 (close-out) has not been run at all; several of the MEDIUMs
are its work and would be discharged by it.

---

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| M1 | MEDIUM | `crates/simulator/tests/local_game_playthrough.rs:457-463`, `:560-568` | **The batch silently zeroed a diagnostic in its own precedent file.** The local `no_orphaned_tokens` split can never match now, so the playthrough's reported transient-token count is permanently 0. **Fix:** split from `game.transient_violations()`. |
| M2 | MEDIUM | `crates/simulator/src/local_game.rs:371-373` | **Plan-mandated comment correction missed.** The `rejections` field doc still names only `MAX_RETAINED_REJECTIONS` as the cap — the wrong constant for the fuzzer's own configuration. **Fix:** name both caps. |
| M3 | MEDIUM | `crates/simulator/src/invariants.rs:463-465` | **Aspirationally-wrong comment.** `check_no_orphaned_tokens` still says "if they remain, something is wrong" — the reading Stage 4 exists to retire. **Fix:** record the transient split and its answering check. |
| M4 | MEDIUM | `crates/simulator/src/invariants.rs:3-11` | Module header still says "**Nine** checks can fire" and enumerates them; the module now exports a tenth. **Fix:** say ten, and say the tenth is end-of-game only, not in `check_all`. |
| M5 | MEDIUM | `docs/mtg-engine-simulator.md:250-253` | Item #10 states legal-action soundness is "enforced only by ... `local_game_playthrough.rs` ... not by the fuzzer" — **false as of this batch**. **Fix:** mark #10 served at run scope; keep #11 unwritten; do not close `OOS-SIM3-2`. |
| M6 | MEDIUM | `crates/simulator/src/report.rs:77-84`, `:99-106` | Two thresholds cite a **5-game** measurement when the batch's own committed **20-game** artefact measures the same configuration differently (78.6% waste, 21.118‰). **Fix:** re-quote from the 20-game run; re-judge the 85 ceiling at 78.6%. |
| M7 | MEDIUM | `crates/simulator/tests/pb_dx32_fuzz_output.rs:532-535` | T3.1's non-vacuity floor is `total_taps > 0` where its sibling T2.2 uses a measured 80%-of-baseline floor. **Fix:** `total_taps >= 77` (80% of the measured 97), message carrying the measurement. |
| M8 | MEDIUM | `crates/engine/tests/core/decision_gate.rs:1503`, `:1484-1493` | The source gate strips **line** comments only; a `/* … */`-wrapped `UNOBSERVABLE_ROW_IDS` tuple stays green while vanishing from the compiled roster. A duplicated id is also hidden by the `BTreeSet`. **Fix:** strip block comments too, and compare counts as well as sets. |
| L1 | LOW | `crates/simulator/src/bin/fuzzer.rs:667-670` | Rejection-class extractor splits at the first `(` only; struct-like variants produce junk class rows (see the committed evidence). **Fix:** split at the first of `(` or `{`. |
| L2 | LOW | `crates/simulator/src/bin/fuzzer.rs:805-820` | The printed decision-coverage header never states that 5 is 5 **of 22**. **Fix:** print the ratio in the header. |
| L3 | LOW | `crates/simulator/src/local_game.rs:52-56` | The `#[allow(clippy::large_enum_variant)]` justification says the `GameResult` is "constructed at most once per game" — `advance()` rebuilds it on every call once the game is over. **Fix:** reword (the `allow` itself is fine). |
| L4 | LOW | `invariants.rs:487-489`, `local_game.rs:536-538`, `report.rs:66-69`, `pb_dx32_fuzz_output.rs:547-552` | The CR 704.3 parenthetical reads as if the rule prescribes step-entry-only SBA checks. It requires them whenever a player would get priority. **Fix:** "in deviation from CR 704.3 (`OOS-M11-7`)". |
| L5 | LOW | `crates/simulator/tests/sim5_bot_cast_discipline.rs:465-475` | T3.2's controlled half asserts an absolute `tap_runs == 1`, not the equivalence the test is named for, though the fixture has `record_journal: true`. **Fix:** compare against `metrics_of(&mid_run_game)`. |
| L6 | LOW | `crates/simulator/tests/pb_dx32_fuzz_output.rs:700-703` | `MOVED_MSG` does not name the other seed-dependent gates a corpus flip will also redden. **Fix:** list T2.2/T3.1/T4.1/T4.3/T6.3. |
| L7 | LOW | `crates/simulator/src/report.rs:84`, `:106` | `MAX_BOT_REJECTION_PER_MILLE` and `MAX_RANDOM_BOT_WASTED_TAP_PCT` are enforced by the binary alone — no test reads either, and F19 says the binary is not in CI. **Fix:** say so at each constant. |
| L8 | LOW | `crates/simulator/src/bin/fuzzer.rs:13`, `:29-61` | `--stop-on-error` help still says "first violation" (now: first HARD violation), and the boundary-event block has no PB-DX32 row. **Fix:** both (the latter is Stage 7). |
| L9 | LOW | `crates/simulator/tests/pb_dx32_fuzz_output.rs:854-857` | The message claims "the set of ids `row_id_for` can ever return"; the test observes only its five fixtures. Exhaustiveness is what bounds the set. **Fix:** reword. |
| L10 | LOW | `crates/simulator/src/invariants.rs:498-513` | `check_no_leaked_tokens` treats a token on the **Stack** at game end as a hard violation, while its sibling `check_no_orphaned_tokens` deliberately exempts the Stack. Faithful to the mirrored `local_game_playthrough.rs:470` and measured 0/20, but the divergence is undocumented. **Fix:** one sentence at the function. |

---

### Finding Details

#### M1 — the batch zeroed a diagnostic in its own precedent file

**Severity**: MEDIUM
**File**: `crates/simulator/tests/local_game_playthrough.rs:457-463` and `:560-568`
**Issue**: Plan fact **F11** names this file as the treatment Stage 4 copies. That file does its own
post-hoc split:

```rust
for v in game.violations() {
    if v.check == "no_orphaned_tokens" { result.transient_token_violations.push(...) }
    else { result.violations.push(...) }
}
```

As of Stage 4, `LocalGame::violations()` **can never contain** `no_orphaned_tokens` — the split now
happens upstream in `record_violations`. So the `if` branch is dead, `transient_token_violations`
is permanently empty, and the run report at `:560-568` now prints
`… 0 transient-token reports (OOS-M11-7)` on every seed, forever. A reader of that output would
conclude the transient class stopped firing. It did not; it moved.

Nothing is *asserted* on that vector (the file says so at `:78-79`), so no test went vacuous and no
assertion weakened — which is why this is MEDIUM and not HIGH. But it is precisely the
`OOS-DX22-13` failure the plan quotes at its own §0 ("a number whose meaning changed without saying
so"), committed in the file the batch cites as the pattern it is generalising.

**Fix**: change the loop to read `game.transient_violations()` for the transient half and
`game.violations()` for the hard half; keep the field docs, and add one line noting the split is now
`LocalGame`'s (PB-DX32 Stage 4) and this is a read of it, not a re-derivation.

#### M2 — the third plan-mandated comment correction was not made

**Severity**: MEDIUM
**File**: `crates/simulator/src/local_game.rs:371-373`
**Issue**: Plan §5 Stage 2 step 1 lists **three** doc comments to rewrite: `:338-341`, `:443-445`,
`:1017-1024`. Two were rewritten well (the `rejections()` accessor at `:646-655` and
`record_rejection` at `:1228-1251` both past-tense the old account and explain the change). The
struct **field** doc was not:

```rust
/// Bot-seat commands the engine refused (SIM-5 fix (3)). Retention is capped at
/// [`MAX_RETAINED_REJECTIONS`]; `rejection_count` is not, so truncation is
/// visible rather than silent.
rejections: Vec<RejectedCommand>,
```

For the configuration Stage 2 exists to serve (`record_journal: false`, i.e. `mtg-fuzzer`), the cap
is `MAX_SAMPLED_REJECTIONS = 8`, not 256. A reader of the field doc gets the wrong constant.

**Fix**: "Retention is capped at [`MAX_RETAINED_REJECTIONS`] with the journal on and
[`MAX_SAMPLED_REJECTIONS`] with it off — see [`Self::record_rejection`]; `rejection_count` is
capped by neither."

#### M3 / M4 — two aspirationally-wrong comments in `invariants.rs`

**Severity**: MEDIUM (each)
**Files**: `crates/simulator/src/invariants.rs:463-465`, `:3-11`
**Issue (M3)**: `check_no_orphaned_tokens` still reads *"Tokens in graveyard/exile are cleaned up by
SBAs — if they remain, something is wrong."* That sentence is the exact claim Stage 4 spends a
whole stage refuting: under this engine's CR 704.3 deviation the report is transient by
construction and is now routed to a non-halting bucket. Plan §5 Stage 7 item 1 names this site.
Leaving it is the `memory/conventions.md` "aspirationally-wrong comment" class — a comment arguing
for a treatment the code no longer applies.
**Issue (M4)**: the module header still says *"**Nine** checks can fire"* and enumerates them. The
module now exports a tenth (`check_no_leaked_tokens`), deliberately outside `check_all`. Plan Stage
7 item 1 mandates the correction *and* the "not in `check_all`" qualification.
**Fix**: M3 — append: this check's output is split out as transient by `LocalGame::record_violations`
and answered by `check_no_leaked_tokens` at both terminal paths; a report here is a checkpoint
artefact, not a defect. M4 — "Ten checks exist; nine of them fire from `check_all` … the tenth,
`check_no_leaked_tokens`, is an end-of-game check and is deliberately not in `check_all`."

#### M5 — a doc statement the batch made false

**Severity**: MEDIUM
**File**: `docs/mtg-engine-simulator.md:250-253`
**Issue**: The twelve-check list's item #10 reads: *"**NOT IMPLEMENTED** (OOS-SIM3-2). Nothing in
`invariants.rs` checks this. It is the SR-38 property, and it is currently enforced only by the
assertions in `local_game_playthrough.rs` …, not by the fuzzer."* Every clause after the first is
now false: the fuzzer enforces it at run scale (`print_sr38_summary` → `MAX_BOT_REJECTION_PER_MILLE`
→ `std::process::exit(1)`), and `cargo test` enforces it at gate scale (T2.2). Only "nothing in
`invariants.rs` checks this" survives — and the plan §3.0 explains why that is deliberate.
**Fix**: exactly what plan Stage 7 item 1 prescribes: mark #10 **served at run scope by PB-DX32's
SR-38 invariant, not by a `check_all` function**; say plainly that #11 (SBA idempotency) is still
unwritten; and do **not** mark `OOS-SIM3-2` closed.

#### M6 — the thresholds cite the smaller sample when the bigger one is in the same commit

**Severity**: MEDIUM
**File**: `crates/simulator/src/report.rs:77-84` and `:99-106`
**Issue**: Both binary-scope constants are documented from a **5**-game measurement:

* `MAX_BOT_REJECTION_PER_MILLE = 30` — *"5 fuzz-shaped games, 23,613 commands, 542 rejections =
  22.953 per mille"*
* `MAX_RANDOM_BOT_WASTED_TAP_PCT = 85` — *"5 fuzz-shaped games, 200 turns: 1,986 wasted of 2,641
  taps = 75%"*

The batch's **own committed artefact** for the same command line over **20** games
(`memory/primitives/pb-dx32-stage4-fuzz-after.txt`) reports:

* `:41` — `1995 rejections / 94467 commands = 21.118 per mille`
* `:74` — `8423 taps of 10720 total = 78.6%`

The five games are literally seeds 1-5 of the twenty (their per-seed command counts sum to 23,613
exactly, matching plan §0.3), so the 20-game figure is strictly the better estimate of the same
population, and it was in hand before the constants were written. It matters materially for the
waste pin: real headroom is **6.4** points (78.6 → 85), not the ~10 the doc's 75% implies. The
plan opens (§0) by quoting `OOS-DX22-13` — *"before the PB-DX22 fix cycle the binary's only
by-`check` output was a five-offending-game sample, so every historical claim is a sample unless
it says otherwise"* — and then pins two constants from a five-game sample without saying so.

**Fix**: re-quote both docs from the 20-game run, naming
`memory/primitives/pb-dx32-stage4-fuzz-after.txt` as the source and its line numbers. Then
re-decide `MAX_RANDOM_BOT_WASTED_TAP_PCT`: either keep 85 and state that headroom is 6.4 points, or
raise/re-derive it deliberately. Do **not** leave the 5-game number standing.

#### M7 — T3.1's non-vacuity floor is a token gesture next to T2.2's

**Severity**: MEDIUM
**File**: `crates/simulator/tests/pb_dx32_fuzz_output.rs:532-535`
**Issue**: T2.2, on the identical 3-seed × 25-turn fixture, carries a real floor derived from its
own Stage-0 measurement (`total_commands >= 2_200`, stated as 80% of the measured 2,767) *and* a
`total_rejections > 0`. T3.1 carries only `total_taps > 0`. Stage 0 measured **97** taps at this
configuration. With a floor of one tap, a change that collapsed the tap population — a bot scoring
change, an offer-gate change, a `build_fuzz_state` change — could leave a single unwasted tap and
pass at 0%, and the gate would report green while measuring nothing. This is the same sampling
weakness the batch is elsewhere careful about, and it is inconsistent with its own sibling gate
twelve lines away.
**Fix**: `assert!(total_taps >= 77, "non-vacuity floor: total_taps {total_taps} is far below the
Stage-0 measurement (97) at this configuration — a run that stopped tapping cannot pass this gate
trivially");` — same shape and same 80% rule T2.2 uses.

#### M8 — the source gate is still satisfiable by a *block* comment

**Severity**: MEDIUM
**File**: `crates/engine/tests/core/decision_gate.rs:1503` (strip), `:1484-1493` (extract),
`:1530-1540` (set comparison)
**Issue**: The mandatory T6.1(b) revert was line comments, and against line comments the gate is
sound — `strip_line_comments` runs over the whole file before extraction, so a `//`-prefixed tuple
is genuinely gone. But `strip_line_comments` (`:1124-1132`) truncates at `//` and knows nothing
about `/* … */`. Wrap a `UNOBSERVABLE_ROW_IDS` tuple in a block comment and:

* the compiler drops it → `UNOBSERVABLE_ROW_IDS` has 16 entries, `ROW_COUNT` becomes 21, the row
  disappears from `print_decision_coverage`'s unobservable list;
* `quoted_strings` still finds both of its string literals → `union` is still the full 22-id set →
  **T6.1 stays green**, including its `union.len() >= MIN_ROWS` floor.

Nothing else covers it: nothing asserts `ROW_COUNT == 22`, and T6.2 only constrains the OBSERVABLE
half (a block-commented observable id *would* be caught there, because `reachable != observable`
would fire). So the blast radius is limited to the unobservable roster and to `ROW_COUNT`. It is
still the comment-satisfiable-gate class the plan itself flags as "found in this exact family, do
not ship one again", and the fix is cheap. A second, smaller hole in the same test: `observable` and
`unobservable` are `BTreeSet`s, so a duplicated id is invisible to the comparison while `ROW_COUNT`
silently grows.

**Fix**: two lines. (1) Strip `/* … */` spans as well as `//` before extraction (a
`strip_block_comments` helper beside `strip_line_comments`, or fold it in). (2) Add a count
assertion that catches both the block-comment case and duplicates:
`assert_eq!(observable_raw.len() + unobservable_all.len() / 2, ROWS.len(), "roster id COUNT must
equal ROWS.len() — a duplicate id, or a row hidden inside a /* */ comment, is invisible to the set
comparison above");` where `observable_raw` is the pre-`BTreeSet` `Vec`.

---

## What I attacked and found **clean** — stated so it is not mistaken for unexamined

These are the dispatch's own six load-bearing questions. Each was checked; each passed.

**(2) The transient/hard split is honest.** `check_no_leaked_tokens` runs at *both* `GameResult`
paths through the single `result_snapshot` call site (`local_game.rs:492`), reached from
`advance()`'s GameOver return (`:697`) and `GameDriver`'s Halted arm (`driver.rs:149`). There is no
third `GameResult`-producing path with a live game (the other two literals are pre-game build
failures). It lands in the **hard** bucket. `record_violations` splits on one string and one string
only, so `player_consistency` (114) and `attachment_validity` (11, `OOS-DX22-8`) are untouched —
confirmed both in source and in the committed A/B, where the HARD block contains exactly those two.
`check_no_leaked_tokens` is genuinely *stronger* than `check_no_orphaned_tokens` (it drops the Stack
exemption), so the split is answered by a superset property, not a sibling one.

**(3) The thresholds are ratchets, and the `_AT_GATE_CONFIG` duplication is forced, not evasive.**
This was the question I most expected to fail, and it does not. Both gate-config populations measure
**higher** than the binary populations — 31.081‰ vs 21-23‰, and 89% vs 78.6% — so reusing the
binary constants in the tests would have made both gates red on arrival. That is the direction that
proves the duplication is a real population difference and not a way of dodging red: an evasive
duplication produces a *looser* twin for a *lower* measurement, and these do the opposite. The
stated mechanism (a shorter game's early taps and early actions have proportionally fewer castable
targets) is plausible and is written at the constant rather than asserted in a handoff. Each of the
five constants carries a measurement, a date, named open seeds (`OOS-SIM5-3/-5`, `OOS-SIM6-3`,
`OOS-CARDS2-4`, `OOS-SIM4-2`, `OOS-SIM2-1`) and a ratchet-downward instruction. My only complaints
are M6 (wrong sample cited) and M7 (weak floor) — neither touches the design.

**(4) The `waste()` open-run close.** `waste()` takes `let mut tally = self.waste;` — `WasteTally`
is `Copy`, so it is a real copy and `self` is untouched. `fold_waste` is a faithful transcription of
`metrics_of`'s per-record body, including the "different player interleaved closes the old run
unclassified" arm, and is called at both fold sites immediately beside `mechanics.record`. Both fold
sites see exactly the set of commands the journal receives (both `apply_sequence` and
`apply_command` journal the same records they fold, on the same branch), so the streaming fold and
the journal walk really are the same measurement, not two measurements that happen to agree. The
R8 problem was hit for real, root-caused rather than worked around, and the added human-`submit()`
fixture does discriminate: `submit()` does **not** call `advance()` (verified at `:966-1010`), so
`waste_run` is still `Some` when `.waste()` is called, and dropping the close yields `tap_runs: 0`
against `total_taps: 1` — exactly the quoted failure. My only note is L5 (the controlled half
asserts an absolute, not the equivalence).

**(5) Decision-coverage honesty.** `OBSERVABLE_ROW_IDS` is exactly the five `Served` rows — I
enumerated all 22 `ROWS` ids and classes independently and the partition matches, 5 Served / 14
AutoChosen / 2 Gated / 1 NoDecision, and `UNOBSERVABLE_ROW_IDS`'s own header states that 14/2/1
split correctly. Both matches in `row_id_for` are exhaustive with no wildcard (3 `BlockingDecision`
variants, 4 `EffectChoiceQuestion` variants). `CleanupDiscard → None` is CR-correct: CR 514.1 is a
turn-based action that doesn't use the stack (MCP-confirmed), and `ROWS` has no cleanup row, while
`discard_cards` is unambiguously the CR 701.9b *effect* row (`cr: "701.9 / 701.9b"`, served by
ENG-1) — so the mapping does not conflate the two. The re-observation-weighting caveat **is** in the
printed header, not only in the module doc. The 4-of-5 (debug/60-turn) and 5-of-5 (release/200-turn)
results are both recorded rather than only the flattering one. Only L2 (the header never says "5 of
22") is open here.

**(6) Architecture Invariant 7.** `game_over_view` (`view.rs:2487-2505`) gained nothing —
`GameOverView` still carries winner/turn_count/total_commands/halted/reason/violations, and
`violation_summary` (`:2454`) reduces each violation to `"{check} (turn {turn})"`, discarding the
`description` that interpolates `obj.characteristics.name`. So neither `RejectedCommand`'s `Command`
`Debug` nor `transient_violations`'s token names nor `decision_coverage` can reach a seat payload.
Worth recording as a positive: the batch made this channel *cleaner*, because the token-naming
`no_orphaned_tokens` class left the HARD bucket that feeds `GameOverView.violations`; the
`leaked_tokens` entries that replace it are a strictly smaller, end-of-game-only population, and
they are name-redacted by the same summariser. The `tools/` footprint is one line at
`main.rs:3339`, in `#[cfg(test)]`, verified by grepping every `..Default::default()` in that file.

**(7) `OOS-CARDS2-3`'s gate.** `commander_pool()` is a clause-for-clause mirror of `deck.rs:42-46`
— `completeness.is_complete()`, `supertypes.contains(Legendary)`, `card_types.contains(Creature)`,
same order, with the line range named in the comment. T5.2's membership half is the right
anti-drift partner and the plan's own prediction about it (green under the drop-Creature revert,
because a superset still contains the pick) is correct reasoning, not an excuse. The failure
message tells a card-def author what to do and in which commit. Only L6 (it doesn't name the other
seeded gates in the same file that will also redden) is open.

---

## Stage 7 (close-out) has not been performed

This is not a finding against the code — it is a status report the runner needs. `memory/primitive-wip.md`
ends at Stage 6 and the tree shows none of Stage 7's five sub-lists done:

| Stage 7 item | status |
|---|---|
| Comment corrections, 7 named sites | **partial** — `local_game.rs`'s accessor + `record_rejection` done; `local_game.rs:371` (M2), `invariants.rs:3-11` (M4), `invariants.rs:463` (M3), `sim5_bot_cast_discipline.rs:39-58`/`:96-106`, `fuzzer.rs:29-61` (L8), `docs/mtg-engine-simulator.md` (M5), `docs/mtg-engine-feedback-engineering.md` §2.3 — all NOT done |
| Seed dispositions in `docs/audits/decision-point-audit.md` §8.1 (`OOS-SIM3-3`/`-4`/`OOS-CARDS2-3` CLOSED, `OOS-SIM3-2` **partial**) | **not done** — no PB-DX32 disposition anywhere in that doc |
| New seeds `OOS-DX32-1..n` (incl. the `player_consistency` 26.8% finding, the 17-row AutoChosen blind spot, the RandomBot-waste-is-bot-design note, T6.3's `surveil` gap, and `--replay`'s empty `command_history`) | **not filed** |
| `CLAUDE.md` delta + tests pin + `memory/workstream-state.md` handoff | **not done** — neither file mentions PB-DX32 |
| `memory/primitives/seed-rerank-2026-08-02.md` untouched | **satisfied** (correctly — coordinator's at collect) |

Discharging Stage 7 would close M2, M3, M4, M5 and L8 as a side effect. M1, M6, M7, M8, and the
remaining LOWs are independent of it.

---

## CR Coverage Check

| CR Rule | Observed correctly? | Tested? | Notes |
|---|---|---|---|
| **514.1** | Yes | T6.2 | `CleanupDiscard → None`, asserted explicitly rather than left unexercised. MCP-confirmed: turn-based action, no stack, no `ROWS` row. |
| **704.3** | Yes, with a citation-phrasing nit | T4.1, T4.2 | The engine's step-entry-only sweep is a *deviation* from 704.3, not an expression of it — see L4. The transience argument itself is sound. |
| **704.5m / 704.5n** | Untouched | — | `attachment_validity` stays HARD and its dispositions were not re-inverted. Correct per plan §8. |
| **500.4** | Yes | T3.1, T3.2, T3.3 | `ManaPoolsEmptied` counted identically by the streaming fold and the journal walk. |
| **601.2c** | Yes | T3.2 (`targeted_casts` parity) | Counted, not re-implemented. |
| **603.3d** | Yes | T6.2 (`triggered_targets`) | |
| **701.9b** | Yes | T6.2 (`discard_cards`) | Correctly the ENG-1 effect row, not CR 514.1. |
| **701.22a / 701.23a / 701.25a** | Yes | T6.2 (scry / search_library / surveil) | `surveil` unreached at T6.3's debug budget, reached 30× at release budget; both recorded. |
| **104.2a / 104.3a** | Yes | T1.1, T2.3 | Concede-driven GameOver used deliberately to force a non-zero parity fixture. |
| **800.4a** | Hypothesis only, not acted on | — | Correct per plan §7 R1/R2: `player_consistency` stays HARD and undiagnosed. |

## Test Review

| test | file | discriminating? | note |
|---|---|---|---|
| T1.1 | `pb_dx32_fuzz_output.rs:237` | Yes | Halted half goes through the real `GameDriver`, checked against an independent shadow `LocalGame`; carries a `total_commands > 0` floor written explicitly against the revert. Good. |
| T2.1 | `:355` | Yes | Non-vacuity is a named Stage-0 measurement, not a hope. |
| T2.2 | `:386` | Yes | The only threshold gate with a properly derived floor. Model for M7's fix. |
| T2.3 | `:432` | Yes | Two purpose-built bots so the parity is checked against a **non-zero** count — a genuinely thoughtful defence against `Default`-value vacuity. |
| T3.1 | `:520` | Yes, weakly | See **M7**. |
| T3.2 | `sim5_bot_cast_discipline.rs:376` | Yes | R8 hit for real, root-caused, fixture built. See **L5** for the one-line strengthening. |
| T3.3 | `:483` | Yes | `OOS-SIM2-1` named at the pin, as criterion (b) requires literally. |
| T4.1 | `pb_dx32_fuzz_output.rs:554` | Yes | Asserts the split in **both** directions plus the end-state property. |
| T4.2 | `:591` | Yes | Paired probe (clean + broken), following `invariants.rs`'s own convention. |
| T4.3 | `:626` | Yes | Hand-built half pins the order guarantee (first occurrence, turn 3 survives); real-seeded half proves the collapse on engine output. |
| T5.1 / T5.2 | `:695` / `:733` | Yes | Exact both directions; T5.2 discriminates a filter divergence T5.1 cannot. |
| T6.1 | `decision_gate.rs:1502` | Yes for line comments; see **M8** for block comments | |
| T6.2 | `pb_dx32_fuzz_output.rs:785` | Yes | The non-vacuity partner T6.1 needs. See **L9** for the overclaiming message. |
| T6.3 | `:883` | Yes, and brittle by design | Exact partition, with a message that says "report this, do not retune". Correct call. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| — | — | — | — | **No card definitions were touched or should have been.** Plan §8 requires an empty `crates/card-defs/` diff and unmoved 1,133/1,803 coverage; the coordinator executed both checks. T5.1 now *pins* the corpus so a future flip announces itself. |

## Previous Findings

Not a re-review. No prior `pb-review-DX32.md` existed.
