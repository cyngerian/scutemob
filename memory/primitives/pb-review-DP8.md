# Primitive Batch Review: PB-DP8 — Triggered-ability target choice (DP-6 / OOS-M11-4)

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 603.3, 603.3a, 603.3b, 603.3c, 603.3d, 601.2c, 101.4, 104.3a, 117.3a/b, 726, 800.4d, 800.4j
**Engine files reviewed**: `crates/engine/src/rules/abilities.rs`, `rules/engine.rs`, `rules/combat.rs`,
`rules/resolution.rs`, `rules/priority.rs`, `rules/protocol.rs`, `rules/events.rs`, `rules/command.rs`,
`rules/loop_detection.rs`, `state/mod.rs`, `state/hash.rs`, `state/builder.rs`,
`card-types/src/state/stubs.rs`, `testing/replay_harness.rs`, `testing/script_schema.rs`,
`crates/simulator/src/{legal_actions,local_game,random_bot,heuristic_bot}.rs`,
`tools/tui/src/play/{input,app,panels/action_menu}.rs`,
`tools/replay-viewer/frontend/src/lib/eventFormat.js`
**Tests reviewed**: `crates/engine/tests/primitives/pb_dp8_trigger_target_choice.rs` (18 tests),
`crates/simulator/tests/local_game.rs` (3 DP8 tests), `crates/engine/tests/scripts/script_replay.rs`,
`test-data/generated-scripts/stack/138_emerge_elder_deep_fiend.json`
**Card defs reviewed**: 0 edited (verified). Roster spot-checked against oracle text:
`elder_deep_fiend`, `sword_of_sinew_and_steel`, `cloud_of_faeries`, `skullsnatcher`,
`tamiyo_field_researcher`, `teferi_temporal_archmage`, `marang_river_regent`, `sorin_lord_of_innistrad`

## Verdict: needs-fix

