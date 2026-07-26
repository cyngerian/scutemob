# Primitive WIP — PB-DP7 (DP-3: cleanup discard has no `Command`) · PLAN

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP6 (DP-15: intervening-if not checked at queue time) — SHIPPED**
> `scutemob-154`, merge `d52fe5b6`, PROTOCOL 27 / HASH 64 unmoved, tests **3,809**.
> Its record lives in `docs/audits/decision-point-audit.md` §4.8/§5 DP-15/§8 + §8.1
> (OOS-DP6-1..10), `memory/primitives/pb-plan-DP6.md` + `pb-review-DP6.md`, and the
> CLAUDE.md changelog entry.

- **PB**: PB-DP7 — **DP-3** (CR **514.1**). The cleanup-step discard-to-hand-size has **no
  `Command` at all**. `rules/turn_actions.rs::cleanup_actions` loops, takes
  `zone.object_ids().last()` — the **highest `ObjectId`**, i.e. the most recently drawn card —
  and discards it, one at a time, with no player input. Madness (CR 702.35a) is honoured on
  that path, so the auto-picker can **involuntarily exile-and-Madness a card the player would
  never have chosen**.
- **Task**: `scutemob-155`
- **Branch**: `feat/pb-dp7-cleanup-discard-command-pilot-for-blocking-pending-de`
- **Class**: CORRECTNESS (Tier 0, class **B**). Rank 7 of the PB-DP suite, and the **first
  wire change** of the suite.
- **Phase**: implement
- **Binding spec**: `docs/audits/decision-point-audit.md`
  - §4.11 table, **line 400** — the "Hand-size discard | 514.1 | **B**" row
  - §5 **line 449** (DP-3 row) — the finding proper, with both cited sites
  - §8 **line 602** (PB-DP7 row) — *"new `Command` ⇒ PROTOCOL bump … Smallest possible pilot
    for the pending-decision pattern: one player, one list, one moment"*
  - §8 **sequencing note** (lines ~606-614) — **the design mandate**, see below
  - §9.3 (**line ~744**) — "the engine already has the right pattern — it just doesn't block"
  - §9.4 recs 5 and 8 — the DTO shape and the "engine chose this for you" annotation
  - §10 — after this lands, §3.1's 277-def sweep must be re-derived (coordinator's job)
  - §8.1 — where new seeds get filed
- **Plan file**: `memory/primitives/pb-plan-DP7.md`
- **Review file**: `memory/primitives/pb-review-DP7.md`

## Acceptance criteria (ESM `scutemob-155`)

1. (5539) Cleanup discard over hand size is a player choice via a new `Command`; the engine
   **blocks** (does not advance past cleanup) until answered for a human seat; bot seats
   auto-answer via `LegalActionProvider`; tests cite **CR 514.1**.
2. (5540) The blocking pending-decision mechanism is **designed and documented for reuse by
   PB-DP8/DP9** (a dedicated plan-file section) and **genuinely gates step advancement** —
   verified by a test that passes priority / advances and observes the block.
3. (5541) Madness triggers **only** off the chosen discard; test citing **CR 702.35**.
4. (5542) `PROTOCOL_VERSION` bumped with fingerprint re-pin per the SR-8/SR-27 gates; HASH
   bump if applicable; replay-viewer + TUI exhaustive matches updated (`cargo build
   --workspace` clean).
