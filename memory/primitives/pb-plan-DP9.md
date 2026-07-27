# Primitive Batch Plan: PB-DP9 — Search / scry / surveil become player choices (CR 608.2d)

**Generated**: 2026-07-27
**Task**: `scutemob-157` · branch `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`
**Primitive**: the engine's **first resolution-time decision channel** —
`GameState.pending_effect_choice` + `GameEvent::EffectChoiceRequired` →
`Command::AnswerEffectChoice`, backed by an **abort-and-restart** continuation for
`resolve_top_of_stack` rather than a resumable effect-list cursor.
**Findings**: DP-7 / DP-8 / DP-9 (`docs/audits/decision-point-audit.md` §4.9, §5 Tier 1 rows,
§8 row PB-DP9)
**CR Rules**: **608.2d** (the unifying rule), **701.23a–j**, **701.22a–d**, **701.25a–d**,
401.4, 401.7, 400.7, 101.4, 608.2c/e/f/m, 104.3a, 800.4j, 121.3
**Class**: AGENCY (all three are audit class **B**, not D — see §9 item 5) with two genuine
correctness residuals surfaced in planning (§9 items 8 and 12).
**Cards affected**: audit claims **74 / 16 / 8 = 98** `Complete` defs. **Unverified** — the
roster must be enumerated from `all_cards()` per SR-36 (§5). Grep on this branch finds **100
files** with `Effect::SearchLibrary`, **20** with `Effect::Scry`, **9** with `Effect::Surveil`
(all completeness levels). **Predicted card-def source edits: 0. Predicted completeness flips: 0.**
**Wire**: **PROTOCOL 30 → 31** and **HASH 67 → 68**, both expected, both with a stated falsifier (§4).
**Baseline**: PROTOCOL **30** (`rules/protocol.rs:286`), HASH **67** (`state/hash.rs:636`),
tests **3,878**.
**Dependencies**: PB-DP7 (`BlockingDecision`, `blocking_decision()`, the `process_command`
admission gate, `GameState::blocking_decision()`), PB-DP8 (the `LegalAction` /
`DecisionKind` / `LocalGame` / TUI / **replay-harness pump** shape, and every lesson in its
audit row), PB-RS1 (`Zone::top_n`, `move_object_to_bottom_of_zone`), PB-DP1 (priority-to-actor),
M11-local S1 (`LocalGame`).
**Deferred items carried in**: **OOS-DP8-14** (the harness pump-skip predicate is keyed on
action vocabulary — this batch adds exactly **one** new action string, which is part of why the
command is unified; §2.1). **OOS-DP8-6** (`GameEvent::private_to()` does not exist) is
**closed by this batch**, deliberately — §2.4. **OOS-DP5-5** (a deferred draw does not suspend
the rest of its effect) is **not** closed and is explicitly out of scope; §9 item 1 explains
why this batch's mechanism does not generalise to it for free.

---

## 0. Executive summary — the design decision

`pb-plan-DP7.md` §1.6 offers three options and says (a) — "a resumable effect-list cursor on
the stack object" — is the real answer. **Option (a) as written is impossible, and the
replacement is better than any of the three.**

**(a) is impossible because there is no stack object.** `resolve_top_of_stack`
(`crates/engine/src/rules/resolution.rs:36`) **pops** the object at `:39-42`, before a single
effect runs. Nothing that lives on the stack object can carry a continuation for effect
execution. (§9 item 1.)

**The design this plan adopts: suspension is an ABORT, not a pause.**

> `resolve_top_of_stack` clones the state at entry. When an effect needs a choice it records
> the *question* and returns without doing anything else. The wrapper **restores the clone
> wholesale** — the stack object is back, no card has moved, no event has happened — records
> the pending entry on that restored state, and returns exactly one event: the question. When
> the answer arrives, the answer is appended to a per-resolution **answer bank** and
> `resolve_top_of_stack` is called **again, from the top**. Execution is deterministic, so it
> retraces the identical path, and when it reaches the choice point it consumes the banked
> answer instead of asking. A second choice in the same resolution suspends again, banks a
> second answer, and replays again.

Five consequences, each of which is why this beats a cursor:

1. **There is no continuation data structure.** No path cursor, no per-frame `remaining`, no
   serialized `EffectContext`, no `Effect` residual. `Sequence`, `Conditional`, `ForEach`,
   `Repeat`, `MayPayThenEffect`, and the multi-player `for p in resolve_player_target_list(..)`
   loops **inside** all three effects are all handled with zero machinery, because the replay
   re-executes them. The brief's "second, inner dimension of suspension" costs nothing.
2. **The re-entrancy audit is three units, not twenty** (§1.3). All **15** `execute_effect`
   call sites in `resolution.rs` are inside the single function `resolve_top_of_stack`
   (`:36`–`:7828`). The only other production callers are `rules/mana.rs:864` (a triggered
   mana ability — **gated**, CR 605.1b argument) and `rules/replacement.rs:1964` (a hardcoded
   `Effect::CreateToken` — **provably unreachable**).
3. **PB-DP8's hardest recurring bug class does not exist here.** DP-8 shipped, then re-fixed
   twice, a *debt-discharge* bug: a guard that returns early inherits the obligation of the
   statements it skipped (`FlushResumeSite`, `repair_departed_priority_holder`, OOS-DP8-13).
   A total state restore has **no debt**: the post-suspension state is byte-identical to the
   pre-resolution state, so nothing was skipped. `handle_all_passed`'s two statements after
   `resolve_top_of_stack` (`engine.rs:2119` `maybe_clear_lki_objects`, `:2124` `is_game_over`)
   are provable no-ops on a restored state, so **zero guard sites outside the wrapper**.