The batch is well-built and its central claims survive adversarial checking: the suspend/resume
machinery does not lose or duplicate triggers on the normal path, CR 603.3b batch order is preserved
byte-for-byte, the wire/hash bumps are gate-computed and append-only, the `HashInto` impls are written
with bare names and hash every field, `reveals_hidden_info() == false` is defensible (every candidate
is battlefield- or graveyard-scoped or a player), the forced-choice narrowing is CR-correct, and the
`UpToN` behaviour flip (divergence #1) is **confirmed against `main`'s source and against CR 601.2c/603.3d**
— it fixes a genuine live-wrong bug on `Complete` cards. But there are **two HIGH findings** and **five
MEDIUM**: the answered targets can be applied to the *wrong* trigger on the resume path (`head_targets`
is taken lazily behind a guard that can fire), the `UpToN` fix is only half-done (`count: 4` is capped
at one target, so Elder Deep-Fiend and Cloud of Faeries still do not match oracle text), and a **31st
`check_and_flush_triggers` call site was missed** — `handle_all_passed`'s overdue-payment branch grants
priority unconditionally after a flush that can now suspend. Both of the runner's deliberately-deferred
seeds (OOS-DP8-9, OOS-DP8-10) are, on inspection, defects that should be closed in this batch rather
than deferred; OOS-DP8-9 in particular is the precondition that makes HIGH-1 reachable and can fire a
`debug_assert` panic in any debug build.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `rules/abilities.rs:7934-7941` | **`head_targets` can be consumed by the wrong trigger.** The `head_targets.take()` sits *behind* the CR 603.3d "required slot has no candidates" guard, so if the head trigger is removed at resume time its answer leaks to the next trigger of the batch. **Fix:** take `head_targets` unconditionally on the first iteration of `flush_sorted`. |
| 2 | **HIGH** | `rules/abilities.rs:8925-8937`, `card-types/src/state/stubs.rs:766-784` | **`UpToN { count: N>1 }` is capped at one target.** `TriggerTargetOption` carries no maximum; the cardinality check is `<= 1`. Elder Deep-Fiend ("up to **four**") and Cloud of Faeries ("up to **two**"), both `Complete`, still cannot announce their full slot. **Fix:** carry `count` on the slot, allow `<= count`, add a per-slot duplicate check (CR 601.2c). |
| 3 | MEDIUM | `rules/engine.rs:2139` | **A 31st `check_and_flush_triggers` site was missed, and it grants priority.** `handle_all_passed`'s `force_resolve_overdue_payments` branch flushes and then unconditionally grants priority at `:2152-2168`. **Fix:** add the suspension guard + `mark_flush_owes_priority` + early return, mirroring the other four. |
| 4 | MEDIUM | `rules/engine.rs:2249-2252`, `:2298-2301`; `rules/abilities.rs:8792-8819` | **CR 726 loop detection and the cleanup ratchet are skipped on every suspension** (broader than seed OOS-DP8-10). `finish_resumed_flush` re-implements only the priority grant. **Fix:** fold `check_for_mandatory_loop` and `cleanup_sba_rounds += 1` into the resume path. |
| 5 | MEDIUM | `rules/engine.rs:2404-2488` | **`handle_concede` is ungated under another player's outstanding announcement** (seed OOS-DP8-9). It can reach `handle_all_passed` → resolution → `flush_pending_triggers`, firing the `debug_assert!` at `abilities.rs:7611` (panic in debug builds), and can advance the whole turn under a suspended batch. **Fix:** skip the priority-advance / turn-advance blocks while a foreign `blocking_decision()` is outstanding. |
| 6 | MEDIUM | `rules/abilities.rs:8920-8951` | **Positional index shift when an optional slot is under-filled.** `chosen` is a flat concatenation, so `[[], [artifact]]` places the artifact at `DeclaredTarget { index: 0 }`. Newly reachable on the trigger path. **Fix:** pad each slot to its declared width (folds into Finding 2). |
| 7 | MEDIUM | `tests/primitives/pb_dp8_trigger_target_choice.rs:818` | **Only one of the guard sites is tested.** T12 exercises `enter_step`'s has-priority branch alone; the Cleanup branch, both combat guards and the resolution tail have no test, and `finish_resumed_flush`'s dead-active-player fallback is untested. This is why Finding 3 was not caught. **Fix:** extend T12 to all guard sites plus the new one. |
| 8 | LOW | `rules/abilities.rs:7587-7589` | **A question with exactly one legal answer is still asked** when a slot is `optional` with an empty candidate set. **Fix:** treat an empty-candidate optional slot as forced (answer `[]`). |
| 9 | LOW | `rules/engine.rs:160-191` vs the four raw-field guards | **Liveness filter and raw-field guards disagree.** `blocking_decision()` hides a dead owner's entry; the in-crate guards and `flush_pending_triggers`'s early return read the raw field. An entry whose owner dies by any route other than `handle_concede` becomes invisible to the gate but permanently blocks every flush. **Fix:** clear the field wherever a player is eliminated, or make the raw-field guards liveness-aware. |
| 10 | LOW | `rules/abilities.rs:7988` | **`flush_sorted`'s `players_passed` reset is skipped on the suspend return.** Harmless on the normal path (the resume re-runs it) but not if the resumed batch places nothing. **Fix:** reset before the `return events;`, or document why not. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `elder_deep_fiend.rs` | **Oracle:** "tap up to **four** target permanents." Engine Finding 2 caps the announcement at one. `Complete`. **Fix:** engine-side (Finding 2); no def edit. |
| 2 | **HIGH** | `cloud_of_faeries.rs` | **Oracle:** "untap up to **two** lands." Same cap. `Complete`, no `completeness` marker. **Fix:** engine-side (Finding 2). |
| 3 | LOW | `sword_of_sinew_and_steel.rs` | Two parallel `UpToN{count:1}` slots with `DeclaredTarget{0}`/`{1}`. Correct today only because both effects are `DestroyPermanent`; Finding 6's index shift would otherwise mis-target. **Fix:** engine-side (Finding 6). |
| 4 | LOW | `test-data/.../138_emerge_elder_deep_fiend.json` | **Stale, self-contradicting notes.** Steps at `:155`/`:173` correctly describe the cast trigger on the stack; `:184`/`:187` still say "the cast trigger ... does not fire" / "WhenCast not yet implemented", and `disputes[0]` still claims the trigger is unimplemented. **Fix:** update both notes and resolve the dispute. |

---

### Finding Details

#### Finding 1: `head_targets` can be consumed by the wrong trigger

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:7934-7941`
**CR Rule**: 603.3d — "If a choice is required when the triggered ability goes on the stack but no
legal choices can be made for it ... the ability is simply removed from the stack"; 601.2c — the
announcement is per-ability.

**Issue.** In `flush_sorted` the resume answer is consumed lazily:

```rust
if slots.iter().any(|s| !s.optional && s.candidates.is_empty()) {
    None                                   // <- head can exit HERE with head_targets still Some
} else if let Some(pre) = head_targets.take() {
    Some(pre)
}
```

There is a second escape earlier in the same arm: `if ability_targets.is_empty() { Some(vec![]) }`
(`:7916-7918`), which is reached when `state.objects.get(&trigger.source)` is `None`. On either exit
`head_targets` survives to the **next** iteration, where it is bound to a *different* `PendingTrigger`
— potentially a different controller's ability with entirely different `TargetRequirement`s. The
resulting stack object carries targets that were never validated against its own requirements.

The plan's §3.3 argues this cannot happen because "`state` cannot have changed" between offer and
resume. That argument is **false**: `process_command`'s admission gate (`engine.rs:213-226`) admits
`Command::Concede { .. }` from *any* player, and `handle_concede` for a third player can reach
`handle_all_passed` (`:2449`) — resolving the top of the stack and destroying the head's only
candidate — or the `active_player == player` turn-advance branch (`:2456-2488`). Either mutates the
board between the offer and the resume.

**Concrete failure.** 4-player game. P1's Ravenous Chupacabra-style ETB ("destroy target creature an
opponent controls") suspends with two candidates, both controlled by P3; `remaining` holds P2's
"whenever ... deals damage to target player" trigger. P3 concedes; `handle_concede` runs
`handle_all_passed`, a Wrath resolves, both creatures die. P1 answers with the (still-accepted, because
validation is against the frozen `entry.slots`) creature id. On resume the head's candidate set is now
empty → head removed → `head_targets` is still `Some([Target::Object(dead_creature)])` → **P2's
trigger goes on the stack targeting a dead creature id in a slot that requires a player.**

**Fix:** compute the head binding by position, not lazily. In `flush_sorted`, at the top of the loop
body, `let this_head = if next_index == 1 { head_targets.take() } else { None };` and use `this_head`
in the branch. Add a regression test that concedes a third player mid-announcement and asserts each
placed trigger's targets satisfy its own requirements.

---

#### Finding 2: `UpToN { count: N }` is capped at one target

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:8923-8937`; `crates/card-types/src/state/stubs.rs:766-784`
**Oracle**: Elder Deep-Fiend — "When you cast this spell, tap **up to four** target permanents."
Cloud of Faeries — "When this creature enters, untap **up to two** lands."
**CR Rule**: 601.2c — "If the spell has a variable number of targets, the player announces how many
targets they will choose before they announce those targets."

**Issue.** `TargetRequirement::UpToN { count, inner }` (`card_definition.rs:3022-3025`) declares a
`count`-wide slot, and `casting.rs`'s two-pass validator honours it for spells. `TriggerTargetOption`
drops `count` entirely — it records only `optional: bool` — and `handle_choose_trigger_targets`
enforces `submitted.len() <= 1` for an optional slot. So the only announcements PB-DP8 can express for
"up to four target permanents" are zero targets or one.

This is a genuine improvement over `main` (which removed the trigger outright — see the divergence
verification below), but the batch's own commit narrative presents the `UpToN` flip as fixing Elder
Deep-Fiend, and it does not: the card still cannot tap more than one permanent. Cloud of Faeries is
`Complete` with no marker and is in the same position. `skullsnatcher` (`count: 2`) is `partial`, so
lower stakes.

Note that the missing per-slot **duplicate** check is latent behind the same cap: with `<= 1` a slot
cannot repeat a target, so CR 601.2c's "the same target can't be chosen multiple times for any one
instance of the word 'target'" is satisfied by accident. Raising the cap without adding the check
would introduce a rules violation.

**Fix:** add `max: u32` (or `count`) to `TriggerTargetOption`, set it from
`TargetRequirement::UpToN { count, .. }` (1 otherwise), change the cardinality check to
`submitted.len() <= slot.max` for optional slots and `== 1` otherwise, and reject duplicate `Target`s
within a slot. This is a wire-shape change to a type already in the closure — re-pin
`PROTOCOL_SCHEMA_FINGERPRINT` (no `PROTOCOL_VERSION` bump is owed if it lands before merge, otherwise
30) and `HASH_SCHEMA_*`.

---

#### Finding 3: a 31st `check_and_flush_triggers` call site was missed, and it grants priority

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:2139` (guard absent), grant at `:2152-2168`
**CR Rule**: 603.3b — "... Then the appropriate player gets priority" (i.e. only after every triggered
ability of the batch is on the stack).

**Issue.** The plan's §4.2 and the runner's "verified, not asserted" note both establish that the
**30** `check_and_flush_triggers` calls inside `process_command`'s `match` need no guard. That is
correct — I re-verified all 30 mechanically (`engine.rs:263, 305, 336, 425, 439, 448, 460, 476, 545,
560, 574, 583, 592, 603, 611, 620, 629, 638, 652, 665, 681, 695, 709, 718, 731, 737, 743, 761, 781,
792`; each is followed by exactly `all_events.extend(events);` and the end of the arm, `:592` adds only
a comment, `:545` is the new self-guarded call in the `ChooseTriggerTargets` arm). But
`check_and_flush_triggers` has a **31st** call site outside that match:

```rust
// engine.rs:2134-2169, inside handle_all_passed
let mut payment_events = force_resolve_overdue_payments(state);
if !payment_events.is_empty() {
    check_and_flush_triggers(state, &mut payment_events);   // <- can now suspend
    ...
    if is_alive { let (passed, priority_events) = priority::grant_initial_priority(state); ... }
```

`check_and_flush_triggers` calls `check_triggers` on `payment_events` (`:33-36`), so a PB-DP4 forced
echo/cumulative-upkeep sacrifice that produces a targeted dies-trigger reaches the CR 603.3d
announcement. The flush suspends; the code then grants priority anyway, and never calls
`mark_flush_owes_priority`. Result: `PriorityGiven` is emitted with the CR 603.3b batch half-placed,
and the batch's own stated invariant ("no priority is granted while suspended") is false.

The plan's §16 verification checklist only greps `flush_pending_triggers\(`, which does not see this
site — the checklist itself has the gap.

**Fix:** insert the standard guard after `:2139`:
```rust
if state.pending_trigger_targets.is_some() {
    abilities::mark_flush_owes_priority(state);
    events.extend(payment_events);
    return Ok(events);
}
```
and extend the §16 mechanical check to `rg 'check_and_flush_triggers\(' crates/engine/src` with a
statement of what follows each site.

---

#### Finding 4: CR 726 loop detection and the cleanup ratchet are bypassed on every suspension

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/engine.rs:2249-2252` and `:2298-2301`; `abilities.rs:8792-8819`
**CR Rule**: 726 (handling illegal actions / mandatory loops), 104.4b (draw on a mandatory loop)

**Issue.** `finish_resumed_flush` reproduces exactly one thing the four guarded sites were about to do:
the priority grant. Both `enter_step` guards return *before* `loop_detection::check_for_mandatory_loop`
(`:2258` and `:2305`), and the Cleanup guard additionally returns before `state.turn.cleanup_sba_rounds
+= 1` (`:2255`) and before the `cleanup_sba_rounds < MAX_CLEANUP_SBA_ROUNDS` test.

Consequences, both of which convert a bounded pathological state into an unbounded one:

1. **CR 726 is never checked for any batch that suspends.** A mandatory infinite loop that involves a
   targeted triggered ability (a Kiki-Jiki/Zealous Conscripts-shaped mandatory loop with a targeted
   ETB, say) never produces the `MandatoryLoopDetected` draw; the engine simply cycles forever.
2. **The 100-round cleanup ratchet stops advancing.** Each cleanup round that suspends and resumes
   leaves `cleanup_sba_rounds` where it was, so `MAX_CLEANUP_SBA_ROUNDS` is never reached and the
   cleanup step cannot fall through to auto-advance.

The runner's seed OOS-DP8-10 names only (2), and only for the Cleanup branch. (1) applies to the
has-priority branch as well and is the more serious half.

**Fix:** carry enough site identity in the entry (or, simpler, run the loop check + ratchet inside
`finish_resumed_flush` when `owed`), and add a test that a suspended cleanup batch still advances
`cleanup_sba_rounds`. This should be in this batch: the batch is what removed the checks.

---

#### Finding 5: `handle_concede` is ungated under another player's outstanding announcement

**Severity**: MEDIUM (assessed against the runner's request to judge OOS-DP8-9)
**File**: `crates/engine/src/rules/engine.rs:2404-2488`
**CR Rule**: 104.3a, 603.3b, 800.4j

**Issue.** `drop_conceded_trigger_flush` correctly handles the case where the *entry's own* player
concedes. When a **different** player concedes, it returns `None` and `handle_concede` proceeds through
its full priority-advance and turn-advance logic with the batch still suspended. Three harms:

- **(a) A debug-build panic.** If the conceder held priority (reachable: a `check_and_flush_triggers`
  suspension leaves the actor holding priority, per PB-DP1) and `next_priority_player` returns `None`,
  `handle_concede:2449` calls `handle_all_passed`, which resolves the top of the stack; the resolution
  tail calls `flush_pending_triggers`, which fires
  `debug_assert!(state.pending_trigger_targets.is_none(), "flush_pending_triggers re-entered ...")`
  (`abilities.rs:7611`). Tests and the fuzzer are debug builds. In release it silently resolves a spell
  with the CR 603.3b batch half-placed.
- **(b) A whole turn can advance under a suspended batch.** `:2456-2488` runs `advance_turn` +
  `enter_step`; `enter_step`'s PB-DP7 progress gate at `:2230` stops it *after*
  `execute_turn_based_actions` has already run for the new step. When the answer finally arrives, the
  previous turn's `remaining` triggers are placed in the new turn.
- **(c) It is the state change that makes Finding 1 reachable.**

The runner's argument for leaving it ("gating risks a hang") does not hold: the entry's player is alive
by construction (the flush never asks a dead controller, `abilities.rs:7950-7960`) and remains the only
legal actor besides further concedes, so the block always has an answerer.

**Fix:** in `handle_concede`, after `drop_conceded_trigger_flush`, early-return past the
priority-advance and turn-advance blocks when `blocking_decision(state).is_some()`. Add a test:
3 players, P1's announcement outstanding, P2 concedes, assert no `PriorityGiven`, no turn advance, no
stack resolution, and that P1's answer still completes the batch.

---

#### Finding 6: positional index shift when an optional slot is under-filled

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:8920-8951` (`chosen` accumulation);
`abilities.rs:9025-9027` (`default_spell_targets`, same shape via `filter_map`)
**Oracle**: Sword of Sinew and Steel — "destroy up to one target planeswalker **and** up to one target
artifact"
**CR Rule**: 601.2c

**Issue.** `chosen` is built by pushing each submitted target in slot order into one flat
`Vec<SpellTarget>`, and the resolution layer reads it by absolute index
(`EffectTarget::DeclaredTarget { index }`). An answer of `[[], [artifact]]` produces `chosen =
[artifact]`, so `DeclaredTarget { index: 0 }` — the "destroy up to one target **planeswalker**" clause —
resolves to the artifact and `index: 1` resolves to nothing. `cloud_of_faeries.rs:23-27` documents the
intended contract as "UpToN contributes its declared targets at consecutive indices **starting where the
prior requirement's indices end**", i.e. the slot occupies a fixed width — which the flat concatenation
does not honour.

Pre-PB-DP8 this was unreachable on the trigger path (a permanent-inner `UpToN` removed the trigger, and
`default_trigger_targets` answers every optional slot with zero, keeping all bots and the harness on the
all-empty path). The batch makes partial answers possible for the first time. No `Complete` def is
currently harmed — the only two multi-slot cases, `sword_of_sinew_and_steel` and `cloud_of_faeries`,
have homogeneous per-slot effects — but the next heterogeneous two-slot trigger silently targets the
wrong object.

**Fix:** pad each slot to its declared width when building `chosen` (folds into Finding 2's `max`
field), and mirror it in `default_spell_targets`. Add a test with a `[UpToN{creature}, TargetPlayer]`
trigger answered `[[], [player]]`.

---

#### Finding 7: only one of the guard sites is tested

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dp8_trigger_target_choice.rs:818-867`

**Issue.** The plan's T12 required the suspension/grant behaviour to be pinned at
`handle_declare_attackers`, `handle_declare_blockers` **and** the resolution tail; the shipped
`test_dp8_no_priority_granted_while_suspended_then_granted_on_resume` drives only `enter_step`'s
has-priority branch (via two `PassPriority` commands). Nothing tests:

- `enter_step`'s Cleanup branch (guard #2) — the branch Finding 4 is about;
- `combat.rs:766` / `combat.rs:1552` (guards #4/#5);
- `resolution.rs:7805` (guard #6);
- `finish_resumed_flush`'s dead-active-player fallback (`abilities.rs:8812-8818`) — the one branch
  where the resume grants priority to someone other than the active player;
- `handle_all_passed`'s payment branch — which is precisely why Finding 3 shipped undetected.

Everything else the plan asked for is present and honest: the 18 engine tests match their names and
each cites CR 603.3d/603.3b/601.2c/800.4d; the roster test is a real `all_cards()` enumeration (SR-36
satisfied) with a `>=` pin; the simulator's T16/T17/T18 exist and T16 asserts
`kind == DecisionKind::TriggerTargets` as required; the fail-before evidence (14 of 18 fail with the
suspension disabled, T14b alone fails with the old `UpToN` semantics) is a credible probe rather than a
claim.

**Fix:** add one test per remaining guard site, plus one for the dead-active-player fallback.

---

## Verification of the four items the brief asked to be checked adversarially

**Suspend/resume correctness (brief item 1).** Verified sound on the normal path. `flush_pending_triggers`
drains + sorts and `flush_sorted` does neither, so `remaining` preserves CR 603.3b order exactly.
`remaining` is captured from `sorted[next_index..]` *after* `next_index` was already incremented past
the head, so the head is not duplicated. Triggers already on the stack are never revisited. Re-entrancy
is blocked by a `debug_assert` + early return, and a second suspension during a resume is handled
(`finish_resumed_flush` inherits `owed` onto the new entry). Rejection leaves state untouched at the
`process_command` boundary — `GameState` is taken by value and every `?` discards the local copy; the
`loop_detection::reset_loop_detection` call at `engine.rs:535` runs before validation but is discarded
on `Err`. Criterion 5545 holds. **The one hole is Finding 1**, which is a resume-path hole, not an
ordering one.

**`grant_priority_on_resume` totality (brief item 2).** The discharge logic is total on the paths it
covers: set by exactly the four guards, inherited on re-suspension, discharged once when the batch
completes, and also carried through `drop_conceded_trigger_flush`. `finish_resumed_flush`'s grant is a
faithful reproduction — `priority::grant_initial_priority` returns `(OrdSet::new(), [PriorityGiven{active}])`,
which is what `finish_resumed_flush` writes, and `handle_declare_attackers`'s `Some(player)` is the
active player by the handler's own entry check. No path sets the flag without discharging it, and no
path discharges twice (the `ChooseTriggerTargets` arm's follow-up `check_and_flush_triggers` can only
create a *new* entry with `owed = false`). **But the flag is not set at all at the site in Finding 3**,
which is the actual totality gap — the missing site was never in the enumeration.

**The `UpToN` flip (brief item 3) — CONFIRMED.** I read `main`'s pre-change source directly. At
`main:abilities.rs:7503-7556` the `UpToN` arm returns `None` for any non-player inner, and the caller at
`main:7785-7797` does `if let Some(st) = candidate { selected.push(st) } else { all_satisfied = false;
break; }` → `trigger_targets_opt = None` → `continue` at `main:7803-7806`. So the whole trigger was
removed, exactly as the runner claims, and the pre-existing comment at `main:7497-7502` ("contribute 0
targets by returning None") is aspirationally wrong. CR 601.2c makes zero targets a legal announcement
for an "up to" slot, so CR 603.3d's "no legal choices can be made" clause does not apply; the flip is
CR-correct and the golden-script edit is justified. The recorded rationale in script 138's steps `:155`
and `:173` matches what the code now does. **Two caveats:** the fix is incomplete (Finding 2), and the
script's later notes contradict its earlier ones (Card Finding 4).

**Forced-choice narrowing (brief item 4) — CR-correct.** CR 601.2c requires the controller to *announce*
a choice; where the rules leave exactly one legal option the announcement is determined, and CR 603.3d
inherits 601.2c wholesale. `optional` is set at `abilities.rs:7368` by
`matches!(req, TargetRequirement::UpToN { .. })` and nowhere else, and I confirmed `UpToN` is the only
source of optionality in the current DSL: no other `TargetRequirement` variant admits zero targets,
"you may" triggers have no DSL representation (DP-12, 19 `known_wrong` defs), and modal triggers are
correctly out of scope (OOS-DP3-4). An `UpToN` slot with one candidate correctly stays a real choice —
`trigger_target_choice_is_forced` rejects any `optional` slot. The only refinement owed is Finding 8
(an optional slot with *zero* candidates has exactly one legal answer and should not be asked).

**Consult-site completeness (brief item 5) — re-verified independently; one site missing.** See
Finding 3. The four named guards are present and correct at `engine.rs:2249`, `engine.rs:2298`,
`combat.rs:766`, `combat.rs:1552`, `resolution.rs:7805` (five, not four — the plan's "four" counts
`enter_step` once). `resolution.rs:7799` is guarded. `handle_all_passed`'s `enter_step` and resolution
sub-paths are covered by the existing PB-DP7 progress gate and the resolution guard respectively.

**Validation totality (brief item 6).** `handle_choose_trigger_targets` checks, in order and before any
mutation: entry exists → sender is the entry's player (SR-29, distinct error from the admission gate's,
and T9 asserts both specifically) → `choice_id` moment guard → slot count → per-slot cardinality →
membership in the *frozen offered candidate set* (so `zone_at_cast` comes from the engine, never the
wire, and the predicate is never forked) → narrow cross-slot `TargetPermanentDistinctFrom` distinctness
→ `state.player(player)?`. Gaps: no per-slot duplicate check (latent behind the `<= 1` cap — Finding 2)
and the cardinality bound ignores `count` (Finding 2).

**Liveness (brief item 7).** The engine never asks a dead controller (`abilities.rs:7950-7960`, T10b
pins the *absence* of an entry, not just the outcome). The conceding-owner path is implemented in the
implement phase as the plan required, drops the owner's own triggers per CR 800.4d, keeps the rest of
the batch per CR 800.4j, and can legitimately re-suspend on another player. **OOS-DP8-9 is a defect,
not a safe deferral — see Finding 5.** **OOS-DP8-10 is a defect and understated — see Finding 4.**

**Blast radius on driving loops (brief item 8).** Verified. The TUI's `should_stop_auto_pass` and
`acting_player` read `blocking_decision()` and generalise unchanged; the `'n'` key, the menu hint and
the event-formatter arm are present. `LocalGame::advance`'s acting-player chain is now an exhaustive
`match` on `BlockingDecision` (`local_game.rs:350-355`), closing the hard-coded-`CleanupDiscard` latent
bug the plan found. `StubProvider`, `RandomBot` and `HeuristicBot` are compile-forced and present; the
bot submits the offered default verbatim (correct for this batch, seeded OOS-DP8-1). The
`LegalActionProvider` obligation sentence is present (`legal_actions.rs:173-179`). The replay-harness
pump exists, goes through `process_command`, is bounded, and is wired into the script driver
(`script_replay.rs:352-380`) with a next-action skip so a script can choose. Two small pump caveats,
not filed as findings: the skip only looks within the same step's `actions`, and it skips on
`discard_to_hand_size` even when the outstanding decision is `TriggerTargets`.

**Wire/gate integrity (brief item 9).** `PROTOCOL_VERSION 29` with a `- 29:` history line, an appended
`ProtocolEpoch { version: 29 }` whose fingerprint equals `PROTOCOL_SCHEMA_FINGERPRINT`, and no edited
row (v27/v28 unchanged). `HASH_SCHEMA_VERSION 66` with an appended `HashSchemaEpoch { version: 66 }`
carrying both fingerprints; v65 unchanged. `GameEvent` discriminant `130u8` added to the no-`_`-arm
match (`hash.rs:5405`). Both `HashInto` impls use **bare** struct names (`hash.rs:3044`, `:3051`) and
hash **every** field (3/3 and 7/7, including `grant_priority_on_resume`); the runner's delete-a-field
demonstration is recorded. `pending_trigger_targets` is folded into `public_state_hash` (`:7858`) and
`loop_detection.rs:159`. The field is `pub(crate)` with a read-only accessor and no `_mut` (SR-3).
`reveals_hidden_info() == false` is **correct**: `trigger_battlefield_target_matches` returns false for
anything not on the battlefield (`abilities.rs:7096`), the two graveyard scans are zone-restricted, the
player arms name players, and every other `TargetRequirement` falls to `_ => false`; no library-, hand-
or exile-scanning requirement reaches this site, and only `ObjectId`s (never card identities) are
carried, so face-down permanents are safe too.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.3 | Yes | Yes | `test_dp8_admission_gate_while_suspended` |
| 603.3a | Yes | Yes | `test_dp8_sender_validation` (both paths, specific errors) |
| 603.3b | Yes | Partial | `test_dp8_apnap_sequence_across_two_controllers`, `..._places_each_trigger_exactly_once`; **violated** at the site in Finding 3 |
| 603.3d | Yes | Yes | `test_dp8_flush_blocks_on_a_real_target_choice`, `..._no_legal_candidate_still_removes_the_trigger` |
| 601.2c (announce) | Yes | Yes | `test_dp8_chosen_target_is_honoured_not_first_match` |
| 601.2c ("up to") | **Partial** | Partial | `test_dp8_up_to_n_slot_is_optional_and_zero_targets_is_legal`; **`count > 1` unimplemented** (Finding 2) |
| 601.2c (one legal answer) | Yes | Yes | `test_dp8_forced_single_candidate_asks_nothing`; refinement in Finding 8 |
| 601.2c (no duplicate targets) | Narrow only | Partial | cross-slot `TargetPermanentDistinctFrom` only; per-slot latent (Finding 2) |
| 102.3 / PB-EF6 | Yes | Yes | `test_dp8_target_opponent_never_self_and_never_asks_when_alone` |
| 104.3a / 800.4d / 800.4j | Partial | Partial | `test_dp8_controller_concedes_mid_choice`; foreign-conceder path ungated (Finding 5) |
| 117.3a/b | Yes | Partial | one guard site tested of five/six (Finding 7) |
| 104.4b / 726 | **Regressed** | No | loop check skipped on every suspension (Finding 4) |
| 603.3c (modal triggers) | Out of scope | n/a | OOS-DP3-4 / OOS-DP8-7; scope call agreed |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| (all 1,804 defs) | n/a | 0 | n/a | **0 card-def source edits, 0 completeness flips** — confirmed |
| `elder_deep_fiend` | No | 0 | **No** | "up to four" → at most 1 (Finding 2). Improved from "trigger never placed" |
| `cloud_of_faeries` | No | 0 | **No** | "up to two" → at most 1 (Finding 2). `Complete`, unmarked |
| `sword_of_sinew_and_steel` | Yes | 0 | Yes (by luck) | Two `UpToN{1}` slots; correct only because both effects are `DestroyPermanent` (Finding 6) |
| `skullsnatcher` | No | — | No | `count: 2` capped at 1; already `partial`, low stakes |
| `tamiyo_field_researcher`, `teferi_temporal_archmage`, `marang_river_regent`, `sorin_lord_of_innistrad` | n/a | 0 | n/a | `UpToN{count>1}` on **loyalty/activated** abilities, not `Triggered` — outside this site |

## Seed Assessment (as requested)

| Seed | Runner's disposition | Reviewer's disposition |
|------|---------------------|------------------------|
| **OOS-DP8-9** (`handle_concede` ungated) | deferred, "gating risks a hang" | **Fix in this batch** — Finding 5. The hang argument does not hold (the entry's player is alive by construction); the gap fires a `debug_assert` panic in debug builds, advances turns under a suspended batch, and is the precondition for HIGH Finding 1. |
| **OOS-DP8-10** (Cleanup ratchet skipped) | deferred | **Fix in this batch, and widen it** — Finding 4. The seed understates the problem: CR 726 loop detection is skipped in *both* `enter_step` branches, not just the ratchet in the Cleanup one. Both turn a bounded pathological state into an unbounded one. |
| OOS-DP8-1..8 | filed | Agreed as filed. OOS-DP8-4 (`TargetPermanentDistinctFrom`) and OOS-DP8-7 (modal, PB-DP8b) are correctly scoped out. |

## Divergences from the Plan — Reviewer's Verdict

| # | Divergence | Verdict |
|---|-----------|---------|
| 1 | `UpToN` premise falsified; behaviour flip | **Correct and well-evidenced**, verified against `main`'s source and CR 601.2c/603.3d. **Incomplete** — Finding 2. |
| 2 | `grant_priority_on_resume` added | **Necessary and correctly implemented.** The plan genuinely omitted the discharge. Gap is the missing site (Finding 3), not the mechanism. |
| 3 | No `PartialEq`/`Eq` on `PendingTriggerTargets` | Correct; `PendingTrigger` is SR-7-gated and nothing compares structurally. |
| 4 | TUI key `'n'` not `'t'` | Correct; `'t'` is taken. |
| 5 | 54 sentinels, not 53 | Fine; a discovery, honestly recorded. |
| 6 | Fuzzer A/B oracle falsified | **The right call and honestly stated.** An extra `Command` per non-forced trigger shifts `RandomBot`'s RNG stream, so trace divergence is structural, not a regression signal. The 8-seed measurement with a stated classification is the strongest evidence available given OOS-DP3-9 / OOS-M11-3. |

## Recommended Fix Ordering

1. Finding 1 (one-line; add the concede-mid-announcement regression test)
2. Finding 5 (unblocks the safety argument for 1)
3. Finding 3 (one guard; extend the §16 checklist to `check_and_flush_triggers`)
4. Findings 2 + 6 together (`max` on `TriggerTargetOption`, padding, per-slot duplicates; re-pin
   the protocol/hash fingerprints)
5. Finding 4
6. Finding 7 (tests for the remaining guard sites)
7. Findings 8, 9, 10 and Card Finding 4 opportunistically

---

## Fix-cycle dispositions (2026-07-26, `scutemob-156`, branch `feat/pb-dp8-…`)

Every finding is dispositioned below. Post-fix gates: `cargo build --workspace`,
`cargo test --all` (**3,871 / 0**, up from 3,858), `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs) — all clean.
**PROTOCOL 29 → 30, HASH 66 → 67**, both gate-computed, both `*_HISTORY` rows appended,
never edited; 54 scattered sentinels + 2 `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned.
**0 card-def source edits, 0 completeness flips** (unchanged).

Every fix has a probe that was **run** in the failing direction before the fix was
restored — the reverts and their failure texts are recorded per row.

### Engine findings

| # | Sev | Disposition |
|---|-----|-------------|
| 1 | HIGH | **FIXED.** `flush_sorted` now binds the resume answer positionally — `let this_head = if next_index == 1 { head_targets.take() } else { None };` at the top of the loop body, before every `continue` and before the target chain. `next_index == 1` **is** sufficient (the increment precedes it, and `head_targets` is `Some` only on the resume path, whose head is `sorted[0]`), and it is strictly stronger than the reviewer's minimum: it also covers the CR 603.2c once-per-turn `continue` at the top of the body, which the lazy `take()` leaked through too. Test `test_dp8_answer_is_bound_to_its_own_trigger_not_the_next_one`. **Fail-before run**: with the `let this_head` binding removed, `the answer belonged to Zapper A; Zapper B must not be placed with it`. The test drives the state change directly (`objects_mut`) rather than through a concede, on purpose: Finding 5's gate closes the concede route, so a concede-based probe would go trivially green. |
| 2 | HIGH | **FIXED.** `TriggerTargetOption` gains `max: u32` (`UpToN{count}`, else 1); the cardinality check is `submitted.len() <= slot.max` for an optional slot and `== 1` otherwise; a per-slot duplicate check was added (CR 601.2c "the same target can't be chosen multiple times for any one instance of the word 'target'"), which the `<= 1` cap had been satisfying by accident. This **is** a wire-type change: the gate recomputed both fingerprints and forced PROTOCOL 30 / HASH 67; nothing was hand-invented. Test `test_dp8_up_to_n_accepts_n_targets_not_one`, which reads the two counts off the real defs via `all_cards()` (SR-36) before exercising them. **Fail-before run** twice: with the cap back at 1 (`got 2 target(s); expected 0 to 4`), and with only the duplicate check removed. |
| 3 | MEDIUM | **FIXED.** The 31st `check_and_flush_triggers` site (`handle_all_passed`'s `force_resolve_overdue_payments` branch) carries the guard, marks `FlushResumeSite::GrantPriority`, and returns. The plan's §16 checklist gained a second, corrected mechanical check — `rg 'check_and_flush_triggers\(' crates/engine/src` with a per-site statement of what follows — plus a note saying *why* the original grep could not have found it (it greps the inner function, so it sees the definition and none of the callers). Test `test_dp8_overdue_payment_branch_grants_no_priority_while_suspended`. **Fail-before run**: `left: 1, right: 0` on the after-ask `PriorityGiven` count. |
| 4 | MEDIUM | **FIXED, and widened as the reviewer asked.** `grant_priority_on_resume: bool` became `resume_site: FlushResumeSite` — `None` / `GrantPriority` / `EnterStepPriority` / `EnterStepCleanup` — so `finish_resumed_flush` can reproduce each site's *own* obligation: the CR 726 mandatory-loop check for **both** `enter_step` branches (the half the seed did not name), and `cleanup_sba_rounds += 1` for the Cleanup one. Tests `test_dp8_suspended_cleanup_batch_still_advances_the_ratchet` and `test_dp8_resume_runs_the_cr726_loop_check` (the loop check's having run is observable: `ChooseTriggerTargets` resets the table per CR 104.4b and the resume re-populates it with exactly one position). **Fail-before run** for both. **One deliberate deviation**: the resume ratchets unconditionally rather than under `cleanup_sba_rounds < MAX_CLEANUP_SBA_ROUNDS`. Reproducing the `else` arm would mean re-entering `enter_step`'s auto-advance fall-through from `abilities.rs`, which it cannot do; the next non-suspending cleanup round makes that call, and the CR 726 check is the real bound on a genuinely repeating position. Recorded in the audit's OOS-DP8-10 row rather than re-seeded. |
| 5 | MEDIUM | **FIXED; seed OOS-DP8-9 CLOSED, not deferred.** `handle_concede`'s priority-advance and turn-advance blocks are gated on `blocking_decision(state).is_none()`. `drop_departed_trigger_flush` has already handled the conceder's own entry by that point, so anything still outstanding belongs to somebody else. The runner's hang argument is refuted in the source comment: the entry's player is alive by construction, and `finish_resumed_flush` grants priority itself (routing past a dead active player), so CR 800.4j is satisfied by an ordinary priority round instead of the shortcut. Test `test_dp8_foreign_concede_does_not_step_over_the_suspended_batch`. **Fail-before run**: the step advanced under the suspended batch (`left: BeginningOfCombat, right: PreCombatMain`). |
| 6 | MEDIUM | **FIXED.** New `flatten_slot_answers(slots, per_slot)` builds the flat `Vec<SpellTarget>` with each slot at its declared width; `handle_choose_trigger_targets`, the forced-choice path and `default_spell_targets` all route through it. **Deviation from the prescribed fix, stated because it matters**: `Vec<SpellTarget>` has no representation for a hole, so "pad each slot to its declared width" cannot be done literally without a new `Target` variant (a far larger wire change). Holes are filled with a documented placeholder, `SpellTarget::unchosen_slot()` = `Target::Object(ObjectId::SENTINEL)` — an id the counter never assigns, so `resolve_effect_target_list_indexed` and `is_target_legal` both contribute nothing for it, exactly as an out-of-range index already did. **Only interior holes are padded**; a trailing un-taken slot is omitted, so an all-empty announcement still yields an empty list and cannot trip CR 608.2b's "all targets are illegal" fizzle. Test `test_dp8_under_filled_optional_slot_does_not_shift_later_indices` (both halves). **Fail-before run**: `the declined slot keeps its one-wide position`. The mirror in `casting.rs`'s spell path is **not** fixed here and is filed as **OOS-DP8-11**. |
| 7 | MEDIUM | **FIXED — every remaining guard site now has a test.** `enter_step` Cleanup (T26), `combat.rs` declare-attackers (T29) and declare-blockers (T31), the `resolution.rs` tail (T28), `finish_resumed_flush`'s dead-active-player fallback (T30), and `handle_all_passed`'s payment branch (T25). Each was **fail-before-run** by disabling its own guard (`if false`) or its own branch. The has-priority guard keeps its existing T12 plus the new T27. |
| 8 | LOW | **FIXED.** `trigger_target_choice_is_forced` is now expressed through `trigger_target_slot_forced_answer`, which treats an `optional` slot with an empty candidate set as determined (its only legal answer is zero targets). Test `test_dp8_optional_slot_with_no_candidates_asks_nothing`. **Fail-before run**: it suspended. |
| 9 | LOW | **FIXED, by converging the state rather than the reads.** Neither prescribed option is quite right on its own: clearing the field at every elimination site means touching eight `has_lost = true` sites, and making the guards liveness-aware would let the game proceed while the batch stayed lost inside `flush_pending_triggers`' raw-field early return. Instead `flush_pending_triggers` **reaps** a departed owner's entry at its top, the CR 800.4d way (`drop_conceded_trigger_flush`, renamed `drop_departed_trigger_flush` since it is no longer concede-specific) — the one place the stale entry actually bites. The re-entrancy `debug_assert!` was narrowed so that an entry the reap itself just created is not mistaken for a re-entrance. Test `test_dp8_entry_of_a_player_eliminated_outside_concede_is_reaped`. **Fail-before run**: `CR 800.4d: a departed player's entry must not survive the flush`. |
| 10 | LOW | **FIXED.** `flush_sorted` tracks `placed_any` (set where a stack object is actually pushed) and resets `players_passed` on the suspend return as well as at the tail. `placed_any` rather than `!events.is_empty()` because on the suspend path `events` also carries the `TriggerTargetChoiceRequired` question, and *asking* is not a game action. The tail's own condition moved to the same flag — they were equal there, so no behaviour changed. Covered indirectly by the guard-site tests (T25/T26/T28/T29/T31 all assert the post-resume priority state). No dedicated fail-before probe: the flag is not independently observable at the suspend point without a state accessor this batch has no reason to add. |

### Card / oracle findings

| # | Sev | Disposition |
|---|-----|-------------|
| 1 | HIGH | **FIXED engine-side (Finding 2), no def edit.** `elder_deep_fiend` declares `UpToN { count: 4 }`; the test asserts that count off `all_cards()` and then that four targets are announceable. |
| 2 | HIGH | **FIXED engine-side (Finding 2), no def edit.** `cloud_of_faeries` declares `UpToN { count: 2 }`, asserted the same way. Its def comment's stated contract ("UpToN contributes its declared targets at consecutive indices starting where the prior requirement's indices end") is now what the engine actually does — Finding 6 made it true rather than aspirational. |
| 3 | LOW | **FIXED engine-side (Finding 6), no def edit.** `sword_of_sinew_and_steel`'s two parallel `UpToN{1}` slots no longer depend on both clauses being `DestroyPermanent`. |
| 4 | LOW | **FIXED.** `test-data/generated-scripts/stack/138_emerge_elder_deep_fiend.json`: both stale notes rewritten to say the cast trigger is on the stack, and `disputes[0]` **resolved** with the real history — the dispute's premise was wrong in both directions (the trigger *is* in the def; the engine dropped it, first by reading `UpToN`'s `None` as "no legal target" and then by capping the announcement at one). `resolved_by` / `resolved_date` filled. Script suite re-run: 43/43, 0 new skips (SR-9c). |

### New seeds filed by the fix cycle

- **OOS-DP8-11** (`docs/audits/decision-point-audit.md` §8.1) — the spell path in `casting.rs`
  still concatenates an under-filled `UpToN` slot, so Finding 6's index shift survives there.
  No `Complete` spell is exposed today (the motivating shape is a triggered ability).
- **OOS-DP8-12** — `SpellTarget::unchosen_slot()` is a real value in the target stream that no
  display surface (replay viewer, TUI) special-cases.

### Audit rows updated

`docs/audits/decision-point-audit.md` §8.1: **OOS-DP8-9** and **OOS-DP8-10** rewritten from
open seeds to **CLOSED**, each recording why the original disposition was wrong and what the
residual is. Two new rows appended. `memory/primitives/pb-plan-DP8.md` §16 corrected.

---

## Closing-review dispositions (2026-07-26, `scutemob-156`, second fix cycle)

The closing `/review` (independent Opus, read-only) passed all 5 acceptance criteria and filed
**1 HIGH, 1 MEDIUM, 6 LOW**. All 8 are dispositioned below; **8 fixed, 0 deferred**.

Post-cycle gates: `cargo build --workspace`, `cargo test --all` (**3,875 / 0**, up from 3,871),
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (1,804 defs) — all clean. **PROTOCOL 30 / HASH 67 unmoved** (no wire
type changed: the only new engine data is a *value* choice, `make_distinct_slot_defaults`).
**0 card-def source edits, 0 completeness flips** (unchanged). Golden corpus 210 approved,
0 new skips (SR-9c); script 138 re-run green.

Every fix has a probe that was **run in the failing direction first** — the HIGH shipped
precisely because a test constructed the hazardous state and asserted the wrong thing about it.

| # | Sev | Disposition |
|---|-----|-------------|
| 1 | **HIGH** | **FIXED.** The previous cycle's Finding-5 gate (`handle_concede`'s priority/turn advance gated on `blocking_decision().is_none()`) stranded `priority_holder` on the conceded player whenever the suspension's `resume_site` was `FlushResumeSite::None` — the site of all 30 in-match `check_and_flush_triggers` calls. The gate's comment claimed "the resume grants priority itself"; `finish_resumed_flush` returns at its `owed == None` branch without touching the field. **The reviewer's suggested fix was evaluated and deliberately not applied at the concede site**: granting there emits `PriorityGiven` *while the CR 603.3b batch is still half-placed*, which is the exact invariant Findings 3/5 exist to protect, and `next_priority_player` can legitimately return `None` (everyone else already passed) leaving no answer at all. Instead the debt is discharged at the end of `resume_trigger_flush` by the new `abilities::repair_departed_priority_holder` — the earliest moment CR 603.3b permits a grant, and a single choke point that catches *every* route to a departed holder, not just concede. Successor = next player in APNAP order who has not passed; if all have, fall through to the CR 603.3b active-player grant (`grant_priority_after_batch`, factored out of `finish_resumed_flush` so the two cannot drift). It is a no-op when the batch re-suspends, when the holder is alive, and for every non-`None` resume site (those already grant). Deliberately **not** wired into `drop_departed_trigger_flush`: `handle_concede` runs its own now-ungated advance straight after, and the reap's caller either holds correct priority already or grants it itself. `engine.rs`'s false comment is replaced with the correction. Tests: the missing assertion added to `test_dp8_foreign_concede_does_not_step_over_the_suspended_batch`, plus the new end-to-end `test_dp8_concede_under_a_suspended_batch_does_not_strand_priority` (drives all four steps and then proves the game is still playable: the holder can pass, the conceded seat gets `PlayerEliminated`, everyone else `NotPriorityHolder`). **Fail-before run**: both tests panicked `left: PlayerId(2), right: PlayerId(2)` on "not the conceded player". |
| 2 | MEDIUM | **FIXED.** `flush_pending_triggers` now zeroes the reaped entry's `resume_site` before calling `drop_departed_trigger_flush`, so the reap no longer discharges a debt inside the caller's own flush. Principle recorded in the source: *the debt belongs to a call site whose moment has passed; the current caller's own obligation is what is owed now* — the six guards grant themselves the moment the flush returns with no entry, and the 30 `check_and_flush_triggers` sites already hold correct priority (PB-DP1), where a grant to the ACTIVE player would have been an overwrite, not a duplicate. New test `test_dp8_reap_does_not_double_grant_priority_at_a_guarded_site` drives the reap from `enter_step`'s has-priority guard (the site the reviewer asked for: an owner eliminated by a CR 704.5a SBA, not by `handle_concede`). **Fail-before run**: `left: 2, right: 1` on the `PriorityGiven` count for one step entry. **Residual, seeded as OOS-DP8-13**: the same zeroing drops the *other* two obligations `FlushResumeSite` carries (the `EnterStepCleanup` ratchet bump and the CR 726 loop check). A merge rule rather than a choose-one rule is the proper fix; the grant half is the half that deadlocks, and it is the half pinned. |
| 3 | LOW | **FIXED, both halves.** The code was given the property, *and* the doc comment was corrected — not one or the other. `make_distinct_slot_defaults` runs over the freshly-built slot list in `flush_sorted` and, for the pairs `handle_choose_trigger_targets`' check (8) actually examines (two `TargetPermanentDistinctFrom` slots), moves a colliding default to the first candidate not already taken. Scoped to that requirement only: CR 601.2c forbids repeats within ONE instance of the word "target", not across two, so two ordinary `TargetCreature` slots keep the pre-PB-DP8 first-match value and the determinism pin (T13) is untouched. `default_trigger_targets` gains an explicit "Acceptance guarantee, and its one exception" section naming the single residual (no second candidate exists ⇒ still refused; a genuine CR 603.3d "no legal choices" question, OOS-DP8-4) rather than leaving a comment asserting a property the code lacks — OOS-DP7-2's failure mode. New test `test_dp8_default_answer_satisfies_cross_slot_distinctness` asserts both that the default is distinct and, the real contract, that `process_command` accepts it. **Fail-before run**: `left: [Object(ObjectId(1))], right: [Object(ObjectId(1))]`. |
| 4 | LOW | **FIXED.** The pump-skip predicate is extracted as `next_action_answers_the_block(steps, step_idx, action_idx, &decision)`: it matches the wanted action string off the outstanding `BlockingDecision` variant, and it searches forward across step boundaries (skipping action-less steps) instead of reading only `step.actions[action_idx + 1]`. New unit test `test_pump_skip_is_cross_step_and_kind_aware` covers same-step, cross-step-with-an-empty-step-between, both kind-mismatch directions, and end-of-script. **Fail-before run twice**: restoring the step-local predicate fails the cross-step assertion; making it cross-step but kind-blind fails as well. Seeded forward as **OOS-DP8-14** because PB-DP9 adds three more answering action strings to the same predicate. |
| 5 | LOW | **FIXED.** `138_emerge_elder_deep_fiend.json`: the `stack_resolve` step's two notes said the cast trigger "resolves after" the spell and "is still on the stack", both backwards — the trigger sits ABOVE the spell (as the `priority_round` note at `:172` correctly says) and therefore resolved in the preceding round, which is why *this* resolution empties the stack. Both notes rewritten; the stack-count and priority-round edits themselves were CR-argued and correct and are untouched. Script re-run: green, 0 new skips. |
| 6 | LOW | **FIXED.** `pb_dp5_pending_draw_choice.rs`'s sentinel comment now records **both** PB-DP8 bumps (66/29 at implement close, 67/30 in the fix cycle when Finding 2 put `max: u32` on `TriggerTargetOption`) and says the assertions below are the live values. |
| 7 | LOW | **FIXED, and the rewrite is proven non-vacuous.** The old test hashed `state`, called `process_command(state.clone(), ..)`, and compared against `state` — the copy that was never passed in. It now drives `handle_choose_trigger_targets(&mut state, ..)` directly, once per rejection class (legality ×2, wrong sender, stale `choice_id`, slot count, per-slot cardinality), re-hashing after each, then confirms the block is still answerable and that an ACCEPTED answer *does* move the hash — so the pin cannot pass for the wrong reason. **Probe run**: inserting `state.turn.turn_number += 1;` at the top of the handler makes it fail with the two hashes printed. This is the test for ESM criterion 5545. |
| 8 | LOW | **FIXED.** `docs/audits/decision-point-audit.md` §8's PB-DP8 row: "the real answer is **four guards**" → **six** (with the arithmetic stated: four at implement close, `handle_all_passed`'s payment branch added by the fix cycle, and the plan's own count treating `enter_step` as one site), and `grant_priority_on_resume` → `resume_site: FlushResumeSite` with the reason it widened. The row also gains a closing-review paragraph whose three transferable rules are (i) when a guard skips work, enumerate what that work was going to do and say where each part is picked up; (ii) a test that constructs a hazardous state and does not assert against it is worse than no test; (iii) a debt-carrying pending entry needs a merge rule, not a choose-one rule. |

### Audit rows updated by this cycle

`docs/audits/decision-point-audit.md`: §8 PB-DP8 row (both factual corrections + closing-review
paragraph, tests → 3,875); §8.1 **OOS-DP8-9** amended — the "the resume grants priority itself"
half of its closure argument was wrong and is now recorded as such, with the real closure named;
§8.1 **OOS-DP8-4** narrowed (the colliding-default half is closed, the no-alternative-candidate
half is the residual); two new rows **OOS-DP8-13** (reaped debt dropped rather than merged) and
**OOS-DP8-14** (harness pump-skip predicate keyed on vocabulary rather than on the decision).