5. (5543) `cargo test --all`, clippy, `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
   clean; audit DP-3 row + PB-DP7 row updated.

## The design mandate (audit §8 sequencing note — read this twice)

> "PB-DP7..PB-DP9 are a coherent block that should be planned together, because they all need
> the same missing machinery: **a pending-decision that actually *gates* progress**. The engine
> already has that machinery seven times over (`pending_commander_zone_choices`,
> `pending_zone_changes`, the three payment vectors, `DredgeChoiceRequired`,
> `MiracleRevealChoiceRequired`) — but §4.4 and §4.11 show that **only `pending_zone_changes`
> genuinely blocks**. The generalisable design work is *'make a pending decision actually
> block'*, not *'add another pending vector'*."

So the *deliverable that outlives this PB* is the gate, not the discard. The plan must contain
a **§ "The blocking pending-decision mechanism"** that:

- names the one place (or smallest set of places) a gate must be consulted so that a pending
  decision cannot be stepped over — and argues why that set is complete (`handle_all_passed`'s
  advance branch, `enter_step`'s cleanup auto-advance fall-through, `enter_step`'s
  `has_priority()` branch, `turn_structure::advance_step`/`advance_turn`, and any
  `execute_turn_based_actions` caller are the obvious candidates — **derive the real set,
  don't trust this list**);
- states explicitly how **PB-DP8** (trigger targets, CR 603.3d, at `flush_pending_triggers`
  time) and **PB-DP9** (search/scry/surveil, mid-resolution) would reuse it, and what about
  each of those is *not* covered by the shape chosen here. PB-DP9 in particular pauses
  **inside an `Effect` resolution**, which is a strictly harder suspension problem
  (cf. **OOS-DP5-5**, §8.1) — say so honestly rather than over-claiming reuse;
- decides whether the gate is one generic `pending_decisions` notion or a per-kind vector with
  a shared `fn is_blocked(state) -> bool` predicate, **with the argument**, and notes the hash
  consequence of each;
- specifies who is *allowed* to act while blocked (nothing? only the answering command? mana
  abilities?) and what `process_command` does with an unrelated command that arrives while a
  decision is outstanding — reject with a distinguishable error, or ignore. CR 514.3 says there
  is **no priority in cleanup**, so this block is *not* a priority round.

## Hard constraints

1. **Wire change is EXPECTED and authorised here** — one new `Command` variant (and at least
   one new `GameEvent`) ⇒ **`PROTOCOL_VERSION` 27 → 28** with a `PROTOCOL_SCHEMA_HISTORY` row
   **appended** (never edit an existing row) and `PROTOCOL_SCHEMA_FINGERPRINT` re-pinned to the
   gate-computed value. If pending state lands on `GameState`, **`HASH_SCHEMA_VERSION` 64 → 65**
   the same way. **Never hand-invent a fingerprint** — take the value the failing gate test
   prints. (PB-DP5 is the worked precedent for the HASH half; read what it did.)
2. **`GameState` stays sealed** (Architecture Invariant #3 / SR-3): new pending state is
   `pub(crate)` with a read accessor, and mutation happens only through `process_command`.
3. **SR-8 wire closure**: adding a `Command`/`GameEvent` variant means `state/hash.rs`'s
   event-hashing match and every other exhaustive match must gain an arm. `cargo build
   --workspace` after every phase — `tools/replay-viewer/src/view_model.rs` and
   `tools/tui/src/play/panels/` are the two that runners miss ~50% of the time.
4. **`crates/engine/src/testing/replay_harness.rs`** parses commands from JSON scripts
   (see the `ChooseDredge` arm at `:902`). SR-9c: a golden-script assertion path that is not
   implemented must not silently skip. If cleanup discard now needs an answering command in
   scripts, the harness needs the arm **and** a decision about what existing scripts do.
5. **Determinism (SR-9b / the fuzzer)**: the bot/auto-answer fallback must be deterministic and
   should preserve today's pick (highest `ObjectId`) so `build_initial_state`-driven
   comparisons and the fuzzer baseline don't churn. Say so in the plan and defend it.
6. **Do not break the CR 514.3a extra-cleanup-round machinery** (`enter_step`'s
   `MAX_CLEANUP_SBA_ROUNDS` loop and `handle_all_passed`'s non-advance branch). The new gate
   sits alongside it; the plan must show the interleaving of "discard pending" with "another
   SBA round" and with `cleanup_sba_rounds`.
7. **CR 514.1 simultaneity.** The discard, the damage clear, the "until end of turn" expiry and
   the mana-pool empty are *one* turn-based action performed simultaneously. Pausing in the
   middle of it is a modelling choice — the plan must state where the pause is taken, which of
   the four sub-actions have already happened when it is taken, and why that ordering is the
   least-wrong. Whichever way it goes, **`expire_end_of_turn_effects` must not run before the
   discard choice is made** if the discard can be affected by an effect that is about to expire
   (e.g. a hand-size modifier) — verify, don't assume.
8. **Multiple discards.** Hand size 10 vs max 7 = **three** discards. Decide up front whether
   the `Command` carries one card or the whole subset (audit §9.4 rec 5 says a cleanup discard
   is *"a subset, not an index into an action list"*), and whether the engine asks once or N
   times. State the CR basis (CR 514.1 discards them simultaneously as part of the same
   turn-based action — one subset is the CR-faithful shape) and the consequence for Madness
   (multiple madness cards discarded at once ⇒ multiple triggers, ordered by APNAP/controller
   choice — check CR 603.3b).
9. **`no_max_hand_size`** (CR 402.2, `Thought Vessel`/`Reliquary Tower`, and the persistent
   `no_max_hand_size_permanent` designation) short-circuits the whole thing — the recompute at
   the top of `cleanup_actions` must stay ahead of any pending-decision emission, or a
   Reliquary Tower player gets asked a question they should never see.

## Coordinator pre-survey (a hypothesis for the planner to **falsify**, not a fact base)

> PB-DP3/DP4/DP5/DP6's wip files all recorded pre-survey bullets that were wrong in *both*
> directions (PB-DP3's yield went 3 → 40; PB-DP5 had a third emit site the audit never named;
> PB-DP6's roster was 14 sites, not 3, and two paths the audit credited as *already correct*
> were both carrying their own defects). Verify every line below against the source **as it
> exists on this branch**, and record in the plan which bullets turned out to be wrong.
> Line numbers below were read on this branch today but drift with every edit — re-derive.

### A. The site

- `crates/engine/src/rules/turn_actions.rs::cleanup_actions` — around **`:1263`** (the audit's
  `:1280-1293` is the discard `loop` inside it). Order of business inside the function today:
  1. CR 402.2 `no_max_hand_size` recompute (layer-resolved, PB-AC8; OR'd with
     `no_max_hand_size_permanent`, PB-AC9);
  2. the discard `loop` — `obj_ids.last()`, `expect_move_object_to_zone`, emits
     `GameEvent::DiscardedToHandSize { player, object_id, zone_from, zone_to }`, and on a
     Madness card retargets the destination to `ZoneId::Exile` and pushes a
     `PendingTrigger { data: Some(TriggerData::Madness { exiled_card, cost }), .. }`;
  3. `clear_damage` + `GameEvent::DamageCleared`;
  4. CR 702.171b saddle clear;
  5. `layers::expire_end_of_turn_effects` (CR 514.2);
  6. `empty_all_mana_pools` (CR 500.4) — normally a no-op by this point;
  7. `GameEvent::CleanupPerformed`.
- Reached from `turn_actions.rs:29` (`Step::Cleanup => Ok(cleanup_actions(state))`) inside
  `execute_turn_based_actions`, which `rules/engine.rs::enter_step` calls at the top of its
  loop.
- `crates/card-types/src/state/zone.rs:130-135` — `object_ids()` on an `Unordered`
  (`OrdSet`) hand yields **ascending**, so `.last()` is the highest id. Confirm.

### B. Where the gate has to bite (candidate set — derive the real one)

`crates/engine/src/rules/engine.rs`:
- `handle_all_passed`, around **`:1954-1975`** — the `if state.turn.step != Step::Cleanup`
  advance branch (this is the CR 514.3a non-advance guard).
- `enter_step`, around **`:1979-2060`** — the cleanup SBA-round block (`MAX_CLEANUP_SBA_ROUNDS
  = 100`, `state.turn.cleanup_sba_rounds`), and the **fall-through to auto-advance** when no
  SBAs fired. That fall-through is the path that would step over an unanswered discard.
- `enter_step`'s `if state.turn.step.has_priority()` branch — cleanup has **no** priority
  (`state/turn.rs:63-65`), which is exactly why this block cannot be modelled as a priority
  round and needs its own gate.
- `rules/turn_structure.rs::advance_step` / `advance_turn`.
- Anything in `crates/simulator` that drives steps (`GameDriver`, `LocalGame::advance`).

### C. Wire surface

- `crates/engine/src/rules/command.rs:19` — the `Command` enum. `ChooseDredge` at **`:299-305`**
  is the closest existing shape (`{ player, card: Option<ObjectId> }`) and its doc comment says
  *"Sent in response to a `DredgeChoiceRequired` event"*. There is currently **no** `Discard*`
  variant of any kind in this file — confirm with a grep.
- `crates/engine/src/rules/events.rs:63` — `GameEvent`. `DredgeChoiceRequired` at **`:848-850`**
  is the template; note its doc claims *"the engine pauses"* — **check whether it actually
  does**, because §4.11/§9.3 says only `pending_zone_changes` blocks. If `ChooseDredge`'s
  "pause" is also fictional, that is a finding worth recording (and possibly a seed) even
  though fixing it is out of scope.
- `crates/engine/src/rules/protocol.rs:260` — `PROTOCOL_VERSION = 27`; `:277` the fingerprint;
  `:294+` the append-only `PROTOCOL_SCHEMA_HISTORY`, whose doc comment at `:318-321` spells out
  the bump procedure. **Follow that comment literally.**
- `crates/engine/src/state/hash.rs:591` — `HASH_SCHEMA_VERSION = 64`; the `GameEvent` hashing
  match around `:4806` (`DredgeChoiceRequired` is discriminant 72 — new events append, they do
  not renumber); `:7736` is where `pending_zone_changes` is folded into the state hash, i.e.
  the pattern any new pending vector follows.
- `crates/engine/src/state/mod.rs:138-155` and `:275-301` — the existing pending vectors, their
  `pub(crate)` declarations, and the read/`_mut` accessor pattern (`:439-441`, `:725-727`).

### D. Consumers that must not be forgotten

- `crates/simulator/src/legal_actions.rs:16` — the `LegalAction` enum (simulator-internal, **not**
  a wire type). PB-DP4's precedent: three new `LegalAction`s arrived inside the **existing**
  `PendingDecision` with **no** new `DecisionKind` and **no** `local_game.rs` edit. That will
  **not** work here, because cleanup has no priority window — this decision is genuinely
  out-of-band, so expect to need a `DecisionKind` variant and a `LocalGame` path. Audit §9.4
  rec 1 asks for `DecisionKind` to become `#[non_exhaustive]`; doing that here is cheap and in
  the spirit of the mandate.
