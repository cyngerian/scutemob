# Primitive Batch Plan: PB-DX2 — gate the resolution-time commands nothing gates

**Generated**: 2026-08-01
**Task**: `scutemob-162` · branch `feat/pb-dx2-gate-the-resolution-time-commands-nothing-gates-oos-d`
**Seeds**: **OOS-DP5-7** (headline) + **OOS-DP7-2** (the doc side of the same subject)
**Riders**: **OOS-DP2-1** (`handle_keep_hand` validates only a count), **OOS-DP9-14** (`pending_effect_choice` trap state)
**Class**: **CORRECTNESS — live exploit, trust boundary.** `Command::ChooseDredge { card: None }`
is a free card for any player at any time; `card: Some(x)` dredges at will.
**CR**: **702.52 / 702.52a / 702.52b** (primary), 614.11 / 614.11a, 616.1 / 616.1e / 616.1f,
121.2 / 121.2c, 103.5 / 103.5c, 608.2d, 104.4b, 702.94a, 400.7
**Baseline**: tests **3,945 / 0**; PROTOCOL **32** / HASH **69**; base `main` @ `27b0a1ec`
**Predicted wire**: **PROTOCOL 32 unmoved AND HASH 69 unmoved** — gate-computed, falsifier in §7.
**Predicted card yield**: **0 completeness flips, 0 card-def edits.** Roster is **one** dredge def
(`golgari_grave_troll.rs`, already `Complete`); see §6.
**TODO sweep** (roster-recall gate, `memory/feedback_planner_roster_recall.md`):
`rg 'TODO.*[Dd]redge|TODO.*pending.?draw|TODO.*KeepHand|TODO.*mulligan' crates/card-defs/src/defs/`
→ **0 cards with matching comments**. Positive assertion: the gate was run and produced no
additions. This is an engine-trust-boundary batch, not a DSL-expressiveness batch, so a card-def
TODO naming it would have been surprising.

---

## 0. Reading order for the runner

1. §1 — premise (extends `memory/primitive-wip.md`; adds seven facts, four of them load-bearing).
2. §2 — CR text, MCP-verbatim. **Read 702.52a and 614.11a together**; the second is why §4 changes
   `perform_one_draw`'s return handling and not just `handle_choose_dredge`.
3. §3 — **THE DECISION: (a)/(b)/(c)/(d), and the wire tension the brief flags.** Load-bearing.
   Do not write a line of production code before reading it.
4. §4 — the fix shape for item 1, edit by edit.
5. §5 — item 2 (the five doc sites, one of which the brief does not name, plus miracle).
6. §6 — roster (derive from `all_cards()`, never grep) — expected 1 def, 0 flips.
7. §7 — the wire prediction, its falsifier, and the exact gate commands.
8. §8 — riders: OOS-DP2-1 (§8.1) and OOS-DP9-14 (§8.2).
9. §9 — tests. §10 — seeds. §11 — ordered step list. §12 — "done". §13 — risks.
10. §14 — parallel-task collision surface (`scutemob-163`).

---

## 1. Premise (extends the WIP re-verification; not a repeat)

`memory/primitive-wip.md` already confirmed seven cites on this worktree (one drift:
OOS-DP2-1's `commander.rs:877-885` → **`:891`**). **Nothing in the brief was falsified.**
Planning added the following, all verified by reading source in this worktree.

**P1 — the `None` arm is worse than "validates nothing"; it is a *complete* free-card path.**
`handle_choose_dredge`'s `None` arm (`rules/replacement.rs:2932-2939`) calls
`draw_card_skipping_dredge` (`:3034-3047`), which guards only `has_lost || has_conceded` and then
runs `perform_one_draw(state, player, false, true, HashSet::new(), 0)`. No graveyard contents, no
dredge card, no pending draw, no priority, no active-player check, no phase check. **A player who
merely exists draws a card, at any time, as many times as they send the command.**

**P2 — the exploit also defeats CR 104.4b loop detection.** `rules/engine.rs:540` runs
`loop_detection::reset_loop_detection(&mut state)` *before* dispatching to the handler, and today
the handler always succeeds on `card: None`. So `ChooseDredge { card: None }` is simultaneously a
free card **and** an unlimited loop-detection reset — a player in a mandatory loop can spam it to
keep the game from being declared a draw. The gate closes both at once. (Rejected commands are
harmless here: `process_command` takes `GameState` by value and the mutated state is dropped on
`Err`, so a reset on a rejected path is never observed.)

**P3 — `Effect::DrawCards { count: n }` with a dredge card in the graveyard draws ZERO cards and
emits `n` prompts. This is a second live bug, on the same code, that the seed does not name.**
`effects/mod.rs:9289-9310`'s sequence loop breaks only on
`DrawStepOutcome::Deferred | LostToEmptyLibrary` (`:9303-9308`). `DredgeOffered` is **not** in that
set, so the loop iterates again, `check_would_draw_replacement` finds the same dredge card again,
and emits another `DredgeChoiceRequired`. Nothing is recorded, so at most one of the `n` draws can
ever be recovered — the other `n-1` are **destroyed**. Same omission at `replacement.rs:1397-1400`
and `:1413-1416` (`resolve_pending_draw`'s tail). CR **614.11a** says the sequence resumes *after*
the replacement's actions complete; the engine neither completes them nor remembers the sequence.
This is in scope: the fix for OOS-DP5-7 must record an outstanding draw anyway, and once it does,
`remaining` is exactly the datum CR 614.11a needs. Fixing the gate without fixing this would push
`n` entries per multi-draw and make the accumulation problem in §3.5 worse.

**P4 — the simulator emits `Command::ChooseDredge` NOWHERE.**
`rg -n 'ChooseDredge|Dredge' crates/simulator/src` → **0 matches**. Two consequences, both
load-bearing: (i) **blocking on a dredge offer would deadlock every simulated and fuzzed game** in
which a dredge card reaches a graveyard — this is the decisive argument against candidate (d),
§3.4; (ii) today, every bot game silently *loses* the draw-step draw of any player with a dredge
card in their graveyard, forever. (ii) is pre-existing and not fixed here (a bot policy is
`crates/simulator`' problem, and `scutemob-163` owns that crate this week) — seeded, §10.

**P5 — the two candidate answer-commands share one queue and have no discriminator.**
`handle_order_replacements` (`replacement.rs:158-234`) routes a `Command::OrderReplacements` to a
pending draw with `state.pending_draws.iter().position(|p| p.player == player)` and an
applicability check computed from **`state`**, not from the entry. PB-DP5's disjointness argument
(`:139-157`) is about *zone change vs draw* and is sound; it does **not** extend to *dredge-entry
vs replacement-order-entry*, because both are `ReplacementTrigger::WouldDraw`. §3.3 shows why this
is nevertheless benign under design (b) and why closing it would cost a HASH bump.

**P6 — `PendingDraw`'s existing four fields express everything a dredge offer needs.**
`crates/card-types/src/state/replacement_effect.rs:387-406`:
`player` (the chooser), `already_applied` (CR 614.5; empty for a fresh dredge offer),
`remaining` (CR 614.11a — the *exact* datum P3 needs), `sets_has_drawn_for_turn` (which draw path
raised it). No new field is required to record "an incomplete draw is outstanding for player P".

**P7 — every existing dredge test and the golden script reach the offer first.** Verified by
reading, not by grep:
`tests/mechanics_a_d/dredge.rs` tests 1 (`:104`), 2 (`:164`), 3 (`:282`), 7 (`:453`), 8 (`:540`),
10 (`:726`), 12 (`:802`), 13 (`:918`) and `tests/mechanics_e_l/golgari_grave_troll.rs:359` all
advance to `DredgeChoiceRequired` (or call `turn_actions::draw_card` / `execute_effect` directly)
**before** sending `ChooseDredge`. Test 9 (`:678`) sends `ChooseDredge { card: Some(..) }` with no
offer at all and asserts `is_err()` — it stays `Err` under the gate (different message; the
assertion is `is_err()` only). `tests/primitives/pb_dp5_pending_draw_choice.rs:441` (the
dredge-decline path) also reaches the offer first. Golden script
`test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json` runs
`turn_based_action: draw_card` then `player_action: choose_dredge`. **Roster is clean — no
existing test depends on the ungated behaviour.** That is a positive finding; record it.

**Premise verdict: holds, and is materially stronger than filed** (P1, P2, P3, P4 are all new).

---

## 2. CR text (MCP, verbatim)

**CR 702.52a** — *"Dredge is a static ability that functions only while the card with dredge is in
a player's graveyard. 'Dredge N' means 'As long as you have at least N cards in your library, **if
you would draw a card**, you may instead mill N cards and return this card from your graveyard to
your hand.'"*

> The conditional is the whole batch. Dredge is a **replacement effect on a draw**. With no draw,
> there is nothing to replace, and `Command::ChooseDredge` has no lawful referent. The engine's
> job is to know whether a draw is outstanding — which is precisely what it did not know.

**CR 702.52b** — *"A player with fewer cards in their library than the number required by a dredge
ability can't mill any of them this way."*

**CR 614.11** — *"Some effects replace card draws. These effects are applied even if no cards could
be drawn because there are no cards in the affected player's library."*
**CR 614.11a** — *"If an effect replaces a draw within a sequence of card draws, all actions
required by the replacement are completed, if possible, before resuming the sequence."*

> P3's bug, verbatim. The sequence must **stop** at the replaced draw and **resume** after.

**CR 616.1** — *"If two or more replacement and/or prevention effects are attempting to modify the
way an event affects an object or player, the affected object's controller (or its owner if it has
no controller) or the affected player chooses one to apply, following the steps listed below. …"*
**CR 616.1e** — *"Any of the applicable replacement and/or prevention effects may be chosen."*
**CR 616.1f** — *"Once the chosen effect has been applied, this process is repeated (taking into
account only replacement or prevention effects that would now be applicable) until there are no
more left to apply."*

> Note for §3.3: dredge **is** one of the "applicable replacement effects" of CR 616.1e for a
> draw. The engine's `check_would_draw_replacement` special-cases it *ahead* of the CR 616.1
> machinery (`replacement.rs:654-693`) rather than folding it in. That is a pre-existing
> simplification; PB-DX2 does not change it, but it is why a `ChooseDredge` landing on a
> `NeedsChoice`-origin entry is a **legal** CR 616.1e choice rather than an exploit.

**CR 121.2** — *"Cards may only be drawn one at a time. If a player is instructed to draw multiple
cards, that player performs that many individual card draws."*
**CR 121.2c** — *"If more than one player is instructed to draw cards, the active player performs
all of their draws first, then each other player in turn order does the same."*

**CR 103.5** *(mulligan; rider §8.1)* — *"… To take a mulligan, a player shuffles the cards in
their hand back into their library, draws a new hand of cards equal to their starting hand size,
**then puts a number of those cards** equal to the number of times that player has taken a mulligan
**on the bottom of their library** in any order. …"*
**CR 103.5c** — *"In a multiplayer game and in any Brawl game, the first mulligan a player takes
doesn't count toward the number of cards that player will put on the bottom of their library …"*

> *"those cards"* = the cards of the hand just drawn. Not any object in the game. That is the
> rider's whole CR argument, and the engine implements only the *number*.

**CR 608.2d** *(rider §8.2)* — *"If an effect of a spell or ability offers any choices other than
choices already made as part of casting the spell … the player announces these while applying the
effect. …"*

**CR 104.4b** — *"If a game that's not using the limited range of influence option (including a
two-player game) somehow enters a 'loop' of mandatory actions, repeating a sequence of events with
no way to stop, the game is a draw. Loops that contain an optional action don't result in a draw."*

