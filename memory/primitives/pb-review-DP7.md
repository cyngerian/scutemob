# Primitive Batch Review: PB-DP7 — Cleanup discard is a player choice; the first blocking pending decision

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**Branch**: `feat/pb-dp7-cleanup-discard-command-pilot-for-blocking-pending-de`
**Base**: `1854d3b9` · commits reviewed: `9a4f990c`, `97995583`, `9f983e32`
**CR Rules verified via MCP**: 514.1, 514.2, 514.3, 514.3a, 500.4, 500.5, 703.4n, 703.4q, 402.2, 702.35a, 701.9a/b, 603.3b, 104.3a, **800.4a/800.4j** (not in the plan; decisive for Finding 1)

**Engine files reviewed**: `rules/engine.rs`, `rules/turn_actions.rs`, `rules/command.rs`,
`rules/events.rs`, `rules/protocol.rs`, `rules/priority.rs`, `rules/loop_detection.rs`,
`state/mod.rs`, `state/builder.rs`, `state/hash.rs`, `state/error.rs`,
`card-types/src/state/stubs.rs`, `card-types/src/state/mod.rs`,
`testing/replay_harness.rs`, `testing/script_schema.rs`

**Consumer files reviewed**: `simulator/legal_actions.rs`, `simulator/local_game.rs`,
`simulator/random_bot.rs`, `simulator/heuristic_bot.rs`, `tools/tui/src/play/app.rs`,
`tools/tui/src/play/input.rs`, `tools/replay-viewer/frontend/src/lib/eventFormat.js`

**Card defs reviewed**: 0 edited (as predicted). `fiery_temper.rs`, `stensia_masquerade.rs`,
`markov_baron.rs` are corrected by the engine change alone; T6/T7 pin the behaviour with a
local test-only Fiery Temper def.

## Verdict: needs-fix