- `crates/simulator/src/local_game.rs` — S1's `advance()` / `AwaitingHuman` / `submit(seq, …)`.
  `advance()` must be **idempotent** while the discard is outstanding (S1's review fixed exactly
  that hazard) and must not auto-pass a human seat past it.
- `crates/engine/src/testing/replay_harness.rs:902` region — script command parsing.
- `tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/` — exhaustive matches.
- The fuzzer (`mtg-fuzzer`) — note **OOS-DP3-9**: it already aborts on a stack overflow at ~15
  games on `main`. Don't chase that here; don't let it mask a real regression either.

### E. Existing tests that encode today's wrong behaviour

These assert the auto-pick **by name** and are near-certain fallout:
- `crates/engine/tests/mechanics_m_z/madness.rs:292`, `:354`, `:575` — all three literally say
  *"`cleanup_actions` uses `obj_ids.last()` … so <card> gets discarded"*. Each is evidence the
  fix is real; each change must be justified against CR 514.1/702.35 in the plan, **not**
  adjusted to fit.
- `crates/engine/tests/primitives/pb_ac9_wheel_and_misc.rs:481/:512/:567` and
  `pb_ac8_restrictions_and_wingame.rs:938` call `cleanup_actions` **directly** — a signature or
  contract change here breaks them; they are also the cheapest place to pin the new behaviour.
