# Primitive Batch Plan: PB-DP7 — Cleanup discard is a player choice, and the first pending decision that actually blocks

**Generated**: 2026-07-26
**Task**: `scutemob-155` · branch `feat/pb-dp7-cleanup-discard-command-pilot-for-blocking-pending-de`
**Primitive**: a new wire round-trip (`GameEvent::CleanupDiscardChoiceRequired` → `Command::DiscardToHandSize`)
backed by the engine's **first pending decision that genuinely gates progress**
(`GameState.pending_cleanup_discard`).
**Finding**: DP-3 (`docs/audits/decision-point-audit.md` §5 line 449, §4.11 line 400)
**CR Rules**: **514.1**, 514.2, 514.3, 514.3a, **402.2**, **701.9a/701.9b**, **703.4n**, **702.35a**, 603.3b, 500.4, 104.4b
**Cards affected**: **0 completeness-marker flips**; **3 `Complete`, deck-legal defs stop being live-wrong**
(`fiery_temper.rs`, `stensia_masquerade.rs`, `markov_baron.rs` — all three carry
`AbilityDefinition::Keyword(KeywordAbility::Madness)` and none declares a `completeness:` field, so all
three are `Completeness::Complete` by the `#[default]` at
`crates/card-types/src/cards/card_definition.rs:199-200`). **0 card-def source edits.**
**Wire**: **PROTOCOL 27 → 28** (certain) and **HASH 64 → 65** (certain — see §5).
**Dependencies**: PB-DP1 (priority-to-actor), PB-DP4 (the `pending_*`-consulted-at-a-boundary precedent),
PB-DP5 (`pending_draws`, the worked HASH-bump precedent), M11-local S1 (`LocalGame`).
**Deferred items carried in**: none. Items deliberately *not* carried in are listed in §10.

---

## 0. Executive summary of the design

Three decisions carry this batch and everything else follows from them.

1. **The gate is derived, not enumerated.** Step/turn advancement in this engine happens at
   exactly the call sites of `turn_structure::advance_step` / `advance_turn`. A workspace-wide
   grep on this branch returns **five** such sites, in **three** functions
   (`rules/engine.rs:1960`, `:1965`, `:2100`, `:2106`, `:2184`). The gate must dominate those
   three functions and nothing else. That is a mechanical completeness argument, not a hand list —
   it is the same method §10 of the audit prescribes for `Zone::push_front` after DP-2. Full
   inventory in **§4**.
2. **The pending state is an `Option`, not a `Vector`, and the shape is derived from the CR.**
   CR 514.1 gives the discard to **the active player** only, once per cleanup step. At most one
   cleanup discard can be outstanding, ever. The three payment vectors are `Vector` because
   CR 702.24b/702.30a genuinely allow several at once; copying that shape here would be
   consistency-cargo-culting. **Per-kind field + a shared `fn blocking_decision(&GameState) ->
   Option<BlockingDecision>` predicate**, not one generic `pending_decisions` bag — argued in **§1.4**.
3. **The `Command` carries the whole subset, and the pause is taken before any of CR 514.2.**
   CR 514.1 is a single turn-based action that discards *n* cards; CR 703.4n confirms it happens
   "immediately after the cleanup step begins". So one command with `cards: Vec<ObjectId>` of
   exactly *n* ids, and the pause sits after the CR 402.2 recompute and before the first discard —
   at which point **nothing** of CR 514.2 (damage clear, "until end of turn" expiry) or CR 500.4
   (pool empty) has run. Argued in **§2** and **§3**.

---

## 1. The blocking pending-decision mechanism (ESM criterion 5540)

> **Read this section as the spec for PB-DP8 and PB-DP9.** It is deliberately written so a later
> planner can reuse the parts that generalise and can see, named and stated, the parts that do not.

### 1.1 What "blocking" has to mean in this engine

The engine has seven pending-decision round-trips (§9.3 of the audit) and, per §4.4/§4.11, only
`pending_zone_changes` gates anything — and it gates *narrowly*: `rules/sba.rs:529-533` / `:736-740`
skip the **object** on later SBA passes. It is object-scoped, not game-scoped. Nothing in the engine
today prevents the *game* from moving on past an unanswered question.

There are exactly two things a gate must be able to stop, and they are different:

- **A. Progress.** The game must not leave the moment at which the question was asked. In this
  engine "the moment" is a step, and leaving it means `advance_step` / `advance_turn`.
- **B. Admission.** While the question is outstanding, the engine must not accept an unrelated
  command. Without this, a client can cast a spell during a cleanup step in which CR 514.3 says
  no player has priority.

PB-DP4 chose a **deadline** (auto-decline at the next stack-empty priority boundary) rather than a
block, for a stated reason: gating priority would deadlock any seat that never sends the command,
because `driver.rs` answers a rejected command with a silent `PassPriority` and a refused pass is an
infinite retry with no error. **That reasoning does not transfer here, and the difference is the
whole reason PB-DP7 is the right pilot:**

- A payment has a CR-supplied default ("if you don't, sacrifice it", CR 118.12a). A cleanup
  discard has none — CR 514.1 is mandatory and CR 701.9b says the affected player chooses. There is
  no "didn't answer" branch to apply.
- A payment sits inside a priority window, where a `PassPriority` is a legal move. A cleanup
  discard sits in a step with **no priority at all** (CR 514.3; `state/turn.rs:63-65`
  `has_priority()` returns `false` for `Step::Cleanup`), so `PassPriority` is already rejected
  there by `priority::pass_priority` (`rules/priority.rs:26-33`). The deadlock PB-DP4 feared is
  structurally unreachable: the answering command is the *only* legal move, and every consumer in
  the tree is being taught to offer it (§7).

### 1.2 The predicate

New, in `crates/engine/src/rules/engine.rs` (private to the engine crate, `pub(crate)`), with a
public read accessor on `GameState` for the simulator:

```
/// The one decision, if any, that is currently gating the game.
///
/// CR 514.1 (PB-DP7) is the only kind today. PB-DP8 (CR 603.3d trigger targets) and
/// PB-DP9 (CR 701.22a/701.23/701.25a) append variants; every consult site below is
/// written against this enum and not against the field, so adding a variant needs no
/// new consult site.
pub(crate) enum BlockingDecision {
    CleanupDiscard { player: PlayerId, count: u32 },
}

pub(crate) fn blocking_decision(state: &GameState) -> Option<BlockingDecision>
```

Implementation today: read `state.pending_cleanup_discard`, and return `None` if that entry's
player is no longer alive (`has_lost || has_conceded`) — see the concede hazard in §4.1(3).

**Per-kind field with a shared predicate, not one generic `pending_decisions: Vector<_>`.** The
argument, with the hash consequence stated for each:

| option | argument | hash consequence |
|---|---|---|
| **chosen: per-kind field + shared predicate** | Each pending kind's *cardinality* is a CR fact, and they differ: exactly-one-or-none for CR 514.1, one-per-trigger for CR 603.3d, one-per-resolving-effect for CR 701.23. A per-kind field lets each carry its CR-correct shape (`Option<_>` here, almost certainly `Vector<_>` for DP-8) and lets the handler's validation be total instead of variant-dispatching. The gate's *cost* — the thing PB-DP8/9 actually want to inherit — lives entirely in `blocking_decision` and its consult sites, so it is shared regardless. | one new `GameState` field per PB ⇒ one HASH bump per PB. Predictable, and each bump's History line names exactly one field. |
| rejected: one generic `pending_decisions: Vector<PendingDecision>` | Looks tidier, but forces a sum type whose variants have unrelated payloads, forces every handler to pattern-match-then-reject, and forces an ordering policy between kinds that no CR asks for. It also would have had to be designed *now*, in a batch whose only real instance is a single scalar. | one bump total, but the enum's declared shape moves on every future PB anyway ⇒ the same number of bumps, with a wider blast radius per bump. |