The core of the batch is sound and, in the two places that matter most, verifiably so.
I independently re-derived the step/turn-advancement grep on this branch: `state.turn = …`
occurs at exactly five sites (`engine.rs:2053`, `:2058`, `:2204`, `:2210`, `:2300`) inside
three functions (`handle_all_passed`, `enter_step`, `handle_concede`) plus one game-start
assignment (`engine.rs:2818`, `Step::Untap`), and `turn_structure::advance_step/advance_turn`
are pure. `handle_all_passed`'s unreachability while blocked is confirmed by reading
`priority.rs:22-33` (rejects unless `priority_holder == Some(player)`) together with
`handle_pass_priority`'s `state.turn.priority_holder = None` at `engine.rs:1958` and the
fact that `enter_step`'s new guard at `:2102` returns before either priority-granting branch.
`process_command` is the sole admission point. The **progress gate and admission gate are
complete**, wire discipline is clean (both history arrays appended, never edited; all four
fingerprints gate-computed; `GameEvent` arm 129 appended without renumbering;
`#[serde(default)]` present; `HashInto for PendingCleanupDiscard` covers both fields and the
SR-19 `NOT_HASHED` allowlist is still empty), and the plan's §3.1 CR correction is correct
against the rule text ("First…"/"Second…", only 514.2's two items simultaneous). The nine
existing-test edits are faithful — I found **no** assertion weakened to fit the implementation.

What it does **not** survive is the "can the game get stuck?" question for one reachable
state the plan never considered: **an active player who has already lost to an SBA before
their cleanup step**. `blocking_decision`'s liveness filter stops the *engine* hanging, but
nothing clears the field, so the entry becomes permanently stale — CR 514.2 never runs for
that turn (CR 800.4j requires the turn to complete), and all three consumers, which read the
raw field rather than the predicate, pin on a dead player forever. That is Finding 1 (HIGH),
and Finding 2 (HIGH) is its second half: the handler accepts the answer and re-enters
`enter_step` with no check that the step is still `Cleanup`. Three test-validity MEDIUMs
follow (per `memory/conventions.md` these are fix-phase HIGHs), plus a false doc comment that
also makes the plan's §1.5 reuse promise inaccurate as-shipped (ESM criterion 5540).

**Criterion 5540 (a test observes the block)**: satisfied. T1 discriminates hard.
**Criterion 5541 (madness not on an unchosen card)**: satisfied in substance — the runner's
fail-before probe on `1854d3b9` is real evidence — but the committed test rests on an
unasserted premise (Finding 7).

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `rules/turn_actions.rs:1320-1338` | **A dead/conceded active player gets a pending entry that is never cleared.** CR 800.4j: the turn must continue to completion. CR 514.2 is skipped for that turn and every consumer deadlocks. **Fix:** do not record an entry (skip CR 514.1 entirely) when the active player is `has_lost \|\| has_conceded`. |
| 2 | **HIGH** | `rules/engine.rs:410-419` | **`Command::DiscardToHandSize` is accepted outside `Step::Cleanup` and unconditionally re-enters `enter_step`.** Re-runs turn-based actions for whatever step is current. **Fix:** reject unless `state.turn.step == Step::Cleanup`; make the `enter_step` resume conditional on it. |
| 3 | **MEDIUM** | `rules/engine.rs:63-69` | **The `BlockingDecision` doc comment is aspirationally wrong.** "every consult site is written against this enum and not against the field it wraps, so adding a variant needs no new consult site" — three consumer sites read the field, and the admission gate hard-codes the variant. **Fix:** correct the comment and/or export a public predicate (Finding 4). |
| 4 | **MEDIUM** | `simulator/legal_actions.rs:220`, `simulator/local_game.rs:331-334`, `tools/tui/src/play/app.rs:241` | **Consumers read `pending_cleanup_discard()` (raw field); the engine gates on `blocking_decision()` (liveness-filtered).** They can disagree; when they do the simulator returns `[]` for every seat. **Fix:** export a public liveness-aware accessor and rewire all three. |
| 5 | **MEDIUM** | `rules/engine.rs:2232-2243` + `:2293-2305` | **Conceding while blocked skips the rest of CR 514.2 for the abandoned turn** (CR 800.4j). **Fix:** run the remainder of `cleanup_actions` (or re-enter `enter_step` at Cleanup) before the turn advance. |
| 6 | LOW | `rules/turn_actions.rs:1363`, `:1551`, `rules/engine.rs:2094-2101` | **CR miscitations.** Mana-pool emptying is CR **500.5 / 703.4q**, not 500.4; damage removal is CR **514.2**, not 514.1. **Fix:** correct the three citations. |
| 7 | LOW | `rules/engine.rs:410-419` | **The discard's events never pass through `abilities::check_triggers`.** Harmless only because `check_triggers` has no `DiscardedToHandSize` arm. **Fix:** add a `check_and_flush_triggers` call or a comment naming the dependency. |
| 8 | LOW | `rules/turn_actions.rs:1486-1493` | **`has_madness` reads base `characteristics.keywords`, not layer-resolved** (W3-LC pattern). Moved verbatim from the old loop. **Fix:** note or convert; out-of-scope is acceptable. |
| 9 | LOW | `rules/events.rs:1363-1370` | **`CleanupDiscardChoiceRequired` broadcasts the full hand `ObjectId` set** with no privacy marker (Architecture Invariant 7). **Fix:** doc-note that M10's filter must make this event private-to-`player`. |
| 10 | LOW | `rules/command.rs:341-343` | **`cards` order is an accepted-and-discarded input** (DP-24 class). Deliberate and documented (§2.3 / OOS-DP7-4). **Fix:** none — record the explicit answer in the audit's §5/DP-24 check. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| 11 | MEDIUM* | `pb_dp7_cleanup_discard.rs:789-802` `test_dp7_pending_entry_is_hashed` | **Vacuous.** Compares a blocked state to a freshly-built one; they differ in step, priority, `players_passed`, turn state. Passes even if the field were never hashed. |
| 12 | MEDIUM* | `pb_dp7_cleanup_discard.rs:512-520` `test_dp7_answer_validation` case 7 | **Wrong-sender case never reaches the handler's SR-29 check** — the admission gate intercepts it. The test asserts only `is_err()`, so it cannot tell. Case 4 (id in another player's hand) is explicitly skipped. |
| 13 | MEDIUM* | `pb_dp7_cleanup_discard.rs:314-364` `test_dp7_madness_does_not_fire_on_an_unchosen_card` (criterion 5541) | **Discrimination rests on an unasserted premise** — that Fiery Temper holds the highest hand `ObjectId`. If builder id assignment ever changes, the test passes vacuously. |
| 14 | LOW | `pb_dp7_cleanup_discard.rs:723-733` `test_dp7_madness_discard_runs_an_extra_cleanup_round` | The `while … rounds < 10` loop does not assert the CR 514.3a non-advance behaviour its own comment describes. |
| 15 | LOW | `pb_dp7_cleanup_discard.rs:390-419` `test_dp7_madness_fires_on_a_chosen_card` | `assert!(trigger.is_some() \|\| on_stack)` with the cost check only in the `on_stack` branch; "exactly one" is claimed, not asserted. |
| 16 | LOW | `pb_dp7_cleanup_discard.rs:562-621` T9(b)/T9(c) | Do not assert the turn advanced (T9(a) does). |
| 17 | LOW | `simulator/legal_actions.rs:2496-2547` `provider_offers_only_the_discard_while_blocked` | Records the entry by calling `cleanup_actions` at `Step::End`, so the SR-38 "accepted verbatim" check runs in a state the engine cannot produce — and silently exercises Finding 2's missing guard. |
| 18 | LOW | (coverage gap) | No test for a **second** cleanup pause in the same turn (CR 514.3a extra round in which the hand becomes oversized again). |