- Golden scripts: grep `test-data/generated-scripts/` for cleanup/discard-to-hand-size
  scenarios before assuming there are none.

### F. Yield calibration

Per `feedback_pb_yield_calibration`, discount any card-yield estimate 2–3×. **The honest
prediction for this PB is 0 completeness flips** — nothing in the corpus is `known_wrong`
*because of* the cleanup discard; the yield is a Tier-0 engine-agency fix plus the reusable
gate. Predict the flip count explicitly and be prepared for it to be **0**. If the plan finds
`Complete` defs that are live-wrong today because of this path (a Madness card that gets
involuntarily exiled is the candidate class — check the corpus for Madness defs), say so with
names.

## Out of scope — file as seeds in the plan's seed section, do not fix here

- **DP-16** (sacrifice picks) and **DP-13** (combat damage assignment / `OrderBlockers`, the
  only decision in the audit with no `LegalAction`) — same *class*, different PBs.
- **DP-6 / PB-DP8** (trigger targets) and **DP-7/8/9 / PB-DP9** (search/scry/surveil) — this PB
  must *design for* them, not implement them.
- **Legend-rule "which to keep"** (§4.11, CR 704.5j, `rules/sba.rs:960-965`, MR-SR29-01) — the
  other class-B row in the same table. Tempting; not this PB.
- **CR 502.2 "you may choose not to untap"** (§4.11, class D) — not this PB.
- Retrofitting the other six non-blocking pending mechanisms onto the new gate. If the gate is
  right, that retrofit is a follow-up PB — **seed it, with the list**.
- The audit §3.1 re-derivation of the 277 figure (§10 re-audit trigger) — the coordinator's job
  after this merges; just note it is now due.

## Plan phase output required

`memory/primitives/pb-plan-DP7.md` containing:

1. **The blocking pending-decision mechanism** (criterion 5540) — its own section, written for
   PB-DP8/DP9 to read: the gate predicate, every consult site with a line number, the argument
   that the set is complete, the reuse analysis for DP8 and DP9, and what it deliberately does
   *not* cover.
2. The `Command` + `GameEvent` shape, with the one-card-vs-subset decision (constraint 8)
   argued from CR 514.1, and the validation list (owner, hand membership, exact count,
   duplicates, unknown ids).
3. Where in `cleanup_actions` the pause is taken and what has/hasn't happened at that point
   (constraint 7), with the CR argument.
4. The full consult-site inventory in `engine.rs`/`turn_structure.rs`/`simulator` **with line
   numbers as they exist on this branch**, partitioned into must-gate / already-safe /
   out-of-scope.