The saving from the rejected option is illusory; the cost (guessing DP-8/DP-9's payloads today) is real.

### 1.3 The consult sites — and why the set is complete

**Progress (A).** The completeness argument is mechanical: *nothing advances a step or a turn except
`turn_structure::advance_step` / `advance_turn`, both of which are pure `&GameState -> TurnState`
functions that cannot advance anything themselves.* Their complete call-site set on this branch
(`rg 'advance_step\(|advance_turn\('`, workspace-wide) is:

| site | function | what it does |
|---|---|---|
| `rules/engine.rs:1960` | `handle_all_passed` | advance step |
| `rules/engine.rs:1965` | `handle_all_passed` | advance turn |
| `rules/engine.rs:2100` | `enter_step` (auto-advance tail) | advance step |
| `rules/engine.rs:2106` | `enter_step` (auto-advance tail) | advance turn |
| `rules/engine.rs:2184` | `handle_concede` | advance turn |

Three functions. Gate all three and progress is provably gated. §4 shows that one of them
(`handle_all_passed`) is already unreachable while blocked, so **only `enter_step` needs a new
guard**, plus a *clear-on-concede* in `handle_concede`.

**Admission (B).** One site: the top of `process_command`
(`rules/engine.rs:67-77`), immediately after the existing `is_game_over` check at `:74-76` and before
the `match command` at `:77`. There is exactly one entry point into the engine (Architecture
Invariant 3 / SR-3), so this single site is by construction complete.

### 1.4 Who may act while blocked

| command | while blocked | why |
|---|---|---|
| `Command::DiscardToHandSize` from the named player | **accepted** | it is the answer |
| `Command::Concede` (any player) | **accepted** | CR 104.3a is available at all times; refusing it would make a blocked game unquittable, which is strictly worse than the bug being fixed |
| everything else, including `PassPriority` and `TapForMana` | **rejected** with `GameStateError::BlockedByPendingDecision { player, decision }` | CR 514.3 — no player has priority in cleanup, so no spell can be cast and no ability activated. Mana abilities are **not** exempted: CR 605.3a lets a player activate a mana ability "any time they have priority", and in cleanup nobody does. (Contrast PB-DP4, which deliberately kept `TapForMana` available because *that* decision lives inside a priority window and CR 608.2g applies.) |

`GameStateError` is not in the SR-8 wire closure (`PROTOCOL_SCHEMA_FINGERPRINT` roots at
`Command`/`GameEvent`/`ReplayLog`; `GameStateError` is reachable from none of them) and is a
`thiserror` derive with no exhaustive match anywhere in `crates/` or `tools/` — adding a variant is
free. A distinguishable error, not a silent ignore, is required: `LocalGame::submit` already
surfaces `LocalGameError::Rejected(GameStateError)` verbatim, so the browser client of M11-local
S5 gets a machine-readable reason rather than an unexplained no-op.

### 1.5 What PB-DP8 (CR 603.3d, trigger targets) inherits — and what it does not

**Inherits, unchanged:**
- `BlockingDecision` + `blocking_decision()` + the two consult sites (§1.3). DP-8 adds a variant,
  not a site.
- The `process_command` admission gate and `BlockedByPendingDecision`.
- The `LegalAction` / `DecisionKind` / `LocalGame::advance` plumbing shape from §7 (a decision that
  is not a priority window, resolved by a dedicated `DecisionKind` variant).
- The "engine never auto-picks; a pure exported helper supplies the deterministic default for bots
  and the harness" pattern from §6.

**Does not inherit, and must be designed fresh:**
- **Cardinality.** CR 603.3b puts *all* triggers that have triggered since the last priority on the
  stack in one process; multiple triggers can each need targets, and they are controlled by
  different players in APNAP order. DP-8's pending state is a `Vector`, and answering it is a
  *sequence* of round-trips inside one flush, not one.
- **Location.** DP-7 pauses between two turn-based actions, at a point where `enter_step` has a
  natural early return. DP-8 pauses inside `abilities::flush_pending_triggers`, which is called
  from **both** branches of `enter_step` (`:2012` and `:2053`), from `check_and_flush_triggers`
  (`rules/engine.rs:60`) on ~20 command paths, and from `handle_all_passed`'s payment sweep
  (`:1919`). Its guard set is therefore **not** the guard set derived in §1.3 — DP-8 must re-derive
  from the `flush_pending_triggers` call-site set the same way this plan derived from
  `advance_step`/`advance_turn`. The *method* transfers; the *answer* does not.
- **Partial-flush resumption.** DP-7's resume is trivial because `cleanup_actions` is idempotent
  once the hand is at max size (§3.3), so the resume just re-enters `enter_step`. A half-flushed
  trigger queue has no such property: DP-8 must record where in the flush it stopped.

### 1.6 What PB-DP9 (search / scry / surveil) inherits — and the honest limit

DP-9 inherits `BlockingDecision`, the admission gate, and the DTO/plumbing shape. **It does not
inherit the resume mechanism, and this plan does not solve DP-9's hard problem.**

DP-7 and DP-8 both pause at a point where the engine is *between* actions. DP-9 pauses **inside an
`Effect` resolution**, mid-`execute_effect`, with an effect list still to run. That is a
continuation problem, not a pending-entry problem, and the engine has already met it and lost:
**OOS-DP5-5** records that a deferred draw does not suspend the rest of its effect, so
"draw three, then discard three" runs its second half against a hand that does not yet hold the
drawn cards. That is exactly the shape DP-9 hits on every `SearchLibrary` that is not the last
element of an `Effect::Sequence`.

Concretely, DP-9 needs one of:
(a) a resumable effect-list cursor on the stack object (a `GameState` shape change, HASH bump, and a
    re-entrancy audit of every `execute_effect` caller); or
(b) splitting the affected effects so the choice is always the *last* thing an effect does (works
    for a bare `Effect::SearchLibrary`, fails for `Sequence`); or
(c) accepting OOS-DP5-5's deviation permanently and documenting it per-effect.

**Say (a) is the real answer and budget for it.** Do not let this plan's "the pilot proved the
pattern" be read as "DP-9 is plumbing". It is not.

### 1.7 Retrofitting the other six mechanisms

Explicitly **out of scope** (§10, seed OOS-DP7-1). The six are
`pending_commander_zone_choices` (CR 903.9a, DP-32 — honoured but does not gate),
`pending_echo_payments` / `pending_cumulative_upkeep_payments` / `pending_recover_payments`
(CR 118.12a deadline by deliberate PB-DP4 design — these should probably **stay** deadlines, not
become blocks), the `DredgeChoiceRequired` round-trip (§9 — its "the engine pauses" doc comment is
**false**, confirmed by execution-path reading this session) and `MiracleRevealChoiceRequired`
(same shape, same suspicion, not verified here).

---

## 2. The `Command` + `GameEvent` shape

### 2.1 `Command::DiscardToHandSize`

**File**: `crates/engine/src/rules/command.rs` — append after `ChooseDredge` (`:296-309`) /
`ChooseMiracle` (`:310-328`), i.e. in the answering-command neighbourhood.

```
/// CR 514.1 / CR 701.9b (PB-DP7 / DP-3): the active player's answer to the
/// cleanup-step discard-to-hand-size turn-based action.
///
/// Sent in response to a `GameEvent::CleanupDiscardChoiceRequired`. `cards` is the
/// COMPLETE subset the player discards, not one card at a time: CR 514.1 is a single
/// turn-based action ("discard enough cards to reduce their hand size to that
/// number"), and CR 703.4n confirms it is performed as one action immediately after
/// the cleanup step begins. `cards.len()` must equal the outstanding entry's `count`
/// exactly; over- and under-supply are both rejected (a player may not discard extra
/// cards, CR 514.1).
///
/// The engine performs the discards in ascending `ObjectId` order regardless of the
/// order given here — see the plan's §2.3.
DiscardToHandSize {
    player: PlayerId,
    cards: Vec<ObjectId>,
},
```

Both field types (`PlayerId`, `Vec<ObjectId>`) are already in the wire closure, so the closure's
**type count is unchanged**; `Command`'s declared shape moves, so the digest moves — the same
situation as PROTOCOL v27 (PB-RS2).

### 2.2 `GameEvent::CleanupDiscardChoiceRequired`

**File**: `crates/engine/src/rules/events.rs` — append at the **end** of the enum (after
`TargetsChanged`, `:1341-1348`). Discriminant **129** (next free: the highest in
`state/hash.rs`'s `GameEvent` match is `RemovedFromCombat = 128` at `:5304-5308`).

```
/// CR 514.1 (PB-DP7 / DP-3): the active player has more cards in hand than their
/// maximum hand size and must choose which to discard. The engine BLOCKS — no step
/// or turn advancement, and `process_command` rejects every command except
/// `Command::DiscardToHandSize` from `player` and `Command::Concede` — until the
/// answer arrives. Unlike `DredgeChoiceRequired`, whose identical claim is not
/// implemented (seed OOS-DP7-2), this one is enforced; see `blocking_decision`.
///
/// `count` is how many cards must go. `hand` is the full set of candidate
/// `ObjectId`s at the moment of the pause; it is public information at the
/// `ObjectId` level (identities are not carried) and is supplied so a client can
/// render the choice without a second query.
///
/// Discriminant: 129.
CleanupDiscardChoiceRequired {
    player: PlayerId,
    count: u32,
    hand: Vec<ObjectId>,
},
```

`reveals_hidden_info()` (`events.rs:1361-1384`): leave on the `_ => false` catch-all. The event
carries `ObjectId`s, not card identities, and every id in it is already derivable from the public
`ObjectId` sequence. **Do not** add it to the `true` list. (Note in passing, do **not** fix here:
`DiscardedToHandSize` also returns `false` today while its sibling `CardDiscarded` returns `true`,
and discarding does reveal identity — seed **OOS-DP7-3**.)

### 2.3 One card or the subset — the CR argument (hard constraint 8)

**Decision: the whole subset, one command, one round-trip.**

- CR 514.1: "they discard enough cards to reduce their hand size to that number. **This
  turn-based action doesn't use the stack.**" Singular action, plural cards.
- CR 703.4n restates it as one turn-based action performed immediately after the step begins.
- CR 701.9b supplies who chooses ("effects that cause a player to discard a card allow the affected
  player to choose which card to discard") but not a serialisation order — there is none, because
  there is one action.
- Audit §9.4 rec 5 independently reaches the same shape from the client side: "a cleanup discard is
  a subset" — not an index into an action list.

Asking N times would (a) invent N distinct turn-based actions the CR does not have, (b) let the
player see the consequences of discard 1 before choosing discard 2, which is a real information
difference when a discard has a madness trigger, and (c) cost N wire round-trips for one rules
event.

**Madness consequence (CR 702.35a + CR 603.3b).** Discarding a subset containing k madness cards
exiles all k and queues k `PendingTrigger`s in one batch. All k are controlled by the same player
(the active player, who is the discarder), so CR 603.3b's second sentence applies: that player puts
them on the stack in any order they choose. The engine's existing same-controller ordering is a
stable sort by source `ObjectId` (`rules/abilities.rs:6963-6975`) — **that is DP-14's finding and
is out of scope here.** To avoid opening an undocumented side channel into it, PB-DP7 performs the
discards in **ascending `ObjectId` order regardless of the order supplied in `cards`**, and
validates `cards` as a set. Three reasons:

1. It keeps the engine's behaviour, and therefore the state hash, independent of a cosmetic client
   detail (SR-9b).
2. It keeps DP-14's ordering the single owner of trigger order; a "the subset's order is the
   CR 603.3b order" rule would be a second, hidden owner that DP-14's eventual fix would then
   contradict.
3. It is trivially reversible: seed **OOS-DP7-4** records that letting the subset order carry
   CR 603.3b intent is a genuine improvement, and belongs with DP-14.

### 2.4 Validation list for `handle_discard_to_hand_size`

In order, each returning `GameStateError::InvalidCommand(..)` unless noted:

1. `validate_player_exists(&state, player)` (**not** `validate_player_active` — see below).
2. There **is** an outstanding entry: else `InvalidCommand("no cleanup discard is pending")`.
3. The entry's `player` **equals** the sender: else `InvalidCommand(..)` naming both. (This is the
   SR-29 trust-boundary check; without it, any seat can discard the active player's cards.)
4. `cards.len() as u32 == entry.count` — exact. Under-supply leaves the hand illegal; over-supply
   discards cards CR 514.1 does not authorise.
5. No duplicate `ObjectId`s in `cards` (dedupe-and-compare-length).
6. Every id exists (`state.expect_object(id).is_some()`), else `ObjectNotFound(id)`.
7. Every id is in **`ZoneId::Hand(player)`** — the sender's own hand, not merely "some hand"
   (this is the OOS-DP2-1 failure mode on `handle_keep_hand`, which checks only the count;
   do not repeat it), else `ObjectNotInZone(id, ZoneId::Hand(player))`.
8. Defensive re-derivation (`debug_assert`, not a hard error): recompute
   `hand_len.saturating_sub(max_hand_size)` and assert it equals `entry.count`. Nothing can change
   the hand while blocked (§1.4 rejects every other command), so a mismatch is an engine bug, and
   `state::diagnostics` vocabulary says an engine bug gets an `expect_`/`debug_assert`, not a
   silent fizzle (SR-4).

**Why `validate_player_exists`, not `validate_player_active`:** the check that matters is #3, which
is strictly stronger (the entry's player is by construction the active player, who is by
construction alive — a dead player's entry is dropped in `blocking_decision`). Using
`validate_player_active` as well is harmless but redundant; the precedent for the weaker check on
an answering command is `ChooseDredge` (`rules/engine.rs:302-312`, with its stated reason).

---

## 3. Where the pause is taken (hard constraint 7)

### 3.1 Correcting the constraint's own premise

The wip file's hard constraint 7 asserts: *"The discard, the damage clear, the 'until end of turn'
expiry and the mana-pool empty are one turn-based action performed simultaneously."*

**That is wrong, verified against MCP CR text.** CR 514 has **two** numbered turn-based actions and
one of them is not in CR 514 at all:

- **CR 514.1** — "**First**, if the active player's hand contains more cards than their maximum
  hand size … they discard enough cards … This turn-based action doesn't use the stack."
- **CR 514.2** — "**Second**, the following actions happen **simultaneously**: all damage marked on
  permanents … is removed and all 'until end of turn' and 'this turn' effects end. This turn-based
  action doesn't use the stack."
- The mana-pool empty is **CR 500.4** ("As a step or phase begins…"), not part of CR 514 at all,
  and the engine already empties pools at the End→Cleanup transition (the call at
  `turn_actions.rs:1400` is documented as normally a no-op).

So: *first*, *second*, and only the two items **inside** 514.2 are simultaneous with each other.
The simultaneity the constraint feared does not span the discard.

### 3.2 The pause point

**In `crates/engine/src/rules/turn_actions.rs::cleanup_actions` (`:1263`), the pause is taken after
the CR 402.2 recompute (`:1266-1302`) and before the discard `loop` (`:1305`).** The function
records the entry, pushes `GameEvent::CleanupDiscardChoiceRequired`, and returns immediately with
that single event.

What has run at the pause point:
- CR 402.2 `no_max_hand_size` recompute, layer-resolved, OR'd with `no_max_hand_size_permanent`
  (`:1274-1286`). **This is load-bearing for hard constraint 9** — the `no_max` short-circuit at
  `:1306-1308` is evaluated *before* any entry can be recorded, so a Reliquary Tower / Thought
  Vessel player is never asked. Keep the recompute above the emission; do not reorder.
- `max_hand_size` and `hand_zone` reads (`:1299-1303`).

What has **not** run:
- any discard, any madness exile, any madness `PendingTrigger`;
- `clear_damage` + `GameEvent::DamageCleared` (`:1374-1375`) — CR 514.2;
- the CR 702.171b saddle clear (`:1376-1393`);
- `layers::expire_end_of_turn_effects` (`:1395`) — CR 514.2;
- `empty_all_mana_pools` (`:1400`) — CR 500.4;
- `GameEvent::CleanupPerformed` (`:1401`).

### 3.3 Why this ordering is the least-wrong, and the expiry check the constraint demanded

The constraint asks: *verify that `expire_end_of_turn_effects` does not run before the discard
choice, if the discard can be affected by an effect about to expire.*

**Verified, and it is already correct today, for a reason stronger than the code's own comment.**
CR 514.1 is performed *before* CR 514.2 by rule, so a "until end of turn, you have no maximum hand
size" or "your maximum hand size is 10" effect **must still be in force** when the discard count is
computed. The engine's existing order (recompute → discard → expire) matches the CR exactly. Taking
the pause where §3.2 puts it preserves that order and adds nothing: the count is computed with the
expiring effects live, and the expiry happens after the answer, on the resume pass.

The resume is what makes this cheap. `cleanup_actions` is **idempotent once the hand is at max
size**: the recompute is a pure function of the battlefield, the discard `loop` breaks immediately
at `:1310` when `hand_size <= max_hand_size`, and everything from `:1374` on runs exactly once
because the *first* pass returned before reaching it. So the resume path is simply "perform the
discards, clear the entry, re-enter `enter_step`" — the second pass runs `cleanup_actions` from the
top, finds nothing to discard, and completes CR 514.2 / CR 500.4 / `CleanupPerformed` normally.
No new "resume_cleanup" entry point, no duplicated events.

(Cross-check against hard constraint 6: this is exactly the shape the CR 514.3a extra-round
machinery already relies on — `enter_step` re-runs `cleanup_actions` in full on every extra round
today, so a second full pass is not a new behaviour. §4.3 walks the interleaving.)

---

## 4. Consult-site inventory (line numbers as they exist on this branch)

### 4.1 MUST-GATE

| # | file | site (line) | action |
|---|---|---|---|
| 1 | `crates/engine/src/rules/engine.rs` | `enter_step`, immediately after `is_game_over` at **:1997-2001** and **before** the CR 514.3a block at **:2007** | `if blocking_decision(state).is_some() { return Ok(events); }`. This is the **progress gate**. It must sit after `events.extend(action_events)` (`:1995`) so the `CleanupDiscardChoiceRequired` event reaches the caller, and after the `is_game_over` poll so a game that ended inside the turn-based actions still finalises. It must sit **before** `:2007` so no SBA/trigger round runs on a half-performed turn-based action. |
| 2 | `crates/engine/src/rules/engine.rs` | `process_command`, after `is_game_over` at **:74-76**, before `match command` at **:77** | the **admission gate** of §1.4: allow `DiscardToHandSize`/`Concede`, reject the rest with `GameStateError::BlockedByPendingDecision`. |
| 3 | `crates/engine/src/rules/engine.rs` | `handle_concede`, before the `handle_all_passed` call at **:2171** and before the `advance_turn` + `enter_step` block at **:2178-2189** | **clear** a pending entry whose player is the conceding player. Without this, the conceding active player leaves a stale entry that blocks the game forever at the *next* player's turn. Belt-and-braces: `blocking_decision` also returns `None` for a dead player (§1.2), so the stale entry cannot block even if the clear is missed — but the field must still be cleared or it pollutes the state hash. |
| 4 | `crates/simulator/src/local_game.rs` | `advance`, the acting-player resolution chain — a new branch **ahead of** the `pending_commander_zone_choices` branch at **:306-309** | `else if` is wrong here; the new branch must be **first**. Argument, and it is forced not preferred: the engine's admission gate (#2) rejects `ReturnCommanderToCommandZone` while blocked, so offering the commander choice first would produce a command the engine refuses and `advance()` would return `Halted(EngineError)`. Returns `(entry.player, Some(DecisionKind::CleanupDiscard))`. |
| 5 | `crates/simulator/src/legal_actions.rs` | `StubProvider::legal_actions`, a new early-returning block **ahead of** the commander-zone block at **:198-207** | must early-return (like the commander-zone and mulligan blocks at `:198-214`, unlike the payment block at `:237-327`): CR 514.3 means nothing else is legal. Ordering must match #4 for the same forced reason. Note the site this bypasses: the `priority_holder != Some(player)` early return at **:217-219** would otherwise return an empty list for everyone during cleanup. |
| 6 | `tools/tui/src/play/app.rs` | `acting_player` at **:234-245**, new branch ahead of the commander-zone check at **:236** | without it `execute_bot_turn` (`:247-293`) issues `PassPriority` for the active player, the engine rejects it (#2), `execute_command` (`:295-320`) swallows the error into `status_message`, and the TUI spins forever. See §7.4 for the human-seat half. |

### 4.2 ALREADY-SAFE — argued, no edit

| site | why it cannot step over a block |
|---|---|
| `rules/priority.rs::pass_priority` **:26-33** | rejects unless `priority_holder == Some(player)`. While blocked, `priority_holder` is `None` — written by `turn_structure::advance_step` **:103** on entry to Cleanup (and by `advance_turn` **:140**), and `enter_step`'s guard (#1) returns before either priority-granting branch (`:2032-2036`, `:2071-2093`). |
| `rules/engine.rs::handle_all_passed` **:1877**, advance branch **:1958-1971** | reachable only from `handle_pass_priority` **:1867** (which needs `pass_priority` to succeed — see above) and from `handle_concede` **:2171** (covered by #3). Structurally unreachable while blocked; gate #2 covers it a second time. |
| `rules/engine.rs::handle_all_passed` payment sweep **:1914-1949** | inside the stack-empty branch of a *priority* round; same unreachability. |
| `rules/turn_structure.rs::advance_step` **:40** / `advance_turn` **:119** | pure `&GameState -> TurnState`; they cannot advance anything. Their complete call-site set is the five rows in §1.3, all inside the three gated functions. |
| `rules/engine.rs::start_game_allowing_incomplete` **:2715** (`enter_step`) | enters at `Step::Untap` (`:2703`); `cleanup_actions` cannot have run, so no entry can exist. |
| `rules/turn_actions.rs::execute_turn_based_actions` **:20-35** | the only `Step::Cleanup` dispatch is **:29**. Every other caller in the tree is a test at a non-Cleanup step: `tests/mechanics_a_d/dungeon_resolution.rs:410` (Upkeep), `tests/mechanics_m_z/saga_class.rs:136/:171/:200`. |
| `crates/simulator/src/driver.rs::GameDriver::run_game` **:62** | re-expressed on `LocalGame::advance` (M11-local S1); `:123` asserts `AwaitingHuman` is impossible with empty `human_seats`. Covered by #4. |
| `rules/sba.rs` | SBAs never advance a step. |
| the four direct `cleanup_actions` callers in tests — `tests/primitives/pb_ac9_wheel_and_misc.rs:481`, `:512`, `:567`; `tests/primitives/pb_ac8_restrictions_and_wingame.rs:938` | **all four are `no_max_hand_size` scenarios** and therefore hit the CR 402.2 short-circuit before any entry is recorded. They stay green **provided the signature `pub fn cleanup_actions(&mut GameState) -> Vec<GameEvent>` is unchanged** — which this design does not change. (This corrects pre-survey bullet E, §9.) |

### 4.3 Interleaving with CR 514.3a (hard constraint 6)

Walk the two paths explicitly; the runner should pin both (tests T11, T12).

**Path 1 — discard, no SBA/trigger.** All pass in End → `handle_all_passed` `:1958` sees
`step != Cleanup` → `advance_step` `:1960` → `enter_step` `:1973`. `cleanup_actions` records the
entry and returns the choice event; guard #1 returns. `cleanup_sba_rounds` is still 0 (reset by
`advance_turn` `turn_structure.rs:144`; it is **not** reset by `advance_step`, so it carries within
a turn — unchanged behaviour). Player sends `DiscardToHandSize`. Handler discards, clears the entry,
calls `enter_step`. Second pass: `cleanup_actions` runs to completion (`DamageCleared`,
saddle clear, expiry, `CleanupPerformed`); the `:2007` block finds no SBA and no trigger, so
`had_events` is false and it falls through `:2039` to `:2041`; `Step::Cleanup.has_priority()` is
false; auto-advance at `:2096-2111` → next turn. `cleanup_sba_rounds` never incremented.

**Path 2 — discard queues a madness trigger.** Same until the handler. The handler's discard queues
`PendingTrigger { data: Some(TriggerData::Madness { .. }), .. }` (the existing code at
`turn_actions.rs:1348-1367`, moved into the handler). It then calls `enter_step`. Second pass:
`cleanup_actions` completes; the `:2007` block runs `flush_pending_triggers` `:2012`, which returns
events; `had_events` is true; `cleanup_sba_rounds` 0 → 1 at `:2016`; loop-detection poll `:2019`;
priority granted at `:2032-2036` and `enter_step` returns. Players pass; `handle_all_passed` `:1958`
sees `step == Cleanup` and does **not** advance (`:1954-1958`, the CR 514.3a non-advance guard),
calls `enter_step` `:1973` for round 2. Round 2's `cleanup_actions`: hand is now at max, no entry,
runs to completion; `:2007` block finds nothing; auto-advance. **`MAX_CLEANUP_SBA_ROUNDS = 100` at
`:2008` is untouched and the discard consumes zero rounds** — the block is orthogonal to the
round counter, which is what constraint 6 asks for.

**Hazard to pin (T11a):** the entry must be cleared *before* `enter_step` is re-entered, or round 2's
`cleanup_actions` would re-record it (hand is at max, so it will not — but assert it, because a
`max_hand_size`-lowering effect expiring in CR 514.2 could in principle re-open the gap. Verified:
`expire_end_of_turn_effects` restores a *higher* base `max_hand_size` if anything, and
`max_hand_size` is a `PlayerState` scalar not a layer output, so it cannot drop at expiry. Assert
anyway; the 100-round cap is the backstop.)

### 4.4 OUT OF SCOPE (named, not gated)

`pending_commander_zone_choices` (DP-32), the three payment vectors (PB-DP4's deadline design is
deliberate and should probably stay), `DredgeChoiceRequired`, `MiracleRevealChoiceRequired`,
`pending_zone_changes` (already object-scoped and correct). Seeds in §10.

---

## 5. Wire expectation, and what would falsify it

### 5.1 `PROTOCOL_VERSION` 27 → 28 — **certain**

Adding `Command::DiscardToHandSize` and `GameEvent::CleanupDiscardChoiceRequired` moves two
wire-frame types' declared shapes. Closure **type count is unchanged** (`PlayerId`, `Vec<ObjectId>`,
`u32` are all already reachable).

Procedure, verbatim from `rules/protocol.rs:315-325`, in **one** commit:
1. `PROTOCOL_VERSION` 27 → **28** at `protocol.rs:260`, plus a `- 28:` History line above it.
2. **Append** a `ProtocolEpoch { version: 28, fingerprint: <gate-computed> }` to
   `PROTOCOL_HISTORY` (the array starts at `protocol.rs:330`; **never edit an existing row**) and
   set `PROTOCOL_SCHEMA_FINGERPRINT` (`protocol.rs:277-278`) to the **same** value.
3. Update `protocol_version_sentinel` (`crates/engine/tests/core/protocol_schema.rs:866-869`,
   currently asserting `27`) and re-pin `FROZEN_HISTORY_PREFIX_DIGEST`
   (`protocol_schema.rs:148-149`) to the value `frozen_prefix_is_pinned` prints.

**Never hand-invent a fingerprint.** Both values are printed by the failing gate
(`protocol_schema.rs:852-855` prints the recompute; `:1067-1077` prints the prefix digest). Take
them from the failure text.

**Falsifier**: none realistic. The only way PROTOCOL stays 27 is if no `Command`/`GameEvent`
variant is added — i.e. if the answering command were expressed by reusing an existing variant.
There is no candidate (`rg 'Discard' crates/engine/src/rules/command.rs` returns **zero** matches
on this branch — pre-survey bullet C's "confirm with a grep" is **confirmed**), and reusing
`ChooseDredge`-style overloading would be exactly the accepted-and-discarded-field antipattern
DP-24 catalogues.

### 5.2 `HASH_SCHEMA_VERSION` 64 → 65 — **certain**

`GameState` gains `pending_cleanup_discard: Option<PendingCleanupDiscard>`.

- `crates/card-types/src/state/stubs.rs` — new `pub struct PendingCleanupDiscard { pub player:
  PlayerId, pub count: u32 }`, `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]`.
  (Home chosen to match `PendingTrigger`; `PendingDraw` lives in `replacement_effect.rs` only
  because it *is* a replacement, which this is not.)
- `crates/engine/src/state/mod.rs` — the field next to `pending_draws` (`:139-142`), `pub(crate)`,
  `#[serde(default)]` (so a pre-PB-DP7 snapshot decodes), plus a read accessor
  `pub fn pending_cleanup_discard(&self) -> Option<&PendingCleanupDiscard>` beside
  `pending_draws()` (`:444-446`). **No `_mut` accessor** — SR-3 keeps mutation inside
  `process_command`; the simulator only reads.
- `crates/engine/src/state/builder.rs` — init `None` beside `pending_draws: Vector::new()` (`:321`).
- `crates/engine/src/state/hash.rs` — `impl HashInto for PendingCleanupDiscard` feeding **both**
  fields (SR-19's `every_hashed_struct_field_is_hashed_or_allowlisted` gate at
  `tests/core/hash_schema.rs:1526` fails otherwise, and the `NOT_HASHED` allowlist is empty and
  should stay empty); fold `self.pending_cleanup_discard.hash_into(&mut hasher)` into
  `public_state_hash` beside `:7739` (the blanket `impl<T: HashInto> HashInto for Option<T>` at
  `:999-1009` covers the `Option`). Public, not private: the entry names a player and a count, both
  public information. Also mirror into `rules/loop_detection.rs`'s mandatory-state fingerprint
  beside `:146-150` — a blocked cleanup is a distinct position.
- `HASH_SCHEMA_VERSION` 64 → **65** (`hash.rs:591`) + a `- 65:` History line; **append** a
  `HashSchemaEpoch { version: 65, .. }` row after the v64 row (`hash.rs:892-899`) with **both**
  gate-computed fingerprints; update the `HASH_SCHEMA_VERSION` sentinel in
  `tests/core/hash_schema.rs`.
- Also add the `GameEvent::CleanupDiscardChoiceRequired` arm (discriminant `129u8`) to the
  `GameEvent` hashing match, after `RemovedFromCombat` at `hash.rs:5304-5308`. That match has **no
  `_` arm** — this is the compile-gate constraint SR-8 exists for.

**Falsifier, stated as the wip asks**: *if the pending entry could live outside `GameState`, HASH
stays 64.* It cannot. `process_command(state: GameState, command: Command) -> Result<(GameState,
Vec<GameEvent>), _>` (`engine.rs:67-70`) takes the state by value and returns it; there is no other
carrier between two commands. The only alternative — re-deriving "is a discard pending" from
`step == Cleanup && hand > max && !no_max` without storing anything — is rejected because it cannot
distinguish "the pause has been taken" from "the pause is about to be taken", so `enter_step` would
loop and `CleanupDiscardChoiceRequired` would be re-emitted on every entry. (It also could not
survive a `count` recomputation disagreement, and it makes the entry invisible to
`loop_detection`.)

`PendingCleanupDiscard` is reachable only from `GameState`, never from
`Command`/`GameEvent`/`ReplayLog`, so it contributes **nothing** to `PROTOCOL_SCHEMA_FINGERPRINT` —
exactly the PB-DP5 `PendingDraw` precedent (`hash.rs:587-590`).

### 5.3 Bump both in the same commit

Both gates fail simultaneously on the first `cargo test --all`. Bump both, take both fingerprint
sets from the two failure texts, and say in the commit message that the fingerprints are
gate-computed.

---

## 6. Bot / fuzzer auto-answer and its determinism argument (hard constraint 5)

**Rule: the default pick is the `count` highest `ObjectId`s in hand, which reproduces today's
behaviour exactly.**

Today's loop (`turn_actions.rs:1305-1372`) takes `obj_ids.last()` each iteration.
`Zone::object_ids()` on an `Unordered` zone (`crates/card-types/src/state/zone.rs:130-135`) iterates
an `imbl::OrdSet`, i.e. **ascending by `ObjectId`** — pre-survey bullet A's final claim is
**confirmed**. `ZoneId::Hand` is built as `Zone::new_unordered()` (`state/builder.rs:287`). So
today's *k*-card discard removes the *k* highest ids, one at a time. "The `count` highest ids,
sorted ascending, in one command" is byte-identical in outcome.

**Where the default lives.** A pure exported helper in the engine, called by nobody in the engine:

```
// crates/engine/src/rules/turn_actions.rs
/// CR 514.1 (PB-DP7): the deterministic default cleanup-discard subset — the
/// `count` highest `ObjectId`s in `player`'s hand, ascending. This reproduces the
/// pre-PB-DP7 auto-pick exactly (`obj_ids.last()` on an ascending `OrdSet`), so a
/// bot game's command trace and the fuzzer baseline do not churn (SR-9b).
///
/// The ENGINE NEVER CALLS THIS on a decision path. It exists so the simulator's
/// `StubProvider`, the replay harness and the TUI cannot drift from one another.
pub fn default_cleanup_discard(state: &GameState, player: PlayerId) -> Vec<ObjectId>
```

Consumers: `StubProvider::legal_actions` (§7.1), `replay_harness` (§7.5) when a script's
`discard_to_hand_size` action names no cards, and the TUI (§7.4).

**Determinism argument.**
- `OrdSet` iteration is total-order deterministic; no `HashSet` or `HashMap` iteration is involved
  anywhere in this path (contrast PB-DP5's `already_applied`, which needed an explicit sort).
- The chosen ids are a pure function of `GameState`, so `build_initial_state`-driven cross-regime
  comparisons (SR-9b) and the golden-script harness agree by construction.
- The one thing that **does** change for a bot game: one extra `Command` per cleanup discard is now
  applied and journalled. Consequences the runner must expect and must **not** paper over:
  - `LocalGame.command_count` (`local_game.rs:164`) rises, moving games slightly closer to
    `limits.max_commands` (`:283-287`).
  - `consecutive_passes` (`:163`) is **reset to 0** by the discard command in `apply_command`'s
    bot path — the discard is not a `PassPriority`. This makes the `max_consecutive_passes` valve
    (`:289-293`) slightly *less* likely to trip. Both are safety valves, not semantics.
  - `loop_detection::reset_loop_detection` **should** be called on the discard command, per
    CR 104.4b ("a meaningful player choice"), matching `ChooseDredge` (`engine.rs:307-308`). This
    changes `loop_detection_hashes` in pathological games; that is correct, not a regression.
- **`mtg-fuzzer` caveat**: per **OOS-DP3-9** the fuzzer already aborts on a stack overflow at ~15
  games on `main`, and long games trip `stack_consistency`. Do not chase it here; do not let it mask
  a real regression either. The honest check is a **fixed-seed A/B**: run N seeds on `main` and on
  the branch and confirm the winner and turn count match for every seed that completes on both.

---

## 7. `LegalAction` / `DecisionKind` / `LocalGame` plumbing

### 7.1 `LegalAction`

`crates/simulator/src/legal_actions.rs`, appended to the enum (`:16-144`, currently ending at
`PayRecover` `:140-143`):

```
/// CR 514.1 / CR 701.9b (PB-DP7 / DP-3): answer the outstanding cleanup discard.
/// `count` is how many must go and `hand` is the full candidate set, so a human
/// client can render a real subset picker. `cards` is the deterministic default
/// (`mtg_engine::rules::turn_actions::default_cleanup_discard`) — exactly `count`
/// distinct ids from `hand`, so a bot that submits it verbatim is always accepted
/// (SR-38: never offer an action the engine rejects).
DiscardToHandSize {
    count: u32,
    hand: Vec<ObjectId>,
    cards: Vec<ObjectId>,
},
```

Exactly one such action is offered, and the block early-returns (§4.1 #5). Offering one action per
candidate subset is combinatorial (C(10,3) = 120 for a 10-card hand) and is not done.

Both `crates/simulator/src/random_bot.rs::action_to_command` (`:128-366`) and
`crates/simulator/src/heuristic_bot.rs`'s scorer (`:98-107` neighbourhood) match `LegalAction`
**exhaustively with no `_` arm** — both are compile-forced to gain an arm. `random_bot` maps to
`Command::DiscardToHandSize { player, cards: cards.clone() }`; `heuristic_bot` scores it high (it is
the only legal action; any score works, but a high one documents that it is not optional).

### 7.2 `DecisionKind`

`crates/simulator/src/local_game.rs:92-98`. Add `CleanupDiscard`, **and make the enum
`#[non_exhaustive]`** — audit §9.4 rec 1, which asks for it explicitly and which this batch makes
concretely true (the enum is no longer "the complete set of decisions reachable by this
architecture", the claim §9.2 line 721 makes). Cost: `#[non_exhaustive]` is a no-op within
`crates/simulator` but forces a wildcard arm in downstream crates. On this branch there is no
downstream exhaustive match on `DecisionKind` (`tools/` does not reference it), so the change is
free today and load-bearing for M11-local S5's DTO. Update the enum's doc comment to say it
enumerates **command-submission-time and out-of-band engine-blocking** decisions, with a pointer to
`docs/audits/decision-point-audit.md` for the trigger-time and resolution-time classes it still
cannot reach.

`decision_kind_for` (`:575-592`) needs **no** change — `advance()` supplies `forced_kind`
(`:353`), the same way the commander-zone branch does.

Audit §9.4 rec 2 (`PendingDecision.payload: DecisionPayload`) is **not** done here: this decision
still fits `actions: Vec<LegalAction>` with a single element carrying its own payload, and
reshaping the struct is M11-local Session 3/5's call. Seed **OOS-DP7-5**.

### 7.3 `LocalGame`

- `advance()` `:253-419`: the new first branch (§4.1 #4). The existing idempotence guard at
  `:270-274` already makes a repeated `advance()` return the same `seq` — verify by test (T14),
  because that guard is the thing S1's review added and a browser refresh depends on it.
- The human-seat stop at `:351-355` is reached the same way as for any other decision.
- `submit()` `:425-494`: no change needed. `command_player` (`:559-563`) extracts `player` from the
  externally-tagged JSON, which works for `DiscardToHandSize { player, cards }` — but
  `test_command_player_extracts_acting_player` (`:615` region) pins that property and should gain
  the new variant.
- The empty-legal-actions auto-pass at `:331-349` is **not** reachable for a blocked seat: the
  provider always offers exactly one action.

### 7.4 TUI (`tools/tui/`)

- `play/app.rs::acting_player` **:234-245** — new first branch (§4.1 #6). Required, or the TUI hangs.
- `play/input.rs::handle_normal_mode` **:31-...** — the TUI has **no** exhaustive match on
  `LegalAction` (it uses `matches!` lookups at `:49/:59/:78/:122/:140/:164/:193`), so nothing
  compile-breaks; but a human seat with an outstanding discard has **no key to answer**, which is a
  hang. Minimum viable: a key that submits the offered action's `cards` (the deterministic
  default). A real subset picker is M11-local Session 7 work — seed **OOS-DP7-6**.
- `play/app.rs` event formatter **:575-580** neighbourhood has a `_ => String::new()` catch-all at
  `:594`, so no compile break; add a display arm for `CleanupDiscardChoiceRequired` anyway.

### 7.5 Replay harness (hard constraint 4)

- `crates/engine/src/testing/replay_harness.rs` — new action arm beside `"choose_dredge"`
  (`:900-903`):
  `"discard_to_hand_size"` → resolve each name in a new `discard_cards: Vec<String>` field against
  `ZoneId::Hand(player)`; if the field is empty, fall back to
  `turn_actions::default_cleanup_discard`. Returns `Command::DiscardToHandSize`.
- `crates/engine/src/testing/script_schema.rs` — `ScriptAction::PlayerAction` (`:250-...`) is
  `#[serde(deny_unknown_fields)]`, so the new optional field must be declared:
  `#[serde(default)] discard_cards: Vec<String>`, following the `convoke`/`delve`/`escape` pattern
  (`:292-316`). Also extend the `action:` doc list at `:254-258`.
- **Existing scripts**: a grep of `test-data/generated-scripts/` finds **no** script asserting
  `DiscardedToHandSize`, and the two cleanup-hand-size scripts
  (`stack/038_thought_vessel_no_max_hand.json`, `stack/039_reliquary_tower_no_discard.json`) are
  both no-max scenarios that hit the CR 402.2 short-circuit and never pause. **The runner must still
  run the full script suite** (`cargo test --test run_all_scripts`) — a script that incidentally
  reaches a cleanup step with an oversized hand would now halt, and that is the cheapest possible
  place to discover it.
- SR-9c: no new **assertion** path is introduced, so no `check_assertions` work. Recommended (not
  required): one new approved script exercising choose-a-non-madness-card, which is the scenario
  the JSON regime is best at documenting.

### 7.6 Replay viewer

- `tools/replay-viewer/src/view_model.rs` matches `StackObjectKind` (`:427-...`) and
  `KeywordAbility` exhaustively — **neither** is touched by this batch, so despite the standing
  warning there is likely nothing to do there. **Verify with `cargo build --workspace`, do not
  assume.**
- `tools/replay-viewer/frontend/src/lib/eventFormat.js` — add a `case
  'CleanupDiscardChoiceRequired':` next to `'DiscardedToHandSize'` at `:57-58` and `:434`. JS: no
  compile gate, so this is an easy silent miss.

---

## 8. Test list, with per-test fail-before predictions

**File**: new — `crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs`, registered in
`crates/engine/tests/primitives/mod.rs` (SR-9a: never add a top-level `tests/*.rs`; a dropped `mod`
line silently deletes coverage).
**Simulator tests**: `crates/simulator/src/local_game.rs`'s `mod tests` (`:610+`) and
`crates/simulator/src/legal_actions.rs`'s `mod tests`.

Because the new `Command`, `GameEvent` and accessor do not exist pre-fix, a test that names them
cannot compile on `main`. Each row therefore states its **fail-before probe**: an assertion
expressible against *today's* API that fails today, so the runner can demonstrate the defect before
writing the fix.

| # | test | asserts | fail-before probe (expressible today) |
|---|---|---|---|
| T1 | `test_dp7_cleanup_discard_blocks_step_advance` | 4-player, P1 active, 9 cards; all pass in End. Then: `step == Cleanup`, `priority_holder == None`, `turn_number` unchanged, `pending_cleanup_discard() == Some{player: p1, count: 2}`, hand still 9, exactly one `CleanupDiscardChoiceRequired` in events. **This is criterion 5540's "observes the block".** | assert `state.turn().step == Step::Cleanup` after the four passes — **fails today** (it is `Step::Untap` of P2's turn) |
| T2 | `test_dp7_pass_priority_rejected_while_blocked` | from T1's state, `process_command(PassPriority{p1})` is `Err`; state unchanged | assert the same `PassPriority` errors — **fails today** (it succeeds; there is no blocked state to be in) |
| T3 | `test_dp7_unrelated_command_rejected_while_blocked` | `PlayLand` / `TapForMana` / `CastSpell` from **any** seat → `Err(BlockedByPendingDecision { .. })`; state byte-identical (compare `public_state_hash`) | as T2 |
| T4 | `test_dp7_concede_while_blocked_clears_entry` | active player concedes while blocked → accepted, `pending_cleanup_discard() == None`, game advances to the next player's turn, no hang | assert a mid-cleanup concede is possible at all — **not expressible today**; the state cannot exist. Record as new-surface-only. |
| T5 | `test_dp7_chosen_cards_are_discarded_not_the_highest_ids` | 9 cards; answer naming the two **lowest** ids; those two in graveyard, the two highest still in hand, hand == 7 | assert the two lowest ids are in the graveyard after cleanup — **fails today** (the two highest go) |
| T6 | `test_dp7_madness_does_not_fire_on_an_unchosen_card` (**criterion 5541**) | 8 cards; Fiery Temper built **last** so it holds the highest `ObjectId`; answer naming a plain filler. Assert: Fiery Temper still in `Hand(p1)`, **not** in `Exile`, zero `PendingTrigger` with `PendingTriggerKind::Madness`, no `StackObjectKind::MadnessTrigger`. Cite CR 702.35a + CR 701.9b. | assert Fiery Temper is still in hand after the four passes — **fails today**: it is exiled and its madness trigger is queued, involuntarily |
| T7 | `test_dp7_madness_fires_on_a_chosen_card` | same board, answer naming Fiery Temper → exile + one madness `PendingTrigger` with the `{R}` cost from `fiery_temper.rs`. CR 702.35a. | passes today (it is the current behaviour); it is the regression guard that T6 did not break madness |
| T8 | `test_dp7_answer_validation` (table-driven, 7 cases) | wrong count low / wrong count high / duplicate id / id in another player's hand / id on the battlefield / unknown `ObjectId` / wrong sender → each `Err`, each with a distinct message; state unchanged in every case | new-surface-only |
| T9 | `test_dp7_no_max_hand_size_never_pauses` (**hard constraint 9**) | (a) `NoMaxHandSize` printed keyword, (b) layer-granted `AddKeyword(NoMaxHandSize)` (the PB-AC8 emblem-proxy shape), (c) `no_max_hand_size_permanent` (PB-AC9): each with 10 cards → no pending entry, no `CleanupDiscardChoiceRequired`, turn advances, hand still 10 | passes today (regression guard for CR 402.2, protecting the ordering in §3.2) |
| T10 | `test_dp7_cr_514_2_is_deferred_until_the_answer` (**hard constraint 7**) | mark damage on a creature and register an `UntilEndOfTurn` continuous effect; at the blocked point assert `damage_marked > 0`, the effect still in `continuous_effects`, and **no** `DamageCleared`; after the answer assert `DamageCleared`, `damage_marked == 0`, effect gone, `CleanupPerformed` emitted exactly once | assert `damage_marked > 0` at the moment the cleanup step is entered — **fails today** (CR 514.2 has already run by the time the pass returns) |
| T11 | `test_dp7_madness_discard_runs_an_extra_cleanup_round` (**hard constraint 6**) | after answering with a madness card: priority granted in `Step::Cleanup`, `cleanup_sba_rounds == 1`, madness trigger on the stack; all pass → `handle_all_passed` does **not** advance; round 2 completes and the turn advances. Also T11a: `cleanup_sba_rounds` is **0** at the blocked point (the pause consumes no round). | partially expressible: assert `cleanup_sba_rounds == 1` after a madness cleanup — passes today; the new half (0 at the blocked point) is new-surface-only |
| T12 | `test_dp7_three_discards_one_command` | hand 10 → `count == 3`; one command; exactly three `DiscardedToHandSize` events; hand == 7; graveyard == 3 | assert three discard events from one *command* — new-surface-only; the count-3 outcome passes today via three internal iterations |
| T13 | `test_dp7_default_pick_reproduces_pre_pb_behaviour` (**hard constraint 5**) | `default_cleanup_discard(state, p1)` returns exactly the `count` highest ids, ascending, and equals the set the pre-PB loop would have taken | passes by construction; it is the determinism pin |
| T14 | `test_dp7_local_game_awaits_human_on_cleanup_discard` (simulator) | human seat: `advance()` → `AwaitingHuman { kind: DecisionKind::CleanupDiscard, player, actions.len() == 1 }`; a second `advance()` returns the **same** `seq` (S1's idempotence); `submit(seq, cmd naming another seat)` → `BadParams`; `submit(stale_seq, ..)` → `StaleDecision`; correct `submit` → game proceeds | new-surface-only |
| T15 | `test_dp7_local_game_bot_seat_auto_answers` (simulator) | bot-only `LocalGame`, seeded, forced oversized hand, run ≥3 turns: never `Halted`, at least one `DiscardToHandSize` in the journal | **fails today** in the sense that no such command can exist; the *regression* it guards (a bot game halting at cleanup) is the thing to watch |
| T16 | `test_dp7_stub_provider_offers_only_the_discard` (simulator) | blocked player gets exactly one action, `DiscardToHandSize`, with `cards.len() == count` and every id in `hand`; every other player gets `[]`; and the offered `cards` is accepted by `process_command` (SR-38) | new-surface-only |
| T17 | `test_dp7_pending_entry_is_hashed` | two states differing only in `pending_cleanup_discard` produce different `public_state_hash`es (mirrors `test_hash_cleanup_sba_rounds_affects_hash`, `tests/rules/replacement_effects.rs:3180`) | new-surface-only |
| T18 | `test_dp7_blocked_state_survives_a_clone_roundtrip` | serde round-trip of a blocked `GameState` preserves the entry; a pre-PB-DP7 snapshot without the field decodes (the `#[serde(default)]` pin) | new-surface-only |

### 8.1 Existing tests predicted to change, with the CR justification for each change

Every one of these changes is "the test now *chooses* the card instead of relying on the
auto-picker". **None** of them is an assertion weakened to fit the implementation.

| file:line | test | change | CR justification |
|---|---|---|---|
| `crates/engine/tests/core/turn_actions.rs:183` | `test_cleanup_discards_to_hand_size` | after the fourth pass, send `DiscardToHandSize` with 2 chosen ids; then assert the same 2 discard events / hand 7 / graveyard 2 | CR 514.1 + CR 701.9b: the player chooses; the *count* assertion is unchanged and is the real content |
| `crates/engine/tests/core/turn_actions.rs:318` | `test_cleanup_discard_event_uses_hand_id` (MR-M2-06) | same insertion; the MR-M2-06 assertion (event carries the **old hand** `ObjectId`) is unchanged | unchanged rule; only the trigger of the discard moves |
| `crates/engine/tests/core/card_def_fixes.rs:371` | `test_thought_vessel_no_max_hand_size` | **no change** | CR 402.2 short-circuit; never pauses (T9 covers it) |
| `crates/engine/tests/core/card_def_fixes.rs:437` | `test_no_thought_vessel_discards_to_hand_size` | insert the answering command; assertions unchanged | as above |
| `crates/engine/tests/core/card_def_fixes.rs:511` | `test_thought_vessel_only_affects_its_controller_other_players_discard` | insert the answering command **for P2** (the active player), not P1 | CR 514.1 names the **active** player; the test's whole point |
| `crates/engine/tests/mechanics_m_z/madness.rs:~222` | `test_madness_cleanup_discard_exiles` (test 1) | insert `DiscardToHandSize` naming Fiery Temper; **rewrite the comment at `:238`** | CR 702.35a is about what happens *when* a madness card is discarded; the test's subject is unchanged. The old comment ("Add Fiery Temper last … `last()` picks it for discard") documented an engine artefact, not a rule |
| `crates/engine/tests/mechanics_m_z/madness.rs:286` | `test_madness_non_madness_card_goes_to_graveyard` (test 2) | insert the command naming Plain Instant; **rewrite the comment at `:292`** | CR 702.35a negative case, unchanged |
| `crates/engine/tests/mechanics_m_z/madness.rs:346` | `test_madness_trigger_on_stack_after_discard` (test 3) | insert the command naming Fiery Temper; **rewrite the comment at `:354`** | CR 702.35a, unchanged |
| `crates/engine/tests/mechanics_m_z/madness.rs:569` | `test_madness_decline_goes_to_graveyard` (test 6) | insert the command naming Fiery Temper; **rewrite the comment at `:575`** | CR 702.35a, unchanged |

Comments at `madness.rs:238/:292/:354/:575` and `turn_actions.rs:185/:321` currently *document*
`obj_ids.last()`. They are the evidence the fix is real; they must be replaced with a sentence
naming the chosen card, not deleted silently.

**Not expected to change but must be run and watched**: `tests/core/concede.rs` (library cards, not
hand), `tests/core/six_player.rs:52`, `tests/core/turn_structure.rs:214`,
`tests/core/turn_invariants.rs:15` (all `ZoneId::Library`). The specific hazard in
`turn_structure.rs:227` is `state.turn().priority_holder.unwrap()` inside a loop — it panics on a
`None`, which is now a reachable state during cleanup. Hands there stay small, so it should hold;
verify rather than assume.

### 8.2 Gate tests requiring an edit (not new tests)

- `crates/engine/tests/core/protocol_schema.rs` — `protocol_version_sentinel` `:866-869`
  (27 → 28) and `FROZEN_HISTORY_PREFIX_DIGEST` `:148-149`.
- `crates/engine/tests/core/hash_schema.rs` — the `HASH_SCHEMA_VERSION` sentinel (64 → 65) and both
  epoch fingerprints; `MIN_HASHINTO_IMPLS` / `MIN_NAMED_STRUCTS` are `>=` assertions and need no
  edit.
- `crates/simulator/src/local_game.rs` `test_command_player_extracts_acting_player` (`:615` region)
  — add the new variant.

---

## 9. Pre-survey bullets that turned out to be WRONG

Verified against source on this branch, 2026-07-26. Confirmed bullets are not listed except where
the confirmation is load-bearing.

1. **Hard constraint 7's CR premise is wrong.** It states the discard, damage clear, "until end of
   turn" expiry and mana-pool empty are "*one* turn-based action performed simultaneously". CR 514.1
   and CR 514.2 are **two** turn-based actions ("First…", "Second…"), only the two items *inside*
   514.2 are simultaneous, and the pool empty is CR 500.4, not part of CR 514 at all. The whole
   "pausing in the middle of it is a modelling choice" framing dissolves: the pause is taken at a
   CR-supplied boundary. (§3.1)
2. **Bullet A's line numbers for the discard loop are stale.** The audit cites
   `turn_actions.rs:1280-1293`; on this branch `cleanup_actions` is `:1263`, the CR 402.2 recompute
   `:1266-1302`, the discard `loop` **`:1305-1372`**, the auto-pick `:1318`, the madness branch
   `:1322-1367`, `clear_damage` `:1374`, saddle `:1376-1393`, expiry `:1395`, pools `:1400`,
   `CleanupPerformed` `:1401`. The audit's `:1280-1293` range covers the CR 402.2 comment block,
   not the loop.
3. **Bullet C's `PROTOCOL_SCHEMA_HISTORY` line number is wrong.** It says `:294+`; the array
   `PROTOCOL_HISTORY` begins at `protocol.rs:330`. `:294` is the doc comment's "Why this exists"
   heading. The bump-procedure comment is `:315-325`, not `:318-321`.
4. **Bullet E is wrong about the four direct-call tests.** It calls
   `pb_ac9_wheel_and_misc.rs:481/:512/:567` and `pb_ac8_restrictions_and_wingame.rs:938`
   "near-certain fallout" and "the cheapest place to pin the new behaviour". **All four are
   `no_max_hand_size` scenarios** that hit the CR 402.2 short-circuit and never reach the discard,
   so they neither break nor can pin anything about the discard — provided `cleanup_actions`'s
   signature is unchanged, which this design keeps. They are useful only as constraint-9
   regression guards.
5. **Bullet E understates the madness fallout by one and misattributes it.** It names three lines
   (`:292`, `:354`, `:575`); there are **four** affected tests — test 1 at `:~222` (whose
   equivalent comment is at `:238`, a line the bullet does not name), plus tests 2, 3 and 6.
6. **Bullet C's suspicion about `DredgeChoiceRequired` is CONFIRMED, and it is worse than a doc
   bug.** `events.rs:848` claims "The engine pauses until a `Command::ChooseDredge` is received".
   It does not. `check_would_draw_replacement` returns `DrawAction::DredgeAvailable`
   (`replacement.rs:686-692`); `perform_one_draw` maps it to
   `(vec![event], DrawStepOutcome::DredgeOffered)` at `:828` and records **no** pending state; the
   `DrawStepOutcome` doc at `:764-767` says explicitly "the caller does **NOT** stop". The draw
   simply does not happen and nothing remembers it. `handle_choose_dredge`
   (`replacement.rs:2898-2943`) validates the card but never checks that a dredge was offered —
   that is **OOS-DP5-7**, already filed, and this session confirms it by reading the path. Seed
   **OOS-DP7-2** covers the false doc comment on both `DredgeChoiceRequired` and
   `MiracleRevealChoiceRequired` (`events.rs:836`).
7. **Bullet D's `DecisionKind` prediction is right, and its `LocalGame` prediction is right but
   incomplete.** It correctly predicts a `DecisionKind` variant and a `LocalGame` path. It does not
   mention that **`StubProvider::legal_actions` early-returns an empty list whenever
   `priority_holder != Some(player)`** (`legal_actions.rs:217-219`), which is the *reason* the
   provider needs a block placed above that check — not merely a convenience. Nor does it mention
   `tools/tui/src/play/app.rs::acting_player` (`:234-245`), which hangs without a branch.
8. **Bullet B's candidate gate set is over-broad in one direction and under-specified in another.**
   `handle_all_passed`'s advance branch (`:1958-1971`) does **not** need a gate: `pass_priority`
   (`priority.rs:26-33`) already rejects when `priority_holder` is `None`, which it always is while
   blocked, so the branch is structurally unreachable. `turn_structure::advance_step`/`advance_turn`
   do not need gates either — they are pure. Conversely the bullet omits **`handle_concede`**
   (`engine.rs:2178-2189`), which is a genuine third advancer and a genuine stale-entry hazard.
9. **Bullet F's yield prediction is right on flips and understates the live-wrong exposure.** "0
   completeness flips" is **confirmed**. But the corpus check it asks for finds **three** `Complete`,
   deck-legal Madness defs (`fiery_temper.rs`, `stensia_masquerade.rs`, `markov_baron.rs`), each of
   which can be involuntarily exiled today whenever it is the most recently drawn card at a cleanup
   with an oversized hand. So DP-3 is not merely "an engine-agency fix": it is **live-wrong on three
   `Complete` cards**, in the DP-10/DP-11 sense.

**Confirmed as stated** (recorded so the reviewer knows the checks ran): bullet A's
`zone.rs:130-135` ascending-`OrdSet` claim; bullet C's "there is currently **no** `Discard*` variant
of any kind in `command.rs`" (grep returns zero); bullet C's `hash.rs` pointers (`:591`, `:4806`
discriminant 72, `:7736`); bullet C's `state/mod.rs` pointers (`:138-155`, `:275-301`,
accessors `:439-446` / `:725-727`); bullet B's "cleanup has no priority" via
`state/turn.rs:63-65`.

---

## 10. Seeds to file in `docs/audits/decision-point-audit.md` §8.1

| seed | finding | class |
|---|---|---|
| **OOS-DP7-1** | **Retrofit the other six pending mechanisms onto `blocking_decision`, selectively.** `pending_commander_zone_choices` (CR 903.9a, DP-32) is honoured but does not gate and *should* — it is a real choice with no default. The three payment vectors should **stay** deadlines (PB-DP4's design is deliberate and CR 118.12a supplies the default). `pending_zone_changes` is already object-scoped and correct. So the retrofit list is **one** entry, not six, and the audit's "seed it, with the list" is answered by narrowing the list. | correctness, follow-up PB |
| **OOS-DP7-2** | **`DredgeChoiceRequired` and `MiracleRevealChoiceRequired` both document a pause the engine does not implement.** `events.rs:848` and `:836`. For dredge the path is confirmed (§9 item 6): no pending state is recorded, the draw silently does not happen, and `handle_choose_dredge` never checks that dredge was offered (the live free-card exploit already filed as **OOS-DP5-7**). Miracle is the same shape and was not verified here. Minimum action: fix the two doc comments so they stop asserting a guarantee. Real action: give both a `BlockingDecision` variant. | correctness + false documentation |
| **OOS-DP7-3** | **`GameEvent::DiscardedToHandSize` returns `false` from `reveals_hidden_info()`** (it falls through the `_` arm at `events.rs:1382`) while its sibling `CardDiscarded` returns `true` at `:1367`. Discarding reveals the card's identity either way. M10's safe-rewind-checkpoint logic reads this predicate. One-line fix, deliberately not taken here to keep the batch's wire story clean. | correctness, M10-gated |
| **OOS-DP7-4** | **Let the `cards` subset order the resulting madness triggers (CR 603.3b).** PB-DP7 deliberately sorts the discard ascending so the client's ordering cannot become a hidden, undocumented input to trigger order while DP-14 still owns that decision. When DP-14 lands, the subset order is the natural carrier of "any order they choose" for same-controller triggers, and PB-DP7's sort should be revisited **with** it, not before. | agency, DP-14 scope |
| **OOS-DP7-5** | **`PendingDecision` still has a flat `actions: Vec<LegalAction>`** (audit §9.4 rec 2). PB-DP7 fits inside it by putting the payload on the single `LegalAction`, which works but is the last decision class that will. A `payload: DecisionPayload` reshaping belongs to M11-local Session 3/5, before the browser DTO is locked. | design debt, M11-local |
| **OOS-DP7-6** | **The TUI has no subset picker.** `tools/tui/src/play/input.rs` gets a key that submits the engine's deterministic default, which is agency-preserving for bots and agency-free for the human at the TUI seat. A real picker is M11-local Session 7 (audit §9.4 rec 8/9). | UX gap, M11-local |
| **OOS-DP7-7** | **The audit's §10 re-audit trigger is now DUE.** "After PB-DP7 or any first pending-decision-that-blocks lands — re-run §3.1's sweep and re-derive the 277 figure." Also due from §10's second bullet: PB-DP7 adds a new `Command` variant, so §5/DP-24's "is it another accepted-and-discarded field" check applies (answer: no — every field of `DiscardToHandSize` is validated, §2.4). Coordinator's job, not the runner's. | bookkeeping |

Additional cross-references to update in the audit when this ships: §4.11 line **400** (Hand-size
discard row **B → A**), §5 **DP-3** row (SHIPPED banner + the three corrections in §9 items 1/4/9),
§8 the **PB-DP7** row (wire prediction confirmed: PROTOCOL **and** HASH, where the row predicted
only PROTOCOL — record that), §8 the **sequencing note** (point PB-DP8/DP9 at §1.5/§1.6 of this
plan), §9.3 and §9.4 rec 1 (`DecisionKind` now `#[non_exhaustive]` — rec 1 **done**), §9.4 rec 5
(subset shape **confirmed** by CR 514.1).

---

## 11. Verification checklist

- [ ] `cargo build --workspace` clean after **every** phase (SR-8; `tools/replay-viewer` and
      `tools/tui` are the two runners miss ~50% of the time)
- [ ] `cargo test --all` green — includes `tools/check-defs-fmt.sh` via `core card_defs_fmt` (SR-35)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh`
- [ ] `PROTOCOL_VERSION == 28`, fingerprint **gate-computed**, `PROTOCOL_HISTORY` row **appended**,
      sentinel + `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned
- [ ] `HASH_SCHEMA_VERSION == 65`, both fingerprints **gate-computed**, `HASH_SCHEMA_HISTORY` row
      **appended**, sentinel re-pinned
- [ ] `GameState` still sealed: the new field is `pub(crate)`, there is a read accessor and **no**
      `_mut` accessor, and `cargo build --workspace` passes (SR-3)
- [ ] `crates/engine/src/state/hash.rs`'s `GameEvent` match gained a `129u8` arm (no `_` arm exists
      there — a miss is a compile error, which is the point)
- [ ] `random_bot::action_to_command` and `heuristic_bot`'s scorer gained `LegalAction` arms
      (both exhaustive, compile-forced)
- [ ] full golden-script suite run: `cargo test --test run_all_scripts` — 210 approved, 0 new skips
      (SR-9c)
- [ ] fixed-seed fuzzer A/B vs `main` on seeds that complete on both (OOS-DP3-9 is pre-existing;
      do not chase, do not let it mask)
- [ ] `docs/audits/decision-point-audit.md` §4.11 / §5 DP-3 / §8 PB-DP7 row + sequencing note /
      §9.3 / §9.4 recs 1 & 5 / §10 updated; seeds OOS-DP7-1..7 filed in §8.1
- [ ] `memory/workstream-state.md` handoff + CLAUDE.md Current State snapshot delta

---

## 12. Risks and edge cases

1. **A blocked game with no consumer that answers is a hang, not a slow game.** This is the price of
   choosing a block over PB-DP4's deadline, and it is why §7 enumerates *four* consumers
   (`StubProvider`, `LocalGame`, `random_bot`/`heuristic_bot`, TUI, harness) rather than two. Miss
   any one and a whole test regime deadlocks. The `LocalGame` safety valves (`max_commands`,
   `max_consecutive_passes`) will **not** catch it — a blocked `advance()` returns `AwaitingHuman`
   or `Halted(EngineError)` on the rejected structural pass, and neither counter moves.
2. **Stale entry after a concede or elimination.** Covered by gate #3 plus the liveness check inside
   `blocking_decision`, and pinned by T4. The subtler variant: the active player is eliminated by an
   SBA *during* cleanup — unreachable, because SBAs do not run while blocked (guard #1 returns
   before `:2007`). State it in the code comment so a later reader does not have to re-derive it.
3. **`enter_step`'s guard placement is load-bearing three ways.** After `events.extend(action_events)`
   (`:1995`) or the choice event is lost; after `is_game_over` (`:1997-2001`) or a game that ended in
   the turn-based actions never finalises; before the CR 514.3a block (`:2007`) or an SBA round runs
   against a half-performed CR 514.1. A reviewer should check all three.
4. **Re-entering `enter_step` from a command handler is new.** No existing command handler calls it
   (`handle_concede` at `:2188` is the only precedent, and it is on a turn-advance path). The
   recursion is bounded — the resume pass cannot record a new entry, because the hand is at max size
   — but the argument depends on `max_hand_size` not dropping during CR 514.2. Verified (it is a
   `PlayerState` scalar, not a layer output), asserted in T11a, and backstopped by
   `MAX_CLEANUP_SBA_ROUNDS`.
5. **Determinism churn is real but bounded.** One extra command per discard changes
   `command_count`, `consecutive_passes` and `loop_detection_hashes` in bot games. Expected, argued
   in §6, and the fixed-seed A/B is the check. Any *winner* or *turn count* change on a completing
   seed is a bug, not churn.
6. **`Vec<ObjectId>` on the wire is a validation surface.** Every SR-29 lesson applies: check the
   sender, check zone membership per id, check the exact count, reject duplicates. `handle_keep_hand`
   is the cautionary tale (OOS-DP2-1: it checks only the count and will happily bottom a card from
   another player's hand). Do not repeat it.
7. **The two version bumps land in one commit and both fingerprints come from failure text.** The
   single most likely process error in this batch is hand-editing a fingerprint or editing an
   existing history row. Both are machine-caught (`history_is_append_only`, `frozen_prefix_is_pinned`,
   `declaration_fingerprint_is_pinned`) — read the failures, do not guess.
8. **`#[non_exhaustive]` on `DecisionKind` is free today and will not be later.** Doing it in this
   batch is deliberate (audit §9.4 rec 1); doing it after M11-local S5 builds a DTO against the enum
   is a breaking change to that DTO.