\* per `memory/conventions.md` § "Test-validity MEDIUMs are fix-phase HIGHs" — these three must
be repaired in the fix phase, not deferred.

---

## Finding Details

### Finding 1 (HIGH) — A dead active player's cleanup entry is recorded and never cleared

**File**: `crates/engine/src/rules/turn_actions.rs:1320-1338` (recording site);
`crates/engine/src/rules/engine.rs:103-117` (`blocking_decision` liveness filter)
**CR**: 800.4j — "If a player leaves the game during their turn, that turn continues to its
completion without an active player." CR 800.4a — "all objects owned by that player leave the
game". CR 514.2 — the damage clear and "until end of turn" expiry are mandatory.

**Issue.** `cleanup_actions` records the entry unconditionally for `state.turn.active_player`,
with no aliveness check:

```rust
if !no_max {
    let hand_size = …;
    if hand_size > max_hand_size {
        state.pending_cleanup_discard = Some(PendingCleanupDiscard { player: active, count });
        events.push(GameEvent::CleanupDiscardChoiceRequired { … });
        return events;                      // <-- CR 514.2 not reached
    }
}
```

The active player *can* be dead here. `sba.rs:265-311` marks `has_lost` without removing the
player or their objects (`state/diagnostics.rs:73-78` confirms players are never removed;
`rg` finds no CR 800.4a object-removal implementation), and `enter_step`'s
has-priority branch already handles "active player lost" by passing priority on
(`engine.rs:2186-2196`) — the turn continues with a dead active player until `advance_turn`.
The most common route is the active player losing to a lethal spell or an empty-library draw
on their own turn while holding 8+ cards.

Consequences, all reachable:

1. **CR 514.2 is skipped for that turn.** `cleanup_actions` returned early, so `clear_damage`,
   the CR 702.171b saddle clear, `expire_end_of_turn_effects` and `CleanupPerformed`
   (`turn_actions.rs:1340-1369`) never run. Marked damage and every "until end of turn" effect
   survive into the next player's turn. CR 800.4j says the turn must complete anyway.
2. **The entry is permanently stale.** `blocking_decision` returns `None` for a dead player, so
   the progress gate at `engine.rs:2102` is off and the game advances — but nothing clears the
   field. `handle_concede` (`:2236-2243`) only clears on *concede*, not on SBA loss.
3. **Every consumer pins on the dead player forever.** `StubProvider::legal_actions`
   (`legal_actions.rs:220-237`) early-returns on the raw field: the dead player gets the discard
   action, **every other player gets `[]`**. `LocalGame::advance` (`local_game.rs:331-334`)
   resolves `acting_player` to the dead player. `PlayApp::acting_player` (`app.rs:241-243`)
   does the same. The `LocalGame` safety valves do not catch this (plan §12.1 says so
   explicitly).
4. If a bot then submits the discard, `process_command` accepts it (the admission gate is off)
   and Finding 2 fires.

**Fix.** In `cleanup_actions`, guard the recording site on the active player being alive:

```rust
let active_is_alive = state.expect_player(active)
    .map(|p| !p.has_lost && !p.has_conceded).unwrap_or(false);
if !no_max && active_is_alive { … }
```

CR 514.1 names "the active player"; a player who has left the game performs no turn-based
action, and under CR 800.4a they have no hand to discard from. With the guard, the function
falls through to CR 514.2 and the turn completes per CR 800.4j. Add a test: active player
marked `has_lost` with a 9-card hand, pass into Cleanup, assert no entry, `DamageCleared`
emitted, `CleanupPerformed` emitted once, turn advances.

### Finding 2 (HIGH) — The answer is accepted out-of-step and re-enters `enter_step` unconditionally

**File**: `crates/engine/src/rules/engine.rs:402-420`;
`crates/engine/src/rules/turn_actions.rs:1413-1531`
**CR**: 514.1 / 703.4n — the discard is performed "immediately after the cleanup step begins".

**Issue.** Neither the handler nor the dispatch arm checks `state.turn.step`. After a
successful discard, `process_command` calls `enter_step(&mut state)?` unconditionally, which
runs `execute_turn_based_actions` for **whatever step is current**. With a stale entry
(Finding 1) the current step can be any later step of a later turn, so this re-executes that
step's turn-based actions — untap, draw, upkeep — a second time.

This is not hypothetical plumbing: the T16 simulator test
(`legal_actions.rs:2496-2547`) records an entry at `Step::End` by calling `cleanup_actions`
directly and then successfully runs `process_command(DiscardToHandSize)`, which re-enters
`enter_step` at `Step::End`. The path is already exercised, just not asserted against.

**Fix.** Add to `handle_discard_to_hand_size`'s validation list (plan §2.4, before any
mutation):

```rust
if state.turn.step != Step::Cleanup {
    return Err(GameStateError::InvalidCommand(
        "cleanup discard is only legal during the cleanup step (CR 514.1)".into()));
}
```

and make the `enter_step` resume in `engine.rs:417` conditional on the same. Update T16 to
drive a real priority round (or assert the rejection) rather than calling `cleanup_actions`
at `Step::End`.

### Finding 3 (MEDIUM) — `BlockingDecision`'s doc comment is aspirationally wrong

**File**: `crates/engine/src/rules/engine.rs:63-69`
**Convention**: `memory/conventions.md` § "Aspirationally-wrong code comments are correctness
hazards" — never leave the aspirational version standing.

The comment says: *"every consult site is written against this enum and not against the field
it wraps, so adding a variant needs no new consult site."* As shipped:

- **True** for the progress gate (`:2102`, `blocking_decision(state).is_some()`).
- **Partly false** for the admission gate (`:139-148`): the allow-list is
  `matches!(&command, Command::DiscardToHandSize { player, .. } if …)` — hard-coded to this
  variant. A DP-8 variant must edit this site to add its own answering command.
- **False** for the three consumer sites (Finding 4), which read
  `state.pending_cleanup_discard()` directly.

This is the same claim the plan's §1.5 makes to a future DP-8/DP-9 planner ("DP-8 adds a
variant, not a site"), so it also answers the ESM criterion 5540 honesty question: **the
mechanism is real and the progress gate genuinely generalises, but §1.5 overstates the reuse
by three consumer sites and one allow-list.** §1.6's DP-9 honesty ("say (a) is the real answer
and budget for it") is accurate and matches the shipped code — nothing about DP-7's resume
mechanism was over-generalised.

**Fix.** Do Finding 4 (which makes the sentence true for the consumers), rewrite the sentence
to name the admission gate's allow-list as the one site a new variant must touch, and append
the same correction to the plan's §1.5 so the DP-8 planner reads the true version.

### Finding 4 (MEDIUM) — Consumers read the field, the engine reads the predicate

**Files**: `crates/simulator/src/legal_actions.rs:220`,
`crates/simulator/src/local_game.rs:331-334`, `tools/tui/src/play/app.rs:241`

`blocking_decision` (`engine.rs:103-117`) filters on liveness; all three consumers read
`state.pending_cleanup_discard()` raw. The plan (§1.2) promised "a public read accessor on
`GameState` for the simulator" and what shipped is an accessor for the *field*, not the
predicate — so the consumers cannot apply the same filter even if they wanted to
(`BlockingDecision` and `blocking_decision` are both `pub(crate)`).

