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