5. The exact `PROTOCOL`/`HASH` expectation and **what would falsify it** (e.g. "if the pending
   entry can live outside `GameState`, HASH stays 64").
6. The bot/fuzzer auto-answer rule and its determinism argument (constraint 5).
7. The `LegalAction` / `DecisionKind` / `LocalGame` plumbing, and whether `DecisionKind` becomes
   `#[non_exhaustive]` here (audit §9.4 rec 1).
8. The test list with **per-test fail-before predictions**, including a test that *observes the
   block* (criterion 5540) and one that proves Madness no longer fires on an unchosen card
   (criterion 5541), plus every existing test predicted to change and why each change is
   CR-justified.
9. An explicit list of every pre-survey bullet above that turned out to be **wrong**.
10. A seed list for the out-of-scope items.

---

## Implementation complete (runner close-out)

**Branch**: `feat/pb-dp7-cleanup-discard-command-pilot-for-blocking-pending-de`
**Commits**: `5cf24dd6` (plan, pre-existing), `9a4f990c` (phases 1-3: engine core, wire
bump, consumers, 9 existing-test fixes), `97995583` (phase 4: 21 new tests).
Base commit: `1854d3b9` (PB-DP6 collected, 3,809 tests).

### Phase summaries

**Phase 1 — engine core** (`9a4f990c`):
- `Command::DiscardToHandSize { player, cards: Vec<ObjectId> }` —
  `crates/engine/src/rules/command.rs` (after `ChooseMiracle`).
- `GameEvent::CleanupDiscardChoiceRequired { player, count: u32, hand: Vec<ObjectId> }` —
  `crates/engine/src/rules/events.rs` (end of enum, discriminant 129).
- `GameState.pending_cleanup_discard: Option<PendingCleanupDiscard>` — `pub(crate)`,
  `#[serde(default)]`, read accessor `pending_cleanup_discard()`, **no** `_mut` accessor
  (SR-3). New struct `PendingCleanupDiscard { player, count }` in
  `crates/card-types/src/state/stubs.rs`, re-exported through both
  `card-types/src/state/mod.rs` and `engine/src/state/mod.rs`.
- `rules::engine::BlockingDecision` (`CleanupDiscard { player, count }`, `Display` impl for
  the error message) + `blocking_decision(&GameState) -> Option<BlockingDecision>` — treats a
  dead entry-player (`has_lost || has_conceded`) as absent.
- Progress gate: `enter_step` (`engine.rs`), inserted after the `is_game_over` check and
  before the CR 514.3a cleanup-SBA block — `if blocking_decision(state).is_some() { return
  Ok(events); }`.
- Admission gate: `process_command` (`engine.rs`), inserted right after the existing
  `is_game_over` check and before `match command` — rejects everything except the named
  player's `DiscardToHandSize` and anyone's `Concede`.
- Clear-on-concede: `handle_concede` (`engine.rs`), clears a stale entry belonging to the
  conceding player before the rest of the concede logic runs.
- `cleanup_actions` (`turn_actions.rs`) now pauses: after the CR 402.2 no-max recompute, if
  `hand_size > max_hand_size`, it records the entry, emits the one
  `CleanupDiscardChoiceRequired` event, and returns immediately — the old auto-discard `loop`
  is gone entirely (its madness-exile logic moved into the new handler below).
- `turn_actions::handle_discard_to_hand_size(state, player, cards)` — the §2.4 validation
  list (pending exists, sender matches, exact count, no duplicates, every id exists and is in
  the sender's own hand, a `debug_assert_eq!` re-derivation), discards in **ascending
  `ObjectId` order** regardless of the order supplied (plan §2.3), performs the same
  madness-exile-and-queue logic the old loop had, clears the entry.
- `turn_actions::default_cleanup_discard(state, player) -> Vec<ObjectId>` — the `count`
  highest ids ascending; the engine itself never calls it (bots/harness/TUI only).
- `GameStateError::BlockedByPendingDecision { player, decision: String }` —
  **deviation from the plan**: the plan's §1.4 sketch used `decision: BlockingDecision`
  (the enum itself); I used `decision: String` (the `Display` rendering) instead, because
  `state/error.rs` is data and does not depend on `rules::engine` (module direction
  convention) — embedding the enum type would have created exactly the reverse dependency
  the codebase's `state`/`rules` split avoids. Not part of the SR-8 wire closure either way.
- Hash: `impl HashInto for PendingCleanupDiscard` (both fields), folded into
  `public_state_hash` via `self.pending_cleanup_discard.hash_into(&mut hasher)` (blanket
  `Option<T>` impl), and mirrored into `rules/loop_detection.rs`'s mandatory-state
  fingerprint (defensive — `blocking_decision` already prevents SBAs from running while
  blocked, so that fingerprint is never actually computed in the `Some` state, but the plan
  asked for the mirror and it costs nothing).
- `GameEvent` hashing match in `state/hash.rs` gained the `129u8` arm for
  `CleanupDiscardChoiceRequired` (no `_` arm exists there, so a miss would have been a
  compile error).

**Phase 2 — wire bump** (`9a4f990c`): `PROTOCOL_VERSION` 27→28, `HASH_SCHEMA_VERSION` 64→65.
Both `PROTOCOL_HISTORY`/`HASH_SCHEMA_HISTORY` rows **appended**, never edited. All four
fingerprints (`PROTOCOL_SCHEMA_FINGERPRINT`, the new `ProtocolEpoch.fingerprint`, and both
`HashSchemaEpoch.decl_fingerprint`/`stream_fingerprint`) were read **verbatim from the
failing gate tests' panic text**, never hand-invented, per the hard constraint. Both frozen
prefix digests (`protocol_schema.rs::FROZEN_HISTORY_PREFIX_DIGEST`,
`hash_schema.rs::FROZEN_HISTORY_PREFIX_DIGEST`) were re-pinned the same way. All 4 gate tests
(`protocol_schema_fingerprint_is_pinned`, `frozen_prefix_is_pinned` ×2,
`history_is_append_only` ×2, `history_tail_matches_the_fingerprint_const`,
`protocol_version_sentinel`, `hash_schema_version_sentinel`, `declaration_fingerprint_is_pinned`,
`stream_fingerprint_is_pinned`) pass.

**Phase 3 — consumers** (`9a4f990c`):
- `LegalAction::DiscardToHandSize { count, hand, cards }` — `crates/simulator/src/legal_actions.rs`.
  `StubProvider::legal_actions`'s new **first** branch (ahead of the commander-zone check),
  forced by the admission gate's ordering exactly as the plan predicted.
- `random_bot::action_to_command` and `heuristic_bot`'s scorer both gained the compile-forced
  arm (heuristic scores it 100 — the only legal action while blocked).
- `DecisionKind::CleanupDiscard` + `#[non_exhaustive]` on the enum (audit §9.4 rec 1) —
  `crates/simulator/src/local_game.rs`. `LocalGame::advance`'s acting-player resolution
  chain gained the forced-first branch ahead of the commander-zone branch, same ordering
  reason as `StubProvider`.
- `replay_harness::translate_player_action` gained a `discard_cards: &[String]` trailing
  parameter and a `"discard_to_hand_size"` match arm (falls back to
  `default_cleanup_discard` when the script supplies no names);
  `script_schema::PlayerAction` gained `#[serde(default)] discard_cards: Vec<String>`. All 5
  call sites updated (`crates/engine/tests/scripts/script_replay.rs`,
  `harness_equivalence.rs`, `crates/engine/tests/combat/combat_harness.rs` ×7,
  `tools/replay-viewer/src/replay.rs`).
- TUI (`tools/tui/src/play/app.rs`): `acting_player`'s forced-first branch (same ordering
  reason); a `CleanupDiscardChoiceRequired` display arm in the event formatter.
  (`tools/tui/src/play/input.rs`): a `'d'` key that submits the offered default subset
  verbatim — the plan's "minimum viable" answer (seed OOS-DP7-6 covers a real picker).
- `tools/replay-viewer/frontend/src/lib/eventFormat.js`: a display case and a `'zone'`
  category entry for `CleanupDiscardChoiceRequired` (JS has no compile gate, so this is an
  easy silent miss — verified by reading, not by a build failure).
- 9 existing tests updated to answer the new pending decision instead of relying on the
  auto-pick (`turn_actions.rs` ×2, `card_def_fixes.rs` ×2, `madness.rs` ×4 — comments at the
  cited lines rewritten per the plan, not deleted) — see §8.1 table below for detail.
- **Un-enumerated fallout, not in the plan's §8.1 list**: `turn_structure.rs::test_ten_full_turn_cycles`
  (a bare `priority_holder.expect("no priority holder")`-style loop over 40 turns with
  15-card libraries) started panicking once a hand legitimately exceeded 7 cards partway
  through the run. The plan's §8.1 closing paragraph flagged this exact hazard by name
  ("`turn_structure.rs:227`'s `state.turn().priority_holder.unwrap()`... it panics on a
  `None`, which is now a reachable state during cleanup... verify rather than assume") —
  it just undercounted by one occurrence (there are two such loops in that file; only this
  one actually got exercised into the failure). Fixed with a
  `answer_pending_cleanup_discard` helper that answers deterministically and continues.
- **~40 scattered `HASH_SCHEMA_VERSION`/`PROTOCOL_VERSION` sentinel tests** across
  `crates/engine/tests/primitives/*.rs` and a few others re-pinned to the live values —
  this is the exact "guarded only by scattered `assert_eq!`" debt SR-17's doc comment
  describes as the disease it was designed to cure for the two *headline* sentinels; it
  never claimed to eliminate the older per-PB sentinels each batch leaves behind, and this
  list is now measurably longer than it was at SR-17 time. **Flagging per scope discipline
  convention rather than silently fixing further**: this is a standing, unbounded-growth
  maintenance tax on every future PROTOCOL/HASH bump and is a good candidate for a small
  follow-up (either delete the redundant per-PB copies in favor of the two canonical gate
  tests, or generate them). Not fixed here beyond the mechanical re-pin required to reach
  green — see seed OOS-DP7-8 below.
- **Fold-in fix, found writing T8**: `handle_discard_to_hand_size`'s existence check used
  `state.expect_object(id)` — an SR-4 `debug_assert`-backed accessor reserved for sites that
  require the id to already be known-live (an *engine* invariant). `cards` is untrusted
  command input, so an unknown id is a reachable, player-facing condition, not an engine
  bug; the debug build panicked instead of returning `ObjectNotFound`. Fixed to the fallible
  `state.object(id)` accessor. This is exactly the SR-4 classification the standing
  invariant exists to force, caught by writing the validation test rather than by review —
  recorded here as a genuine (small) defect in the phase-1 code, not a plan deviation.

**Phase 4 — tests** (`97995583`): 18 tests in the new
`crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs` (registered in
`crates/engine/tests/primitives/main.rs`, SR-9a), covering the plan's T1-T13 + T17-T18 (T9
split into 3 tests for the three `no_max_hand_size` sources; T18 split into 2 tests — see
deviation below). T14/T15 in `crates/simulator/tests/local_game.rs` (the existing
`LocalGame` acceptance-test file, not an inline `mod tests` — see deviation below). T16 in
`crates/simulator/src/legal_actions.rs`'s existing `mod tests`.

**Deviation — T18 (serde round-trip)**: the plan's T18 asked for "serde round-trip of a
blocked `GameState` preserves the entry; a pre-PB-DP7 snapshot without the field decodes."
`serde_json::to_string(&GameState)` does not work **at all** in this codebase —
`GameState` has several `OrdMap`s keyed by non-string newtypes (`ObjectId`, `PlayerId`,
`ZoneId`, ...), and `serde_json` requires string map keys. This is a pre-existing structural
property (confirmed: no other test anywhere round-trips a whole `GameState` through
`serde_json`; the established pattern, `test_replacement_effect_serde_roundtrip_*` in
`tests/rules/replacement_effects.rs`, round-trips individual structs). Rewrote T18 as two
tests at the struct level: `test_dp7_pending_cleanup_discard_serde_roundtrip` (mirrors that
established pattern) and `test_dp7_pending_cleanup_discard_defaults_when_absent` (a minimal
stand-in struct with the identical `#[serde(default)] pending_cleanup_discard:
Option<PendingCleanupDiscard>` field declaration, proving the exact mechanism the plan's
§5.2 relies on for old snapshots, without the impossible full-`GameState` premise).

**Deviation — T14/T15 location**: the plan's §8 header says these live in
"`crates/simulator/src/local_game.rs`'s `mod tests`". That module has exactly one existing
test (`test_command_player_extracts_acting_player`, extended in phase 3 with the new
variant) and no `GameStateBuilder`-driven fixture helpers; the established location for
exactly this kind of `LocalGame`/`advance`/`submit` behavioral test is the sibling
integration file `crates/simulator/tests/local_game.rs` (10 pre-existing tests of the same
shape, including `test_local_game_halts_awaiting_human_at_first_priority`, T14's closest
precedent). Placed T14/T15 there instead.

**Deviation — T15 driving mechanism**: building a full deck/library `LocalGame` fixture
(the file's existing `build_state`/`fixed_deck` helpers) makes it hard to force an oversized
hand deterministically in one turn. Instead built a minimal 2-player state directly via
`GameStateBuilder` with P1's hand pre-populated at 8 cards and an empty library for both
players, relying on CR 103.8a's 2-player first-turn draw-skip
(`is_first_turn_of_game && players.len() <= 2`, which `start_game_allowing_incomplete`
always sets regardless of builder input) to guarantee the hand stays at 8 through turn 1's
Cleanup — the fastest deterministic route to the pause, and the same shape the plan's T9/T10
fixtures in the engine-side file already use. T15's game naturally concludes shortly after
(P2 draws from an empty library on turn 2 and loses, CR 104.3b) rather than running to
`max_turns`; the assertion was written against the actual regression class (`EngineError`/
`NoLegalActions` halts from a rejected/impossible discard command), not against "never
halts", so this is not fragile to that natural early conclusion.

### Fail-before probe evidence (mandatory, T1/T2/T5/T6/T10)

Per the runner brief's "PB-DP6 hit a real hazard" warning, probes were run in an **isolated
`git worktree`** pinned at the base commit (`1854d3b9`, PB-DP6 collected) — `/tmp/scutemob-dp7-probe`
— rather than via in-place `git checkout <parent> -- <files>` on this working tree, to make
clobbering structurally impossible rather than merely avoided by care. The probe files were
never committed; the worktree was removed after collecting results
(`git worktree remove --force`), and `git status`/`git diff --stat HEAD` on this worktree
were confirmed clean immediately after.

Each probe asserts the POST-FIX-predicted outcome using only pre-existing (pre-PB-DP7) API,
so a probe that panics on the base commit demonstrates the defect the fix corrects.

| # | probe | predicted (plan §8) | observed on `1854d3b9` | match |
|---|---|---|---|---|
| T1 | `state.turn().step == Cleanup` after 4 passes (9-card hand) | fails — actually `Untap` of P2's turn | **FAILED**: `left: Upkeep, right: Cleanup` (P1 auto-discarded and the game ran clear through to P2's Upkeep, not merely P2's Untap — see note below) | plan's *direction* correct; the plan's own stated post-4-pass step (`Untap`) was one step optimistic — the fifth (Upkeep-entry) auto-advance also completes with no priority window in a 4-player empty board |
| T2 | a further `PassPriority` errors | fails — it succeeds today | **FAILED**: the follow-on pass for the (new) active player's priority holder returned `Ok`, confirming no block exists | matches |
| T5 | the two chosen LOWEST ids end up discarded | fails — the two HIGHEST go | **FAILED**: the two lowest ids were still asserted-present in hand pre-fix is false, i.e. they were NOT discarded (the two highest were, as predicted) | matches |
| T6 | Fiery Temper (highest id) stays in hand when a filler is "chosen" instead | fails — Temper is exiled regardless | **FAILED**: Temper was not in hand post-cleanup (it was involuntarily exiled) | matches |
| T10 | damage_marked > 0 right after entering Cleanup | fails — already 0 | **FAILED**: `damage_marked` was already 0 after the 4-pass sequence returned | matches |

**Predictions that were WRONG**: only T1's exact landing step. The plan's §8 table says the
probe "fails today (it is `Step::Untap` of P2's turn)" — on this branch's actual base
commit, a 4-player empty-board 4-pass sequence from `Step::End` runs the *entire* auto-
cascade (cleanup → next turn's Untap → auto-advance through Untap since it has no priority →
Upkeep, which DOES grant priority) in the same 4 `PassPriority` calls, landing at `Upkeep`,
not `Untap`. This does not change the probe's verdict (it still demonstrates "no pause
exists" exactly as intended) — recorded because the instruction is to report predictions
that were wrong, not just ones whose overall verdict differed.

### Wire sentinels (read directly from source after the change)

- `crates/engine/src/rules/protocol.rs:268` — `pub const PROTOCOL_VERSION: u32 = 28;`
- `crates/engine/src/rules/protocol.rs:285-286` — `PROTOCOL_SCHEMA_FINGERPRINT =
  "bf5f5dded64029f15272c4151edd847c340793ff7ebe7d4ee32ef51be81114b4"`
- `crates/engine/src/state/hash.rs:607` — `pub const HASH_SCHEMA_VERSION: u8 = 65;`

### Test counts

- Base (PB-DP6 collect): **3,809** passing / 0 failing.
- After phases 1-3 (9 existing-test fixes, 0 new tests): **3,809** passing / 0 failing
  (unchanged count, as expected — only fixes, no additions yet).
- After phase 4 (+21 new tests: 18 engine-side + 2 `LocalGame` + 1 `StubProvider`): **3,830**
  passing / 0 failing.
- `cargo build --workspace`: clean (0 warnings under the workspace's `warnings = "deny"`
  lint policy).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean. `tools/check-defs-fmt.sh`: clean (1,804 defs checked).
- `crates/engine/tests/core/bare_lookup_ratchet.rs` (SR-25): all 3 tests pass unchanged — no
  re-pin needed; the counter it tracks did not move.
- Golden-script suite (`cargo test -p mtg-engine --test scripts run_all_scripts`): 8/8 green,
  including `no_script_is_awaiting_triage` and
  `approved_scripts_only_use_allowlisted_untranslatable_actions` (SR-9c) — 0 new skips.
- Fuzzer A/B: **not completed as a clean A/B**. `mtg-fuzzer --games 10 --seed 1` (release,
  full 1,804-card pool) hit the pre-existing **OOS-DP3-9** stack overflow before completing
  even 10 games. Per the plan's explicit instruction ("do not chase it here"), no bisection
  against `main` was attempted; flagging honestly rather than claiming a clean A/B that
  wasn't actually run. The controlled, deterministic bot-only regression this PB specifically
  needs (a bot game not halting because of the new discard command) IS covered directly by
  T15, which does not depend on the large card pool.

### Card-def flip count

**0 completeness-marker flips**, exactly as the plan predicted (§Yield calibration / Fold-in
section). No card-def source files were edited. The plan's live-wrong claim (three
`Complete` Madness defs — `fiery_temper.rs`, `stensia_masquerade.rs`, `markov_baron.rs` —
were involuntarily exile-able pre-fix) is now corrected by the engine change alone; T6/T7
pin the corrected behavior at the engine level using a local test-only Fiery Temper
definition (matching `mechanics_m_z/madness.rs`'s own pattern), not the real card def files.

### Seeds for the coordinator

All seven of the plan's proposed seeds (§10) are still accurate and unfiled by this runner
(filing them in `docs/audits/decision-point-audit.md` §8.1 is explicitly the coordinator's
lane per the dispatch brief) — reproduced here for convenience, plus one new one found during
implementation:

- **OOS-DP7-1** — retrofit `pending_commander_zone_choices` onto `blocking_decision`
  (narrowed list: this one entry only, not all six).
- **OOS-DP7-2** — `DredgeChoiceRequired`/`MiracleRevealChoiceRequired` doc comments assert a
  pause the engine does not implement (dredge path confirmed by source reading this session;
  miracle not verified).
- **OOS-DP7-3** — `GameEvent::DiscardedToHandSize.reveals_hidden_info()` returns `false`,
  inconsistent with sibling `CardDiscarded`'s `true`.
- **OOS-DP7-4** — let the `cards` subset order carry CR 603.3b same-controller madness-trigger
  order once DP-14 lands (deliberately NOT done here — ascending sort is the safe interim).
- **OOS-DP7-5** — `PendingDecision.actions: Vec<LegalAction>` is the last decision class that
  fits without a `payload: DecisionPayload` reshape (audit §9.4 rec 2); M11-local Session
  3/5's call.
- **OOS-DP7-6** — the TUI's `'d'` key submits the deterministic default only; no real subset
  picker exists yet (M11-local Session 7).
- **OOS-DP7-7** — the audit's §10 re-audit trigger ("re-derive the 277 figure") is now due,
  plus the §5/DP-24 "accepted-and-discarded field" check on the new `Command` (answer: no —
  every field of `DiscardToHandSize` is validated, §2.4).
- **OOS-DP7-8** (new, filed by this runner's implementation, not the plan) — the scattered
  per-PB `HASH_SCHEMA_VERSION`/`PROTOCOL_VERSION` sentinel-test pattern SR-17 was supposed to
  retire has instead kept growing (~40 occurrences as of this PB, up from "~29" at SR-17
  time); each future wire/hash bump now pays a larger mechanical tax re-pinning them. A
  follow-up should either delete them in favor of the two canonical gate tests
  (`protocol_version_sentinel`/`hash_schema_version_sentinel` and their fingerprint
  siblings) or generate/derive them so they cannot drift out of a single source of truth.

Also due per the plan's audit cross-reference list (§10, second paragraph — coordinator's
lane, not filed here): §4.11 line 400 (B → A), §5 DP-3 row (SHIPPED banner), §8 PB-DP7 row
(wire prediction confirmed: **both** PROTOCOL and HASH moved, matching the row's prediction),
§8 sequencing note (point PB-DP8/DP9 at this plan's §1.5/§1.6), §9.3 and §9.4 rec 1
(`DecisionKind` now `#[non_exhaustive]` — done), §9.4 rec 5 (subset shape confirmed).