When they disagree — which Finding 1 makes reachable — `StubProvider` returns an empty action
list for **every** seat and `LocalGame` deadlocks or halts. This is precisely the "can the game
get stuck" failure class, and it is the one the brief asked about.

**Fix.** Promote `BlockingDecision` to `pub` and add
`pub fn blocking_decision(&self) -> Option<BlockingDecision>` on `GameState` (read-only, no
`_mut`, SR-3 intact), then rewire `StubProvider::legal_actions`, `LocalGame::advance` and
`PlayApp::acting_player` to it. This is also exactly the surface DP-8/DP-9 need, and it makes
Finding 3's comment true.

### Finding 5 (MEDIUM) — Conceding while blocked abandons CR 514.2

**File**: `crates/engine/src/rules/engine.rs:2232-2243`, `:2293-2305`
**CR**: 800.4j, 514.2

`handle_concede` clears the entry for the conceding player, then (because
`active_player == player`) goes straight to `empty_all_mana_pools` → `advance_turn` →
`reset_turn_state` → `enter_step`. `cleanup_actions` is never re-run for the abandoned turn,
so CR 514.2's damage clear and "until end of turn" expiry never happen for it — marked damage
and until-EOT effects leak into the next player's turn.

Conceding mid-turn already skipped the remainder of the turn before PB-DP7, so this is a
widening rather than a new class; but PB-DP7 makes cleanup a *resting* state where a concede
can land with CR 514.1 half-done, which is new. T4 asserts the entry is cleared and the turn
advances, but asserts nothing about CR 514.2.

**Fix.** In `handle_concede`, when the conceding player is the active player and the step is
`Cleanup`, complete the cleanup turn-based actions before advancing (the simplest correct form
is to clear the entry and call `turn_actions::cleanup_actions` once, which — with Finding 1's
guard in place — will now run straight through). Extend T4 to assert `DamageCleared` and that
an `UntilEndOfTurn` effect registered on the conceded turn is gone.

### Finding 6 (LOW) — CR citations

MCP text: **CR 500.4** = "As a step or phase begins, if there are effects that last until that
step or phase, those effects expire." The mana-pool empty is **CR 500.5** ("As a step or phase
ends … any unspent mana left in a player's mana pool empties"), turn-based action **CR 703.4q**.
Three sites cite 500.4 for the pool: `turn_actions.rs:1363` (pre-existing),
`turn_actions.rs:1533` (`empty_all_mana_pools` doc, pre-existing), and the **new**
progress-gate comment at `engine.rs:2094-2101`. Separately `clear_damage`'s doc
(`turn_actions.rs:1551`) cites CR 514.1 for damage removal, which is CR 514.2 (pre-existing).
**Fix**: correct the new site at minimum; the three pre-existing ones are a free ride-along.

### Finding 11 (MEDIUM, test-validity) — `test_dp7_pending_entry_is_hashed` is vacuous

**File**: `crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs:789-802`

The doc comment says "two states differing only in `pending_cleanup_discard`". The test
compares `advance_to_cleanup_block(state.clone())` against a **freshly built** state, which
differs in `turn.step`, `turn.priority_holder`, `turn.players_passed`, and whatever else the
four passes moved. It would pass unchanged if `self.pending_cleanup_discard.hash_into(…)` were
deleted from `public_state_hash`.

Note that the SR-19 gate `every_hashed_struct_field_is_hashed_or_allowlisted`
(`tests/core/hash_schema.rs:1528`, `NOT_HASHED` empty at `:1252`) already machine-proves both
fields feed the hash — so this test is simultaneously vacuous *and* redundant.

**Fix.** Either (a) delete it and replace with a one-line comment citing the SR-19 gate, or
(b) rewrite so the two states genuinely differ only in the field — e.g. compare the blocked
state's hash against the same state with the field cleared, which requires the public
predicate/accessor from Finding 4 plus a test-only clearing seam. Do not leave a test whose
name asserts a discrimination it does not perform.

### Finding 12 (MEDIUM, test-validity) — validation table does not exercise the SR-29 sender check

**File**: `crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs:427-527`