> **Cite correction, verified this task.** Several comments in this codebase (and
> `memory/gotchas-rules.md` §"#34 — Mandatory infinite loops (CR 726)") cite **CR 726** for
> mandatory loops. **CR 726 is "Restarting the Game" (Karn Liberated)** — MCP-verified, all seven
> subrules are about restart procedure. The mandatory-loop rule is **CR 104.4b**. The runner must
> cite **104.4b** in any comment it writes, must not propagate "CR 726", and must **not** go on a
> repo-wide cite hunt (out of scope) — see seed **OOS-DX2-6**.

**CR 702.94a** *(§5.3)* — *"Miracle is a static ability linked to a triggered ability. 'Miracle
[cost]' means 'You may reveal this card from your hand **as you draw it** if it's the first card
you've drawn this turn. …'"*

**CR 400.7** — object identity on zone change; relevant to §8.1's duplicate-id case (the second
move of the same id fails with `ObjectNotFound` because the first minted a new id).

---

## 3. THE DECISION: (a), (b), (c) or (d) — and the wire tension

### 3.1 Recommendation

**Take (b): reuse the existing `pending_draws: Vector<PendingDraw>` queue.** The
`DrawAction::DredgeAvailable` arm of `perform_one_draw` pushes a `PendingDraw` exactly the way the
`NeedsChoice` arm already does; `handle_choose_dredge` **requires and consumes** one.
**No new type. No new field. No new enum variant. No new `GameState` state. Wire-neutral.**

And: **require-and-consume, do NOT block** (see 3.4). The engine's guarantee after PB-DX2 is
*"`ChooseDredge` is legal only while an outstanding draw for that player stands, and answering
consumes it"* — **not** *"the engine stops until you answer"*. §5 makes every doc site say exactly
that and nothing more.

### 3.2 Why not (a) — a new `GameState.pending_dredge` field / `PendingDredge` type

It is the obvious design and it is **wire-moving**, which fails acceptance criterion 5873.

- `tests/core/hash_schema.rs:675-685`'s `compute_decl_fingerprint` hashes **every declaration in
  the serde closure of `GameState`**, with the closure size in the preimage
  (`hasher.update(format!("types={}\n", closure.len()))`). A new `GameState` field ⇒ closure or
  declaration text moves ⇒ `decl_fingerprint` moves ⇒ `HASH_SCHEMA_VERSION` must bump. This is not
  a prediction: **PB-DP5 moved HASH 63 → 64 for exactly this, for exactly this queue**
  (`state/hash.rs:579-587`, `:982`).
- PROTOCOL would **not** move — and here PB-DX1's lesson (a type reachable from a closure root
  moves PROTOCOL too) is checked and found **not** to apply: `PROTOCOL_SCHEMA_FINGERPRINT`'s
  closure roots are `Command` / `GameEvent` / the replay log, **not** `GameState`.
  `card-types/src/state/stubs.rs:807` records this for `PendingDraw` verbatim, and
  `tests/primitives/pb_dp5_pending_draw_choice.rs:1227-1231` pins it
  (*"PROTOCOL_VERSION unchanged at 27 (`PendingDraw` is reachable only from `GameState`, never
  `Command`/`GameEvent`/`ReplayLog`)"*). So (a) would be **HASH-only**, and still fails AC 5873.
- It buys one thing (b) does not: a discriminator between a dredge offer and a replacement-order
  offer (P5). §3.3 argues that thing is not worth a wire bump because both directions of the
  ambiguity produce **legal** outcomes.
- **A new field on `PendingDraw` is the same cost** — the declaration text of a closure member
  moves, so `decl_fingerprint` moves. There is no cheap discriminator.

**If, during implementation, the runner concludes the correct fix genuinely requires new stored
state**: that is a **stop-and-report against AC 5873**, not a licence to bump. Report the
argument; the coordinator decides. Do not hand-apply a fingerprint under any circumstances
(`memory/conventions.md` "Hash sentinel convention" — the *bump rule* says default-to-bump, but
this batch has an explicit acceptance criterion pinning the constants, and that criterion outranks
the default).

### 3.3 Why (b) is sound despite having no discriminator (P5)

Under (b) a single queue holds two kinds of outstanding draw and two commands can consume either.
Enumerate the four cases; **all four are legal outcomes**:

| answer | entry it lands on | what happens | verdict |
|---|---|---|---|
| `OrderReplacements` | a **NeedsChoice** entry | unchanged from today | ✔ |
| `OrderReplacements` | a **dredge** entry | `resolve_pending_draw` applies the named replacement, then `perform_one_draw(offer_dredge: false)`. Every ordered id must pass `find_applicable` first, so a well-formed answer names a genuinely applicable `WouldDraw` replacement. Outcome = "declined dredge, applied a legal replacement, drew" | ✔ legal (CR 616.1e); document it |
| `ChooseDredge { None }` | a **NeedsChoice** entry | consume, then `perform_one_draw(offer_dredge: false)` with the entry's `already_applied`/`remaining` — i.e. the player declined dredge for a draw that has other applicable replacements, so it re-defers and re-emits `ReplacementChoiceRequired`. No card is minted, no `remaining` is lost | ✔ no-op-ish; **this is exactly what `pb_dp5_...rs:441` already exercises** |
| `ChooseDredge { Some(x) }` | a **NeedsChoice** entry | the `Some` arm still validates x is in *this player's* graveyard, has `Dredge(n)`, and library ≥ n — which is **byte-for-byte the eligibility predicate** `check_would_draw_replacement` uses at `:666-683`. So the player can only dredge a card that dredge law would have offered, against a draw that genuinely is outstanding | ✔ and arguably **more** CR-correct than the status quo: PB-DP5's `offer_dredge: false` on resume (`:1381`) is an engine simplification, not a CR rule; CR 616.1e lets the player pick dredge from the applicable set |

The residual asymmetry — `position()` picks the FIFO-first entry when a player somehow has two —
is (i) **pre-existing** (`handle_order_replacements` has always used `position()`), (ii) made
*less* likely by §4.2's fold guard, and (iii) deterministic. Document FIFO in
`handle_order_replacements`' doc block and in the new gate; do **not** add a discriminator.
**`Command::ChooseDredge`'s payload cannot carry one without a PROTOCOL bump anyway** — it is
`{ player, card: Option<ObjectId> }` and adding a field is a wire change.

### 3.4 Why not (d) — route through `blocking_decision`

`rules/engine.rs:146-166` / `:220-269` / `:291-308`. Four independent reasons, any one
disqualifying:

1. **It would deadlock every simulated and fuzzed game.** P4: `crates/simulator` never constructs
   `Command::ChooseDredge`. A `BlockingDecision::Dredge` makes `process_command` reject everything
   except that command and `Concede` (`:302-307`), so the first bot game in which a dredge card
   reaches a graveyard hangs. This alone ends the argument, and it is the batch's stated hard
   constraint ("no hang", "no regression to the simulator/fuzzer").
2. **The CR supplies a default, so the codebase's own block-vs-deadline test says deadline.**
   PB-DP7 §1.1 and OOS-DP7-1's filing row both state the criterion: *block only where the CR
   supplies no default*, which is why PB-DP4's three payment vectors stayed deadlines. CR 702.52a
   is *"you **may** instead"* — declining is always legal, so "no answer" has a well-defined,
   harmless meaning. Cleanup discard (CR 514.1), trigger targets (CR 603.3d) and resolution-time
   choices (CR 608.2d) have no such default; that is why *they* block.
3. **It would redden a test that exists specifically to forbid it.**
   `tests/primitives/pb_dp5_pending_draw_choice.rs:1199-1221`
   (`test_dp5_unanswered_pending_draw_does_not_deadlock`) says in so many words: *"if a future
   change ever gates progress (priority, SBAs, step advancement) on `pending_draws`, this test
   starts failing."* Under (b) that test keeps passing, which is the design being validated.
4. **A dredge offer can belong to a non-active player.** `effects::draw_cards_for_player` runs for
   any `PlayerTarget`; "each player draws a card" with a dredge card in an opponent's graveyard
   would hand that opponent a veto over the whole game's progress. The seven-round-trip inventory
   OOS-DP7-1 narrowed itself away from exactly this shape.

**OOS-DP7-1's "either wire these onto `blocking_decision` or correct the comments" is therefore
answered: correct the comments** (§5), with the reasoning above written into them.

### 3.5 Why not (c) — derive the gate from existing state with no stored state at all

Assess honestly: **(c) only appears sound.** After `DredgeChoiceRequired` is emitted, the game
state is byte-identical to a state in which the player simply has not drawn yet. The nearest
derivable predicate is *"it is this player's draw step, `has_drawn_for_turn == false`, and dredge
options exist"*, and it fails three ways:

1. **It cannot see the effect-draw path at all.** `effects::draw_cards_for_player` passes
   `sets_has_drawn_for_turn: false` (`effects/mod.rs:9298`), so nothing distinguishes "an effect's
   draw is outstanding" from "no draw is outstanding". `Effect::DrawCards` is where dredge is most
   often used competitively.
2. **It is not consumable.** The predicate stays true after the answer: `Some(x)` never touches
   `has_drawn_for_turn` (dredge is not a draw, CR 702.52a), so the same command can be replayed
   for unlimited free dredges. A gate that cannot be consumed is not a gate.
3. **It cannot carry `remaining`**, so P3 stays unfixed and CR 614.11a stays violated.

Rejected on evidence, and the rejection is recorded here so the next reader does not re-derive it.

### 3.6 Where the gate lives: **the handler, not the admission arm**

- `rules/engine.rs:534-544` keeps `validate_player_exists` and keeps
  `loop_detection::reset_loop_detection`. **No change to the `ChooseDredge` arm at all.**
