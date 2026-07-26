# Primitive WIP — PB-DP5 (DP-5: the `WouldDraw` multi-replacement prompt is unanswerable) · PLAN

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP4 (DP-10 attack tax never debited · DP-11 echo/CU/recover never
> enforce the "otherwise") — SHIPPED** `scutemob-152`, merge `799dcc0a`, tests 3,781. Its
> record lives in `docs/audits/decision-point-audit.md` §5 DP-10/DP-11 + §8,
> `memory/primitives/pb-plan-DP4.md` + `pb-review-DP4.md`, and the CLAUDE.md changelog entry.

- **PB**: PB-DP5 — **DP-5** (CR **616.1** / **614.11**). The engine asks a question it cannot
  accept an answer to, and the draw is silently destroyed.
  - `replacement::check_would_draw_replacement` returns `DrawAction::NeedsChoice(
    GameEvent::ReplacementChoiceRequired { .. })` when 2+ `WouldDraw` replacements apply.
  - Every caller emits that event and returns early **recording no pending state** — there is
    no draw-pending field on `GameState` at all.
  - `handle_order_replacements` (`rules/replacement.rs:163-172`) hard-requires a matching
    `pending_zone_changes` entry and errors without one. So the `Command::OrderReplacements`
    the player is being asked for is **rejected**, and the draw can never complete.
  - Net effect: the draw is eaten. Reachable with any two `WouldDraw` replacements, including
    in the draw step.
- **Task**: `scutemob-153`
- **Branch**: `feat/pb-dp5-woulddraw-multi-replacement-prompt-is-unanswerable-th`
- **Class**: CORRECTNESS (Tier 0, class **D**). Rank 5 of the PB-DP suite.
- **Phase**: plan
- **Binding spec**: `docs/audits/decision-point-audit.md`
  - §4 table, **line 383** — "`WouldDraw` multi-replacement | **D** | `rules/turn_actions.rs:1186-1189` (and the twin at `effects/mod.rs:8553-8564`) — see **DP-5**"
  - §5 **line 432** (DP-5 row) — the finding proper
  - §8 **line 581** (PB-DP5 row) — *"**HASH bump** (new `GameState` field); no new `Command`
    if it reuses `OrderReplacements`"*
  - §8.1 — where new seeds get filed
- **Plan file**: `memory/primitives/pb-plan-DP5.md`
- **Review file**: `memory/primitives/pb-review-DP5.md`

## Acceptance criteria (ESM `scutemob-153`)

1. (5531) With 2+ `WouldDraw` replacements, the draw records pending state,
   `OrderReplacements` is accepted, and the draw completes through the chosen order; test
   citing CR 616.1/614.11 covers **both** emit sites.
2. (5532) The strengthened `replacement_effects` test asserts the draw **actually completes**
   (card in hand / replacement applied), not merely deferred.
3. (5533) `HASH_SCHEMA_VERSION` bumped for the new `GameState` field per the SR-17 gate;
   PROTOCOL 27 unchanged.