Case 7 sends `DiscardToHandSize { player: p(2), … }`. The **admission gate** rejects it at
`engine.rs:139-148` with `BlockedByPendingDecision`, because `*player == decision.player()` is
false — so `handle_discard_to_hand_size`'s own sender check (`turn_actions.rs:1421-1426`, the
plan's §2.4 item 3, the SR-29 trust-boundary check) is **never reached by any test**. The test
asserts only `r.is_err()`, which cannot distinguish the two rejections.

Case 4 ("an id from a DIFFERENT player's hand") is explicitly not written — the comment says
it is "covered together with case 6", but case 6 is an *unknown* id, rejected by the
`state.object(id)` lookup, whereas an id in another player's hand resolves fine and is rejected
by the `ZoneId::Hand(player)` membership check. (That check *is* covered by case 5's
battlefield id, so this is a coverage nicety, not a hole.)

**Fix.** Assert the specific error for each case (`InvalidCommand` vs `ObjectNotFound` vs
`ObjectNotInZone` vs `BlockedByPendingDecision`). That will immediately surface case 7's real
provenance; then add a genuine handler-level sender case (give p(2) a hand card and pass it as
one of p(1)'s ids for the zone check, and reach the sender check via a direct
`handle_discard_to_hand_size` call or after the Finding 4 accessor lands).

### Finding 13 (MEDIUM, test-validity) — criterion 5541's test rests on an unasserted premise

**File**: `crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs:314-364`

T6 discriminates only if Fiery Temper holds the **highest** `ObjectId` in P1's hand — that is
what makes it the pre-fix auto-pick target. `build_oversized_hand` adds it last and the doc
comment asserts the property in prose, but no assertion checks it. If `GameStateBuilder`'s id
assignment ever changes, T6 silently stops testing anything (the old engine would also have
left Temper in hand).

The runner's fail-before probe on `1854d3b9` did observe Temper being exiled, so the *claim* is
currently true and criterion 5541 is genuinely met — this is about the committed test's
durability, not about whether the fix works.

**Fix.** One line before the answer:
`assert_eq!(temper_id, *hand_ids.iter().max().unwrap(), "T6 only discriminates if Temper is the pre-fix auto-pick target");`
(T5 already self-guards this way by asserting the two highest ids survive — copy that pattern.)

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 514.1 (first turn-based action, discard) | Yes | Yes | T1/T5/T12; player choice honoured per CR 701.9b |
| 514.2 (second, simultaneous) | Yes, **with two holes** | Partly | T10 pins the deferral; Findings 1 and 5 skip it entirely on two paths |
| 514.3 (no priority in cleanup) | Yes | Yes | T2/T3 — admission gate; `priority_holder == None` asserted in T1 |
| 514.3a (extra cleanup rounds) | Yes | Yes | T11; pause consumes 0 rounds, madness costs 1 |
| 500.5 / 703.4q (pool empties) | Yes (pre-existing) | n/a | Miscited as 500.4 — Finding 6 |
| 703.4n (immediately after step begins) | Yes | Indirect | Pause point is at the top of `cleanup_actions` |
| 402.2 (maximum hand size) | Yes | Yes | T9 ×3 — printed keyword, layer-granted, persistent designation |
| 702.35a (madness) | Yes | Yes | T6 (negative, criterion 5541) / T7 (positive) + 4 repaired `madness.rs` tests |
| 701.9b (affected player chooses) | Yes | Yes | T5 |
| 603.3b (trigger ordering) | Deliberately deferred | n/a | Ascending-id sort; OOS-DP7-4 |
| 104.3a (concede any time) | Yes | Yes | T4 |
| **800.4a / 800.4j** | **No** | **No** | Not in the plan. 800.4a object removal is a pre-existing gap; **800.4j is newly violated** — Findings 1 and 5 |

## Wire Discipline (SR-8 / SR-27)