- The require-and-consume check goes in `replacement::handle_choose_dredge`, because it is
  *state* validation of a specific command (like every other handler's validation), not
  *admission* control. `process_command`'s admission gate is exclusively about "what is legal
  while the engine is blocked", and under (b) the engine is never blocked. PB-DP7/DP8/DP9 all put
  their per-command validation in the handler and used the admission gate only for the block.
- Consequence for P2: because the handler now returns `Err` for an unearned `ChooseDredge`, and
  `process_command` discards the mutated state on `Err`, the loop-detection reset at `:540` is
  never observed on the exploit path. **Do not move the reset** — moving it is a behaviour change
  to every other reset site's ordering convention and is out of scope.

---

## 4. Item 1 — the fix shape (`OOS-DP5-7` + P3)

All line numbers are as of base `27b0a1ec`; **re-verify each before editing** (the OOS-DP6-8
documentation-rot class — PB-DX1 found two stale cites in its own brief).

### 4.1 Edit 1 — extract the CR 614.11a tail loop (pure refactor, zero behaviour change)

**File**: `crates/engine/src/rules/replacement.rs`

`resolve_pending_draw`'s resume loop (`:1397-1420`) is about to be needed in a second place. Lift
it verbatim into a private helper *above* `resolve_pending_draw`:

```rust
/// CR 614.11a / 121.2: perform the `remaining` further draws of the sequence a
/// deferred draw belonged to, stopping on a further deferral or an empty library.
///
/// Extracted from `resolve_pending_draw` by PB-DX2 so `handle_choose_dredge` can
/// discharge the same obligation without duplicating it. Behaviour is byte-for-byte
/// the pre-PB-DX2 loop, including `offer_dredge: false` (see OOS-DX2-2: each draw of
/// a sequence is separately replaceable under CR 702.52a, and suppressing dredge for
/// the whole tail is a pre-existing simplification this batch deliberately does not
/// change).
///
/// Terminates in at most `remaining` iterations: `remaining` is a `u32` captured
/// before the loop and `perform_one_draw` never calls back into this function.
fn perform_remaining_draws(
    state: &mut GameState,
    player: PlayerId,
    remaining: u32,
    sets_has_drawn_for_turn: bool,
) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for i in 0..remaining {
        let remaining_after = remaining - 1 - i;
        let (evts, out) = perform_one_draw(
            state, player, false, sets_has_drawn_for_turn, HashSet::new(), remaining_after,
        );
        events.extend(evts);
        if matches!(
            out,
            DrawStepOutcome::Deferred
                | DrawStepOutcome::LostToEmptyLibrary
                | DrawStepOutcome::DredgeOffered
        ) {
            break;
        }
    }
    events
}
```

`resolve_pending_draw:1397-1420` becomes:

```rust
    if !matches!(
        outcome,
        DrawStepOutcome::Deferred
            | DrawStepOutcome::LostToEmptyLibrary
            | DrawStepOutcome::DredgeOffered   // PB-DX2 / P3: a dredge offer now
                                               // records its own entry with the
                                               // correct `remaining`; resuming here
                                               // would double-count it.
    ) && pending.remaining > 0
    {
        events.extend(perform_remaining_draws(
            state, pending.player, pending.remaining, pending.sets_has_drawn_for_turn,
        ));
    }
```

### 4.2 Edit 2 — record the outstanding draw at the offer site

**File**: `crates/engine/src/rules/replacement.rs`, `perform_one_draw`, the `DredgeAvailable` arm
at **`:828`** (today: `=> (vec![event], DrawStepOutcome::DredgeOffered)`).

```rust
        DrawAction::DredgeAvailable(event) => {
            // CR 702.52a + 614.11a (PB-DX2, OOS-DP5-7): the offer REPLACES this
            // draw, so the draw is now an outstanding obligation. Record it in the
            // same `pending_draws` queue the CR 616.1 deferral uses — see
            // `handle_choose_dredge`, which requires and CONSUMES an entry, and
            // pb-plan-DX2.md §3.3 for why one undiscriminated queue is sound.
            // Before PB-DX2 nothing was recorded, so `Command::ChooseDredge` had no
            // gate at all and `card: None` minted a free card for any player at any
            // time.
            //
            // Determinism (SR-9b): sort `already_applied` by ReplacementId before
            // storing — `HashSet` iteration order is not stable and this field is
            // hashed. Same reasoning as the `NeedsChoice` arm below.
            let mut sorted: Vec<ReplacementId> = already_applied.into_iter().collect();
            sorted.sort_by_key(|id| id.0);
            match state.pending_draws.iter().position(|p| p.player == player) {
                // At most ONE dredge-originated entry per player. A second draw for
                // a player who already owes an answer FOLDS into the outstanding
                // obligation (this draw, plus its own tail) instead of pushing.
                // Two reasons, both real: (1) `pending_draws` is in
                // `loop_detection::compute_mandatory_state_hash` (`:159-163`), so an
                // unbounded per-draw push would make two structurally identical
                // CR 104.4b positions fingerprint differently and could mask a
                // mandatory loop; (2) it conserves the draw, which today is simply
                // destroyed (plan §1 P3).
                Some(i) => {
                    if let Some(entry) = state.pending_draws.get_mut(i) {
                        entry.remaining += 1 + remaining_after;
                    }
                }
                None => state.pending_draws.push_back(PendingDraw {
                    player,
                    already_applied: sorted,
                    remaining: remaining_after,
                    sets_has_drawn_for_turn,
                }),
            }
            (vec![event], DrawStepOutcome::DredgeOffered)
        }
```

Notes for the runner:
- Keep the `DredgeOffered` **variant**. It is informative and its doc is one of §5's four sites.
  Do not collapse it into `Deferred`.
- `imbl::Vector::get_mut` exists; if the borrow checker fights, `let mut e = state.pending_draws[i].clone(); e.remaining += …; state.pending_draws.set(i, e);` is the equivalent.
- `already_applied` is moved by `into_iter()`; compute `sorted` **before** the `match` (as written)
  so both arms compile.
- The fold rule applies **only at this site**. `NeedsChoice`'s push is untouched; its own
  multi-entry possibility is pre-existing and seeded (OOS-DX2-3).

### 4.3 Edit 3 — stop the sequence at the offer

**File**: `crates/engine/src/effects/mod.rs`, `draw_cards_for_player`, `:9303-9308`.

Add `DrawStepOutcome::DredgeOffered` to the `matches!` break set, with a CR 614.11a comment naming
plan §1 P3 (*"before PB-DX2 a `DrawCards { count: 3 }` with a dredge card in the graveyard emitted
three prompts and drew zero cards; at most one could ever be answered"*).

### 4.4 Edit 4 — the gate, in `handle_choose_dredge`

**File**: `crates/engine/src/rules/replacement.rs:2925-3020`.

Rewrite the function body in this order. **All validation precedes all mutation** (the SR-23 idiom
`move_object_to_bottom_of_zone` already follows at `state/mod.rs:1767-1786`).

```
0. Dead-player discharge. If `state.expect_player(player)` says `has_lost || has_conceded`:
   remove any `pending_draws` entry for that player and return `Ok(vec![])`.
   (Preserves `draw_card_skipping_dredge`'s guard at `:3040-3043`, and additionally clears the
   obligation so a dead player's entry cannot sit in the hash forever — the OOS-DP9-14 lesson,
   applied prophylactically here.)

1. THE GATE (CR 702.52a):
       let idx = state
           .pending_draws
           .iter()
           .position(|pd| pd.player == player)
           .ok_or_else(|| GameStateError::InvalidCommand(format!(
               "ChooseDredge from player {:?} with no draw outstanding — CR 702.52a: dredge \
                replaces a draw, and the engine records the offer as a PendingDraw entry \
                (GameEvent::DredgeChoiceRequired). PB-DX2 / OOS-DP5-7.", player)))?;
   FIFO: `position` takes the OLDEST outstanding draw for this player, matching
   `handle_order_replacements:205`. `Command::ChooseDredge` carries no discriminator and cannot
   gain one without a PROTOCOL bump; see plan §3.3.

2. For `Some(card_id)`: run the EXISTING validations unchanged (`:2942-2984`) — object exists,
   `obj.zone == ZoneId::Graveyard(player)`, `KeywordAbility::Dredge(n)` present, library >= n.
   Add a comment recording that these three ARE `check_would_draw_replacement`'s eligibility
   predicate (`:666-683`), so a gated `Some` answer can only name a card dredge law would have
   offered. NO new validation is required or wanted.

3. CONSUME: `let pending = state.pending_draws[idx].clone(); state.pending_draws.remove(idx);`

4a. `Some(card_id)`: mill n / move to hand / emit `Dredged` — the existing body, unchanged
    (`:2985-3017`). THEN, CR 614.11a:
        events.extend(perform_remaining_draws(
            state, player, pending.remaining, pending.sets_has_drawn_for_turn));
    (`Dredged` does not set `has_drawn_for_turn`; only the tail draws do, per the entry's flag.)

4b. `None`: resume the replaced draw with the entry's own bookkeeping —
        let (evts, outcome) = perform_one_draw(
            state, player,
            false,                      // CR 702.52a: the player just declined for THIS draw;
                                        // re-offering would loop (dredge.rs test 10).
            pending.sets_has_drawn_for_turn,
            pending.already_applied.iter().copied().collect(),
            pending.remaining,
        );
        events.extend(evts);
        if !matches!(outcome, Deferred | LostToEmptyLibrary | DredgeOffered)
            && pending.remaining > 0
        {
            events.extend(perform_remaining_draws(
                state, player, pending.remaining, pending.sets_has_drawn_for_turn));
        }
```

`draw_card_skipping_dredge` (`:3034-3047`) is now **dead** — it hardcoded
`(HashSet::new(), remaining: 0, sets_has_drawn_for_turn: true)`, which is exactly the bookkeeping
4b must stop discarding. **Delete it** (clippy `-D warnings` will demand it anyway) and move its
doc's still-true parts (why `offer_dredge: false`) into 4b's comment. Its mention in
`check_would_draw_replacement`'s doc (`:636`) must be updated in the same edit.

### 4.5 What must NOT change

- **No progress gate on `pending_draws`.** `pb_dp5_pending_draw_choice.rs:1204` is the pin.
- **`handle_order_replacements` logic is untouched** — only its doc block gains the dredge case
  and the FIFO note (§3.3).
- **`rules/engine.rs`'s `ChooseDredge` arm is untouched** (§3.6), including
  `check_and_flush_triggers` (`memory/gotchas-infra.md` "Command Handler Pattern Gotchas" lists
  the commands that must NOT flush; `ChooseDredge` is not one of them, and dredging can move a
  card between zones, so the flush stays).
- **SR-25 `bare_lookup_ratchet` must not move.** Ceilings on the files this batch touches:
  `src/rules/replacement.rs` **24**, `src/rules/resolution.rs` **100**, `src/rules/commander.rs`
  **6** (`tests/core/bare_lookup_ratchet.rs:112/137/180`). The ratchet fails in **both**
  directions. None of §4's edits adds an `.objects.get(` / `.players.get(` — §8.1's does, and
  §8.1 avoids it deliberately.
- **SR-7**: this batch pushes no `PendingTrigger`.
- `PendingDraw`'s declaration, `Command::ChooseDredge`'s shape, `GameEvent::DredgeChoiceRequired`'s
  shape: **all unchanged**. That is the wire prediction (§7).

---

## 5. Item 2 — `OOS-DP7-2`: the doc sites (there are FIVE, not two)

After §4, the honest description is: *the draw does not happen, an obligation is recorded, the draw
sequence stops, `ChooseDredge` is the only thing that discharges it — and the engine does **not**
block.* Every site must say that and nothing stronger.

| # | site | today | required after |
|---|---|---|---|
| 1 | `rules/replacement.rs:617-620` — `DrawAction::DredgeAvailable` | *"The engine pauses until a `Command::ChooseDredge` is received."* | replace "pauses" with the recorded-obligation wording; name the `PendingDraw` push and `handle_choose_dredge`'s consume |
| 2 | `rules/events.rs:845-853` — `DredgeChoiceRequired` | same false claim | full replacement, text below |
| 3 | `rules/replacement.rs:764-767` — `DrawStepOutcome::DredgeOffered` | *"the caller does NOT stop"* — **true today, false after §4.3** | rewrite: the caller MUST stop (CR 614.11a); an entry was pushed carrying `remaining` |
| 4 | `rules/events.rs:1353-1355` — inside `CleanupDiscardChoiceRequired`'s doc | *"Unlike `DredgeChoiceRequired`, whose identical claim is not implemented (seed OOS-DP7-2), this one is enforced"* | rewrite: dredge is a **deadline**, cleanup discard is a **block**; keep the contrast, drop the "not implemented" (it becomes false), point at `handle_choose_dredge`'s gate. *(This is the third site the brief does not name; the WIP file caught it. If it is left alone it becomes the new lying comment.)* |
| 5 | `rules/events.rs:833-838` — `MiracleRevealChoiceRequired` | *"The engine pauses until a `Command::ChooseMiracle` is received."* | **verified false** — §5.3. Correct the comment; the behaviour fix is seeded |

Model text for site 2:

```rust
    /// One or more dredge cards are available in the player's graveyard and the
    /// player must choose whether to dredge one or draw normally (CR 702.52a).
    ///
    /// **The engine does NOT block on this** (PB-DX2, closing OOS-DP7-2's half of
    /// the claim). What happens is: the draw does not occur; a `PendingDraw` entry
    /// is recorded for `player` (`replacement::perform_one_draw`); and the draw
    /// SEQUENCE stops (CR 614.11a), with the count of further draws carried on the
    /// entry. Priority, state-based actions and step advancement all continue, and
    /// any player may act. `Command::ChooseDredge` is legal ONLY while that entry
    /// stands and CONSUMES it (`replacement::handle_choose_dredge`), which is what
    /// stops the command minting a free card (OOS-DP5-7).
    ///
    /// It is a DEADLINE, not a `rules::engine::BlockingDecision`, for two reasons:
    /// CR 702.52a is "you MAY instead", so declining is always legal and "no answer"
    /// has a well-defined meaning (the block-vs-deadline test PB-DP4/PB-DP7 use);
    /// and `crates/simulator` constructs no `ChooseDredge` at all, so blocking would
    /// deadlock every bot game in which a dredge card reaches a graveyard.
    /// An unanswered offer means the draw simply never happens.
    ///
    /// `options` lists `(ObjectId, u32)` pairs of (dredge card, dredge amount).
```

### 5.3 `MiracleRevealChoiceRequired` — verified, and it is worse than "the doc lies"

The seed says *"same shape and the same suspicion (not verified)"*. **Verified this task:**

- `rules/miracle.rs:115-141` `check_miracle_eligible` emits the event from inside
  `perform_one_draw`'s completed-draw path (`replacement.rs:901-907`) and records **nothing**.
  The engine does not pause. **The doc at `events.rs:836` is false. Correct it.**
- `handle_choose_miracle` (`miracle.rs:44-106`) is **not** a free-card exploit: it validates the
  card is in the player's hand, has `KeywordAbility::Miracle`, and `cards_drawn_this_turn == 1`.
- **But it is not gated on the offer either, and that is a live CR 702.94a violation.** The
  card need not be the card just drawn. Any miracle card *already in hand* — drawn last turn,
  tutored, discarded-and-returned — can be revealed and cast for its miracle cost as long as the
  player has drawn exactly one card this turn. CR 702.94a says *"as you draw it"*.
- **Disposition: doc-fix here, behaviour fix seeded (OOS-DX2-1).** The fix needs a record of
  *which* object was just drawn, which is new stored state ⇒ HASH bump ⇒ out of scope under
  AC 5873. Say so in the corrected doc rather than implying the gate exists.

---

## 6. Roster — derive from `all_cards()`, never from grep (SR-36)

**Planning-time expectation, to be confirmed or falsified by the runner:** exactly **one** card def
in the corpus carries `KeywordAbility::Dredge(_)` — `crates/card-defs/src/defs/golgari_grave_troll.rs:80`
(`Dredge(6)`), which carries **no `completeness` field** and is therefore **`Complete` and
deck-legal**. That is what makes OOS-DP5-7 "wrong in a game you could play today".

Mandated derivation (grep is a planning aid only):

```
for def in all_cards():
    for face_is_transformed in [false, true]:
        for ability in def.effective_abilities(face_is_transformed):
            if let AbilityDefinition::Keyword(KeywordAbility::Dredge(n)):
                record (def.name, def.completeness, n)
```

Walk **both faces** (`effective_abilities`, not `def.abilities`) — PB-OS4b/PB-RS4 made the back
face live, and PB-DP9's roster came out 69/16/7 against a published 74/16/8 precisely because a
flat scan undercounts. **If the enumeration returns anything other than one `Complete` def, the
enumeration wins** and the runner reports the delta.

**Expected yield: 0 completeness flips, 0 card-def edits, 0 new defs.** The batch is pure engine
correctness. Do not open `crates/card-defs/` except to run the enumeration.

**Golden scripts**: `rg -l 'Grave-Troll\|dredge\|choose_dredge' test-data/generated-scripts/` at
planning time → **`replacement/014_golgari_grave_troll_dredge.json` only**. Re-run before starting.
SR-9c forbids silent skips, so a broken script surfaces; any changed expectation needs a one-line
CR citation in the diff. **Do not adjust a script to fit.**

---

## 7. Wire prediction, falsifier, and the exact gates

### 7.1 The prediction

> §4 changes **no type declaration anywhere**: no new struct, no new struct field, no new enum
> variant, no changed field type. `PendingDraw`, `GameState`, `Command::ChooseDredge` and
> `GameEvent::DredgeChoiceRequired` are all byte-identical in declaration. §8.1 and §8.2 likewise
> add only guards and a field *clear*.
>
> **Therefore `PROTOCOL_VERSION` stays 32 and `HASH_SCHEMA_VERSION` stays 69**, and
> `tests/core/protocol_schema.rs` and `tests/core/hash_schema.rs` **must both stay green with no
> edits to `rules/protocol.rs` or `state/hash.rs`.**
>
> Mechanism, checked rather than assumed: `hash_schema.rs:675-685`'s `compute_decl_fingerprint`
> hashes the declarations of the serde closure of `GameState` plus `types={len}`; nothing in that
> closure moves. `protocol_schema.rs`'s closure roots are `Command`/`GameEvent`/the replay log,
> **not** `GameState` (`card-types/src/state/stubs.rs:807` and
> `pb_dp5_pending_draw_choice.rs:1227-1231` both record this for `PendingDraw`), and no
> `Command`/`GameEvent` variant changes.
>
> **Falsifier**: if either gate reddens, some declaration moved — which means the implementation
> deviated from (b) and stored new state. **STOP. Do not read the new digest out of the failure
> text and re-pin.** Report it: AC 5873 pins these constants, and a move is a design escalation,
> not a bookkeeping step. (This is the opposite instruction from PB-DX1, where the bump was
> *predicted in advance*; here the constants are an acceptance criterion.)

Runtime `public_state_hash` **values** do change for states with an outstanding dredge offer —
that is real new state content, not a schema move, and nothing pins a stored hash value
(`state_hashing.rs`, `harness_equivalence.rs` and `loop_detection.rs` all compute live).

### 7.2 The exact commands the runner must run

```
cargo test -p mtg-engine --test core            # protocol_schema + hash_schema + ratchets + fmt gate
cargo test --all                                # 3,945 baseline; expect +16..20
cargo clippy --all-targets -- -D warnings
cargo fmt --check && tools/check-defs-fmt.sh    # SR-35
cargo build --workspace                         # simulator / TUI / replay-viewer
```

Report `HASH_SCHEMA_VERSION` and `PROTOCOL_VERSION` explicitly in the review even though they did
not move — a silent "no bump" is indistinguishable from "did not check".

### 7.3 Bench check

`perform_one_draw` gains one `Vector` scan per draw (`position` over `pending_draws`, which is
empty in the common case). Run `cargo bench -p mtg-engine` against the merge base in a throwaway
worktree (PB-DP9's method) and report `priority_cycle_4p` / `sba_check` / `full_turn_4p`. Baseline
at PB-DX1 close: `full_turn_4p` ≈ **217 µs**. A >5 % regression is a stop-and-report.

---

## 8. Riders

### 8.1 OOS-DP2-1 — `handle_keep_hand` validates only a count: **FIX**

**File**: `crates/engine/src/rules/commander.rs`, `handle_keep_hand` at **`:891`**
(the seed's `:877-885` cite is **stale** — correct it on closure, per the OOS-DP6-8
documentation-rot class).

**The gap is worse than filed.** `Command::KeepHand` has *no pregame gate whatsoever*
(`rules/engine.rs:482-489`: `validate_player_exists` only), and `required_bottom` is derived from
`mulligan_count.saturating_sub(1)`, which never resets. So a player who took two mulligans can, on
turn 30, send `KeepHand { cards_to_bottom: [any ObjectId in the game] }` and the loop at `:913-915`
moves it to the bottom of *their* library — from the battlefield, from a graveyard, or **from
another player's hand**. Repeatable, once per mulligan taken, forever.

**Fix** (per-entry zone guard + duplicate check, all validation before any mutation):

```rust
    // CR 103.5: "then puts a number of THOSE CARDS ... on the bottom of their
    // library" -- "those cards" are the cards of the hand just drawn, not any
    // object in the game. Before PB-DX2 only the COUNT was checked, so a malformed
    // or hostile KeepHand could bottom a permanent from the battlefield, a card
    // from a graveyard, or a card from ANOTHER PLAYER'S HAND (OOS-DP2-1).
    //
    // All validation precedes all mutation (the SR-23 idiom
    // `move_object_to_bottom_of_zone` itself follows): a rejected command must not
    // leave a half-applied bottoming behind.
    let hand_zone = ZoneId::Hand(player);
    {
        // SR-25: read the HAND ZONE's membership, not the object map. `expect_zone`
        // is the NONSWALLOW helper (the hand zone is built pre-turn-1 and never
        // removed, ground truth 2) and this adds no bare `.objects.get(` lookup, so
        // `bare_lookup_ratchet`'s ceiling of 6 for this file does not move.
        let hand = state
            .expect_zone(&hand_zone)
            .ok_or(GameStateError::ZoneNotFound(hand_zone))?;
        let mut seen: std::collections::HashSet<ObjectId> = std::collections::HashSet::new();
        for obj_id in cards_to_bottom.iter() {
            if !seen.insert(*obj_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "KeepHand: object {:?} named twice in cards_to_bottom (CR 103.5: each of \
                     the cards put on the bottom is a distinct card of the hand)",
                    obj_id
                )));
            }
            if !hand.contains(obj_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "KeepHand: object {:?} is not in player {:?}'s hand (CR 103.5)",
                    obj_id, player
                )));
            }
        }
    }
```

Place it **after** the count check (`:902-910`) and **before** the move loop.

**What this does and does not close.** It closes the cross-zone and cross-player half completely:
after the guard, a mid-game `KeepHand` can only bottom the sender's own hand cards, i.e. it
degrades from an attack on other players to self-harm. It does **not** close the missing pregame
phase gate — there is no `mulligan_phase` state to consult (`rg -i mulligan crates/card-types/src/state`
→ `PlayerState::mulligan_count` only), and adding one is new stored state ⇒ HASH bump ⇒ out of
scope under AC 5873. **Seed it (OOS-DX2-4)**, and note in the same seed that
`Command::TakeMulligan` (`engine.rs:477-481`) has the identical hole and is *worse* (mid-game it
shuffles the sender's hand into their library and draws seven).

**Duplicate-id honesty**: today `[a, a]` already errors — the first move mints a new `ObjectId`
(CR 400.7) so the second fails with `ObjectNotFound`. The guard's value is therefore (i) the
*classification* (a duplicate is a malformed client command, not a missing object) and (ii) the
no-partial-mutation property. Test T12 must assert on the **message**, not merely `is_err()`, or
it is a vacuous probe (`memory/conventions.md` "Test-validity MEDIUMs are fix-phase HIGHs").

### 8.2 OOS-DP9-14 — reap `pending_effect_choice` with a dead owner: **FIX, as defensive hardening**

**Honest classification: this is defensive hardening, not a live fix.** The seed's own argument
holds and was re-verified: `discharge_effect_choice_on_concede`'s only caller is `handle_concede`
(`engine.rs:2579-2620`), it clears the field whenever it is `Some` **regardless of who conceded**
(keyed on the game position, `:2530-2538`), and while the block stands the admission gate
(`:291-308`) admits only the answer and `Concede`, so no SBA can run and no other elimination path
exists. **It is unreachable through legal commands today.** It is worth fixing anyway because the
residue is *unrecoverable* (`blocking_decision` returns `None` ⇒ `PassPriority` is admitted ⇒ the
entry `debug_assert!` fires ⇒ in release, `ask_or_consume_effect_choice`'s "already suspended"
guard re-emits the stale question forever) and because the fix is four lines.

**Cite correction**: the seed says *"mirroring `drop_departed_trigger_flush`'s placement
(`engine.rs:2664`)"*. `engine.rs:2664` is inside **`handle_concede`**, not at the top of
`resolve_top_of_stack`. The "mirroring" is of the *shape* (a small reap helper for a departed
owner), not the placement. Record the correction when closing the seed.

**Fix**: at the very top of `rules/resolution.rs::resolve_top_of_stack` (`:90`), **before** the
existing `debug_assert!` at `:91-95`:

```rust
    // CR 608.2d (PB-DX2, closing OOS-DP9-14): a `pending_effect_choice` whose owner
    // has left the game is a trap state, not a question. `blocking_decision`'s
    // liveness filter (`engine.rs:220-227`) already stops such an entry blocking the
    // game, but it does NOT clear the field, so if a future widened admission gate
    // ever let an SBA elimination run here the residue would be unrecoverable.
    // Unreachable through legal commands today (`discharge_effect_choice_on_concede`
    // clears the field on ANY concede) -- this is defensive hardening, and it is
    // deliberately narrow.
    //
    // It must run BEFORE the assert below and must clear ONLY a dead owner's entry:
    // clearing unconditionally would make the assert vacuous, and the assert's real
    // job -- catching re-entry with a LIVE player's question outstanding, which IS
    // an engine bug -- has to keep its teeth. Pinned in both directions by
    // `tests::dx2_*` at the foot of this module.
    if let Some(entry) = state.pending_effect_choice.clone() {
        let owner_alive = state
            .expect_player(entry.player)
            .map(|pl| !pl.has_lost && !pl.has_conceded)
            .unwrap_or(false);
        if !owner_alive {
            state.pending_effect_choice = None;
            state.effect_choice_answers = imbl::Vector::new();
        }
    }
```

**No re-drive is owed here** (unlike `discharge_effect_choice_on_concede`, which must drive the
resolution because at the concede point `priority_holder` is `None` and everyone has passed): we
are *already inside* `resolve_top_of_stack`, so the resolution proceeds on the next line.
State that in the comment.

`expect_player` is the NONSWALLOW helper (SR-14 ground truth 1: players are never removed), so the
`bare_lookup_ratchet` ceiling of 100 for `resolution.rs` does not move.

---

## 9. Tests

**New file**: `crates/engine/tests/primitives/pb_dx2_command_gates.rs`
**Registration**: `crates/engine/tests/primitives/main.rs` — insert `mod pb_dx2_command_gates;`
**after line 30** (`mod pb_dx1_lowered_intervening_if;`), keeping the list sorted. SR-9a: never add
a top-level `tests/*.rs`. **Do not touch `tests/rules/main.rs`** (§14).

**Patterns to copy**: `tests/mechanics_a_d/dredge.rs:22-102` (the `dredge_card_def` /
`build_upkeep_state` / `pass_all` helpers — copy them, do not `mod`-import across targets);
`tests/primitives/pb_dp5_pending_draw_choice.rs` (the `pending_draws()` assertion idiom and T12's
no-deadlock shape).

Every test cites its CR section (Architecture Invariant 8).

### 9.1 Fail-before probes — MANDATORY

| # | test | asserts | fail-before |
|---|---|---|---|
| **T1** | `test_dx2_choose_dredge_none_without_offer_is_a_free_card_today` | 2 players, `Step::PreCombatMain`, dredge card in p1's graveyard, library ≥ N, **no draw attempted**. `ChooseDredge { player: p1, card: None }` → `Err(InvalidCommand)`; p1's hand count unchanged. CR 702.52a. | **FAILS** — today it returns `Ok` and p1 draws a card |
| **T2** | `test_dx2_choose_dredge_some_without_offer_dredges_at_will_today` | same fixture, `card: Some(ggt)` → `Err`; graveyard, library and hand all unchanged. CR 702.52a / 702.52b. | **FAILS** — today it mills N and returns the card to hand |
| **T3** | `test_dx2_choose_dredge_is_consumed_by_its_answer` | reach the draw-step offer; send `ChooseDredge { None }` (ok); send it **again** → `Err`. CR 702.52a. This is the *consume* half, distinct from T1's *require* half. | **FAILS** — today the second answer draws a second card |
| **T5** | `test_dx2_multi_draw_sequence_stops_at_the_dredge_offer` | `Effect::DrawCards { count: 3 }` for a player with a dredge card: **exactly one** `DredgeChoiceRequired`, **zero** `CardDrawn`, exactly one `pending_draws()` entry with `remaining == 2`. Then `ChooseDredge { None }` → **three** `CardDrawn` total and `pending_draws()` empty. CR 614.11a / 121.2. | **FAILS** — today: three prompts, zero draws, and a decline yields one card (two draws destroyed) |
| **T6** | `test_dx2_dredge_then_remaining_draws_complete` | same fixture, `ChooseDredge { Some(x) }` → one `Dredged` + **two** `CardDrawn`. CR 702.52a + 614.11a. | **FAILS** — today one `Dredged`, zero further draws |
| **T10** | `test_dx2_keep_hand_rejects_a_card_in_another_players_hand` | mid-game state, `p1.mulligan_count = 2`, `KeepHand { player: p1, cards_to_bottom: vec![<an object in p2's hand>] }` → `Err`; the object is still in p2's hand. CR 103.5. | **FAILS** — today it succeeds and moves p2's card to p1's library |
| **T11** | `test_dx2_keep_hand_rejects_a_battlefield_permanent` | same with a permanent p1 controls. CR 103.5. | **FAILS** |
| **T12** | `test_dx2_keep_hand_rejects_duplicate_ids` | `p1.mulligan_count = 3` (required 2), `cards_to_bottom: vec![a, a]` → `Err` **whose message names the duplicate** (`assert!(msg.contains("twice"))`). CR 103.5 / 400.7. | **FAILS on the message** — today it errors as `ObjectNotFound` after already moving the card once. §8.1 explains why `is_err()` alone would be a vacuous probe |
| **T14** | `test_dx2_reap_clears_a_dead_owners_pending_effect_choice` | **in-src unit test** (see 9.3). Build a state with one stack object and `pending_effect_choice = Some(PendingEffectChoice { player: p2, .. })`, mark p2 `has_conceded`, call `resolve_top_of_stack`. After: entry cleared, `effect_choice_answers` empty, resolution proceeded. CR 608.2d. | **FAILS** — today the entry `debug_assert!` panics |

**Protocol**: write T1, T2, T5, T10 and T14 **first, before any production code**, run them, and
paste the failure text into the review. *A probe that passes before the fix is a test-validity
HIGH, not a LOW* (`memory/conventions.md`). If any of them passes pre-fix, **STOP and report** —
the premise is falsified.

### 9.2 The rest

| # | test | asserts | fail-before |
|---|---|---|---|
| T4 | `test_dx2_dredge_offer_records_a_pending_draw` | after the draw-step offer: `pending_draws().len() == 1`, `[0].player == p1`, `[0].remaining == 0`, `[0].already_applied.is_empty()`. CR 702.52a / 614.11a. | new (nothing recorded today) |
| T7 | `test_dx2_dredge_offers_do_not_stack_entries` (§4.2's fold) | draw-step offer (unanswered), then `Effect::DrawCards { count: 2 }` for the same player: still **exactly one** entry, `remaining == 2`. One `ChooseDredge { None }` then yields **three** `CardDrawn`. CR 614.11a + 104.4b (the fingerprint argument). | new |
| T8 | `test_dx2_unanswered_dredge_offer_does_not_deadlock` | **hard constraint.** With an outstanding dredge entry, both players pass priority through several steps: no `Err`, no hang, the turn advances, `state.blocking_decision().is_none()`, and the entry is still present (nothing resolves it for the player). Mirrors `pb_dp5_...rs:1204`. | passes before and after; its value is post-fix |
| T9 | `test_dx2_dead_players_dredge_entry_is_discharged` | **3 players** (so the game is not over). p1 has an outstanding entry, p1 concedes, then `ChooseDredge` from p1 → `Ok(vec![])` and `pending_draws()` empty. | new; drop it and say so if `is_game_over` / admission ordering makes it unreachable |
| T13 | `test_dx2_keep_hand_still_accepts_the_players_own_hand_cards` | non-regression: `mulligan_count = 2`, one of p1's own hand cards → `Ok`, card at the bottom of p1's library, `MulliganKept` emitted. CR 103.5 / 103.5c. | passes before and after |
| T15 | `test_dx2_reap_does_not_silence_a_live_owners_entry` | **in-src, `#[cfg(debug_assertions)]`, `#[should_panic]`**: same fixture as T14 but the owner is **alive**; `resolve_top_of_stack` must still trip the entry `debug_assert!`. **This is the test that proves the reap did not make the assert vacuous** — it is not optional. | passes before and after; pins §8.2's narrowness |
| T16 | `test_dx2_wire_version_sentinels` | `HASH_SCHEMA_VERSION == 69`, `PROTOCOL_VERSION == 32`. The wire-neutrality pin and AC 5873's machine check. | passes before and after **by design** |

### 9.3 The in-src unit tests (T14, T15)

`GameState.pending_effect_choice` is `pub(crate)` (SR-3, `state/mod.rs:168`) and there is no
test-only setter, so an integration test in `crates/engine/tests/` **cannot** construct the trap
state — and it cannot be reached through legal commands either (§8.2). Put T14/T15 in a
`#[cfg(test)] mod tests` at the foot of `crates/engine/src/rules/resolution.rs`. Precedent exists:
four engine source files already carry in-src test modules (`testing/replay_harness.rs`,
`state/diagnostics.rs`, `rules/layers.rs`, `rules/casting.rs`).

**Do not** add a setter to `state/test_util.rs` — that widens the SR-3 seal for a test's
convenience, which is the opposite of the invariant's purpose. `PendingEffectChoice`'s fields are
all `pub` (`card-types/src/state/stubs.rs:962-981`), so in-crate construction is a struct literal.

### 9.4 Existing tests: predicted impact

| test / corpus | prediction | reasoning |
|---|---|---|
| `mechanics_a_d/dredge.rs` tests 1,2,3,7,8,10,12,13 | **UNCHANGED** | P7: every one reaches the offer before answering; the offer now pushes an entry the answer consumes |
| `mechanics_a_d/dredge.rs` test 9 (`:678`, no offer, `Some`) | **UNCHANGED** | asserts `is_err()` only; the gate makes it `Err` earlier with a different message |
| `mechanics_a_d/dredge.rs` test 13 (`:918`, two `draw_card` calls) | **UNCHANGED** | two *separate* single draws, not a sequence; each pushes and each is consumed |
| `mechanics_e_l/golgari_grave_troll.rs:359` | **UNCHANGED** | reaches the offer first |
| `primitives/pb_dp5_pending_draw_choice.rs:441` (dredge-decline) | **UNCHANGED** — traced explicitly: offer pushes entry (len 1) → `ChooseDredge{None}` consumes (len 0) → resume hits `NeedsChoice` → pushes fresh entry (len 1) ⇒ the `len() == 1` assertion still holds and `OrderReplacements` still answers it | |
| `primitives/pb_dp5_pending_draw_choice.rs:1204` (no-deadlock) | **UNCHANGED** | §4.5: no progress gate is added |
| golden `replacement/014` | **UNCHANGED** | `turn_based_action: draw_card` then `player_action: choose_dredge`; harness `translate_player_action`'s `"choose_dredge"` arm (`replay_harness.rs:1149-1152`) needs no edit |
| 211 golden scripts | **UNCHANGED** | §6: one name hit, traced above. Re-verify before starting; SR-9c forbids silent skips |
| `tests/core/decision_gate.rs` `BASELINE` | **UNCHANGED** | 0 card-def edits |
| `crates/simulator` tests | **UNCHANGED** | no `ChooseDredge` producer (P4); no blocking added |

---

## 10. Seeds to file (`docs/audits/decision-point-audit.md` §8.1)

| seed | finding | class |
|---|---|---|
| **OOS-DX2-1** | **`Command::ChooseMiracle` is not gated on the offer, and CR 702.94a's "as you draw it" is unenforced.** `handle_choose_miracle` (`rules/miracle.rs:44-106`) validates hand + `KeywordAbility::Miracle` + `cards_drawn_this_turn == 1`, but **not** that the named card is the card just drawn. A miracle card already in hand (tutored, drawn last turn, returned) can be revealed and cast for its miracle cost on any turn whose first draw has happened. Closing it needs a record of the just-drawn object ⇒ new `GameState` state ⇒ HASH bump, which is why PB-DX2 corrected only the doc. Same doc-vs-code family as OOS-DP7-2. | correctness, **live**, HASH-bumping |
| **OOS-DX2-2** | **A resumed draw sequence never re-offers dredge for its remaining draws.** `perform_remaining_draws` (PB-DX2, extracted from `resolve_pending_draw:1402`) passes `offer_dredge: false` for every tail draw, so after one deferral the rest of a "draw three" is dredge-immune. CR 702.52a applies to each individual draw (CR 121.2), so each should be separately replaceable. PB-DP5 plan §3.3's argument for `false` is about not restarting a CR 616.1 application *on the same draw* and does not cover the tail. Behaviour preserved deliberately by PB-DX2 to keep the batch minimal. | correctness, latent |
| **OOS-DX2-3** | **`pending_draws` can still hold two entries for one player, and `handle_order_replacements` routes by `position()` (FIFO) with an applicability set computed from `state`, not from the entry.** PB-DX2 bounded the *dredge* contribution (§4.2's fold) but left `NeedsChoice`'s. With two entries an answer can resolve the older one with the newer one's intent. Also: `pending_draws` is in `loop_detection::compute_mandatory_state_hash`, so unbounded growth could mask a CR 104.4b mandatory loop. Closing it properly wants a per-entry id on `PendingDraw` ⇒ HASH bump. | correctness, latent |
| **OOS-DX2-4** | **`Command::KeepHand` and `Command::TakeMulligan` have no pregame phase gate at all.** `engine.rs:477-489` checks only `validate_player_exists`; `mulligan_count` never resets, so `required_bottom` stays positive for the whole game. PB-DX2's per-entry hand guard reduces `KeepHand` to self-harm, but `TakeMulligan` is untouched and is worse: mid-game it shuffles the sender's hand into their library, reshuffles, and draws seven. There is no `mulligan_phase` state to consult (`PlayerState::mulligan_count` is the only mulligan field), so the fix is new stored state ⇒ HASH bump. | correctness, **live**, HASH-bumping |
| **OOS-DX2-5** | **The bots never dredge, so every simulated game silently loses the draw of any player with a dredge card in their graveyard.** `crates/simulator` constructs no `Command::ChooseDredge` (0 grep hits in `src/`), and `LegalActionProvider` never offers it. After PB-DX2 the draw is *recorded* rather than destroyed, so the loss becomes a permanently outstanding obligation instead of a silent hole — better, but still not played. Owner: `crates/simulator` / M11-local. | simulator gap |
| **OOS-DX2-6** | **"CR 726" is the wrong cite for mandatory loops throughout this repo.** MCP-verified: **CR 726 is "Restarting the Game" (Karn Liberated)**; the mandatory-loop rule is **CR 104.4b** (*"…somehow enters a 'loop' of mandatory actions… the game is a draw"*). The wrong cite appears in `memory/gotchas-rules.md` §"#34", in `rules/engine.rs`'s `BlockingDecision` obligation (7) doc, and in PB-DP9's plan/review prose. A repo-wide sweep is out of PB-DX2's scope; PB-DX2 cites 104.4b in every comment it writes. | documentation rot |

**Close in `docs/audits/decision-point-audit.md` §8.1**: **OOS-DP5-7**, **OOS-DP7-2**,
**OOS-DP2-1**, **OOS-DP9-14**. **Correct these stale cites while closing them** (the OOS-DP6-8
class — PB-DX1 found two, so re-verify every cite you touch):
- OOS-DP2-1's `rules/commander.rs:877-885` → **`:891`**;
- OOS-DP9-14's *"mirroring `drop_departed_trigger_flush`'s placement (`engine.rs:2664`)"* →
  `:2664` is inside `handle_concede`, not the top of `resolve_top_of_stack` (§8.2).

Also update **OOS-DP7-1**'s row: its "the dredge/miracle pair is OOS-DP7-2's problem first" is now
answered for dredge (**deadline, not block** — §3.4), and re-pointed for miracle (OOS-DX2-1).

---

## 11. Ordered step list for the runner

**Phase 0 — probes first, no production code.**
1. Create `crates/engine/tests/primitives/pb_dx2_command_gates.rs` + the `main.rs` line. Write
   **T1, T2, T5, T10** and run them. **All four must fail.** Paste the failure text into the
   review. If any passes, STOP and report.
2. Write **T14** in a new `#[cfg(test)] mod tests` at the foot of `rules/resolution.rs`. Run it.
   **It must fail** (the entry `debug_assert!` panics). Paste the panic.

**Phase 1 — item 1, the dredge gate.**
3. `replacement.rs`: extract `perform_remaining_draws` (§4.1) and re-express
   `resolve_pending_draw:1397-1420` on it, adding `DredgeOffered` to its `matches!` set.
   `cargo test -p mtg-engine --test primitives --test mechanics_a_d` — everything still green
   (pure refactor + a set the current code can never hit yet).
4. `replacement.rs:828`: the `DredgeAvailable` arm records a `PendingDraw`, with the fold guard
   (§4.2). Run **T4**, **T7**.
5. `effects/mod.rs:9303`: add `DredgeOffered` to `draw_cards_for_player`'s break set (§4.3).
   Run **T5**.
6. `replacement.rs:2925`: rewrite `handle_choose_dredge` per §4.4 (steps 0-4b) and **delete**
   `draw_card_skipping_dredge`, updating `check_would_draw_replacement`'s doc reference at `:636`.
   Run **T1, T2, T3, T6, T8, T9**. Then the full dredge corpus:
   `cargo test -p mtg-engine --test mechanics_a_d --test mechanics_e_l --test primitives`.
7. `cargo build --workspace` (simulator / TUI / replay-viewer — the historic runner-miss is ~50 %,
   even when no enum changed).

**Phase 2 — item 2, the doc sites.** *(separate commit)*
8. Fix all five sites in §5's table, using the model text for site 2. Verify by reading that no
   surviving comment on this path claims a pause, a block, or a guarantee the code does not make.

**Phase 3 — rider OOS-DP2-1.** *(separate commit)*
9. `commander.rs:891`: insert §8.1's guard between the count check and the move loop. Run
   **T10, T11, T12, T13** plus the existing mulligan/commander suites.
   Confirm `bare_lookup_ratchet` did not move.

**Phase 4 — rider OOS-DP9-14.** *(separate commit)*
10. `resolution.rs:90`: insert §8.2's reap **above** the existing `debug_assert!`. Run **T14** (now
    passes) and write and run **T15** (`#[should_panic]`, `#[cfg(debug_assertions)]`) — it must
    pass, proving the assert still has teeth.

**Phase 5 — gates.**
11. `cargo test -p mtg-engine --test core`. **`protocol_schema` and `hash_schema` must be GREEN
    with no edits to `rules/protocol.rs` or `state/hash.rs`.** If either reddens: STOP and report
    (§7.1's falsifier). Add **T16**.
12. `cargo test --all`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check`;
    `tools/check-defs-fmt.sh`; `cargo build --workspace`. Run the 211 golden scripts and confirm
    0 new skips (SR-9c).

**Phase 6 — roster + benches.**
13. Run §6's `all_cards()` enumeration (as a one-off, not a committed test — the roster is one
    card and a permanent gate would be theatre; say so in the review). Report the count even if it
    equals the predicted 1/0 flips. Run §7.3's benches against the merge base in a throwaway
    worktree and report the three numbers.

**Phase 7 — bookkeeping.**
14. File seeds **OOS-DX2-1..6** in `docs/audits/decision-point-audit.md` §8.1; close
    **OOS-DP5-7 / OOS-DP7-2 / OOS-DP2-1 / OOS-DP9-14** with the two cite corrections; update
    **OOS-DP7-1**'s row; move `memory/primitive-wip.md` to the review phase; update
    `memory/workstream-state.md` and CLAUDE.md's snapshot.

---

## 12. What "done" looks like — falsifiable

- [ ] T1, T2, T5, T10, T14 each **fail** on unmodified code, with the failure text quoted in the review.
- [ ] T3 exists and fails pre-fix — the answer is demonstrably **consumed**, not merely required.
- [ ] T5 and T6 exist and fail pre-fix — CR 614.11a's sequence stop-and-resume is demonstrably fixed, not just the gate.
- [ ] T15 exists and passes — the OOS-DP9-14 reap did **not** make `resolve_top_of_stack`'s entry `debug_assert!` vacuous.
- [ ] `rg -n 'draw_card_skipping_dredge' crates/` → **0** (the function is deleted, not orphaned).
- [ ] No comment on the dredge or miracle path claims the engine "pauses"; all five §5 sites reconciled.
- [ ] `PROTOCOL_VERSION == 32` and `HASH_SCHEMA_VERSION == 69`, **unchanged**, with `rules/protocol.rs` and `state/hash.rs` untouched by `git diff`. Stated explicitly in the review.
- [ ] `bare_lookup_ratchet` green with **no** ceiling edits (it fails in both directions).
- [ ] `cargo build --workspace`, `cargo test --all`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` — all green.
- [ ] 211 golden scripts green, 0 new skips; `replacement/014` unchanged.
- [ ] `git diff` over `crates/card-defs/src` is **empty**; 0 completeness flips; roster reported from `all_cards()`.
- [ ] `crates/simulator` and `tools/` build; no test in `crates/simulator/tests` moves.
- [ ] Benches reported against the merge base; `full_turn_4p` within 5 % of ≈217 µs.
- [ ] Seeds OOS-DX2-1..6 filed; four seeds closed; both stale cites corrected; OOS-DP7-1's row updated.
- [ ] Test count reported; expected ≈ **3,945 → 3,961-3,965**.

---

## 13. Risks

1. **Adding a progress gate by accident.** The single way this batch can hang the game. Any code
   that reads `pending_draws` from `enter_step`, `handle_all_passed`, `blocking_decision` or the
   SBA loop is out of bounds. T8 and `pb_dp5_...rs:1204` are the antidotes; neither is optional.
2. **The design drifting into a new field.** The moment someone writes "just add a `kind: DredgeOffer`
   to `PendingDraw`", HASH moves and AC 5873 fails. §3.2/§3.3 exist so that argument is already
   had. If it genuinely cannot be avoided: **stop and report**, do not bump.
3. **The fold guard losing a draw.** §4.2 folds `1 + remaining_after` into the existing entry. Get
   the arithmetic wrong and draws vanish — which is the bug being fixed, re-introduced. T7 pins it
   with an explicit total (draw-step 1 + effect 2 = 3 cards after one answer).
4. **Breaking `pb_dp5_pending_draw_choice.rs:441`.** It is the one existing test that exercises the
   dredge→NeedsChoice hand-off. The trace in §9.4 says it holds; if it reddens, the `None` arm is
   not threading the entry's `already_applied`/`remaining` correctly (§4.4 step 4b).
5. **`ChooseDredge` on a `NeedsChoice` entry looking like a hole.** It is argued legal in §3.3
   (CR 616.1e). Do not "fix" it by adding a discriminator; document it.
6. **The OOS-DP9-14 reap swallowing a real bug.** A reap that clears unconditionally deletes the
   only detector this codebase has for CR 608.2d re-entrancy. Narrow (dead owner only), placed
   above the assert, pinned by T15.
7. **`handle_keep_hand`'s guard placed after the move loop.** Then a rejected command has already
   moved a card. Validation-before-mutation is the whole point; T12's message assertion is what
   catches a runner who reordered it.
8. **Scope creep into `TakeMulligan` / miracle / the CR 726 cite sweep.** All three are seeded
   (OOS-DX2-4, OOS-DX2-1, OOS-DX2-6) and all three are HASH-bumping or repo-wide. Leave them.
9. **Skipping `cargo build --workspace`.** No enum changed here, which makes it *feel* skippable;
   the historic miss rate on the replay-viewer / TUI exhaustive matches is ~50 % and the build is
   the gate, not the reasoning.

---

## 14. Parallel-task collision surface (`scutemob-163`, M11-local S3)

`scutemob-163` runs concurrently and touches `crates/engine`: **new** `rules/queries.rs` and
`tests/rules/queries.rs`, plus `rules/mod.rs`, `lib.rs`, and visibility changes in `casting.rs`.

**PB-DX2 touches**: `rules/replacement.rs`, `rules/engine.rs` *(doc only — §3.6 says the
`ChooseDredge` arm is unchanged; in practice PB-DX2 edits **no** line of `engine.rs`)*,
`rules/commander.rs`, `rules/resolution.rs`, `rules/events.rs`, `effects/mod.rs`, plus
`tests/primitives/pb_dx2_command_gates.rs` and `tests/primitives/main.rs`.

**Overlap: none.** Three standing instructions:
- **Do not add anything to `crates/engine/src/lib.rs`.** Every new test uses existing public API
  (`process_command`, `GameStateBuilder`, `state.pending_draws()`, `state.blocking_decision()`);
  T14/T15 are in-crate and need no export.
- **Do not touch `rules/mod.rs`** — no new module is created (T14/T15 live inside
  `rules/resolution.rs`).
- **Do not touch `tests/rules/main.rs`** — all PB-DX2 integration tests go in the `primitives`
  target. If something forces an edit to any of `lib.rs`, `rules/mod.rs`, `casting.rs` or
  `tests/rules/main.rs`, **flag it in the review** so the collect can sequence the merges.