4. **"The object whose resolution is suspended leaves the stack" is structurally unreachable**
   (the brief's fifth exit). The object is put *back*; the admission gate admits only the
   answer and `Concede`; and the answer re-derives everything from the restored state. With a
   cursor design this is a live hazard; here it is a theorem.
5. **The internal poison checks are an optimisation and a robustness measure, not a
   correctness requirement** (§3). Missing one cannot corrupt state — it can only waste work
   or reach a panic. This inverts DP-8's §4 problem, where a missed guard granted priority
   mid-batch.

The price, stated honestly: the resolution runs **k+1 times** for a resolution containing *k*
choices (k is 1 or 2 in the whole corpus), one extra `GameState` clone per stack resolution,
and a hard dependency on the engine's determinism (SR-9b) that is now *load-bearing at
runtime*, not just in tests. §6 and §10 own that.

**Second decision, and the one a reviewer should weigh hardest: the deterministic defaults
are NOT status-quo-preserving for scry and surveil.** Search keeps its lowest-`ObjectId` pick
(zero churn). Scry and surveil default to the **identity** answer — keep everything on top,
order unchanged — instead of today's "bottom everything" / "mill everything". Argued in §2.5,
with the DP-8 precedent that points the other way stated and answered.

---

## 1. The continuation mechanism

### 1.1 The `GameState` shape change

Three new fields. `crates/engine/src/state/mod.rs`, beside `pending_trigger_targets` (`:158`),
all `pub(crate)` with read-only accessors and **no** `_mut` accessor (SR-3):

```rust
/// CR 608.2d (PB-DP9 / DP-7/8/9): the one resolution-time choice the engine is
/// currently blocked on, if any. Recorded on a state that has been RESTORED to
/// the moment before `resolve_top_of_stack` began, so nothing of the aborted
/// resolution survives it. See `rules::engine::blocking_decision`.
#[serde(default)]
pub(crate) pending_effect_choice: Option<PendingEffectChoice>,

/// CR 608.2d: answers already given for THIS resolution, in the order the
/// engine asked for them. Consumed positionally by the replay (PB-DP8's
/// HIGH-1 lesson: bind positionally, never lazily). Cleared when a resolution
/// completes, aborts with an error, or is abandoned.
#[serde(default)]
pub(crate) effect_choice_answers: Vector<AnsweredEffectChoice>,

/// Monotone source of `choice_id` moment guards. Deliberately SEPARATE from
/// `timestamp_counter`: `timestamp_counter` seeds shuffles and coin flips and
/// is consumed by `next_object_id`, so bumping it between an abort and its
/// replay would change the replay's execution. Nothing but the recorder reads
/// this field.
#[serde(default)]
pub(crate) next_choice_id: u64,
```

New types in `crates/card-types/src/state/stubs.rs` (the home `PendingCleanupDiscard` and
`PendingTriggerTargets` already use):

```rust
/// CR 608.2d (PB-DP9): the suspended resolution-time choice.
///
/// Reachable from `GameEvent::EffectChoiceRequired`, so `EffectChoiceQuestion`
/// IS in the SR-8 wire closure. `PendingEffectChoice` itself is reachable only
/// from `GameState` (the `PendingCleanupDiscard` / `PendingTriggerTargets`
/// precedent) and contributes nothing to `PROTOCOL_SCHEMA_FINGERPRINT`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingEffectChoice {
    /// The moment guard. Must be echoed by `Command::AnswerEffectChoice`.
    pub choice_id: u64,
    /// CR 608.2d: the player the effect names, and the ONLY player who may answer.
    /// NOT necessarily the resolving object's controller ("each player scries 1").
    pub player: PlayerId,
    /// The question, with its full legal answer space.
    pub question: EffectChoiceQuestion,
    /// The 0-based index of this choice within the current resolution — i.e.
    /// `effect_choice_answers.len()` at the moment of the abort. The replay
    /// compares it, so an answer can never be applied to a different choice.
    pub index: usize,
}

/// CR 608.2d: an answer already given, paired with the question it answered.
/// The replay asserts the recomputed question equals `question` before consuming
/// `answer` — a mismatch is a determinism violation, i.e. an engine bug (SR-4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnsweredEffectChoice {
    pub question: EffectChoiceQuestion,
    pub answer: EffectChoiceAnswer,
}
```

**Falsifier for the HASH bump**: none. `process_command(state: GameState, …) -> Result<(GameState, …)>`
(`engine.rs:209-212`) is the only carrier between two commands, so the answer bank has nowhere
else to live. Deriving "is a choice pending" from the board is impossible — the whole point is
that the resolution has been rolled back and looks exactly like a resolution that has not
started.

### 1.2 The abort wrapper

`crates/engine/src/rules/resolution.rs`. Rename the existing 7,800-line body to
`resolve_top_of_stack_inner` **unchanged** except for the §3 poison checks, and add:

```rust
/// CR 608.1 / CR 608.2d (PB-DP9). Resolve the top of the stack, with one
/// exception: if an effect needs a CR 608.2d choice that has not been answered,
/// the ENTIRE resolution is rolled back to the moment before it began, the
/// question is recorded on `GameState.pending_effect_choice`, and the only event
/// returned is `GameEvent::EffectChoiceRequired`.
///
/// The roll-back is a whole-state restore, not a field-by-field undo. That is
/// what makes the suspension free of the "who discharges the debt the guard
/// skipped" problem PB-DP8 met three times (audit §8, transferable rules i/iv):
/// nothing was skipped, because nothing happened.
pub fn resolve_top_of_stack(state: &mut GameState) -> Result<Vec<GameEvent>, GameStateError> {
    debug_assert!(state.pending_effect_choice.is_none(),
        "resolve_top_of_stack re-entered while a CR 608.2d choice is outstanding");
    let restart_point = state.clone();
    let result = resolve_top_of_stack_inner(state);
    match result {
        Ok(events) => {
            if let Some(pending) = state.pending_effect_choice.clone() {
                // ROLL BACK. This also discards any DP-8 trigger suspension the
                // inner pass reached after the aborted effect (§1.5).
                *state = restart_point;
                state.next_choice_id = state.next_choice_id.wrapping_add(1);
                let q = GameEvent::EffectChoiceRequired { /* from `pending` */ };
                state.pending_effect_choice = Some(pending);
                Ok(vec![q])
            } else {
                // A completed resolution ends the answer bank's life.
                state.effect_choice_answers = Vector::new();
                Ok(events)
            }
        }
        Err(e) => {
            // `process_command` discards the mutated state on `?`, so no restore is
            // owed; clear the flag so a stale question cannot escape on a path that
            // re-uses the state (the harness and `LocalGame` both do).
            state.pending_effect_choice = None;
            state.effect_choice_answers = Vector::new();
            Err(e)
        }
    }
}
```

**`choice_id` must come from `next_choice_id`, taken on the RESTORED state, and nothing else.**
Taking it from `timestamp_counter` would either (a) be read from the aborted probe, which is
discarded, or (b) bump the counter on the restored state, which changes the seeds every shuffle
and `next_object_id` in the replay consumes — silently breaking the replay's determinism
premise. This is the single subtlest trap in the batch.

### 1.3 The `execute_effect` caller re-entrancy audit (the brief's centre of gravity)

Complete production caller set on this branch
(`rg 'execute_effect\(' crates/*/src` — 20 hits, of which 2 are the definition and its inner
call):

| # | caller | file:line | verdict | argument |
|---|---|---|---|---|
| 1–15 | fused split L/R (`:220`, `:225`); modal / per-mode (`:492`, `:500`, `:506`); splice (`:530`); ability + trigger resolution (`:1926`, `:1948`, `:1967`, `:2203`, `:2296`); `:5377`; `:7386`; `:7454`; Rooms (`:7561`) | `rules/resolution.rs` | **RESUMABLE** | **All fifteen are inside `resolve_top_of_stack`** (`:36`–`:7828`; the next `fn` is `execute_gift_effect` at `:7829`). Verified by function-boundary grep. One restartable unit, one wrapper, one resume entry point. |
| 16 | triggered mana ability (CR 605.4a) | `rules/mana.rs:864` | **GATED** | Reached only when `targets.is_empty() && is_mana_producing_effect(effect)` (`:858`). CR 605.1b: an ability is a mana ability only if it *could* add mana and doesn't target; CR 605.4a resolves it immediately, outside the stack, and there is no object to roll back to. A `Sequence([AddMana, Scry])` could in principle pass `is_mana_producing_effect`, so the gate is a runtime fact, not a type fact: **the three effect arms use the default when `state.effect_choice_gate_closed` is set** (a plain `bool` on `EffectContext`, not on `GameState` — it never crosses a command boundary). `mana.rs` sets it. A `debug_assert` records if it ever fires, and a roster test asserts no `Complete` def puts one of the three effects inside a mana ability. |
| 17 | Servo-token fallback for a modular-style replacement | `rules/replacement.rs:1964` | **PROVABLY UNREACHABLE** | The effect argument is a literal `&Effect::CreateToken { spec: servo_spec }` constructed two lines above (`:1944-1968`). No `Effect::Scry`/`Surveil`/`SearchLibrary` can reach it. |
| — | `execute_effect_inner` self-recursion: `Sequence` `:3299`, `Conditional` `:3308`/`:3310`, `ForEach` players `:3359` / objects `:3400`, `Repeat` `:3410`, `Choose` `:3416`, `MayPayOrElse` `:3421`, `MayPayThenEffect` `:3455`, `CounterUnlessPays` `:3468`, coin-flip `:3987`/`:3989`, `:4016` | `effects/mod.rs` | **RESUMABLE, no machinery** | The replay re-executes them from the top. This is the nesting story: **there is nothing to design.** §3 adds a one-line poison check at each for robustness only. |
| — | 237 direct `execute_effect(` calls in `crates/engine/tests` across 67 files | tests | **fallout, bounded** | Only those exercising the three effects are affected; §8.1 names the 9 candidate files. |

**Nesting story, stated explicitly because the brief asks for it.** A `SearchLibrary` inside
`ForEach::EachPlayer` inside a `Conditional` inside a `Sequence`, with three players each
searching, produces: abort → question(P1) → answer → replay → the `Sequence` re-runs, the
`Conditional` re-evaluates (same state ⇒ same branch), the `ForEach` re-collects (same state ⇒
same list), P1's search consumes bank[0], P2's search aborts → question(P2) → … Three commands,
three replays, no cursor, and the `Conditional`'s branch and the `ForEach`'s collection are
re-derived rather than remembered — which is *correct*, not merely convenient, because they are
pure functions of a state that has not changed (§1.4).

### 1.4 Why the replay is sound, and what would falsify it

Sound iff execution is a deterministic function of `(GameState, Command)`. Load-bearing facts,
each verified on this branch:

- `state.objects` is an `imbl::OrdMap` (ascending `ObjectId`); `state.zones` entries are
  `imbl::Vector`; `state.players` is an `OrdMap`. No `HashSet`/`HashMap` iteration order feeds
  any of the three effects' candidate derivation.
- All randomness is seeded from `state.timestamp_counter` (`effects/mod.rs:3039`, `:3138`, the
  coin-flip/dice arms at `:3981-4014`), which the restore rolls back.
- Between the abort and the replay, `process_command`'s admission gate (§2.6) admits only the
  answering command and `Concede`. `Concede` **does** mutate — that is exit 2 in §1.5, and it
  is handled by *abandoning the bank*, not by trusting it.
- The three new `GameState` fields are read by exactly three places: the three effect arms
  (bank), the recorder (`next_choice_id`), and `blocking_decision`. **No effect reads them**,
  so the replay's starting state is execution-equivalent to the original.
- **Exception, and it must be handled**: `rules/loop_detection.rs`'s mandatory-state
  fingerprint. PB-DP7 and PB-DP8 both folded their pending field into it. **PB-DP9 must NOT**
  — the entry and the bank grow between replay *k* and replay *k+1*, so including them would
  make two structurally identical positions fingerprint differently and could silently mask a
  CR 726 mandatory loop. Deviation from precedent, deliberate, argued here, and pinned by a
  test. `public_state_hash` **does** include all three (they are real state; SR-19's gate
  requires every field of a hashed struct).

**`EffectContext.target_remaps` is a `std::collections::HashMap`** (`effects/mod.rs:189`). If
any code path iterated it in a way that affects outcomes, the replay could diverge *across
processes* (Rust's default hasher is per-process randomised). Within one process it is stable,
so the batch is safe today; but the runner must **grep every read of `target_remaps` and
confirm none iterates**, and seed the finding if one does. This is an SR-9b hazard that
predates the batch and that the batch makes runtime-relevant.

**Mismatch handling.** On replay, before consuming `bank[i]`, the effect arm recomputes the
question and compares it to `bank[i].question`. Equal ⇒ consume. Unequal ⇒ a determinism
violation, i.e. an engine bug: `debug_assert!` with a diagnostic naming both questions
(`state::diagnostics` `expect_` vocabulary, SR-4), and in release **truncate the bank at `i`
and re-ask** — which cannot hang (the new question is offered to a live player) and cannot
corrupt (the resolution is aborted again). A `PendingEffectChoice.index` guard plus a bounded
`MAX_EFFECT_CHOICES_PER_RESOLUTION` (suggest 64) prevents an ask/re-ask cycle from becoming
unbounded; exceeding it applies defaults for the remainder and emits a diagnostic.

### 1.5 Every exit from the block, and who discharges it (PB-DP8 transferable rule iv)

The state at the block is: the stack object is on the stack, `priority_holder == None`,
`players_passed` full, `pending_effect_choice = Some(..)`.

| exit | who clears | who resumes the game | argument |
|---|---|---|---|
| **1. Answered** | `handle_answer_effect_choice` | it calls `resolve_top_of_stack` itself, which grants priority at its own tail (`resolution.rs:7802-7805` neighbourhood) and flushes triggers (`:7799`). Then it mirrors `handle_all_passed`'s two post-statements (`maybe_clear_lki_objects`, `is_game_over` → `check_game_over`). | The only debt in the batch, and it is two lines. Factor them into a shared `finish_stack_resolution(state, &mut events)` called from **both** `handle_all_passed:2116-2127` and the answer handler, so they cannot drift. |
| **2. The entry's own player concedes** | `handle_concede`, in the same block that clears `pending_cleanup_discard` / calls `drop_departed_trigger_flush` | clear the entry **and the whole bank**, then call `resolve_top_of_stack`; the effect arms see a dead player (`has_lost \|\| has_conceded`) and use the **default** without asking (the DP-8 §8.1 case-1 pattern). Resolution completes; its tail grants priority. Then `finish_stack_resolution`. | Without this the game deadlocks: `priority_holder` is `None`, nobody can pass, and nothing else drives `handle_all_passed`. **The bank must be dropped, not kept**: the concede mutated the board, so banked answers are answers to questions that may no longer be the ones asked. |
| **3. Another player concedes** | nobody — the entry survives | nobody — the block persists, correctly | PB-DP8's obligation 5 gate (`handle_concede` refuses to advance priority/turn while `blocking_decision(state).is_some()`) already covers this and **generalises for free** because it reads the predicate, not the field. Verify by reading; do not re-implement. |
| **4. The entry's player is eliminated by an SBA, not a concede** | a **reap** at the top of `resolve_top_of_stack` and in `blocking_decision`'s liveness filter | the same helper as exit 2 | Unreachable while blocked (no SBA runs — the admission gate rejects everything). Defended anyway: this is exactly DP-8's Finding 9 / `drop_departed_trigger_flush`. Call the discharge helper from the **end of `handle_concede`**, next to `repair_departed_priority_holder` — the placement DP-8's *second* closing review earned. |
| **5. The suspended object leaves the stack** | — | — | **Structurally unreachable.** The object was never removed (the restore put it back) and no admitted command removes stack objects. State this in the source comment with the argument, not as an assertion. |
| **6. The game ends while blocked** | `check_game_over` | — | `process_command` answers `GameAlreadyOver` thereafter. The entry is inert. Clear it in `check_game_over` anyway so it does not pollute the terminal hash. |

`blocking_decision` (`rules/engine.rs:173`) gains a third lookup. **Order: `pending_trigger_targets`
→ `pending_effect_choice` → `pending_cleanup_discard`**, with the argument that the first two are
mutually exclusive by construction (a suspended CR 603.3b flush means no resolution has begun;
an effect choice is recorded on a state whose flush has not run) and the restore in §1.2 is what
guarantees it — the inner pass *can* set both, and the whole-state restore is what un-sets the
trigger one. Write that argument in the code, because it is not obvious.

---

## 2. The wire shapes

### 2.1 One command, not three — the CR argument

**CR 608.2d** is the rule all three implement, verbatim:

> *"If an effect of a spell or ability offers any choices other than choices already made as
> part of casting the spell, activating the ability, or otherwise putting the spell or ability
> on the stack, the player announces these while applying the effect. The player can't choose
> an option that's illegal or impossible…"*

CR 701.22a / 701.23a / 701.25a are three *instances* of one rule, not three rules. The CR gives
no reason to serialise them differently, and it does give a reason to unify: the announcement's
*timing* ("while applying the effect"), its *validity condition* ("can't choose an option that's
illegal or impossible") and its *actor* are identical across all three. So:

```rust
/// CR 608.2d (PB-DP9): the player's answer to an outstanding resolution-time choice.
///
/// `choice_id` must equal the outstanding entry's — the MOMENT guard (PB-DP7 lesson 2,
/// PB-DP8 §8.2). The engine does not trust any positional information in `answer`
/// beyond identity: every id is re-checked against the question the engine itself
/// recorded, never against the wire.
AnswerEffectChoice {
    player: PlayerId,
    choice_id: u64,
    answer: EffectChoiceAnswer,
},
```

Three engineering consequences that follow from the CR argument rather than from taste, and
which the reviewer should check:
- **one** admission-gate allow-list entry, **one** `LegalAction`, **one** `DecisionKind`, **one**
  `BlockingDecision` variant, **one** harness action string (`"answer_effect_choice"`), **one**
  `eventFormat.js` case;
- **OOS-DP8-14** predicted "PB-DP9 adds three more answering action strings" to
  `script_replay.rs::next_action_answers_the_block`. It adds **one**. Record the correction.
- DP-16 (edicts/discard), DP-17 (proliferate), DP-25 and every other §4.9 row become an
  `EffectChoiceAnswer` variant and **zero** new plumbing. That reuse is the batch's real
  deliverable and is why the unified shape is worth arguing for.

### 2.2 The question and answer payloads

```rust
/// CR 608.2d (PB-DP9): the outstanding resolution-time question, with its full
/// legal answer space, so a client can render a picker without a second query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectChoiceQuestion {
    /// CR 701.23a: every card in the searched zone(s) that matches the effect's
    /// filter, in ascending `ObjectId` order (the order `state.objects`, an
    /// `OrdMap`, yields — the same order the pre-PB-DP9 `min_by_key` scanned).
    SearchLibrary {
        candidates: Vec<ObjectId>,
        /// CR 701.23b vs 701.23d: `true` iff the effect's `TargetFilter` states a
        /// quality, in which case the player may legally decline to find even
        /// though a match exists. `false` for an unrestricted "search for a card",
        /// where CR 701.23d makes finding MANDATORY. See §2.3.
        may_fail_to_find: bool,
    },
    /// CR 701.22a: the top N, **top-first** (`Zone::top_n`'s own order).
    Scry { looked_at: Vec<ObjectId> },
    /// CR 701.25a: the top N, **top-first**.
    Surveil { looked_at: Vec<ObjectId> },
}

/// CR 608.2d (PB-DP9): the answer. Every variant's legality is checked against the
/// engine's own recorded question (§2.7), never re-derived from the board.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectChoiceAnswer {
    /// CR 701.23a/b: the found card, or `None` to fail to find.
    SearchLibrary { found: Option<ObjectId> },
    /// CR 701.22a. `bottom` and `top` PARTITION the question's `looked_at`
    /// (union equal, no duplicates, nothing else). `top` is top-first: after the
    /// effect, `top[0]` is the library's top card. `bottom` is also top-first
    /// among the bottomed cards: `bottom.last()` ends up bottom-most.
    Scry { bottom: Vec<ObjectId>, top: Vec<ObjectId> },
    /// CR 701.25a. `graveyard` + `top` partition `looked_at`; `top` is top-first.
    /// `graveyard` order is the order the cards are put there (CR 608.2f: the
    /// controller chooses the relative order of same-controller actions).
    Surveil { graveyard: Vec<ObjectId>, top: Vec<ObjectId> },
}
```

**Ordering contract, pinned against PB-RS1 and verified against the code on this branch.**
`Zone::top_n(n)` returns top-first; the library's **top is the last element** and its **bottom
is index 0** (`move_object_to_bottom_of_zone` = `push_front`; `move_object_to_zone` appends).
So:
- `bottom`: iterate **in order**, `push_front` each ⇒ `bottom.last()` finishes at index 0
  (bottom-most). ✅
- `top`: iterate **in reverse**, append each ⇒ `top[0]` finishes last (top-most). ✅

**Do not implement the `top` reordering with `move_object_to_zone`.** It creates a fresh
`ObjectId` unconditionally (`state/mod.rs:1739-1740`, and the same at `:1739` in
`move_object_to_bottom_of_zone`) — a same-zone "move" renumbers the card, which is a CR 400.7
deviation (a card that stays in the library does not become a new object) and which today's
scry-to-bottom already commits (§9 item 8). Add a `Zone`-level permutation helper (e.g.
`Zone::reposition_within(&mut self, ids_top_first: &[ObjectId], to_bottom: &[ObjectId])`, or two
narrow `move_to_top_within` / `move_to_bottom_within` methods) that permutes the `Vector` and
touches neither `state.objects` nor `next_object_id`. Surveil's graveyard half is a **real** zone
change and keeps `move_object_to_zone`.

Fixing the renumber is a behaviour change to scry-to-bottom. It is in scope because this code is
being rewritten anyway, and the runner must enumerate the fallout (any test asserting post-scry
`ObjectId`s) rather than paper over it. If the fallout turns out to be large, **stop and flag** —
preserving the renumber and seeding it is an acceptable narrower outcome.

### 2.3 Fail-to-find — the exact CR carve-out (ESM criterion 5549)

The CR text, verbatim from MCP:

> **701.23b** — *"If a player is searching a hidden zone for cards with a stated quality, such
> as a card with a certain card type or color, that player isn't required to find some or all
> of those cards even if they're present in that zone."*
>
> **701.23c** — *"If a player is instructed to search a hidden zone for cards that match an
> undefined quality, that player may still search that zone but can't find any cards."*
>
> **701.23d** — *"If a player is searching a hidden zone simply for a quantity of cards, such as
> 'a card' or 'three cards,' that player must find that many cards (or as many as possible, if
> the zone doesn't contain enough cards)."*
>
> **701.23e** — *"If the effect that contains the search instruction doesn't also contain
> instructions to reveal the found card(s), then they're not revealed."*

Mapped onto the engine's one search shape, `Effect::SearchLibrary { player, filter, reveal,
destination, shuffle_before_placing, also_search_graveyard }` (`effects/mod.rs:2936-2943`):

| engine shape | rule | `may_fail_to_find` |
|---|---|---|
| `filter == TargetFilter::default()` — the all-permissive filter, i.e. "search your library for **a card**" | **701.23d** | **`false`** — finding is mandatory when a candidate exists |
| any other `filter` (a card type, subtype, name, colour, mana-value bound, basic/nonbasic, controller, …) | **701.23b** | **`true`** |
| `candidates.is_empty()` | 701.23d's "as many as possible" | no question is asked at all; the effect is a no-op, exactly as today |

`TargetFilter` derives `Default` (`card_definition.rs:3035`), so the predicate is literally
`*filter == TargetFilter::default()`. The runner must confirm `TargetController::default()` is
the unrestricted variant; if it is not, the predicate needs the field excluded.

**Two things the brief got wrong here, recorded in §9 (item 4):** (i) CR 701.23's carve-out is
*not* "except where the search reveals the card" — **CR 701.23e says revealing has no bearing on
finding**, and the engine ignores `reveal` entirely (`reveal: _` at `:2939`, a pre-existing gap
seeded below); (ii) the carve-out that *does* exist is CR 701.23d's quantity-only search.

**Zone nuance, deliberately not modelled**: CR 701.23b is scoped to a **hidden** zone. With
`also_search_graveyard: true` the search spans a hidden zone and a public one, and strictly the
player may decline to find from the library but must find from the graveyard. `Effect::SearchLibrary`
always searches the library, so `may_fail_to_find` is computed from the filter alone and the
graveyard nuance is a seeded residual (**OOS-DP9-5**), not a silent one.

### 2.4 `GameEvent::EffectChoiceRequired` and hidden information (Architecture Invariant 7)

`crates/engine/src/rules/events.rs`, appended at the **end** of the enum after
`TriggerTargetChoiceRequired` (`:1405-1416`). **Discriminant 131** — the current maximum in
`state/hash.rs`'s no-`_`-arm `GameEvent` match is **130** (`hash.rs:5447`).

```rust
/// CR 608.2d (PB-DP9 / DP-7/8/9): `player` must answer a resolution-time choice.
/// The engine BLOCKS — the resolution has been ROLLED BACK to the moment before
/// it began (the spell/ability is still on the stack, nothing has moved), and
/// `process_command` rejects every command except `Command::AnswerEffectChoice`
/// from `player` and `Command::Concede` — until the answer arrives.
///
/// HIDDEN INFORMATION (Architecture Invariant 7). Unlike
/// `TriggerTargetChoiceRequired`, every id here names a card in a HIDDEN zone:
/// the library candidates that matched a search filter (CR 401.2) or the top N a
/// player is looking at (CR 701.22a/701.25a). Knowing WHICH ids match is itself
/// hidden information even though no card identity is carried. Therefore
/// `reveals_hidden_info()` is `true` AND `private_to()` returns `Some(player)`.
///
/// Discriminant: 131.
EffectChoiceRequired {
    player: PlayerId,
    choice_id: u64,
    /// The resolving spell or ability's source object, for display.
    source_object_id: ObjectId,
    question: EffectChoiceQuestion,
},
```

**`GameEvent::private_to()` does not exist on this branch** — `rg 'private_to' crates/` returns
only two doc-comment mentions in `events.rs` (`:1400`) and CLAUDE.md's Architecture Invariant 7.
PB-DP8 filed that as **OOS-DP8-6**. The brief's constraint 3 requires it. **This batch adds it**,
narrowly:

```rust
/// Architecture Invariant 7: the seat this event may be broadcast to, if it is
/// private. `None` = public. The M10 network layer is the consumer; there is no
/// network layer today, so this is a declaration the engine can be TESTED against,
/// not an enforcement point. Closes OOS-DP8-6's "the invariant names a surface
/// that does not exist" half; the filter itself remains M10 work.
pub fn private_to(&self) -> Option<PlayerId> {
    match self {
        GameEvent::EffectChoiceRequired { player, .. } => Some(*player),
        // OOS-DP7-3(b): CleanupDiscardChoiceRequired broadcasts a hand's exact
        // ObjectId composition and belongs here too. Added, closing that half.
        GameEvent::CleanupDiscardChoiceRequired { player, .. } => Some(*player),
        _ => None,
    }
}
```

This is the batch's **one deliberate scope addition**. The alternative — `reveals_hidden_info()
== true` plus a seed — was considered and rejected because the brief makes a leak *test* an
acceptance criterion and a boolean cannot express "for this seat". The catch-all keeps the cost
at ~20 lines and no wire change (a method is not a declared shape; PROTOCOL is unaffected by it).

Also add `EffectChoiceRequired` to `reveals_hidden_info()`'s `true` list, and — separately, and
**only if the runner agrees after reading** — note that `Scried` and `Surveilled` currently
return `false` while plainly committing to hidden library order. That is **OOS-DP9-6**, not a
fix here.

### 2.5 The deterministic defaults — and why scry/surveil do NOT preserve the status quo

The engine never auto-picks on a decision path (constraint 1). Three pure exported helpers, in
`crates/engine/src/effects/mod.rs`, **called by nobody in the engine**:

```rust
pub fn default_search_answer(q: &EffectChoiceQuestion) -> EffectChoiceAnswer
pub fn default_scry_answer(q: &EffectChoiceQuestion) -> EffectChoiceAnswer
pub fn default_surveil_answer(q: &EffectChoiceQuestion) -> EffectChoiceAnswer
// plus the dispatcher the callers actually use:
pub fn default_effect_choice_answer(q: &EffectChoiceQuestion) -> EffectChoiceAnswer
```

| effect | default | vs. today | argument |
|---|---|---|---|
| **Search** | `found: Some(candidates[0])` — the lowest `ObjectId`, byte-identical to `candidates.iter().min_by_key(\|&&id\| id.0)` at `effects/mod.rs:3026` | **unchanged** | The engine's pick is one legal card among several with no systematic bias. Pure agency loss; preserving it keeps the search half of the batch at **zero churn** and keeps the SR-9b cross-regime comparison meaningful. |
| **Scry** | `bottom: [], top: looked_at` — the **identity** | **CHANGED** (today: everything to the bottom, `:3084-3092`) | See below. |
| **Surveil** | `graveyard: [], top: looked_at` — the **identity** | **CHANGED** (today: everything to the graveyard, `:3117-3130`) | See below. |

**The DP-8 precedent points the other way and is answered.** PB-DP8 preserved its default
byte-for-byte, deliberately, so the corpus would not churn and the fuzzer A/B would survive. That
was right there because the first-match pick was *neutral* among legal answers. Here it is not:

- Both defaults are CR-**legal** (CR 701.22a/701.25a permit "any number", which includes all).
  The audit classifies all three rows **B**, and the brief's "these are correctness bugs"
  is a severity judgement, not the audit's classification (§9 item 5).
- But "all to the bottom" / "all to the graveyard" is the **systematically maximal-harm** legal
  answer, and for surveil it is harmful in a way the CR notices: cards leave the library, so
  `Surveil N` can contribute to a CR 704.5b deck-out loss the player never agreed to. `Scry N`
  cannot deck, but it inverts the mechanic into a forced Bottom-N.
- The engine's default should be **what a player who declines to engage would say**, and for a
  choice that is optional in every direction that is the null action.
- Practical bonus, discovered in planning: the identity answer performs **no zone move at all**,
  which sidesteps the CR 400.7 renumbering in §2.2 entirely for every bot game.

**Cost, stated so nobody is surprised**: the scry/surveil half of this batch has **real
behavioural churn** on the existing corpus (§8.1), and a bot game's scry becomes a no-op rather
than a self-mill. Both are improvements; both are visible. If the reviewer disagrees, the lever
is one line per helper.

Bots submit the offered default **verbatim, never randomised** — randomising would change every
fuzzer seed's outcome and is a bot-quality improvement, seeded as **OOS-DP9-1** (the sibling of
OOS-DP8-1).

**No forced-choice narrowing is available.** PB-DP8's §5.3 narrowing (don't ask when exactly one
legal answer exists) kept its fallout tractable. Here:
- a quality-stated search with one candidate still has **two** legal answers (take it, or CR
  701.23b decline), so it is never forced;
- a scry/surveil of N ≥ 1 always has at least 2^N legal partitions;
- the only genuinely forced cases are a **quantity-only search with exactly one candidate**
  (CR 701.23d makes finding mandatory) and **N = 0** (CR 701.22b/701.25c: no event occurs).
  Implement both narrowings — they are CR-derived — and expect them to remove almost none of the
  churn. That is honest and it is why §8.1 matters so much.

### 2.6 Admission gate and `BlockingDecision`

`rules/engine.rs:226-239`, extend the allow-list — **this is the one site every new variant must
edit** and the doc block at `:87-119` says so:

```rust
|| matches!(&command, Command::AnswerEffectChoice { player, .. } if *player == decision.player())
```

New variant on `BlockingDecision` (`rules/engine.rs:121-132`), keeping `Copy`:

```rust
/// CR 608.2d (PB-DP9): `player` must answer a resolution-time choice from the
/// spell or ability resolving at `source` before the resolution can be retried.
EffectChoice { player: PlayerId, choice_id: u64, source: ObjectId },
```

`player()` and `Display` gain arms (both exhaustive). `blocking_decision` gains the third lookup
per §1.5 with the same liveness filter.

**Update the six-obligation doc block at `rules/engine.rs:87-119` while you are in the file** —
PB-DP8 wrote it after paying for four of them, and PB-DP9 discharges them as follows, which is
worth recording because it is the first evidence the list generalises:
(1) admission gate — **yes**, one line; (2) `handle_concede` clear — **yes**, §1.5 exits 2 and 4;
(3) two by-name hash sites — **`public_state_hash` yes, `loop_detection` deliberately NO**, §1.4;
(4) `LocalGame`'s exhaustive `match` — compile-forced; (5) `handle_concede`'s foreign-decision
gate — **inherited free**, it reads the predicate; (6) resume-site debt — **does not apply**, the
restore makes it vacuous (§0 point 3). **Add obligation (7): a new blocking kind must state
whether its pending state belongs in the loop-detection fingerprint, and argue it.**

### 2.7 Validation list for `handle_answer_effect_choice`

In `crates/engine/src/effects/mod.rs` (beside the choice machinery), dispatched from
`engine.rs`'s match with `validate_player_exists` (the `ChooseDredge` / `DiscardToHandSize` /
`ChooseTriggerTargets` precedent) and `loop_detection::reset_loop_detection` (CR 104.4b — a
resolution-time announcement is a meaningful player choice).

**All checks before any mutation**, in order:

1. `validate_player_exists(&state, player)`.
2. An entry exists — else `InvalidCommand("no resolution-time choice is pending")`.
3. `entry.player == player` — else `InvalidCommand` naming both. **SR-29 trust boundary.**
   Reachable only by a direct handler call (the admission gate catches the wire path first) —
   which is exactly the hole PB-DP7's review Finding 12 found, so the test must assert the
   **specific** error on each of the two paths, not merely `is_err()`.
4. `choice_id == entry.choice_id` — else `InvalidCommand("stale choice: expected N")`. **The
   moment guard.** Bound positionally: it names *this* question, not "a" question.
5. **Variant agreement**: `answer`'s variant must match `entry.question`'s variant — else
   `InvalidCommand`. (A `Scry` answer to a `Surveil` question is not merely wrong, it is a
   different rule.)
6. Per variant:
   - `SearchLibrary { found: Some(id) }` — `id ∈ question.candidates`. `found: None` — legal
     **iff** `question.may_fail_to_find` (CR 701.23d), else
     `InvalidCommand("CR 701.23d: this search must find a card")`.
   - `Scry { bottom, top }` / `Surveil { graveyard, top }` — the concatenation must be a
     **permutation** of `question.looked_at`: equal length, no duplicates within or across the
     two vectors, every id present in `looked_at`, and every id in `looked_at` present exactly
     once. Reject each failure with its own message. This is the *ordering payload's legality*
     the brief asks for: the order is data, but the multiset is a constraint.
7. Only then mutate: push `AnsweredEffectChoice { question: entry.question, answer }` onto
   `effect_choice_answers`, clear `pending_effect_choice`, call `resolve_top_of_stack`, then
   `finish_stack_resolution`.

**State untouched on rejection.** `process_command` takes `GameState` by value and every `?`
discards the local copy, so this holds by construction *provided* the handler validates before
mutating. Test it the way PB-DP8's closing review had to re-teach: drive
`handle_answer_effect_choice(&mut state, ..)` **directly**, once per rejection class, re-hashing
after each, and then confirm an *accepted* answer **does** move the hash — otherwise the pin
passes for the wrong reason. (ESM criterion 5545.)

---

## 3. The guard/consult-site derivation

**The rule, from PB-DP8's audit row**: *the guard set is every statement that executes after the
suspending call, within the same `process_command` invocation.*

Applied here, and then **weakened on purpose**:

| layer | derived set | needed? |
|---|---|---|
| after `execute_effect(..)` inside `resolve_top_of_stack_inner` | the **15** sites in §1.3 rows 1–15 | **robustness only.** Add `if state.pending_effect_choice.is_some() { … }` early-exit after each. Correctness does not depend on completeness — the wrapper restores the whole state — but continuing to execute after a suspension can panic (an arm that assumes the search found a card) or spin. |
| inside `execute_effect_inner`'s 13 recursion/loop sites (§1.3 row 4) + the `for p in players` loops in the three effect arms | 13 + 3 | **robustness only**, same argument. Also the reason a second question in one resolution is deferred to the next replay rather than overwriting the first: the recorder does nothing when `pending_effect_choice.is_some()`. |
| after `resolve_top_of_stack(..)` in `handle_all_passed` (`engine.rs:2114`) | `maybe_clear_lki_objects` (`:2119`), `is_game_over` (`:2124`) | **NO GUARD.** Both are provable no-ops on a restored state: the stack is non-empty (the object is back), so `maybe_clear_lki_objects` short-circuits; `is_game_over` was false at `process_command`'s entry and the state is byte-identical. **State the argument in a comment — PB-DP8's transferable rule (i) is that a comment justifying skipped work is a reachability claim.** |
| everything else | — | unreachable: `resolve_top_of_stack` has exactly **one** production caller (`rg 'resolve_top_of_stack\(' crates/*/src` ⇒ 2 hits: the definition and `engine.rs:2114`). |

**Mechanical verification the runner must run and record** (PB-DP8's checklist gap was that it
grepped the inner function and saw the definition instead of the callers):
```
rg -n 'execute_effect\(' crates/*/src          # 20 hits; 17 production callers + 3 definition/inner
rg -n 'resolve_top_of_stack' crates/*/src      # 2 hits
rg -n 'execute_effect_inner\(' crates/engine/src/effects/mod.rs   # 13 recursion sites + the definition
```
and, per site, a one-line statement of what executes after it.

---

## 4. Wire prediction, and what would falsify each half

### 4.1 `PROTOCOL_VERSION` 30 → **31** — expected

`Command::AnswerEffectChoice` and `GameEvent::EffectChoiceRequired` move two wire-frame types'
declared shapes, and **the closure's type count changes**: `EffectChoiceQuestion` and
`EffectChoiceAnswer` are new and reachable from both.

Procedure, verbatim from the comment at `rules/protocol.rs` (the bump block above
`PROTOCOL_VERSION` at `:286`), in **one** commit:
1. `PROTOCOL_VERSION` 30 → **31**, plus a `- 31:` History line saying the closure's type count
   changed and naming the two new types.
2. **Append** `ProtocolEpoch { version: 31, fingerprint: <gate-computed> }` to
   `PROTOCOL_HISTORY` — **never edit an existing row** — and set `PROTOCOL_SCHEMA_FINGERPRINT`
   to the same value.
3. Re-pin `protocol_version_sentinel` and `FROZEN_HISTORY_PREFIX_DIGEST` in
   `crates/engine/tests/core/protocol_schema.rs`.

**Never hand-invent a fingerprint** — every value is printed by the failing gate
(`declaration_fingerprint_is_pinned`, `frozen_prefix_is_pinned`, `history_is_append_only`).

**Falsifier**: PROTOCOL stays 30 only if both the question and the answer reuse existing
variants. There is no candidate: `Command::OrderReplacements` (PB-DP5's reuse) is keyed to a
`pending_zone_changes`/`pending_draws` applicability test and carries no per-kind payload;
`Command::ChooseTriggerTargets` carries `Vec<Vec<Target>>` with no way to express a partition or
a fail-to-find. Reusing either would be the DP-24 accepted-and-discarded-field antipattern.
Reject explicitly and record.

`private_to()` and `reveals_hidden_info()` are **methods**, not declared shapes — they do not
move the fingerprint. Say so in the commit message so a reviewer does not go looking.

### 4.2 `HASH_SCHEMA_VERSION` 67 → **68** — expected

`GameState` gains three fields; two new hashed structs plus two new hashed enums; `GameEvent`
gains discriminant 131.

1. `HASH_SCHEMA_VERSION` 67 → **68** (`state/hash.rs:636`) + a `- 68:` History line.
2. **Append** `HashSchemaEpoch { version: 68, decl_fingerprint, stream_fingerprint }` after the
   v67 row; both gate-computed; no existing row edited.
3. `GameEvent` hashing match: append a **`131u8`** arm after `TriggerTargetChoiceRequired`
   (`hash.rs:5447`). **That match has no `_` arm** — a miss is a compile error, which is the point.
4. `HashInto` impls **with BARE type names** — `impl HashInto for PendingEffectChoice`,
   `for AnsweredEffectChoice`, `for EffectChoiceQuestion`, `for EffectChoiceAnswer` — beside
   `impl HashInto for PendingTriggerTargets` (`hash.rs:3085`). **OOS-DP7-11**: a path-qualified
   name (`impl HashInto for crate::state::stubs::Foo`) silently falls out of SR-19's
   `every_hashed_struct_field_is_hashed_or_allowlisted` gate with no diagnostic. Every field of
   every struct hashed; the `NOT_HASHED` allowlist is empty and **stays empty**.
5. **Runner obligation, non-negotiable** (a gate cited in a comment is a claim): after writing
   them, delete one `hash_into` line from each new impl, run the SR-19 gate, confirm it fails
   **by name**, restore, and record the result in the commit message.
6. `public_state_hash`: fold in all three new `GameState` fields beside `:7888`/`:7892`.
7. `rules/loop_detection.rs`'s mandatory-state fingerprint: **deliberately NOT extended** (§1.4).
   Write the argument in the source and pin it with a test.

**Falsifier**: HASH stays 67 only if the answer bank could live outside `GameState`. It cannot —
`process_command` by-value is the only carrier between two commands, and the bank must survive
between the question and the answer.

### 4.3 The scattered sentinel re-pin — a find recipe, not a table

PB-DP8 shipped a 44-file table and **still missed one** (`pb_dp5_pending_draw_choice.rs` spells
the constant `mtg_engine::HASH_SCHEMA_VERSION` and escaped a numeric regex). OOS-DP7-8 complains
about the growth. A table is the wrong artefact; here is a recipe that is complete by
construction:

```
rg -n --no-heading -g '!target' 'HASH_SCHEMA_VERSION|PROTOCOL_VERSION' crates/ tools/
```

**Grep the symbol, never the number.** Any assertion on the version must name the constant to
compare against it, so this catches symbolic, path-qualified and re-exported spellings alike. On
this branch it returns ~44 test files plus the two canonical gate files
(`tests/core/protocol_schema.rs`, `tests/core/hash_schema.rs`) and `tests/core/protocol_roundtrip.rs`.

Second sweep, for any assertion on a bare literal that never names the constant: **run
`cargo test --all` and fix what fails**. State in the commit message that both sweeps ran.

**Do not add a new sentinel in the PB-DP9 test file** — OOS-DP7-8 is a standing complaint about
exactly this growth, and adding to it while citing it is poor form.

---

## 5. Rosters — enumerated, not grepped (SR-36)

The audit's **74 / 16 / 8** are claims. PB-DP8's row records that the audit's 84 and the
planner's grep-derived 74 were **both** wrong and the enumerated answer was 77. Assume the same
here.

**The runner's first task is `test_dp9_roster_enumeration`** in the new test file:

> Walk `mtg_card_defs::all_cards()`. For every `CardDefinition`, including `back_face` and
> `adventure_face` via `effective_abilities(true)` / `effective_abilities(false)`, walk **every
> `Effect` tree** with a recursive `effect_contains(&Effect, &dyn Fn(&Effect) -> bool)` helper
> (the effects nest — `Sequence`, `ForEach`, `Conditional`, `Repeat`, `MayPayThenEffect`,
> `Choose`, coin-flip arms — so a flat scan undercounts). Count, separately, defs containing
> `Effect::SearchLibrary`, `Effect::Scry`, `Effect::Surveil`, split by
> `completeness == Completeness::Complete`. **Print all three rosters by name.** Pin with three
> `assert!(n >= …)` (a `>=` assertion, so the authoring campaign cannot redden it).
>
> **Write the three printed numbers into the commit message and into audit §5's DP-7/DP-8/DP-9
> rows.** If they are 74/16/8 the audit is confirmed; if not, the audit is corrected. Either way
> the number becomes a fact.

Grep prediction for the reviewer to check the enumeration against (files, all completeness
levels, on this branch): `Effect::SearchLibrary` **100 files / 114 occurrences**; `Effect::Scry`
**20 files**; `Effect::Surveil` **9 files**. Spot-check contents so the roster is sanity-checkable:
the ten fetchlands (`polluted_delta`, `scalding_tarn`, `arid_mesa`, `flooded_strand`,
`marsh_flats`, `misty_rainforest`, `verdant_catacombs`, `windswept_heath`, `wooded_foothills`,
`bloodstained_mire`) plus `evolving_wilds` / `terramorphic_expanse` / `prismatic_vista` /
`fabled_passage`; the tutors (`demonic_tutor`, `vampiric_tutor`, `worldly_tutor`,
`mystical_tutor`, `enlightened_tutor`, `imperial_seal`, `grim_tutor`, `chord_of_calling`,
`green_suns_zenith`, `natural_order`, `birthing_pod`); the ramp package (`cultivate`,
`kodamas_reach`, `rampant_growth`, `farseek`, `three_visits`, `natures_lore`, `skyshroud_claim`,
`harrow`, `solemn_simulacrum`); the six Temples + `opt` / `preordain` / `serum_visions` /
`read_the_bones` / `senseis_divining_top` for scry; `consider` / `connive` / `doom_whisperer` /
`undercity_sewers` / `thundering_falls` / `underground_mortuary` for surveil.

### 5.1 Card-def yield prediction

**0 source edits, 0 completeness flips** — the same result PB-DP1..DP8 all landed. Argument: this
is an engine-agency batch; the defs are already `Complete` and already legal; nothing about their
DSL changes. The scry/surveil default flip changes *behaviour* of `Complete` defs **without
editing them**, which is the point.

### 5.2 MANDATORY pre-existing TODO sweep (roster-recall gate)

Ran on `crates/card-defs/src/defs/` with two patterns:
`(?i)(TODO|known_wrong|partial|inert)[^\n]{0,200}(scry|surveil|search)` and
`(?i)(scry|surveil|search|tutor)[^\n]{0,200}(lowest|auto-pick|auto-select|engine picks|not interactive|deferred to M10|bottom of the library|always)`.

**Result: 21 matches examined; ZERO forced adds that PB-DP9 as scoped closes.** Not an omission —
a positive assertion. Every match is an *adjacent* DSL gap, and three of them are valuable enough
to record here so they are not rediscovered later:

| file:line | TODO | closed by PB-DP9? |
|---|---|---|
| `halimar_depths.rs:24` | *"DSL gap — no 'rearrange top N' effect. **Scry 3 is wrong (allows bottoming)**"* | **No.** "Look at the top three and put them back in any order" is a top-only rearrange; PB-DP9's scry answer permits bottoming, so a `Scry` lowering is still wrong. Becomes a **one-variant** follow-on once this batch's choice surface exists. **OOS-DP9-2.** |
| `wrenn_and_seven.rs:36` | *"Known DSL gap. Partial: Scry 4 as approximation."* | **No**, same family as above. **OOS-DP9-2.** |
| `tooth_and_nail.rs:34/:57`, `buried_alive.rs:20`, `myriad_landscape.rs:48`, `sarkhan_unbroken.rs:73`, `goblin_recruiter.rs:4`, `protean_hulk.rs:23`, `tiamat.rs:9` | *"up to two / up to three / any number of X cards" — `SearchLibrary` finds one card* | **No.** `Effect::SearchLibrary` has no count. This is the **largest card-yield item adjacent to this batch** (~7 `partial` defs) and it is a natural successor: with the choice surface built, a `count: EffectAmount` on `SearchLibrary` turns `found: Option<ObjectId>` into `found: Vec<ObjectId>` and needs no new plumbing. **OOS-DP9-3.** |
| `scion_of_the_ur_dragon.rs:6` | *"needs `EffectTarget::LastSearchResult`"* | **No**, but PB-DP9's `AnsweredEffectChoice` is the natural carrier for it. **OOS-DP9-4.** |
| `path_of_ancestry.rs`, `aqueous_form.rs`, `scheming_symmetry.rs`, `woodland_bellower.rs`, `fauna_shaman.rs`, `the_world_tree.rs`, `wight_of_the_reliquary.rs`, `archdruids_charm.rs`, `myriad_landscape.rs:7`, `path_to_exile.rs` | condition / filter / second-search / `MayPayOrElse` gaps | **No** — different primitives (DP-25 family, filter gaps, `Condition` DSL). |

---

## 6. Bot / fuzzer determinism

**The default is a pure function of `EffectChoiceQuestion`**, and the question is a pure function
of `GameState`, so the class of determinism SR-9b requires is preserved by construction (§1.4's
fact list). Two things genuinely change and must not be papered over:

- **One extra `Command` per non-narrowed choice.** Every fetchland crack, every tutor and every
  scry/surveil in a bot game now costs a round trip. `LocalGame.command_count` rises toward
  `limits.max_commands`; `consecutive_passes` resets on each answer, making
  `max_consecutive_passes` slightly *less* likely to trip. Both are safety valves, not semantics.
  **Check the fuzzer's `max_commands` default and raise it if the measurement shows games
  truncating** — a truncated game is a *false* determinism signal.
- **`loop_detection::reset_loop_detection` fires on each answer** (CR 104.4b), changing
  `loop_detection_hashes` in pathological games. Correct, not a regression.

**The A/B-vs-`main` oracle is invalid, and this is inherited knowledge, not a discovery.** PB-DP8
established it: an extra `Command` shifts `RandomBot`'s RNG stream, so trace divergence between
branches is *structural*. Do not run an A/B and call it evidence.

**The determinism test that IS valid, and it is mandatory here in a way it was not for DP-7/DP-8,
because the replay's soundness depends on it at runtime:**

1. `test_dp9_same_seed_twice_is_byte_identical` — run `LocalGame` (bots only) with the same seed
   twice **on this branch**, and assert the two journals and the two final `public_state_hash`es
   are byte-identical. This is the direct test of §1.4.
2. `test_dp9_replay_retraces_the_same_path` — a fixture whose resolution contains **two**
   choices; assert `effect_choice_answers.len()` goes 0 → 1 → 2, that each recorded
   `AnsweredEffectChoice.question` equals the question the replay recomputes, and that the
   resolution's final events are emitted **exactly once** (the discarded runs leak nothing).
3. Fixed-seed fuzzer run for crash/panic surface only, with **OOS-DP3-9** (stack overflow at ~15
   games on `main`) and **OOS-M11-3** (nondeterminism in 150–200+ turn games) named as
   pre-existing. Do not chase them; do not let them mask a regression; say in the commit message
   exactly what was and was not run.

Benchmark obligation: `resolve_top_of_stack` now clones `GameState` unconditionally. Run the
existing criterion benches (`full_turn_4p`, `priority_cycle_4p`). If `full_turn_4p` regresses
more than ~5%, add a cheap `effect_may_ask(&Effect) -> bool` pre-scan and clone only when it is
true. Record the measured numbers either way.

---

## 7. Plumbing — the driving loops first, then the surfaces

PB-DP7's closing review: *a gate that stops the engine also stops every loop built on top of it.*
PB-DP8 then found the loops had already generalised. Verify, do not assume.

| loop | file | expected status | work |
|---|---|---|---|
| TUI auto-pass | `tools/tui/src/play/mod.rs` via `PlayApp::should_stop_auto_pass` (`app.rs`) | already reads `blocking_decision().is_some()` — generalises free | **none**; verify by reading |
| TUI bot loop | `tools/tui/src/play/mod.rs` via `PlayApp::acting_player` (`app.rs`) | already `if let Some(decision) = …blocking_decision() { return decision.player() }` | **none**; verify by reading |
| `LocalGame::advance` | `crates/simulator/src/local_game.rs:344-356` | **exhaustive `match` on `BlockingDecision`** since PB-DP8 | **compile-forced** arm → `DecisionKind::EffectChoice` |
| `GameDriver::run_game` | `crates/simulator/src/driver.rs` | expressed on `LocalGame`; asserts `AwaitingHuman` unreachable with empty `human_seats` | none directly; T-sim-2 proves it |
| `mtg-fuzzer` | `crates/simulator/src/bin/fuzzer.rs` | uses `GameDriver` | none directly; §6 |
| **replay-harness script pump** | `crates/engine/src/testing/replay_harness.rs::auto_answer_blocking_decisions` (`:354`) + `crates/engine/tests/scripts/script_replay.rs:352-400` | **built by PB-DP8** | **extend, do not rebuild** — one new `match` arm for the new decision kind, and one new string in `next_action_answers_the_block` (`script_replay.rs:401`). This is why the 210-script corpus survives: **without it, every script that reaches a search/scry/surveil halts at once.** |

Surfaces:

- **`LegalAction`** (`crates/simulator/src/legal_actions.rs`, appended after
  `ChooseTriggerTargets` `:163`):
  ```rust
  /// CR 608.2d (PB-DP9): answer an outstanding resolution-time choice. `question`
  /// is the full legal answer space so a human client can render a picker;
  /// `answer` is the engine's own deterministic default
  /// (`mtg_engine::effects::default_effect_choice_answer`), which the engine is
  /// guaranteed to accept (SR-38: never offer an action the engine rejects).
  AnswerEffectChoice { choice_id: u64, question: EffectChoiceQuestion, answer: EffectChoiceAnswer },
  ```
  `StubProvider::legal_actions`'s PB-DP7/DP8 block (`:247-280`) already `match`es
  `BlockingDecision` exhaustively ⇒ **compile-forced**. Exactly one action; the block
  early-returns (nothing else is legal).
- **Bots** — `random_bot::action_to_command` (`:368-380`) and `heuristic_bot`'s scorer (`:111-115`)
  both match `LegalAction` **exhaustively with no `_` arm** ⇒ compile-forced. `random_bot` maps to
  `Command::AnswerEffectChoice` with the offered `answer` **verbatim**; `heuristic_bot` scores 100.
- **`DecisionKind`** — `local_game.rs:105`, add `EffectChoice`. Already `#[non_exhaustive]`
  (PB-DP7). Update its doc comment (`:94-104`), which currently says the enum does not reach the
  resolution-time class.
- **`LocalGame::submit` / `command_player`** — no change; `command_player` extracts `player` from
  the externally-tagged JSON and works for `AnswerEffectChoice { player, .. }`. Add the variant to
  `test_command_player_extracts_acting_player`.
- **TUI** — `tools/tui/src/play/input.rs`: a key mirroring the PB-DP8 `'n'` key at `:66-90` that
  submits the offered default. The TUI has **no** exhaustive `LegalAction` match (it uses
  `matches!` probes), so nothing compile-breaks and a missing key is a **hang** — not optional.
  `panels/action_menu.rs`: a hint. `app.rs`'s event formatter (`:631`/`:637` neighbourhood): a
  display arm for `EffectChoiceRequired`. A real picker is M11-local Session 7 — **OOS-DP9-7**.
- **Replay viewer** — `tools/replay-viewer/frontend/src/lib/eventFormat.js`: a
  `case 'EffectChoiceRequired':` in **both** places, beside `'TriggerTargetChoiceRequired'` at
  `:65` (display) and `:408` (category). **JS has no compile gate; this is the easiest silent miss
  in the batch.** `tools/replay-viewer/src/view_model.rs` matches `StackObjectKind` and
  `KeywordAbility` exhaustively; **neither moves here**, so there is likely nothing to do —
  **verify with `cargo build --workspace`, do not assume** (the standing ~50%-miss warning).
- **Script schema** — `crates/engine/src/testing/script_schema.rs`: one new action string
  `"answer_effect_choice"` in `ScriptAction::PlayerAction`'s doc list (`:259-262`) and one new
  `#[serde(default)]` field carrying the answer (the variant is `#[serde(deny_unknown_fields)]`,
  so an undeclared field is a hard error). Follow the `trigger_targets` pattern (`:492`).
  `replay_harness.rs`: a new arm beside `"choose_trigger_targets"` (`:1006`).
  **SR-9c**: no new *assertion* path ⇒ no `check_assertions` work.

---

## 8. Tests, with per-test fail-before predictions

**New file**: `crates/engine/tests/primitives/pb_dp9_effect_choice.rs`, registered in
`crates/engine/tests/primitives/mod.rs`. **SR-9a: never add a top-level `tests/*.rs`; a dropped
`mod` line silently deletes coverage.**
**Simulator tests**: `crates/simulator/tests/local_game.rs` and `legal_actions.rs`'s `mod tests`.

**Shared helper** (mirroring PB-DP7/DP8's, and panicking when nothing is pending so it can never
mask a missing block):
```rust
/// Answer any outstanding CR 608.2d choice with the engine's own default, through
/// `process_command`. Panics if nothing is pending.
fn answer_pending_effect_choice(state: GameState) -> (GameState, Vec<GameEvent>)
```

| # | test | asserts | fail-before probe (expressible on `main`) |
|---|---|---|---|
| T1 | `test_dp9_search_blocks_and_rolls_back` | Cast a tutor with 3 matching cards. After the resolving pass: `pending_effect_choice().is_some()`, the **spell is still on the stack**, the library is untouched, no `SpellResolved`, exactly one `EffectChoiceRequired` with `candidates.len() == 3`. **ESM "a test observes the block".** CR 608.2d/701.23a | assert the spell is still on the stack after the pass — **fails today** (it has resolved and a card is in hand) |
| T2 | `test_dp9_chosen_card_is_found_not_the_lowest_id` | answer naming the **highest**-`ObjectId` candidate ⇒ that card reaches the destination, the other two stay in the library. CR 701.23a | assert the highest-id match is in hand — **fails today** (`min_by_key` takes the lowest, `effects/mod.rs:3026`) |
| T3 | `test_dp9_legal_fail_to_find` (**criterion 5549**) | quality-stated filter, 2 candidates, answer `found: None` ⇒ resolution completes, **no** card moves, `shuffle_before_placing` still shuffles if set, the spell goes to the graveyard, priority is granted. CR **701.23b** | not expressible — the state cannot exist. New-surface-only |
| T4 | `test_dp9_unrestricted_search_may_not_fail_to_find` | `filter == TargetFilter::default()` with ≥1 candidate ⇒ `may_fail_to_find == false`; `found: None` is **rejected** with the CR 701.23d message; state hash unchanged. CR **701.23d** | new-surface-only |
| T5 | `test_dp9_scry_keeps_cards_on_top_in_a_chosen_order` | Scry 3 (top-first A,B,C). Answer `bottom: [B], top: [C, A]` ⇒ library reads from the top: C, A, …rest…, and B is the **bottom-most** card. Cite CR 701.22a **and** PB-RS1's orientation. | assert any scried card is still on top after the effect — **fails today** (all three go to the bottom, `:3084-3092`) |
| T6 | `test_dp9_surveil_keeps_cards_on_top` | Surveil 2 (A,B). Answer `graveyard: [B], top: [A]` ⇒ A is the library's top card, B is in the graveyard, `Surveilled { count: 2 }` still fires (CR 701.25d). | assert a surveilled card is still in the library — **fails today** (Surveil N ≡ Mill N, `:3117-3130`) |
| T7 | `test_dp9_answer_validation` (table-driven) | for each of: non-candidate id; `found: None` on an unrestricted search; wrong `EffectChoiceAnswer` variant; a partition missing an id; a partition with a duplicate; a partition containing an id not in `looked_at`; wrong sender; stale `choice_id`; no entry — **assert the specific error**, and re-hash after each (drive the handler **directly**, not via a cloned-into-the-call state — PB-DP8's closing-review LOW-7). Then assert an **accepted** answer *does* move the hash. **ESM criterion 5545.** | new-surface-only |
| T8 | `test_dp9_admission_gate_while_blocked` | `PassPriority` / `CastSpell` / `TapForMana` / `PlayLand` from **any** seat ⇒ `Err(BlockedByPendingDecision)`, `public_state_hash` unchanged in every case. CR 608.2d | new-surface-only |
| T9 | `test_dp9_rollback_is_total` | Build a resolution whose effect list is `Sequence([DealDamage, SearchLibrary, Shuffle])`. At the block, assert **the damage has not been dealt**, the library order is untouched, and the state hash differs from the pre-command hash **only** by the three new fields. After the answer, assert the damage is dealt **exactly once**. | assert damage is not yet dealt at the moment the search would ask — new-surface; the "exactly once" half is the guard against a partial-apply bug |
| T10 | `test_dp9_two_choices_in_one_resolution` | `Sequence([SearchLibrary, Scry])`: question 1 (search) → answer → question 2 (scry) with a **different `choice_id`** → answer → both applied, both `EffectChoiceRequired`s emitted once each, `effect_choice_answers` empty at the end. Cite CR 608.2c (instructions in the order written). | new-surface; this is the replay mechanism's own test |
| T11 | `test_dp9_choice_inside_for_each_each_player` | "Each player searches their library for a basic land" (or a scry-each-player fixture) in a 4-player game: four questions in sequence, each naming the right player, each with that player's own candidates; all four applied. Cite CR 608.2e/101.4 **and record the APNAP deviation** (§9 item 7). | new-surface; this is the brief's "second, inner dimension" |
| T12 | `test_dp9_choice_inside_conditional_and_sequence` | `Sequence([A, Conditional{ cond, if_true: Sequence([SearchLibrary, Shuffle]), if_false: B }, C])`: the search asks, and after the answer **`Shuffle` and `C` both run and `A` does not run twice**. | new-surface; the nesting proof |
| T13 | `test_dp9_stale_choice_id_rejected` | answer with `choice_id + 1`, and with the `choice_id` of the *previous* choice in the same resolution ⇒ `Err`, hash unchanged. **PB-DP7 lesson 2 / PB-DP8 HIGH-1 (bind positionally).** | new-surface |
| T14 | `test_dp9_owner_concedes_mid_choice` | 3 players; P1's search question outstanding; P1 concedes ⇒ entry **and bank** cleared, the resolution completes using the **default** (CR 608.2d — a departed player makes no choice), the spell leaves the stack, **priority is granted to a live player**, and that player can actually `PassPriority`. Assert the resulting holder is not the conceded seat. CR 104.3a/800.4j. **This is PB-DP8's exact deadlock class — assert against the hazardous state, do not merely construct it.** | new-surface |
| T15 | `test_dp9_foreign_concede_does_not_step_over_the_block` | P2 (not the entry owner) concedes ⇒ no `PriorityGiven`, no turn advance, no stack resolution, entry intact, and P1's answer still completes the resolution. | new-surface; pins PB-DP8's obligation-5 gate generalising |
| T16 | `test_dp9_object_cannot_leave_the_stack_while_blocked` | at the block, assert the source stack object is present and `stack_objects.len()` is unchanged; then assert every command that could remove it is rejected. Documents §1.5 exit 5 as a property, not a comment. | new-surface |
| T17 | `test_dp9_defaults_reproduce_the_stated_behaviour` | `default_search_answer` == the pre-PB `min_by_key` pick (**unchanged**); `default_scry_answer` / `default_surveil_answer` are the **identity** and are explicitly **not** the pre-PB behaviour — assert both directions so the flip is documented by a test, not by a commit message. §2.5 | the search half passes by construction; the scry/surveil halves are the flip's pin |
| T18 | `test_dp9_scry_zero_and_surveil_zero_ask_nothing` | `Scry 0` ⇒ no event, no question (CR **701.22b**); `Surveil 0` ⇒ no event, no question (CR **701.25c**) | passes today; regression guard |
| T19 | `test_dp9_forced_quantity_search_asks_nothing` | unrestricted filter with **exactly one** candidate ⇒ no question, card found directly (CR 701.23d makes it determined) | new-surface (the absence of a question) |
| T20 | `test_dp9_private_to_leak_probe` (**constraint 3**) | `EffectChoiceRequired::private_to() == Some(searcher)` and `reveals_hidden_info() == true`; for a 4-player fixture assert **no other seat's id** is returned; and assert the event carries **only `ObjectId`s**, never a card name or `CardId`. State in the test's doc comment that there is no network filter to enforce it yet (M10) — the test pins the declaration. | new-surface; closes OOS-DP8-6's declaration half |
| T21 | `test_dp9_loop_detection_fingerprint_excludes_the_choice_state` | two states differing **only** in `pending_effect_choice` / `effect_choice_answers` produce the **same** loop-detection fingerprint and **different** `public_state_hash`es. §1.4 | new-surface; this is the deliberate precedent deviation's pin |
| T22 | `test_dp9_mana_ability_gate` | a fixture whose triggered mana ability's effect contains a `Scry`: the effect uses the default and **no** entry is recorded (`mana.rs:864`, CR 605.1b/605.4a); plus a roster assertion that no `Complete` def has one of the three effects inside a mana ability | new-surface |
| T23 | `test_dp9_roster_enumeration` | §5's `all_cards()` walk; prints three rosters; three `>=` pins | new-surface; **its printed numbers are the deliverable** |
| T-sim-1 | `test_dp9_local_game_awaits_human` | human seat: `advance()` ⇒ `AwaitingHuman { kind: DecisionKind::EffectChoice, player, actions.len() == 1 }`; a second `advance()` returns the **same** `seq`; `submit(seq, cmd naming another seat)` ⇒ `BadParams`; `submit(stale_seq, …)` ⇒ `StaleDecision`; correct `submit` proceeds. **Must assert `kind == EffectChoice`** | new-surface |
| T-sim-2 | `test_dp9_bot_game_never_halts_on_an_effect_choice` | bot-only `LocalGame`, seeded, a deck containing a tutor **and** a scry **and** a surveil card, ≥5 turns: never `Halted`, ≥1 `AnswerEffectChoice` of each kind in the journal | new-surface; guards `driver.rs`'s `unreachable!()` |
| T-sim-3 | `test_dp9_stub_provider_offers_only_the_answer` | the blocked player gets exactly one action; every other player gets `[]`; and the offered default is **accepted by `process_command`** (SR-38) | new-surface |
| T-det-1/2 | §6's two determinism tests | as §6 | new-surface |

### 8.1 Existing tests and golden scripts predicted to change

**None may be repaired by weakening an assertion.** Every change is either "the test now
*chooses*" or "the test now pins the new, CR-correct default".

| file | why | CR justification for the change |
|---|---|---|
| `crates/engine/tests/mechanics_m_z/library_ordering.rs` (11 `execute_effect`/`Effect::Scry` hits) | direct scry/surveil execution | CR 701.22a — the *bottoming* assertions must be kept and re-pointed at an **explicit answer** that bottoms, so PB-RS1's orientation stays pinned. **This is the brief's "reuse PB-RS1's tests" requirement.** |
| `crates/engine/tests/core/pb_rs1_roster_sweep.rs` | PB-RS1's `Zone::top_n` roster sweep | same — supply the bottom-everything answer explicitly |
| `crates/engine/tests/mechanics_m_z/surveil.rs` | Surveil ≡ Mill assertions | CR 701.25a — a surveil that mills is now a *choice*; supply it |
| `crates/engine/tests/mechanics_e_l/library_search.rs` | search assertions | CR 701.23a |
| `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs`, `pb_ef10_sacrifice_driven_amounts.rs`, `primitive_sr36_scaled_mana_and_life_costs.rs`, `mechanics_a_d/adventure_tests.rs`, `core/pb_rs3_combat_trigger_roster.rs` | contain one of the three effects | per-effect |
| `test-data/generated-scripts/baseline/009_read_the_bones_scry_draw.json` | Read the Bones = "Scry 2, then draw two" | with the identity default the two scried cards are the two drawn. **Add an explicit `answer_effect_choice` action** so the script documents the choice rather than inheriting the pump's default — the JSON regime is the best place to document that the choice is real |
| `test-data/generated-scripts/stack/071_consider_surveil_then_draw.json` | Consider = "Surveil 1. Draw a card." | same; make the mill-or-keep decision explicit and update the step notes. **This is the script that best demonstrates the fix.** |
| `test-data/generated-scripts/etb-triggers/205_nadaar_ventures_on_etb.json` | contains a scry-adjacent step | verify; may be unaffected |
| Anything asserting post-scry `ObjectId`s | §2.2's CR 400.7 renumber fix | CR 400.7 — a card that stays in the library does not become a new object |

**Indirect fallout is not enumerable by grep**: any test whose deck contains one of the ~129
roster files reaches a question. The procedure: run `cargo test --all`, and for each failure
decide whether the repair is (a) insert the answer helper, or (b) recognise that the question
should not have been asked (a narrowing bug). **(b) is a bug in this batch, not in the test** —
treat every unexpected question as a finding before treating it as fallout.

**Golden scripts**: run the full suite (`cargo test --test run_all_scripts`), 210 approved,
**0 new skips** (SR-9c). If a script halts, the §7 pump extension is wrong — **fix the pump, do
not edit the script.**

---

## 9. Pre-survey bullets that turned out to be WRONG

Verified against source and MCP CR text on this branch, 2026-07-27.

1. **`pb-plan-DP7.md` §1.6's option (a) is impossible as written.** It prescribes *"a resumable
   effect-list cursor **on the stack object**"*. `resolve_top_of_stack` pops the stack object at
   `resolution.rs:39-42`, before any effect runs; there is no stack object during
   `execute_effect`. The audit's §8 PB-DP7 row repeats the phrase. Both must be corrected.
2. **The re-entrancy audit is three units, not "every `execute_effect` caller".** DP-7 §1.6, the
   audit §8 row and the brief all imply a wide sweep. There are **17** production callers and
   **15 of them are inside one function** (`resolve_top_of_stack`, `resolution.rs:36`–`:7828`;
   the next `fn` starts at `:7829`). The other two are `mana.rs:864` (gated) and
   `replacement.rs:1964` (a literal `Effect::CreateToken` — provably unreachable).
3. **The audit's `effects/mod.rs` line cites are stale.** §4.9 and §5 cite `:3032` (search),
   `:3089-3098` (scry), `:3123-3130` (surveil). On this branch: search auto-pick **`:3026`**
   (arm opens `:2936`), scry bottom-loop **`:3084-3092`** (arm opens `:3072`), surveil
   **`:3117-3130`** (arm opens `:3101`). `memory/primitive-wip.md`'s cites are correct; the
   audit's are not. Same class as OOS-DP6-8: *a site cite in that document is a snapshot.*
4. **The brief's fail-to-find carve-out is wrong.** It says fail-to-find is permitted *"EXCEPT
   where the search is for a card with a stated quality that the search itself reveals."* CR
   **701.23e** says revealing is orthogonal ("if the effect doesn't also contain instructions to
   reveal, they're not revealed") and has no bearing on finding. The real carve-outs are
   **701.23d** (a *quantity-only* search — "a card", "three cards" — **must** find) and
   **701.23c** (an undefined quality — can't find anything). The stated-quality case is exactly
   the one where failing to find **is** allowed (701.23b). §2.3 implements the CR, not the brief.
5. **"Scry and surveil actively invert the printed mechanic — these are correctness bugs" is a
   severity judgement, not a classification.** The audit's own §4.9 marks all three rows class
   **B** (a choice exists and the engine makes it silently), and both current defaults are
   CR-**legal** answers under 701.22a/701.25a's "any number". The correctness arguments that do
   survive are narrower, and this plan uses those: surveil-as-mill can contribute to a CR 704.5b
   deck-out the player never agreed to (§2.5), and scry-to-bottom renumbers `ObjectId`s in
   violation of CR 400.7 (item 8).
6. **The brief's CR 121.1 cite for top/bottom orientation is wrong.** CR 121.1 is the *draw*
   rule ("a player draws a card by putting the top card of their library into their hand"). The
   CR gives no "top = / bottom =" numbering rule; the rules that bear on library position are
   **CR 401.4** (the owner arranges cards put in the same position at the same time — which is
   exactly what scry's "in any order" relies on) and **CR 401.7**. PB-RS1's own source comments
   cite 121.1 (`effects/mod.rs:3087-3090`) and are stale in the same way. The *convention*
   (`push_front` = bottom, last element = top) is correct and load-bearing; only the cite is wrong.
7. **`PlayerTarget::EachPlayer` is not APNAP.** `resolve_player_target_list`
   (`effects/mod.rs:6926-6937`) iterates `state.players.keys()` — an `OrdMap`, i.e. **ascending
   `PlayerId`**. CR **701.22c** / **701.23i** / **608.2e** all require the per-player decisions to
   be made in **APNAP** order. This is a pre-existing deviation that PB-DP9's question order
   inherits and makes *visible* for the first time. Do **not** fix it in this batch (it changes
   every multi-player effect's order, far beyond this roster) — seed it and state it in T11's
   comment. **OOS-DP9-8.**
8. **`move_object_to_bottom_of_zone` renumbers on a same-zone move.** `state/mod.rs:1739-1740`
   calls `next_object_id()` unconditionally, so today's scry-to-bottom gives every scried card a
   **new `ObjectId`** even though it never left the library — a CR 400.7 deviation, and it also
   consumes `timestamp_counter` values (the shuffle/coin-flip seed source). §2.2 prescribes a
   `Zone`-level permutation instead. New finding; nothing in the audit or either prior plan names it.
9. **`GameEvent::private_to()` still does not exist** (OOS-DP8-6 open). The brief's constraint 3
   assumes it. This batch adds it (§2.4) rather than seeding it again.
10. **CR 701.22c also requires the moved cards to move *simultaneously*** across players ("those
    cards move at the same time"). The engine moves each player's cards inside the per-player
    loop. Pre-existing; unchanged by this batch; seeded with item 7.
11. **`Effect::SearchLibrary`'s `reveal` field is inert** — destructured as `reveal: _` at
    `effects/mod.rs:2939`. CR 701.23e means the found card is revealed only when the effect says
    so, and the engine never reveals. Pre-existing, out of scope, **OOS-DP9-9**.
12. **CR 701.23h is unmodelled**: "search a library for one or more cards more than once before
    shuffling … the player searches that library only once." The engine treats each
    `Effect::SearchLibrary` as an independent search, so a two-search effect asks twice. Under
    PB-DP9 that is *visible* (two questions) where before it was silent. Out of scope, seeded.
13. **The audit's roster numbers are unverified** (§5) and PB-DP8's precedent is that both the
    audit's number and the planner's grep were wrong. Grep on this branch: 100 / 20 / 9 **files**.
14. **OOS-DP8-14 predicted three new harness action strings.** The unified command means **one**.

**Confirmed as stated** (recorded so the reviewer knows the checks ran): the wip file's three
site cites; `resolve_top_of_stack` has exactly one production caller (`engine.rs:2114`);
`PROTOCOL_VERSION == 30` (`protocol.rs:286`) and `HASH_SCHEMA_VERSION == 67` (`hash.rs:636`);
`GameEvent`'s maximum hashed discriminant is **130** (`hash.rs:5447`), so 131 is next;
`BlockingDecision` is a plain `enum` (no `#[non_exhaustive]`), so `LocalGame`'s `match` is
compile-forced; `DecisionKind` is `#[non_exhaustive]`; `TargetFilter` derives `Default`;
`Zone::top_n` returns top-first and `expect_move_object_to_bottom_of_zone` is `push_front`
(PB-RS1); `auto_answer_blocking_decisions` and `next_action_answers_the_block` exist and are
wired into `script_replay.rs:352-400`; `random_bot`/`heuristic_bot`/`StubProvider` all match
exhaustively with no `_` arm.

---

## 10. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

| seed | finding | class |
|---|---|---|
| **OOS-DP9-1** | **`RandomBot` answers a CR 608.2d choice with the engine's default, not randomly.** Correct for this batch (a random answer would change every fuzzer seed and destroy any baseline), wrong for bot quality — a bot that always keeps every scried card on top is as un-strategic as one that always bottoms them. A randomising `RandomBot` + a library-aware `HeuristicBot` + a fuzzer-baseline re-pin belong together. Sibling of OOS-DP8-1. | simulator quality, deferred |
| **OOS-DP9-2** | **"Look at the top N and put them back in any order" has no DSL representation** — `halimar_depths.rs:24` and `wrenn_and_seven.rs:36` both say so in their own source, and both lower to `Scry`, which wrongly permits bottoming. With PB-DP9's choice surface built, this is one `Effect` variant (or one `bottom_allowed: bool`) plus a question flag. **Found by the mandatory TODO sweep; not a PB-DP9 forced add because it needs a different primitive.** | DSL gap, cheap follow-on |
| **OOS-DP9-3** | **`Effect::SearchLibrary` finds exactly one card.** ~7 `partial` defs say so in their own source (`tooth_and_nail`, `buried_alive`, `myriad_landscape`, `sarkhan_unbroken`, `goblin_recruiter`, `protean_hulk`, `tiamat`). CR 701.23 searches for "one or more cards". On PB-DP9's machinery this is `count: EffectAmount` on the effect and `found: Vec<ObjectId>` on the answer, with **zero** new plumbing. **The largest card-yield item adjacent to this batch** and the strongest candidate to rank next. | DSL gap, high card yield |
| **OOS-DP9-4** | **`EffectTarget::LastSearchResult` does not exist** (`scion_of_the_ur_dragon.rs:6`). PB-DP9's `AnsweredEffectChoice` is the natural carrier — the found card is now a first-class recorded value rather than a local. | DSL gap, narrow |
| **OOS-DP9-5** | **`may_fail_to_find` is computed from the filter alone, ignoring the zone.** CR 701.23b is scoped to a **hidden** zone; with `also_search_graveyard: true` the search spans a hidden and a public zone, and strictly the player must find from the graveyard even while declining from the library. Deliberately not modelled; the conservative choice (allow the decline whenever the filter states a quality) is stated in the code. | correctness, narrow |
| **OOS-DP9-6** | **`GameEvent::Scried` and `GameEvent::Surveilled` return `false` from `reveals_hidden_info()`** while committing to hidden library order — the same class as OOS-DP7-3. Not fixed here to keep the batch's hidden-info story to the one new event. | correctness, M10-gated |
| **OOS-DP9-7** | **The TUI answers with the default, not a picker** — sibling of OOS-DP7-6 / OOS-DP8-2. A scry picker is the single most valuable M11-local Session 7 widget because it is the one a human uses every game. | UX gap, M11-local |
| **OOS-DP9-8** | **Multi-player resolution-time choices are made in ascending `PlayerId` order, not APNAP, and do not move simultaneously.** CR 701.22c / 701.23i / 608.2e. `resolve_player_target_list` (`effects/mod.rs:6926-6937`) iterates an `OrdMap`. Pre-existing and far wider than this roster (every `ForEach::EachPlayer` effect), which is why it is not fixed here — but PB-DP9 makes it *observable* for the first time, because the questions are now asked in that order. | correctness, wide |
| **OOS-DP9-9** | **`Effect::SearchLibrary`'s `reveal` field is inert** (`effects/mod.rs:2939`, `reveal: _`) and **CR 701.23h ("search twice before shuffling is one search") is unmodelled.** Both were invisible before; the second is now visible as two questions. | correctness, narrow |
| **OOS-DP9-10** | **`EffectContext.target_remaps` is a `std::collections::HashMap`** (`effects/mod.rs:189`). The abort-and-replay design makes execution determinism a *runtime* requirement, not just a test requirement (SR-9b). If any code path's outcome depends on iterating that map, the replay can diverge across processes. The runner must audit every read and record the result; if one iterates, this becomes a correctness finding rather than a hygiene one. | determinism, gate-adjacent |
| **OOS-DP9-11** | **`state/mod.rs`'s same-zone `move_object_to_bottom_of_zone` renumbers (CR 400.7).** PB-DP9 fixes it for scry by permuting the `Zone` instead. **Every other same-zone caller of `move_object_to_zone` / `move_object_to_bottom_of_zone` should be swept for the same defect** — this is the OOS-RS-1 method applied to a different axis. | correctness, sweep |
| **OOS-DP9-12** | **§10's re-audit triggers are due again.** A new `Command` ⇒ DP-24's accepted-and-discarded-field check (answer: no — `choice_id`, the variant, every id and the partition are all validated). A new `GameEvent` ⇒ the `reveals_hidden_info` sweep (answered in §2.4, and it found OOS-DP9-6). Also still owed from OOS-DP7-7: §3.1's 277-def re-derivation. | bookkeeping |

**Audit cross-references to update when this ships**: §4.9 the three rows (Scry / Surveil /
Library search pick, **B → A**, with the corrected line numbers from §9 item 3); §5 Tier 1
**DP-7 / DP-8 / DP-9** rows (SHIPPED banner + the enumerated roster numbers + the CR-cite
corrections in §9 items 4 and 6); §8 the **PB-DP9** row (wire prediction, the abort-vs-cursor
design and **why option (a) as specified was impossible**, and the three-unit re-entrancy result
vs the "every `execute_effect` caller" prediction); §8's sequencing note (correct the "resumable
effect-list cursor on the stack object" phrasing); the `pb-plan-DP7.md` §1.6 correction; §9.3 /
§9.4 recs; §10.

---

## 11. Risks and edge cases

1. **The replay's determinism premise is the batch.** If execution is not a pure function of
   `GameState`, an answer can be applied to a question the engine never asked. Mitigations: the
   question-equality check before consuming each banked answer (§1.4), T-det-1/2, and the
   `target_remaps` audit (OOS-DP9-10). **A reviewer should attack this first.**
2. **Test fallout is the batch's schedule risk, and there is no narrowing lever.** Unlike DP-8,
   almost no choice is forced (§2.5), so *every* fetchland, tutor, scry and surveil in *every*
   test and script now asks. The pump absorbs the 210-script corpus; engine unit tests do not
   have a pump. §8.1 names 9 candidate files but the real number is discovered by running the
   suite. **Treat every unexpected question as a possible narrowing bug before treating it as
   fallout**, and if the volume is unmanageable, **stop and flag** rather than weaken assertions.
3. **The scry/surveil default flip is a deliberate behaviour change to `Complete` cards.** It is
   argued in §2.5 and pinned by T17, and it is the reason §8.1 exists. If the reviewer disagrees
   with the call, the change is one line per helper — but the *tests* encode the decision, so
   reversing it means re-pointing them.
4. **`resolve_top_of_stack` now clones `GameState` on every resolution.** Persistent structures
   make this cheap in principle; measure it (§6). The `effect_may_ask` pre-scan is the escape
   hatch, and it must not become a correctness dependency.
5. **The concede exits are where PB-DP8 bled three times.** Exit 2 (the owner concedes) must
   *drive* the resolution, not merely clear the entry — otherwise `priority_holder == None` with
   everyone passed is an unrecoverable deadlock. T14 must assert the resulting holder can
   actually act, not merely that the entry is gone (PB-DP8's closing-review rule (ii): a test
   that constructs a hazardous state and does not assert against it is worse than no test).
6. **The loop-detection exclusion is a deliberate deviation from the DP-7/DP-8 precedent.** If
   the argument in §1.4 is wrong, a mandatory-loop game could be mis-detected in either
   direction. T21 pins the behaviour; the *argument* is what a reviewer must check.
7. **Hidden information is materially harder here than in DP-7/DP-8**: the candidate list is
   itself the secret. `private_to()` is added but nothing consumes it, so the guarantee is a
   declaration, not an enforcement. Say that in the test and in the commit message; do not let
   the batch read as though the leak is closed.
8. **Two version bumps in one commit and five gate-computed fingerprints.** The most likely
   process error is hand-editing a fingerprint or editing an existing history row. All of it is
   machine-caught — read the failures, do not guess. And re-pin the sentinels with the **symbol**
   grep (§4.3), not a numeric regex.
9. **A missed consumer deadlocks a whole regime.** Five must answer: `StubProvider`, `RandomBot`,
   `HeuristicBot`, the TUI key, the harness pump. The first three are compile-forced; **the TUI
   key and the pump are not.** The pump is the loud one (210 scripts fail at once, which fails
   safe); the TUI key is the silent one.
10. **`GameStateError::BlockedByPendingDecision` already exists** and is outside the wire closure
    — no new error variant is owed. Do not add one.

---

## 12. Verification checklist

- [ ] `cargo build --workspace` clean after **every** phase (SR-8; `tools/replay-viewer` and
      `tools/tui` are the two runners miss ~50% of the time)
- [ ] `cargo test --all` green — includes `tools/check-defs-fmt.sh` via `core card_defs_fmt` (SR-35)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
- [ ] `rg -n 'execute_effect\(' crates/*/src` — 17 production callers, each classified per §1.3,
      and the classification written into the source at each of the three units
- [ ] `rg -n 'resolve_top_of_stack' crates/*/src` — exactly 2 hits; the wrapper is the only
      suspension-aware site, and `handle_all_passed`'s two post-statements carry the **argued**
      no-guard comment (§3)
- [ ] `PROTOCOL_VERSION == 31`, fingerprint **gate-computed**, `PROTOCOL_HISTORY` row **appended**,
      `protocol_version_sentinel` + `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned
- [ ] `HASH_SCHEMA_VERSION == 68`, **both** fingerprints gate-computed, `HASH_SCHEMA_HISTORY` row
      **appended**, sentinel + `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned
- [ ] all sentinels re-pinned via the **symbol** grep of §4.3 (`HASH_SCHEMA_VERSION|PROTOCOL_VERSION`
      over `crates/ tools/`), plus a post-`cargo test --all` sweep for bare-literal assertions;
      **no new sentinel added** (OOS-DP7-8)
- [ ] `state/hash.rs`'s `GameEvent` match gained a `131u8` arm (no `_` arm exists — a miss is a
      compile error, which is the point)
- [ ] all four new `HashInto` impls written with **bare** type names; the delete-a-field
      demonstration run and its result recorded (OOS-DP7-11)
- [ ] `NOT_HASHED` allowlist still empty
- [ ] the three new fields folded into `public_state_hash` and **deliberately excluded** from
      `loop_detection.rs`'s fingerprint, with the argument in the source and T21 as the pin
- [ ] `GameState` still sealed: new fields `pub(crate)`, read accessors, **no** `_mut` (SR-3)
- [ ] `rules/engine.rs`'s six-obligation doc block updated with the PB-DP9 discharge record and
      the new obligation (7) (§2.6)
- [ ] `local_game.rs`'s `BlockingDecision` match, `random_bot`, `heuristic_bot`, `StubProvider`
      all gained arms (compile-forced) and the bot submits the **default verbatim**
- [ ] TUI: key, menu hint, event-formatter arm; auto-pass loop and `acting_player` **verified
      unchanged and still correct** by reading
- [ ] `eventFormat.js` gained an `EffectChoiceRequired` case in **both** places — no compile gate,
      verify by reading
- [ ] the harness pump and `next_action_answers_the_block` **extended** (one new arm, one new
      action string), and the full golden suite runs: 210 approved, **0 new skips** (SR-9c)
- [ ] `test_dp9_roster_enumeration` run; **the three printed counts written into the commit
      message and into audit §5's DP-7/DP-8/DP-9 rows** (SR-36 — never ship a grep-derived roster)
- [ ] `private_to()` added, `reveals_hidden_info()` extended, T20 green; the commit message says
      plainly that nothing consumes `private_to()` yet
- [ ] criterion benches run (`full_turn_4p`, `priority_cycle_4p`) and the numbers recorded; the
      `effect_may_ask` pre-scan added only if the regression exceeds ~5%
- [ ] same-seed-twice determinism test green; fuzzer run for crash surface only, with
      OOS-DP3-9 / OOS-M11-3 named as pre-existing and **no A/B-vs-`main` presented as an oracle**
- [ ] `git diff --stat -- crates/card-defs/` shows **0 card-def source edits, 0 completeness flips**
- [ ] `docs/audits/decision-point-audit.md` §4.9 / §5 DP-7/8/9 / §8 PB-DP9 row + sequencing note /
      §9.3 / §9.4 / §10 updated; seeds **OOS-DP9-1..12** filed in §8.1; the
      `pb-plan-DP7.md` §1.6 "cursor on the stack object" correction recorded
- [ ] `memory/workstream-state.md` handoff + CLAUDE.md Current State snapshot delta