| Check | Result |
|---|---|
| `PROTOCOL_VERSION` 27 → 28 (`protocol.rs:268`) | PASS |
| `- 28:` History line appended above the const (`:260-267`) | PASS |
| `ProtocolEpoch { version: 28 }` **appended** at `:501-507`, v26/v27 rows untouched | PASS |
| `PROTOCOL_SCHEMA_FINGERPRINT` == the new epoch's fingerprint | PASS (`bf5f5dd…`) |
| `HASH_SCHEMA_VERSION` 64 → 65 (`hash.rs:607`) | PASS |
| `- 65:` History line appended (`:591-606`) | PASS |
| `HashSchemaEpoch { version: 65 }` appended at `:917-926`, v64 untouched | PASS |
| `GameEvent` hashing arm 129, appended after `RemovedFromCombat = 128`, no renumbering, no `_` arm | PASS (`hash.rs:5341-5351`) |
| `impl HashInto for PendingCleanupDiscard` feeds **both** fields | PASS (`hash.rs:3003-3008`) |
| Folded into `public_state_hash` | PASS (`hash.rs:7786`) |
| Mirrored into `loop_detection.rs` mandatory-state fingerprint | PASS (`:151-156`) |
| `#[serde(default)]` on `GameState::pending_cleanup_discard` | PASS (`state/mod.rs:151`) |
| Read accessor, **no** `_mut` accessor (SR-3) | PASS (`state/mod.rs:459-465`) |
| `builder.rs` init `None` | PASS (`:322`) |
| `NOT_HASHED` allowlist still empty | PASS (`hash_schema.rs:1252`) |
| ~50 scattered per-PB sentinels re-pinned to 28/65 | PASS — and OOS-DP7-8 is a fair complaint |
| `GameStateError::BlockedByPendingDecision` outside the wire closure | PASS — `GameStateError` is unreachable from `Command`/`GameEvent`/`ReplayLog` |

## Consumer Coverage

| Consumer | Covered? | Ordering constraint honoured? | Notes |
|---|---|---|---|
| `StubProvider::legal_actions` | Yes (`:210-237`) | **Yes — first**, above commander-zone and above the `priority_holder != Some(player)` early return | Reads the raw field (Finding 4) |
| `LocalGame::advance` | Yes (`:324-334`) | **Yes — first** in the acting-player chain | Reads the raw field (Finding 4) |
| `DecisionKind` | Yes, `CleanupDiscard` + `#[non_exhaustive]` (audit §9.4 rec 1) | n/a | Done as planned |
| `random_bot::action_to_command` | Yes (`:365-371`) | n/a | Compile-forced |
| `heuristic_bot` scorer | Yes (`:107-111`, scores 100) | n/a | Compile-forced |
| `replay_harness::translate_player_action` | Yes (`:903-917`) | n/a | `discard_cards` trailing param, 5 call sites updated |
| `script_schema::PlayerAction` | Yes (`:471-478`) | n/a | `#[serde(default)] discard_cards` under `deny_unknown_fields` |
| `tools/tui` `acting_player` | Yes (`app.rs:235-243`) | **Yes — first** | Reads the raw field (Finding 4) |
| `tools/tui` event formatter | Yes (`app.rs:603-608`) | n/a | Below the `_ =>` catch-all, so opt-in |
| `tools/tui` input | Yes — `'d'` key (`input.rs:43-62`) | n/a | Deterministic default only; OOS-DP7-6 |
| `replay-viewer` `eventFormat.js` | **Yes — verified by reading** (`:60-61` display, `:438` category) | n/a | No compile gate; correctly caught |
| `replay-viewer` `view_model.rs` | Not touched | n/a | `StackObjectKind`/`KeywordAbility` unmoved, as predicted |
| `GameDriver::run_game` | Via `LocalGame` | n/a | T15 covers the bot path |

## Stuck-State Analysis (brief item 2)

| Scenario | Outcome | Verdict |
|---|---|---|
| Blocked player never answers | Game waits indefinitely | **By design** — the block is a block, not a deadline; §1.1's argument (CR 514.1 has no CR-supplied default, unlike CR 118.12a) is sound and every consumer is taught to offer the answer |
| Blocked player concedes | Entry cleared (`:2236-2243`), `priority_holder` is `None` so `handle_all_passed` is not called, turn advances | Progress OK; **CR 514.2 skipped — Finding 5** |
| Another player concedes while blocked | Entry untouched, no advance, block persists | Correct |
| Another player concedes making the game over | `GameAlreadyOver` thereafter; stale entry harmless | Correct |
| Active player **already lost to an SBA** | Entry recorded for a dead player, `blocking_decision` returns `None`, field never cleared | **BROKEN — Finding 1** |
| Last player standing | `is_game_over` is checked before both gates | Correct |
| CR 514.3a extra rounds | Pause consumes 0 rounds; `MAX_CLEANUP_SBA_ROUNDS = 100` untouched; a fresh oversized hand in round *n* re-pauses correctly (a new cleanup step begins per CR 514.3a, so CR 514.1 legitimately re-applies) | Correct; untested (Finding 18) |
| Gate placement in `enter_step` | After `events.extend(action_events)` (`:2087`), after `is_game_over` (`:2089-2093`), before the CR 514.3a block (`:2110`) | All three load-bearing conditions from plan §12.3 verified |