4. (5534) `cargo test --all`, clippy, `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
   clean; audit DP-5 row + PB-DP5 row updated.

## Hard constraints

1. **No new `Command` variant, no new `GameEvent` variant, no new `Effect` variant.** Reuse
   `Command::OrderReplacements` and the existing `GameEvent::ReplacementChoiceRequired`.
   PROTOCOL **27** must be unchanged. **If the design appears to require a PROTOCOL bump, STOP
   and say so in the plan rather than bumping** — that is a re-scope decision, not a worker
   call.
2. **HASH bump is expected and allowed**: a new `pub(crate)` field on `GameState` requires
   `HASH_SCHEMA_VERSION` 63 → 64 per the SR-17 gate, with a `HASH_SCHEMA_HISTORY` entry.
   Confirm empirically (the gate test will tell you) rather than assuming — PB-DP2's predicted
   bump was **falsified**, and PB-DP3's and PB-DP4's "no change" predictions both held. Do not
   bump a constant the gate does not demand.
3. **Do not hang the game.** A deferred draw that nobody ever answers must not deadlock a
   sequential effect (`ForEach::EachPlayer` draws, etc.) or a fuzzer/simulator seat that never
   sends `OrderReplacements`. Whatever the resume design, state the chosen semantics
   explicitly and defend them. PB-DP4's constraint (b) is the precedent: hanging the game is
   strictly worse than the status quo.
4. Architecture invariants #2/#3: `GameState` is sealed `pub(crate)` (SR-3); mutation only
   through commands. A new field needs builder init + accessor (+ a `_mut` escape hatch only
   if genuinely needed, gated the way its siblings are) + `hash_into` in `state/hash.rs`.
5. SR-4: any new silent-failure site in `effects/mod.rs` / `rules/resolution.rs` must pick a
   side (`expect_*` vs `lki_*`).
6. `crates/simulator`, `tools/tui` and `tools/replay-viewer` have exhaustive matches that break
   on new enum variants — run `cargo build --workspace` after every phase.

## Coordinator pre-survey (a hypothesis for the planner to **falsify**, not a fact base)

> PB-DP3's and PB-DP4's wip files both record pre-survey bullets that were wrong in *both*
> directions. Verify every line below against the source as it exists on this branch, and
> record in the plan which bullets turned out to be wrong.

**Emit sites — the task description names two; I believe there are three.**

- `crates/engine/src/rules/turn_actions.rs` — `draw_card`, `DrawAction::NeedsChoice` arm
  (~`:1186`). Returns `Result<Vec<GameEvent>, GameStateError>`.
- `crates/engine/src/effects/mod.rs` — `draw_one_card`, same arm (~`:8553`). Returns a plain
  `Vec<GameEvent>` — **no `Result`**, which constrains any error-returning helper shared
  between the two.
- `crates/engine/src/rules/replacement.rs` — `draw_card_skipping_dredge` (~`:2666`) has the
  **same** `ReplacementResult::NeedsChoice` early return with no pending state. This is the
  post-dredge path. **The audit did not name it.** Confirm whether it is a genuine third
  instance of the same bug and include it if so; if it is unreachable, say why.

**The answer path.**

- `handle_order_replacements` (`rules/replacement.rs:~139`) does, in order: (a) reject empty
  `ids`; (b) reject unknown ids; (c) find a `pending_zone_changes` entry whose
  `affected_player == player`, else **error** — this is the rejection DP-5 is about;
  (d) rebuild a `WouldChangeZone` trigger from that pending entry and require every ordered id
  to be in `find_applicable(...)`; (e) call `resolve_pending_zone_change(state, ids[0],
  pending_idx)`.
- Steps (c)–(e) are all zone-change-specific. The draw case needs a parallel arm keyed off the
  new pending-draw state, rebuilding a `ReplacementTrigger::WouldDraw { player_filter:
  PlayerFilter::Specific(player) }` instead. **Keep both security checks** the existing
  doc-comment argues for: the sender must be the affected chooser, and every ordered id must be
  *currently applicable* (not merely registered) — a hostile client must not be able to apply
  an arbitrary registered replacement.
- Precedence question the plan must answer: what if a player has **both** a pending zone change
  and a pending draw? Pick a rule and defend it against CR.

**What "complete the draw through the chosen order" means (CR 616.1 / 616.1f).**

- Apply `ids[0]`, add it to an `already_applied` set, then **re-check** the remaining
  applicable replacements against that set (CR 616.1f). The zone-change path already does this
  inside `resolve_pending_zone_change` — read it and mirror the shape rather than inventing a
  second one. Possible outcomes: another `NeedsChoice` (pending state persists, a second
  `ReplacementChoiceRequired` is emitted), a single auto-apply, or nothing left → perform the
  draw.
- Today the only `ReplacementModification` the draw path honours is `SkipDraw` — the else-branch
  in `check_would_draw_replacement` says *"other modifications are not applicable to draws —
  proceed normally"*. So with two `SkipDraw`s, applying either yields a skipped draw and the
  chosen order is **unobservable**. **Do not let that make criterion 5532 vacuous**: find or
  construct a scenario where the two branches are *distinguishable*, so a test can prove the
  **chosen** replacement was honoured rather than an arbitrary one. If no such scenario exists
  without widening the draw path beyond `SkipDraw`, say so explicitly and propose the minimum
  that makes the criterion meaningful — that is a scope call to surface in the plan, not to
  make silently.
- The draw-completion body currently lives inline in three places. Factor it once if that is
  the clean move — but note `draw_card` also sets `has_drawn_for_turn`, increments
  `cards_drawn_this_turn` and runs the CR 702.94a miracle check, while `draw_one_card` does a
  subset. That difference is either a real distinction or a latent bug; decide which and say
  so.

**Tests.**

- `crates/engine/tests/rules/replacement_effects.rs:~2984-3000` —
  `test_draw_needs_choice_emits_replacement_choice_required` currently asserts only
  "deferred": event emitted, library still 1, hand still 0. Strengthen it per criterion 5532.
- SR-9a: integration tests are 9 targets under `crates/engine/tests/<group>/` with a `mod`
  line in the group's `main.rs`. **Never** add a top-level `tests/*.rs`; a missing `mod` line
  silently deletes coverage.
- Every new test cites CR 616.1 and/or 614.11 (architecture invariant #8).
- Fail-before/pass-after evidence is expected: run the new suite against the pre-fix source
  (`git show`-restore the touched files, run, restore byte-identical) and record actual
  observed pre-fix behaviour per test, the way PB-DP4's close-out does.
- Watch the SR-25 `bare_lookup_ratchet` gate (`crates/engine/tests/core/bare_lookup_ratchet.rs`)
  — it fires on any change up **or** down in the swept files, and `replacement.rs` may be one
  of them.

**Out of scope — file as seeds in the plan's seed section, do not fix here:**

- Giving `OrderReplacements` a `LegalAction` so a bot / M11-local seat can answer at all
  (this is the same class as PB-DP4's §9 recommendation).
- Widening the draw replacement path beyond `SkipDraw` modifications, unless the 5532 argument
  above forces a minimal widening.
- The §9 recommendation that `advance()` yield `AwaitingHuman` for a non-empty pending vector.

## Plan phase output required

`memory/primitives/pb-plan-DP5.md` containing:

1. Verified site inventory with line numbers **as they exist on this branch** (not the audit's).
2. The pending-state shape and the argument for it (what the resume needs; why a `Vector` vs a
   single `Option`; whether more than one player can have a deferred draw at once).
3. The resume algorithm written against CR 616.1 / 616.1f, including the re-check loop.
4. The precedence rule vs `pending_zone_changes`.
5. The deadlock-avoidance answer for hard constraint 3, with the deviation stated explicitly.
6. The exact hash/protocol gate expectation, and what will falsify it.
7. The test list with per-test fail-before predictions.
8. An explicit list of every pre-survey bullet above that turned out to be **wrong**.
9. A seed list for the out-of-scope items.

## Implementation complete (runner close-out)

**Status: SHIPPED (pending review).** All three phases + tests landed, all gates green.

### Change summary

- **Phase 1** (`8f184175`) — `GameState.pending_draws: Vector<PendingDraw>` (new
  `pub(crate)` field, next to `pending_zone_changes`) + the new `PendingDraw` struct in
  `crates/card-types/src/state/replacement_effect.rs` (`player`, `already_applied`,
  `remaining`, `sets_has_drawn_for_turn`). Builder init, `pending_draws()` /
  `pending_draws_mut()` accessors (SR-3), `HashInto` impl, `public_state_hash` feed, and
  a `loop_detection.rs` mirror block. `HASH_SCHEMA_VERSION` 63 → 64 (confirmed empirically
  by the gate test, not hand-bumped — both `declaration_fingerprint_is_pinned` and
  `stream_fingerprint_is_pinned` moved as predicted), `HASH_SCHEMA_HISTORY` v64 row
  appended, `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned, all 42 `HASH_SCHEMA_VERSION` sentinels
  bumped (41 files under `tests/primitives/` + `tests/casting/` + `tests/mechanics_e_l/` +
  `tests/rules/`, plus the local `hash_schema_version_sentinel`). `PROTOCOL_VERSION`
  unchanged at 27 (confirmed: `PROTOCOL_SCHEMA_FINGERPRINT` never moved).
- **Phase 2** (`b3e8e435`) — factored the three near-duplicate draw-completion bodies into
  `replacement::perform_one_draw` (+ `DrawStepOutcome` enum), parameterizing
  `check_would_draw_replacement` on `already_applied: &HashSet<ReplacementId>` (CR 614.5
  threading) and `offer_dredge: bool` (never re-offered on resume, §3.3). On
  `NeedsChoice`, `perform_one_draw` pushes a `PendingDraw` (with `already_applied` sorted
  by `ReplacementId` for hash determinism, SR-9b) and returns `Deferred` instead of
  emitting an unanswerable event into the void. Rewired all three emit sites:
  `turn_actions::draw_card` (`crates/engine/src/rules/turn_actions.rs`), the third,
  previously-unnamed site `replacement::draw_card_skipping_dredge` (reached via
  `Command::ChooseDredge { card: None }`), and `effects::draw_one_card` — renamed
  `draw_cards_for_player(state, player, n)` and now **owning** the CR 614.11a sequence
  loop, **breaking** on `Deferred`/`LostToEmptyLibrary` instead of continuing to iterate
  (pre-fix this loop kept calling the old `draw_one_card` after a deferral, so
  `Effect::DrawCards { count: 3 }` emitted three unanswerable prompts and drew zero
  cards — confirmed live in the fail-before probe, see below). SR-25
  `bare_lookup_ratchet`: `effects/mod.rs` ceiling re-pinned 111 → 110.
- **Phase 3** (`724a2c67`) — `handle_order_replacements` grows a second routing arm:
  tries a pending zone change first (byte-for-byte pre-PB-DP5 behavior, so no existing
  test regresses), then falls through to a pending draw. Routing is by applicability,
  which `trigger_matches` makes total (a `WouldChangeZone` replacement is never
  applicable to a `WouldDraw` event and vice versa — the two candidate sets are provably
  disjoint), so a well-formed answer can never be misrouted. New
  `resolve_pending_draw` (modelled on `resolve_pending_zone_change`): applies the chosen
  replacement (emitting `ReplacementEffectApplied` for it first — the order
  discriminator), `SkipDraw` ends the chain (CR 614.10/616.1f), anything else re-checks
  via a single `perform_one_draw` call (which **is** the CR 616.1f re-check — no
  additional loop needed, since `check_would_draw_replacement`'s own `AutoApply`
  dispatch is already terminal per call), and if the entry's `remaining` sequence count
  is nonzero, resumes the rest of the sequence (CR 614.11a). Both SR-29 trust-boundary
  checks (affected-chooser, currently-applicable) preserved in the new arm.
- **Tests** (`3bd7a029`) — 13 new tests in
  `crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs` (T1–T13; `mod` line
  added to `tests/primitives/main.rs`) + the existing
  `test_draw_needs_choice_emits_replacement_choice_required` in
  `crates/engine/tests/rules/replacement_effects.rs` strengthened in place (T0):
  added `pending_draws()` state assertions and drove the choice through
  `Command::OrderReplacements` to prove the chain actually resolves end to end
  (satisfying acceptance criterion 5532's "replacement applied" branch — this
  particular scenario is two `SkipDraw` effects, so the card-in-hand branch is proven
  by the separate `test_dp5_draw_completes_through_chosen_order`). All 13 new tests +
  the strengthened T0 passed on the **first** run against post-fix source — no debug
  cycle needed. `PendingDraw` re-exported from the crate root alongside its
  `PendingZoneChange` sibling for test-file convenience.

### Fail-before / pass-after evidence (OBSERVED, not predicted)

Method: reverted the 10 touched engine/card-types source files to `9fb09fc4` (the
pre-PB-DP5 parent commit) via `git checkout 9fb09fc4 -- <files>`, kept the new/modified
test files as committed, wrote a throwaway probe file
(`zzz_dp5_failbefore_probe.rs`, deleted before this close-out) using only
pre-fix-stable API signatures (`draw_card`, `execute_effect`, `process_command`,
`Command::OrderReplacements`/`ChooseDredge` — none of these signatures changed) so it
would compile against the reverted source, ran it with `--nocapture`, then restored all
10 files with `git checkout HEAD -- <files>` and confirmed `git diff` was empty before
re-running the full gate suite.

| # | test | OBSERVED pre-fix behavior | plan's prediction | match? |
|---|---|---|---|---|
| T0 | strengthened `test_draw_needs_choice_...` | **does not compile** — `error[E0599]: no method named `pending_draws` found for struct `GameState`** (19 such errors across the file, confirmed by compiling the real committed test files against reverted source) | "does not compile pre-fix (new API)" | ✅ |
| T1 | order-answering | `Err(InvalidCommand("player PlayerId(1) is not the affected player of any pending replacement choice"))` | exact match | ✅ |
| T2 | chosen order [601,600] | `Err(InvalidCommand("player PlayerId(1) is not the affected player of any pending replacement choice"))`, zero `ReplacementEffectApplied` events | "FAILS: command rejected, zero events" | ✅ |
| T3 | mirrored [600,601] | same `Err` as T2 | "FAILS: as T2" | ✅ |
| T4 | draw completes | `Err(...)`; hand stayed empty, library stayed 1 | "FAILS: command rejected; hand empty, library 1" | ✅ |
| T5 | effect-draw path | `execute_effect` emitted `ReplacementChoiceRequired`; the paired `OrderReplacements` probe returned the same `Err` as T1 | "paired probe: rejected identically to T1" | ✅ |
| T6 | sequence stop/resume | **3** `ReplacementChoiceRequired` events, **0** cards drawn to hand | "FAILS loudly: three prompts, zero cards" | ✅ |
| T7 | dredge-decline (3rd emit site) | `DredgeChoiceRequired` then `ReplacementChoiceRequired`(the same bug on this site); `OrderReplacements` → same `Err` as T1 | "the prompt is emitted, the OrderReplacements is rejected" | ✅ |
| T8 | wrong player (p2) | `Err(InvalidCommand("player PlayerId(2) is not the affected player of any pending replacement choice"))` | "passes pre-fix for the wrong reason" | ✅ (rejects, but for "no pending event at all" not "not applicable") |
| T9 | inapplicable id | `Err(InvalidCommand("player PlayerId(1) is not the affected player ..."))` — i.e. rejected via the **same** "no pending event" class as T8, not an applicability message | "passes pre-fix for the wrong reason" | ✅ |
| T10 | precedence | zone-change answer: `Ok` (`ReplacementEffectApplied` + `CommanderZoneRedirect`) — pre-existing path unaffected; draw answer: `Err(...)` same as T1 | "first command passes; second FAILS" | ✅ |
| T11 | 616.1f re-check | both submission orders (B-first, A-first) returned `Err(...)` — no `ReplacementEffectApplied` at all | "FAILS: both commands rejected" | ✅ |
| T12 | no-deadlock | `PassPriority` for both players succeeded with no error — this is a **regression guard**, so it was expected (and observed) to pass pre-fix too, for the reason stated in the plan (nothing to deadlock on) | "passes pre-fix — this is a regression guard" | ✅ |
| T13 | wire sentinels | `HASH_SCHEMA_VERSION=63, PROTOCOL_VERSION=27` — the `assert_eq!(.., 64u8)` half fails | n/a (assertion mismatch, not a behavior prediction) | ✅ |

All 14 rows (T0–T13) match the plan's per-test prediction exactly, including the two
"passes pre-fix for the wrong reason" cases (T8/T9) where the OBSERVED error was
specifically the "not the affected player of any pending replacement choice" class, not
an applicability-class message — confirming the fix-phase's test-validity requirement
that T9 must show a **different** error class post-fix (verified: post-fix T9 returns
`"none of the ordered replacement ids [...] are applicable to player PlayerId(1)'s
pending replacement choice (zone change pending: false, draw pending: true)"`).

### Test counts

- Parent pin (PB-DP4 collect): **3,781** passing, 0 failing.
- After PB-DP5 (13 new + 1 strengthened in place, net +13): **3,794** passing, 0 failing.

### Wire check (read directly from source after the change)

- `crates/engine/src/state/hash.rs`: `pub const HASH_SCHEMA_VERSION: u8 = 64;`
- `crates/engine/src/rules/protocol.rs`: `PROTOCOL_VERSION` unchanged at **27**
  (verified via `test_dp5_wire_version_sentinels` and by the fact
  `PROTOCOL_SCHEMA_FINGERPRINT` never moved in any gate run).

### Plan deviations

None. The plan's §3 Phases 1–3, §7 test list (with T0 strengthened per the explicit
runner brief), and §11 verification checklist were followed as written. One
implementation-level simplification versus the plan's literal §3.2 pseudocode: the
plan describes `resolve_pending_draw`'s CR 616.1f re-check as an explicit loop with a
termination proof; the actual implementation makes a **single** call to
`perform_one_draw` (which itself makes a single call to `check_would_draw_replacement`)
because `check_would_draw_replacement`'s own `AutoApply` dispatch is already terminal
per call (it only ever fires when exactly one replacement remains applicable, so there
is nothing left to loop over afterward) — behaviorally identical to an explicit loop in
every traced scenario (T4, T6, T11 all pass), just without a redundant `loop {}`
construct. This is documented in the doc comments on both `perform_one_draw` and
`resolve_pending_draw` (mirroring `resolve_pending_zone_change`'s own single-call
shape, which the plan explicitly said to model this on).

### Un-enumerated sites hit

None beyond the plan's own inventory. `cargo build --workspace` was run after every
phase; `tools/tui`, `tools/replay-viewer`, and `crates/simulator` had zero occurrences
of `Command::OrderReplacements` (confirmed by the plan's §1.7) and needed no changes —
this held true throughout implementation.

### Gates (all green)

`cargo build --workspace`, `cargo test --all` (3,794/0), `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804
defs clean), and `cargo test -p mtg-engine --test scripts run_all_scripts` (8/8,
including `replacement/014_golgari_grave_troll_dredge.json`).

**0 card-def edits** — as predicted (§11 checklist), the corpus has zero `WouldDraw`
replacement registrations (§1.4 / seed W1), so no card definition needed touching.