## Plan §1.5 / §1.6 Reuse Honesty (ESM criterion 5540)

**§1.6 (DP-9): honest.** "Say (a) is the real answer and budget for it" is accurate; nothing
in the shipped code implies DP-9 is plumbing, and the DP-7 resume genuinely depends on
`cleanup_actions`'s idempotence-at-max-hand-size, which the code and its comment both state.

**§1.5 (DP-8): overstated, and the overstatement was copied into the source.** The progress
gate (one line, enum-typed) does transfer verbatim. The admission gate's allow-list and the
three consumer sites do not — see Finding 3. §1.5's "does not inherit" list (cardinality,
location, partial-flush resumption) is correct and valuable, and the `flush_pending_triggers`
call-site re-derivation instruction is exactly right. **Fix**: amend §1.5 and the source
comment together (Findings 3 + 4).

## Runner Deviations Assessed

| Deviation | Assessment |
|---|---|
| `BlockedByPendingDecision { decision: String }` instead of the enum | **Correct call.** `state/error.rs` must not depend on `rules::engine`. Wire-neutral either way. |
| T18 rewritten as two struct-level tests | **Correct call.** `GameState` genuinely cannot round-trip through `serde_json` (non-string map keys). Honestly documented in the test's own doc comment. The stand-in struct proves the serde mechanism, not that `GameState` carries it — but `state/mod.rs:151` is a one-line visual check. |
| T14/T15 moved to `crates/simulator/tests/local_game.rs` | **Correct call.** Established location, 10 precedent tests. |
| T15 driven off CR 103.8a's 2-player first-turn draw-skip | Acceptable and well-documented; the assertion targets the real regression class (`EngineError`/`NoLegalActions`), not "never halts". |
| `expect_object` → `object()` SR-4 fix found writing T8 | **Genuine defect caught by the right mechanism.** No siblings found: the other `expect_object` uses in the handler (`:1483`, `:1487`) run *after* the ids are validated, so the invariant genuinely holds there. |
| Fuzzer A/B not completed (OOS-DP3-9) | Honest, and correct per the plan's explicit instruction. T15 covers the specific regression. |
| `turn_structure.rs::test_ten_full_turn_cycles` un-enumerated fallout | Repaired with an `answer_pending_cleanup_discard` helper that panics if nothing is pending — the right shape (it does not mask a genuine `priority_holder == None` bug). |

## Recommended Disposition

1. **Findings 1, 2** — fix before collect. Both are reachable illegal game states.
2. **Findings 3, 4** — fix together in one change (export the predicate, rewire the three
   consumers, correct the comment and plan §1.5). This is also the DP-8 prerequisite.
3. **Finding 5** — fix, or file as a seed with the CR 800.4j citation if the coordinator
   judges the concede-abandons-turn shape to be pre-existing scope.
4. **Findings 11, 12, 13** — fix in the fix phase (test-validity → fix-phase HIGH per
   `memory/conventions.md`).
5. **Findings 6-10, 14-18** — fix opportunistically or file as seeds. Finding 6's new-site
   miscitation and Finding 9's privacy doc-note are one-liners worth taking now.
6. New seeds suggested from this review, in addition to the runner's OOS-DP7-1..8:
   - **OOS-DP7-9** — CR 800.4a object removal on player loss is unimplemented; a dead player's
     hand, library and graveyard persist. Root cause of Finding 1's reachability and a broader
     multiplayer-correctness gap.
   - **OOS-DP7-10** — a cleanup discard emits `DiscardedToHandSize`, which `check_triggers`
     has no arm for, so "whenever you discard a card" abilities (Waste Not, Bone Miser,
     Containment Construct) never fire off CR 514.1. Pre-existing; Finding 7 makes it visible.
