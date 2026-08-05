# Primitive Batch Plan: PB-DX23 — dredge has no answer channel for anyone

**Generated**: 2026-08-04
**Task**: `scutemob-201` · worktree `/home/skydude/projects/scutemob/.worktrees/scutemob-201`
**Verified at**: HEAD `e490153b` (PB-DX21 merge)
**Primitive**: `LegalAction::ChooseDredge { card: Option<ObjectId>, mill: u32 }` — a
simulator-local answer channel for the CR 702.52a dredge offer, plus one shared engine
query (`rules::queries::dredge_options`) so the offer and the engine's own scan are one
derivation.
**Seeds**: `OOS-DX2-5` (primary) + `OOS-DX2-2` (rider) + `OOS-DX2-7` (rider, record-only) ·
watch item `OOS-DX2-3`
**Brief (authoritative)**: `memory/primitives/seed-rerank-2026-08-02.md` §4, lines 987-1034;
seed rows at :229, :273, :312-315.
**CR rules**: 702.52a/b · 121.1 / 121.2 / 121.6b · 614.11 / 614.11a · 616.1e / 616.1f ·
400.7 · 400.1 · 104.3c · 117.3 / 117.4 · 514.1 (admission gate) · 103.8a/c
**Dependencies**: PB-DP4 (`PayEcho` deadline-action precedent), PB-DP5 (`PendingDraw`),
PB-DP7/DP8/DP9 + UI-1 (blocking-decision machinery — **read, then deliberately NOT used**,
see §3 Q1), PB-DX2 (the gate this batch answers), SIM-6 (`ActivationCostPlan` provider
precedent), PB-DX20 (`queries.rs` one-derivation-two-consumers shape).
**Baseline (must be re-measured on the branch BEFORE any edit)**: tests **4,398 / 0 / 5**,
**PROTOCOL 35**, **HASH 73**.

**TODO sweep (mandatory roster-recall gate, run)**:
`grep -rn "Dredge(" crates/card-defs/src/defs/` → **1 hit**, `golgari_grave_troll.rs:80`.
`grep -rni "dredge" crates/card-defs/src/defs/` → **2 files**: `golgari_grave_troll.rs` and
`thrasios_triton_hero.rs` — the latter is a **false positive**, its `Completeness::partial`
note names the WouldDraw/dredge channel only to explain why a `RevealAndRoute` zone move
bypasses it; that card has no dredge ability and is not unblocked by this batch.
**TODO sweep result: 0 cards added.** `golgari_grave_troll` is the corpus's only dredge def
and this batch adds **no card definition and edits no card definition body** (one comment
correction only — §4 Stage 6).

---

## §1. The defect, restated with verified line numbers

### 1.1 Primary — `OOS-DX2-5`: nobody can answer a dredge offer

`grep -rn "ChooseDredge" crates/simulator/src/ tools/` returns **zero hits** (re-verified at
`e490153b`; the workspace-wide grep finds 40 files, none of them under `crates/simulator/` or
`tools/`). There is no `LegalAction::ChooseDredge` variant, so:

* no bot can answer (`StubProvider` never emits it, `params.rs` cannot map it,
  `heuristic_bot` has no arm for it);
* **the human seat in the shipped browser cannot answer either** — `tools/play-server`
  renders `LegalAction`s, and one that is never emitted is never rendered.

The consequence is not a lost option. It is a **permanent draw-cadence corruption**:

| step | file:line | what happens |
|---|---|---|
| draw step draws with `offer_dredge: true` | `crates/engine/src/rules/turn_actions.rs:1252-1259` | `perform_one_draw(state, player, true, true, {}, 0)` |
| dredge scan finds an eligible card | `crates/engine/src/rules/replacement.rs:699-733` | returns `DrawAction::DredgeAvailable(GameEvent::DredgeChoiceRequired{..})` at `:729` |
| the offer REPLACES the draw | `replacement.rs:953-977` | pushes a `PendingDraw` at `:970` and returns `DrawStepOutcome::DredgeOffered` — **no card is drawn** |
| nothing ever answers | — | the entry stands; priority, SBAs and step advancement all continue (`events.rs:860-875`) |
| next turn's draw discharges the stale entry FIRST | `replacement.rs:945-950` | `position(|p| p.player == player)` → `remove(i)` → `resolve_declined_pending_draw(..)` (`:1122-1135`) |
| …then defers the CURRENT draw | `replacement.rs:953-977` | a fresh entry is pushed |

So the player is permanently **one draw behind**, forever, and the drawn card comes off a
library that has had a full turn cycle to be reordered. `golgari_grave_troll.rs` declares no
`completeness` field, so it is `Complete` by derive (`#[default]`), deck-legal, and the
corpus's **only** dredge def.

### 1.2 Rider — `OOS-DX2-2`: the multi-draw TAIL is dredge-immune

`perform_remaining_draws` (`replacement.rs:1563-1591`) loops `remaining` times calling
`perform_one_draw(state, player, false, …)` at `:1572-1579`. Its own doc at `:1544-1562`
names `OOS-DX2-2` and says suppressing dredge for the whole tail is *"a pre-existing
simplification this batch deliberately does not change"*. **That doc is what this batch must
update.**

The asymmetry is sharper than the seed row states, and the plan states it precisely because
the runner will otherwise mis-scope the fix: `effects/mod.rs:9495-9525`
(`draw_cards_for_player`, the *fresh* multi-draw sequence) already passes `offer_dredge:
true` on **every** iteration (`:9503`). So a "draw three" that is never interrupted offers
dredge on all three draws. It is only once the sequence has been **interrupted and resumed**
that the tail becomes dredge-immune — three resume sites, all passing `false`:

| site | file:line | verdict (§3 Q3) |
|---|---|---|
| `resolve_declined_pending_draw`'s own re-draw | `replacement.rs:1127-1135` (the `false` at `:1130`) | **KEEP `false`** — same draw |
| `perform_remaining_draws`' loop | `replacement.rs:1572-1579` (the `false` at `:1575`) | **FLIP to a caller-supplied flag** |
| `resolve_pending_draw`'s CR 616.1f re-check | `replacement.rs:1668-1675` (the `false` at `:1671`) | **KEEP `false`** — same draw |

### 1.3 Rider — `OOS-DX2-7`: the stale-entry discharge is an unrecorded AUTO-CHOSEN decision

`replacement.rs:935-950` auto-declines on the player's behalf. It is invisible to
`crates/engine/tests/core/decision_gate.rs` **by construction**: that gate walks card-def
`Effect`/`Condition` DSL variant names over `all_cards()`
(`crates/engine/tests/core/decision_site_walk.rs`), and dredge is a `KeywordAbility` reached
through none of them. That is a fresh instance of `OOS-DP10-9` (a decision with no DSL
trace), not a gate bug. It is already narrated in `docs/audits/decision-point-audit.md:947-962`
but has **no row** in §3.1.

### 1.4 Watch item — `OOS-DX2-3` (REOPENED; do NOT re-close on a structural proof)

`pending_draws` is **not** bounded to one entry per player. `replacement.rs:946-950`
discharges; `resolve_declined_pending_draw` re-enters `perform_one_draw` (`:1127`); both
`:970` and `:991` `push_back` unconditionally, so the inner call can push between them.
PB-DX2 closed this on a **structural** proof ("both push sites are downstream of the
discharge") — a claim about *where* the pushes are, not *when* they run — and a re-review
reproduced it empirically and REOPENED it. **Pin**: `crates/engine/tests/primitives/
pb_dx2_command_gates.rs:1272` (`test_dx2_needschoice_redefer_grows_the_queue`).

Two comment corrections owed (the function doc at `replacement.rs:853-904` was properly
corrected; the two **push-site** comments were not):

* `replacement.rs:959-962` — *"This push is always into an EMPTY slot for `player` — the
  discharge above guarantees it, so there is no fold/accumulate case here anymore"*
* `replacement.rs:983-984` — *"As above, this push is always into an empty slot for
  `player`"*

Both still assert the retracted claim. A reader who greps to the push site gets the
falsified story.

---

## §2. CR research (verbatim from MCP)

**702.52a** — "Dredge is a static ability that functions only while the card with dredge is
in a player's graveyard. 'Dredge N' means 'As long as you have at least N cards in your
library, if you would draw a card, you may instead mill N cards and return this card from
your graveyard to your hand.'"

**702.52b** — "A player with fewer cards in their library than the number required by a
dredge ability can't mill any of them this way."

**121.1** — "A player draws a card by putting the top card of their library into their hand.
This is done as a turn-based action during each player's draw step. It may also be done as
part of a cost or effect of a spell or ability."

**121.2** — "Cards may only be drawn one at a time. If a player is instructed to draw
multiple cards, that player performs that many individual card draws."

**121.6b** — "If an effect replaces a draw within a sequence of card draws, the replacement
effect is completed before resuming the sequence."

**614.11a** — "If an effect replaces a draw within a sequence of card draws, all actions
required by the replacement are completed, if possible, before resuming the sequence."

**616.1e** — "Any of the applicable replacement and/or prevention effects may be chosen."

**616.1f** — "Once the chosen effect has been applied, this process is repeated (taking into
account only replacement or prevention effects that would now be applicable) until there are
no more left to apply."

**104.3c** (via `replacement.rs:1024-1026`) — being required to draw more cards than remain
in the library causes a loss.

**400.1** — a graveyard is a **public** zone. **400.7** — an object that changes zones
becomes a new object.

### 2.1 What the rules decide for this batch

1. **CR 121.2 + 614.11a/121.6b make each individual draw separately replaceable.** "Draw
   three" is three draws; dredge applies "if you would draw **a** card". So the tail of an
   interrupted sequence is dredge-offerable, and `perform_remaining_draws`' blanket `false`
   is a CR deviation. This is the whole of `OOS-DX2-2`.
2. **PB-DP5 §3.3's argument is about ONE draw, not the sequence.** Its `false` exists so a
   CR 616.1 application the player already began is not restarted *on the same draw event*.
   `resolve_declined_pending_draw`'s call at `:1130` and `resolve_pending_draw`'s at `:1671`
   are both the **same draw** resuming; `perform_remaining_draws`' loop is a **different**
   draw each iteration. **This distinction must appear in the commit message** (acceptance
   criterion 3).
3. **CR 702.52a is "you MAY instead".** Declining is always legal, so "no answer" has a
   well-defined meaning — which is exactly the block-vs-deadline test PB-DP4/PB-DP7 use, and
   why `GameEvent::DredgeChoiceRequired`'s doc (`events.rs:870-875`) classifies the offer as
   a DEADLINE rather than a `BlockingDecision`. See §3 Q1.
4. **CR 400.1 makes the offer's contents public.** A dredge offer names GRAVEYARD cards
   only. It does **not** disclose library order — the mill it causes reads library cards but
   the offer itself carries none. Architecture Invariant 7 is satisfied without any new
   redaction work (§6 R7).
5. **CR 702.52b is a legality floor, not a survival rule.** `library_count >= n` is what the
   engine checks (`replacement.rs:714`, `:3350`). Nothing in the CR stops a player dredging
   themselves toward CR 104.3c. A bot needs its own margin (§3 Q4).

---

## §3. The seven design decisions

### Q1 — Where does the provider emit `ChooseDredge`?

**Decision: (a) an ordinary priority-window action**, appended at the END of the PB-DP4
pay-or-lose-it block in `StubProvider::legal_actions` — immediately after the `PayRecover`
loop (`crates/simulator/src/legal_actions.rs:604-618`) and before the `is_main_phase`
computation at `:620`.

**Why not (c), a fourth `BlockingDecision` variant.** It is CR-wrong and expensive:

* CR 702.52a is "you **may** instead". A `BlockingDecision` is for a decision the game
  cannot proceed past; the engine's own doc says so at `events.rs:870-872`, and the engine
  *already* deliberately does not block (`BlockingDecision` at
  `crates/engine/src/rules/engine.rs:145-166` has exactly three variants, and
  `DredgeChoiceRequired`'s doc at `events.rs:860-875` spends fifteen lines on why).
* Blast radius, from `engine.rs:87-144`'s own **seven-obligation** checklist: the admission
  allow-list at `engine.rs:304-314` (`Command::ChooseDredge` is *not* in it today), a
  `handle_concede` clear, **`state/hash.rs::public_state_hash` by name → HASH 73 → 74**, a
  `loop_detection` decision, `local_game.rs:765-769`'s exhaustive match, the foreign-concede
  gate, and a resume-site debt. That is a HASH bump for a decision the CR says is optional.
* It would also *retire* CR 702.52a's own "no answer is an answer": blocking means the
  player can no longer decline by inaction, and the CR 616.1 deadline residual `OOS-DP5-2`
  would have to be closed in the same batch.

**Why not (b), an exclusive early-return before the `priority_holder` check
(`legal_actions.rs:508-510`).** The `DiscardToHandSize` precedent early-returns because the
**engine's admission gate enforces exclusivity** (`engine.rs:304-318` rejects every other
command). For dredge the engine enforces nothing: `PassPriority` stays legal, every cast
stays legal. A provider that offered `ChooseDredge` **alone** would be **lying about
legality in the withholding direction** — the mirror image of SR-38, and worse for a human
client, which would lose every other button while an *optional* offer stands.

**The non-priority-holder interaction, resolved rather than waved at.** A `PendingDraw` can
belong to a player who does not hold priority (an effect-draw for a non-active player), and
`legal_actions.rs:508-510`'s early return means such a player is offered nothing. `advance()`
(`local_game.rs:754-801`) resolves exactly one acting seat per iteration, so the offer is
simply **not surfaced yet**. That is acceptable and the reason is CR 117.3d: after any spell
or ability resolves, the active player receives priority and then **every** player receives
priority in turn order before the game advances a step. So a living player always reaches a
priority window with the entry still standing, long before their own next draw (which is the
only event that auto-discharges it). The offer is therefore *deferred*, never *lost*.
**Record as a seed** (`OOS-DX23-1`) with the one residual: a player who is dead or has
conceded never gets priority — but `handle_choose_dredge`'s step 0 (`replacement.rs:3267-3283`)
already discharges their entry, so there is nothing to offer.

**Two SR-38 consequences the placement buys for free**, both to be asserted:

* while a `BlockingDecision` stands, the provider early-returns at `legal_actions.rs:420-487`
  and never reaches this block — which is **required**, because `engine.rs:304-314`'s
  allow-list does not name `Command::ChooseDredge` and would reject it with
  `BlockedByPendingDecision`;
* a dead/conceded player never holds priority, so the offer never reaches one.

### Q2 — Is decline offered separately from each `Some(card)`?

**Decision: emit `ChooseDredge { card: None, mill: 0 }` PLUS one
`ChooseDredge { card: Some(id), mill: n }` per currently-eligible dredge card — and emit
NOTHING AT ALL when no dredge card is currently eligible.**

Both halves are CR-argued:

* **`None` is always legal** (CR 702.52a "may"), and `handle_choose_dredge`'s `None` arm
  (`replacement.rs:3300-3310`) accepts it against *any* entry. Offering it is what makes the
  channel able to discharge an entry the player does not want to use.
* **`Some(id)` is offered only for what CR 702.52a/b currently permit**, re-derived at OFFER
  time from the live graveyard and library — never read off the stale
  `DredgeChoiceRequired` event, whose `options` were computed when the entry was pushed and
  can have gone stale (the card exiled, the library milled below `n`). The engine's own
  `Some` validation (`replacement.rs:3313-3355`) is byte-for-byte
  `check_would_draw_replacement`'s eligibility predicate, so this keeps the offer a strict
  subset.
* **Suppressing the offer entirely when nothing is eligible** is the answer to the brief's
  "does that hand bots a way to fake-answer?". It does something worse than fake-answer: a
  `NeedsChoice`-origin entry answered with `ChooseDredge { None }` **re-defers** — the
  decline's resume hits `NeedsChoice` again and pushes a fresh entry, pinned by
  `pb_dx2_command_gates.rs::test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry`
  (`:1200-1249`, `pending_draws().len()` back to 1). A bot scoring `ChooseDredge` above
  `PassPriority` would decline forever and burn `max_commands`. Suppressing the offer when
  no dredge card is eligible removes that loop **structurally**, at zero CR cost:
  * a `NeedsChoice` entry's correct answer is `Command::OrderReplacements`, which the
    provider does not offer either, so nothing is lost that the provider had;
  * corpus reach is **zero** — `grep -rn "ReplacementTrigger::WouldDraw"
    crates/card-defs/src/defs/` returns 0, so a `NeedsChoice`-origin entry cannot arise from
    a legal deck;
  * the entry still self-heals via the `perform_one_draw` discharge.
  **Trade-off, stated**: a `NeedsChoice` entry remains unanswerable through this channel.
  File as `OOS-DX23-2`, with the note that `LegalAction::OrderReplacements` is the real fix
  and is out of scope (it needs a per-entry replacement-id list on the offer).

**The derivation must be shared, not mirrored.** Re-deriving CR 702.52a/b eligibility inside
`crates/simulator` is the `OOS-RS-2` drift class. Add a read-only engine query:

```rust
// crates/engine/src/rules/queries.rs
/// CR 702.52a/b: every card in `player`'s graveyard that could replace a draw
/// right now, as `(card, N)`, sorted by `ObjectId` for determinism.
pub fn dredge_options(state: &GameState, player: PlayerId) -> Vec<(ObjectId, u32)>
```

and make `check_would_draw_replacement` (`replacement.rs:699-733`) **call it** rather than
keep its own copy. One arithmetic, two consumers — the PB-DX20 shape. **PB-DX20's own lesson
applies and must be honoured**: a differential probe between two consumers proves
*consistency*, not *correctness*, so the query also gets its own direct CR 702.52a/b probes
(§5 T2.x).

### Q3 — The `OOS-DX2-2` tail flip

**Decision, per call site:**

| # | site | today | after | reason |
|---|---|---|---|---|
| 1 | `resolve_declined_pending_draw` → `perform_one_draw` (`replacement.rs:1130`) | `false` | **`false` (UNCHANGED)** | This is the **same draw** resuming. CR 702.52a's "may" was already answered *no* for this draw event; re-offering it would be an infinite choice loop. Pinned by `dredge.rs::test_dredge_decline_does_not_reoffer` (`:931-1004`, "Test 10"). PB-DP5 §3.3's argument DOES cover it. |
| 2 | `resolve_pending_draw` → `perform_one_draw` (`replacement.rs:1671`) | `false` | **`false` (UNCHANGED)** | Same draw, CR 616.1f re-check. PB-DP5 §3.3 covers it identically. |
| 3 | `perform_remaining_draws` → `perform_one_draw` (`replacement.rs:1575`) | `false` (hard-coded) | **caller-supplied `offer_dredge: bool`** | Each iteration is a **different draw** (CR 121.2). CR 614.11a/121.6b say the replacement completes *and then the sequence resumes*; the resumed draws are new "would draw" events. PB-DP5 §3.3 does **not** reach them. |

**`perform_remaining_draws` gains a parameter rather than a hard `true`, and this is the
load-bearing part of the decision.** Its three callers do not want the same thing:

| caller | file:line | passes | why |
|---|---|---|---|
| `handle_choose_dredge`, `Some` arm | `replacement.rs:3394-3400` | **`true`** | The player is answering; the tail is theirs to answer too. |
| `resolve_pending_draw` (CR 616.1 resume) | `replacement.rs:1697-1702` | **`true`** | Same reason; zero corpus reach either way. |
| `resolve_declined_pending_draw` | `replacement.rs:1145-1150` | **the flag it was given** | See below. |

`resolve_declined_pending_draw` therefore also gains a parameter,
`tail_offers_dredge: bool`, passed **`true`** by `handle_choose_dredge`'s explicit-decline
arm (`replacement.rs:3309`) and **`false`** by `perform_one_draw`'s implicit stale-entry
discharge (`replacement.rs:949`).

**Why the discharge path must NOT offer dredge in the tail — this is the `OOS-DX2-3`
consequence the brief asked the taker to own, and it is REACHABLE POST-FIX with
`golgari_grave_troll` alone.** Trace, with an unconditional `true` at `:1575`:

1. Turn N: an effect draws three with the Troll in the graveyard. Draw 1 offers dredge →
   entry `E{remaining: 2}` pushed, sequence stops (`effects/mod.rs:9515-9522`).
2. The player does not answer (post-fix this is still legal and still reachable: the provider
   offers `PassPriority` alongside, per Q1, and a human may simply not click).
3. Turn N+2: `draw_card` → `perform_one_draw(offer_dredge: true)`. Its FIRST act
   (`:946-950`) discharges `E` → `resolve_declined_pending_draw` → `perform_one_draw(false)`
   completes draw 1 → `perform_remaining_draws(2)` → with an unconditional `true`, draw 2
   offers dredge, pushes `E2{remaining: 1}`, `break`.
4. Control returns to the OUTER `perform_one_draw`, whose discharge already ran, so it now
   runs `check_would_draw_replacement(offer_dredge: true)` → `DredgeAvailable` → pushes
   `E3{remaining: 0}` at `:970`.
5. `pending_draws` now holds **two dredge-originated entries** for one player.

That breaks the ONE invariant the discharge does establish, stated verbatim at
`replacement.rs:886-890`: *"at most one **dredge-originated** entry can exist per player,
because that arm only runs when `offer_dredge` is true, which is never the case on a
re-entrant discharge call"*. It would make `OOS-DX2-3` — LOW, zero-corpus-reach today —
**live and reachable from the corpus's only dredge card**, and it would grow the queue by
one per turn cycle in the unanswered regime. Threading the flag preserves the invariant
verbatim, at the cost of one extra parameter and one honest CR deviation: **an
auto-discharged sequence's tail is auto-declined whole, not just at its head.** That
deviation is *consistent with what the discharge already is* (an engine-made auto-decline,
`OOS-DX2-7`) rather than a new kind of deviation, and it is strictly smaller than today's
(today the whole tail is dredge-immune for **every** caller).

**Alternative considered and REJECTED**: unconditional `true` at `:1575`, accept the growth,
pin it. Rejected because the growth is unbounded across turns, `pending_draws` is hashed
(`state/hash.rs`), and widening a reopened seed's reach as a side effect of closing a
different one is exactly the class this project keeps getting burned by. **The runner must
record which option it took and why** — if it deviates to the unconditional flip it owes an
explicit test pinning the two-entry state and an `OOS-DX2-3` row update.

**`PendingDraw.remaining` bookkeeping stays correct.** `perform_remaining_draws` computes
`remaining_after = remaining - 1 - i` at `:1571` and hands it to `perform_one_draw`, which
stores it verbatim on the pushed entry at `:973`. So an entry raised at tail position `i`
carries exactly the number of draws still owed after it, and `handle_choose_dredge`'s
`pending.remaining` (`:3394`) resumes from the right place. **No field changes; HASH
untouched.**

**Termination argument (required by the brief).** Bounded by the sequence length, and the
bound is structural rather than empirical:

* `perform_remaining_draws` is a `for i in 0..remaining` over a `u32` captured before the
  loop (`:1570`) and `break`s on `DredgeOffered` (`:1581-1588`);
* each accepted `ChooseDredge` (either arm) **consumes exactly one entry**
  (`replacement.rs:3308` / `:3359`) and completes or replaces exactly one draw, then resumes
  with a strictly smaller `remaining`;
* `remaining` is monotonically non-increasing across the whole chain — nothing anywhere
  increases it (the pre-PB-DX2 `remaining += 1 + remaining_after` fold is gone,
  `replacement.rs:860-872`);
* dredging removes the dredge card from the graveyard (CR 400.7, `:3377`), so a single
  dredge card cannot re-offer itself.

Therefore a "draw N" produces at most N offer/answer round trips. **No ping-pong.**

### Q4 — The heuristic bot's dredge policy

**Decision** — in `crates/simulator/src/heuristic_bot.rs::score_action`
(`:186`, table at `:192-341`):

```
LegalAction::ChooseDredge { card: None, .. }      => 2
LegalAction::ChooseDredge { card: Some(_), mill } =>
    if library_count(state, player) >= 2 * (*mill as usize) { 3 } else { 0 }
```

with `_player` at `:186` promoted to `player` (it is currently unused).

Stated rule, not a preference:

* **Decline scores 2** — the `PayEcho { pay: false }` precedent verbatim
  (`heuristic_bot.rs:319-327`): just above `PassPriority`'s 1, so the bot always discharges
  an outstanding offer rather than sitting on it, and below every real play
  (`PlayLand` 100, `CastSpell` 50+, `ActivateAbility` 40, `DeclareAttackers` 30+), so
  answering never displaces a play.
* **Dredge scores 3 only with 2× library headroom.** CR 702.52b's `library_count >= n` is a
  *legality* floor already enforced by the engine and mirrored by the offer; `2 * n` is a
  *survival* rule against CR 104.3c and is the bot's only defence against milling itself
  out. Below the margin it scores **0** — the project's "below `PassPriority`, above
  nothing" idiom (`heuristic_bot.rs:187-191`, `:275-276`), so the action stays choosable when
  it is all there is and the resulting command is one the engine ACCEPTS.
* `RandomBot` picks uniformly and therefore keeps exercising both arms end to end
  (`random_bot.rs` reaches `action_to_command` with `ActionParams::default()`).

**Note for the runner**: with the current corpus the mill-out risk is small in practice
because dredging returns the Troll to **hand** (`replacement.rs:3377`), removing it from the
graveyard and ending the offer chain. The margin rule is written for the general case and
must not be dropped on that observation.

**Recorded fuzz seeds: YES, they move — say so plainly.** A new action in
`StubProvider`'s list shifts every `RandomBot` RNG draw downstream of it. `OrderBlockers`'
doc (`legal_actions.rs:276-296`) explains the precedent and why it was kept OUT of the
provider for exactly this reason — **and that precedent does not transfer**, for the reason
its own item 1 gives: `Command::OrderBlockers` is *optional* and a bot that never issues it
plays a legal game, whereas an unanswered dredge offer is a permanent draw-cadence
corruption. The cost of keeping this out of the provider is the defect this batch exists to
close.

Scope of the shift, measured rather than assumed:

* the shift occurs **only in games where a dredge offer arises**, which requires
  `golgari_grave_troll` in the deck, which requires a green-inclusive commander identity
  (`deck.rs:58-97`, 60 non-lands drawn from the identity-filtered `Complete` pool);
* the play-server's seeded pins (`UI1_SEED`/`SIM1_SEED`/`UI2_SEED`/`UI6_SEED`,
  `COMBAT_SEED`, `TARGET_SEED`, `UI3_SPLIT_COMBAT_SEED`, `DISTINCTIVE_SEED`) all use
  `DeckSource::Fixed` decks with no dredge card, so they **must not** move — if one does,
  that is a finding, not a re-pin;
* the ratchets that CAN redden are listed in §6 R1.

### Q5 — The mandatory probe's shape

**Decision: `crates/simulator`'s `LocalGame`, started from a `GameStateBuilder`-assembled
pre-game state via `LocalGame::start` (`local_game.rs:432-442`).** Not the engine's script
harness — the whole point is that the *provider* and the *bot* answer, and neither exists in
the script regime.

**"No state pokes" is defined as follows, and the definition goes in the test's own doc:**

* **PERMITTED**: anything expressible on `GameStateBuilder` before `LocalGame::start` — the
  registry, players, zone contents (including `ObjectSpec::card(p1, "Golgari Grave-Troll")
  .in_zone(ZoneId::Graveyard(p1)).with_card_id(CardId("golgari-grave-troll"))`), library
  stocking, the starting step. `LocalGame::start` runs the real `start_game`, so Architecture
  Invariant 9 is enforced on the fixture.
* **FORBIDDEN**: any mutation of `GameState` after `start` — in particular
  `state.pending_draws` — and any direct call from the test body to `perform_one_draw`,
  `turn_actions::draw_card`, `replacement::*` or `process_command`. **Every** command must
  come from `advance()`'s own bot path.

Fixture (`crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`):

* 2 players, **both bot seats** (`human_seats` empty → `advance()` runs to completion),
  `HeuristicBot`, fixed seed, `check_invariants: true`, `record_journal: true`.
* `p1` graveyard: the real `golgari_grave_troll` def through a `CardRegistry`, enriched
  (`enrich_spec_from_def`) so `characteristics.keywords` actually carries `Dredge(6)` — the
  standing `ObjectSpec::card` gotcha.
* Libraries: ≥ 40 filler objects each with **no `card_id`** (so Invariant 9 never sees them
  — the PB-DX32 T1.1 recipe, `pb_dx32_fuzz_output.rs`), enough to survive the run and to keep
  `library_count >= 12` for the bot's 2× margin.
* Run `advance()` until the turn counter passes **6** (three of `p1`'s own turns: 1, 3, 5).
  CR 103.8a makes `p1` skip the draw on turn 1 in a two-player game
  (`turn_actions.rs:1226-1233`), so "three turns" is three of `p1`'s turns, two of which have
  a real draw step. **The runner must state this in the test doc rather than letting a reader
  assume three draws.**

**Assertions** (A1 is primary because it is robust to bot policy; A2 is the arithmetic one;
A3 is the non-vacuity floor):

| id | assertion | pre-fix | post-fix |
|---|---|---|---|
| A1 | `game.state().pending_draws()` is **EMPTY** at the halt | exactly **1** entry, for `p1` | **0** |
| A2 | `count(CardDrawn{player: p1}) + count(Dredged{player: p1})` over the journal `==` number of `p1` draw steps that occurred | short by exactly **1** | equal |
| A3 | at least one `DredgeChoiceRequired { player: p1 }` in the journal | ≥ 1 | ≥ 1 |

**The exact integers must be MEASURED at Stage 0 (pre-fix) and recorded in the plan's
execution notes, not predicted here.** The predictions above are the *shape* of the
discriminator; the runner writes the literal numbers into the test only after observing
them, per this project's standing "measured, not guessed" rule. A3 exists because a fixture
that never reaches a dredge offer would make A1 and A2 vacuously true.

**Revert to watch red**: delete the `ChooseDredge` push block from
`legal_actions.rs::legal_actions`. A1 and A2 must both redden, with the rebuild confirmed
(`Compiling mtg-simulator` observed in the captured output).

### Q6 — The play-server answer shape

**Decision: NO new `AnswerShapeView` variant, NO new `ActionParams` field, NO new
`ActionParamsDto` field, NO new picker component, and ZERO frontend production lines.**

The choice lives **in the `LegalAction` itself**, one action per choice — the
`PayEcho { permanent, pay }` shape verbatim. Consequences, each verified against source:

* `params.rs::action_to_command_with_params` gets a one-line arm building
  `Command::ChooseDredge { player, card: *card }`. It stays **outside** the nine-arm
  parameterisation allowlist at `params.rs:271-286`, so announcing any param on it is
  refused with `ParamError::UnsupportedParam` — correct, since it has no param channel.
* "Absent means accept the default" **does not arise**: there is no params field to be
  absent. The nearest thing to a default is the *decline* action, which is a distinct offer
  the client must click, not an implicit fallback. This is strictly better than a params
  channel and the runner should say so at the arm.
* `tools/play-server/src/view.rs` needs three arms only —
  `action_kind` → `"ChooseDredge"`, `action_object` → `*card` (already `Option<ObjectId>`;
  a decline correctly has no object), `action_label` → `format!("Dredge {} (mill {mill})",
  card(*id))` / `"Decline dredge — draw normally"`.
* `blocking_decision_view` (`view.rs:2057-2269`) needs **no** arm: it has `_ => None` at
  `:2267`, and a dredge offer is correctly not a blocking decision.
* `api.rs::validate_decision_params` / `validate_combat_params` need **no** arm (both end in
  a catch-all).
* The browser renders it with no change: `ActionBar.svelte:261-263` puts every kind not in
  `controlKinds` (`PassPriority`) or `relocatedKinds` (`Concede`, `TapForMana`) into the
  `plays` group as a plain labelled button. The picker chain (`option.decision`) is not
  entered because `decision` is absent. **This is the `PayEcho`/`PayCumulativeUpkeep`/
  `PayRecover` path, already live in the shipped browser.**

**Divergence from the brief, stated loudly.** Acceptance criterion 1 says "play-server
**blocking-decision UI** surfaces it". It will not, and must not: the engine deliberately
does not block (`events.rs:860-875`), and routing a dredge offer through the
blocking-decision UI would mean adopting Q1 option (c) with its HASH bump and its CR-wrong
exclusivity. **The substance of the criterion — the human answer channel is live end to
end — is met and is proven by an HTTP probe** (§5 T5.1), not by a picker.

**Which gate pins it.** A `tools/play-server/src/main.rs` *source* gate is the wrong
instrument here — there is no frontend change to guard. The pin is a real HTTP probe driving
`POST /api/game/action` with the **non-default** choice (`Some(troll)`, not the decline), on
a fixture built through `session::new_game` with `DeckSource::Fixed` (the `ui6_install`
recipe, `main.rs:3981-4000`). The Troll reaches the graveyard **by legal means**: cast it
with an empty graveyard, it enters as a 0/0 and dies to SBAs (CR 704.5f) — already proven by
`crates/engine/tests/mechanics_e_l/golgari_grave_troll.rs::test_golgari_grave_troll_empty_graveyard_dies_to_sba`
(`:287`). Budget note for the runner: {4}{G} needs five lands, so the drive is longer than
`ui1_drive_to_question`'s; if the drive budget proves impractical, **stop and report** rather
than falling back to a state poke — a `mono_green` fixture with cheap ramp is the first
alternative to try.

### Q7 — Wire

**Prediction: PROTOCOL 35 unmoved, HASH 73 unmoved.** Reasons: `Command::ChooseDredge`
(`crates/engine/src/rules/command.rs`) and `GameEvent::DredgeChoiceRequired`
(`events.rs:903-906`) already exist and gain no field; `PendingDraw` gains no field; the new
`offer_dredge` / `tail_offers_dredge` parameters are function arguments, not state;
`LegalAction` lives in `crates/simulator` and is not in the SR-8 wire closure;
`rules::queries::dredge_options` is a read-only query.

**These must be EXECUTED, not predicted** (name both, both are in the `core` test target):

```
cargo test -p mtg-engine --test core hash_schema
cargo test -p mtg-engine --test core protocol_schema
```

Key sub-tests: `hash_schema::hash_schema_version_sentinel`
(`crates/engine/tests/core/hash_schema.rs:1249-1252`, pinned at **73**) and
`protocol_schema::protocol_schema_fingerprint_is_pinned`
(`crates/engine/tests/core/protocol_schema.rs:850`).

---

## §4. Staged implementation order

### Stage 0 — observations and baseline (no source edits)

1. `cargo test --workspace --no-fail-fast` **to a file** (never `| tail` — the 2026-08-02
   lesson): expect **4,398 / 0 / 5**, residual list empty. Record the actual number.
2. `cargo test -p mtg-engine --test core hash_schema --test core protocol_schema` — record
   HASH **73** / PROTOCOL **35**.
3. `cargo test -p play-server` — record the pin (expect **78 / 0**).
4. Write the Q5 probe fixture **first**, run it against unmodified source, and **record the
   pre-fix values of A1/A2/A3 literally** into `memory/primitives/pb-DX23-execution-notes.md`.
   This is the batch's before-picture and the only way the revert-watch is a discriminator
   rather than a tautology.
5. `grep -rn "ChooseDredge" crates/simulator/src/ tools/` → confirm **0** hits at HEAD.
6. `grep -rn "ReplacementTrigger::WouldDraw" crates/card-defs/src/defs/` → confirm **0**
   (the zero-corpus-reach premise behind Q2 and §1.4).
7. Run the golden-script corpus and record the baseline, because
   `test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json` exercises
   `ChooseDredge` through `replay_harness.rs`.

### Stage 1 — engine: the shared query (behaviour-NEUTRAL)

* **New**: `crates/engine/src/rules/queries.rs::dredge_options(state, player) ->
  Vec<(ObjectId, u32)>` — CR 702.52a/b, sorted by `ObjectId`.
* **Rewire**: `replacement.rs:699-733`'s inline scan **calls** it. The returned `DrawAction`
  must be byte-identical.
* Gate: `cargo test -p mtg-engine --test mechanics_a_d dredge` and `--test primitives
  pb_dx2_command_gates` both green with no test edited. Full-suite count unmoved apart from
  the new probes.

### Stage 2 — engine riders: `OOS-DX2-2` (the tail flip) + `OOS-DX2-3` comment corrections

* `perform_remaining_draws` (`replacement.rs:1563`) gains `offer_dredge: bool`; the `false`
  at `:1575` becomes that parameter. **Rewrite its doc at `:1544-1562`** — the "a
  pre-existing simplification this batch deliberately does not change" sentence is now false.
* `resolve_declined_pending_draw` (`:1122`) gains `tail_offers_dredge: bool`; passes it at
  `:1145-1150`. Its own `perform_one_draw` call at `:1127-1135` keeps `false` **and the
  comment at `:1130-1131` must say why in the new two-axis vocabulary** (same draw vs.
  different draw).
* Callers: `perform_one_draw:949` → `false`; `handle_choose_dredge:3309` → `true`;
  `handle_choose_dredge:3395` → `true`; `resolve_pending_draw:1697` → `true`.
* **Correct the two push-site comments** at `:959-962` and `:983-984` — drop the retracted
  "always into an EMPTY slot / the discharge above guarantees it" claim and point at
  `perform_one_draw`'s own corrected "Per-player invariant" doc (`:853-904`) and
  `OOS-DX2-3`.
* Also update `memory/gotchas-rules.md:43-45` ("Declining re-checks other `WouldDraw`
  replacements but does not re-offer dredge for THAT SAME draw") — still true, but it now
  needs the tail sentence beside it.
* Gate: `pb_dx2_command_gates.rs:1272` must still be **green and unedited**.

### Stage 3 — simulator: the variant, the provider, `params.rs`

* `legal_actions.rs`: new `LegalAction::ChooseDredge { card: Option<ObjectId>, mill: u32 }`
  with a doc naming CR 702.52a, the Q1 placement argument and the Q2 suppression rule.
* `StubProvider::legal_actions`: the emission block, appended after the `PayRecover` loop
  (`:604-618`). Guarded on `state.pending_draws().iter().any(|p| p.player == player)` **and**
  a non-empty `queries::dredge_options(state, player)`.
* `params.rs`: the arm. Stays outside the `:271-286` allowlist.
* Gate: `cargo build --workspace` — the compiler now points at every exhaustive site (§4
  Stage 5 table).

### Stage 4 — bot policy

* `heuristic_bot.rs::score_action`: the two arms from Q4; `_player` → `player`.
* Gate: `cargo test -p mtg-simulator` and the seeded fixtures; record every seed that moves.

### Stage 5 — the exhaustive-match sweep

Compile-forced sites (let `cargo build --workspace` confirm; this table is so the runner does
not discover them one at a time):

| file | match | line | action |
|---|---|---|---|
| `crates/simulator/src/params.rs` | `action_to_command_with_params`'s `match action` | `:287` | new arm → `Command::ChooseDredge` |
| `crates/simulator/src/heuristic_bot.rs` | `score_action`'s `match action` | `:192` | two arms (Q4) |
| `tools/play-server/src/view.rs` | `action_kind` | `:1204` | `"ChooseDredge"` |
| `tools/play-server/src/view.rs` | `action_object` | `:1235` | `*card` |
| `tools/play-server/src/view.rs` | `action_label` | `:1270` | Q6 label |

**Not** compile-forced, verified by reading, listed so nobody adds a redundant arm:
`view.rs::action_needs_x` (`:1370`), `action_modes` (`:1407`), `action_target_requirements`
(`:1458`), `target_query_source` (`:1478`), the combat-options match (`:1536`),
`blocking_decision_view` (`:2064`, `_ => None` at `:2267`), `api.rs::validate_combat_params`
(`_ => Ok(())` at `:436`), `api.rs::validate_decision_params`, and everything under
`tools/tui/` (which uses `matches!`/`if let`, never an exhaustive `match` — so **the TUI gets
no dredge channel**; record as `OOS-DX23-3`, the `OOS-UI2-5`/`OOS-DX6-5` family).
`crates/view-model` does not depend on `crates/simulator` and is untouched.

### Stage 6 — docs, audit row, card-def comment

* `docs/audits/decision-point-audit.md` §3.1: add the `OOS-DX2-7` AUTO-CHOSEN row (§5 T6.1
  spells out the content). Cross-reference §3.2 ("decisions that need no card at all"), which
  is where it structurally belongs, and say so.
* `crates/card-defs/src/defs/golgari_grave_troll.rs:30-31` — the comment says "Engine
  machinery already exists (rules/replacement.rs `DredgeAvailable` + Command::ChooseDredge)",
  which was true and misleading: the machinery existed and **nothing could reach it**. Extend
  it to name the answer channel. **Comment only — zero DSL lines**, so coverage stays
  **1,133/1,803 = 62.8%**, proven by regenerating `tools/authoring-report.py` to a
  byte-identical body.
* `tools/play-server/README.md` — the human dredge channel in the limitations/routes table.
* `crates/engine/src/rules/events.rs:873-875` — `DredgeChoiceRequired`'s doc gives as one of
  its two reasons for being a deadline that *"`crates/simulator` constructs no `ChooseDredge`
  at all, so blocking would deadlock every bot game"*. **That reason is now false and must be
  struck**; the FIRST reason (CR 702.52a is "you may instead") stands alone and is the real
  one.

### Stage 7 — tests, gates, revert-watch

Per §5. Every new gate proven red by **executing** its revert with the rebuild confirmed.

---

## §5. Test inventory

New file: **`crates/simulator/tests/pb_dx23_dredge_answer_channel.rs`** (the batch's home;
`crates/simulator/tests/*.rs` are per-file targets there, unlike SR-9a's engine grouping).
Engine-side probes append to the EXISTING grouped target
**`crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs`** with its `mod` line
added to `crates/engine/tests/primitives.rs` (SR-9a: never add a top-level `tests/*.rs`; a
dropped `mod` line silently deletes coverage).

### T1 — the mandatory probe (acceptance criterion 2)

| id | test | CR | revert to watch red |
|---|---|---|---|
| T1.1 | `test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence` (simulator) | CR 702.52a, 121.1, 103.8a | delete the `ChooseDredge` push block from `StubProvider::legal_actions` — A1 and A2 both redden |

Shape and "no state pokes" definition per §3 Q5. Includes A3 non-vacuity.

### T2 — the shared query (Q2)

| id | test | CR | revert |
|---|---|---|---|
| T2.1 | `test_dx23_dredge_options_matches_cr_702_52a_eligibility` | 702.52a | make `dredge_options` ignore the graveyard-zone filter — a battlefield dredge card appears |
| T2.2 | `test_dx23_dredge_options_respects_the_library_floor` | 702.52b | change `<=` to `<` in the library comparison — the exact-count card disappears (mirrors `dredge.rs::test_dredge_exact_library_count_is_eligible`, `:586`) |
| T2.3 | `test_dx23_offer_and_engine_scan_are_one_derivation` | 702.52a | inline a second scan in `check_would_draw_replacement` that drops the sort — the two disagree on order. **Consistency, not correctness** — that is why T2.1/T2.2 exist separately (the PB-DX20 lesson) |

### T3 — the tail flip (acceptance criterion 3, `OOS-DX2-2`)

| id | test | CR | revert |
|---|---|---|---|
| T3.1 | `test_dx23_tail_of_an_answered_multi_draw_offers_dredge_again` | 121.2, 614.11a, 121.6b | restore `false` at `perform_remaining_draws`' `perform_one_draw` call — the second `DredgeChoiceRequired` vanishes |
| T3.2 | `test_dx23_declining_does_not_reoffer_for_the_same_draw` | 702.52a, 616.1f | pass `true` at `replacement.rs:1130` — a re-offer appears on the SAME draw. Guards the boundary `dredge.rs` test 10 (`:931`) also guards, from the other side |
| T3.3 | `test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry` | 702.52a, 614.11a | pass `true` from `perform_one_draw:949` — `pending_draws().len()` becomes **2** (the exact §3 Q3 trace). **This is the `OOS-DX2-3` guard and the reason the flag is threaded** |
| T3.4 | `test_dx23_remaining_bookkeeping_survives_a_tail_deferral` | 614.11a, 121.2 | hard-code `remaining_after: 0` at `replacement.rs:1571` — the resumed tail loses draws |

### T4 — provider and bot (acceptance criterion 1, bot half)

| id | test | CR | revert |
|---|---|---|---|
| T4.1 | `test_dx23_provider_offers_decline_plus_one_per_eligible_card` | 702.52a/b | drop the `None` push — the decline disappears |
| T4.2 | `test_dx23_provider_offers_nothing_when_no_dredge_card_is_eligible` | 702.52a, 616.1e | remove the `dredge_options(..).is_empty()` guard — a `NeedsChoice`-origin entry gets a bare decline offer (the Q2 loop) |
| T4.3 | `test_dx23_provider_is_silent_while_a_blocking_decision_stands` | 514.1 (admission gate, `engine.rs:304-314`) | move the emission block ABOVE the `blocking_decision()` early return at `legal_actions.rs:420-487` — the offer appears alongside a cleanup discard, i.e. an action the engine would reject (SR-38) |
| T4.4 | `test_dx23_every_offered_action_is_engine_accepted` | SR-38 | offer a `Some(id)` for a graveyard card without the keyword — `process_command` returns `Err` |
| T4.5 | `test_dx23_heuristic_bot_declines_rather_than_milling_itself_out` | 702.52b, 104.3c | drop the `2 * mill` margin — the bot dredges at a library below the margin |

**T4.4 hazard, stated because four PB-DX21 probes got this wrong**: `process_command`'s
`Err` arm carries **no `GameState`**, so "the rejection mutated nothing" is structurally
vacuous through it. Assert on the `Ok`/`Err` discriminant and on a state obtained from a
separate accepted call — never on a state the failing call was supposed to return.

### T5 — the human channel (acceptance criterion 1, human half)

| id | test | CR | revert |
|---|---|---|---|
| T5.1 | `test_dx23_browser_can_answer_a_dredge_offer` (`tools/play-server/src/main.rs` test module) | 702.52a, 400.1 | drop the `action_kind` arm → the option renders as a compile error, so instead: drop the provider push and watch the drive find no `ChooseDredge` option |

Must use a **non-default** answer (`Some(troll)`, not the decline) so game state
distinguishes the human's choice from any fallback — the UI-4/SIM-6 standard. Assert the
`Dredged` event and the Troll in hand, and assert the option carries **no** `decision` key
(it is not a blocking decision — this is the pin on the Q6 divergence).

### T6 — record-keeping (acceptance criteria 4 and 5)

| id | test / artefact | what |
|---|---|---|
| T6.1 | `docs/audits/decision-point-audit.md` §3.1 row | `OOS-DX2-7`: "stale `PendingDraw` auto-discharge (CR 702.52a / 614.11a)" — **AUTO-CHOSEN**, `Complete` defs = **1** (`golgari_grave_troll`), marked NON-DSL and invisible to `decision_gate` by construction, a fresh `OOS-DP10-9` instance. Verify `core::decision_gate::named_residual_seed_ids_still_exist_in_the_audit` (`decision_gate.rs:1435-1462`) stays green — it reads this file as text |
| T6.2 | `pb_dx2_command_gates.rs:1272` | unedited and green. **Do not re-close `OOS-DX2-3`.** T3.3 pins the invariant that *is* preserved; the reopened seed stays reopened |
| T6.3 | `replacement.rs:959-962` and `:983-984` | corrected; no test, verified by reading |

### Full-suite gates (all EXECUTED, output captured to a file)

```
cargo build --workspace
cargo test --workspace --no-fail-fast            # to a file; residual list must be empty
cargo test -p mtg-engine --test core hash_schema        # HASH 73 unmoved
cargo test -p mtg-engine --test core protocol_schema    # PROTOCOL 35 unmoved
cargo test -p play-server                                # 78/0 pin
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
tools/check-defs-fmt.sh                                  # SR-35
python3 tools/authoring-report.py                        # body byte-identical; 62.8% unmoved
```

Plus the golden-script corpus (Stage 0 step 7's baseline re-run), because
`replacement/014_golgari_grave_troll_dredge.json` drives `ChooseDredge`.

---

## §6. Risk and blast radius

**R1 — recorded seeds move, and these are the ratchets that can redden.** Any fuzz-shaped or
`build_initial_state` fixture whose deal contains `golgari_grave_troll` shifts its RNG stream
from the first dredge offer onward. Named, with their pinned values:

| gate | file | pin |
|---|---|---|
| `test_dx32_sr38_bot_rejection_rate_is_ratcheted` (T2.2) | `crates/simulator/tests/pb_dx32_fuzz_output.rs` | `MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG = 40` |
| `test_dx32_random_bot_waste_ratio_is_bounded` (T3.1) | same | `MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG = 95`, floor `total_taps >= 77` |
| `test_dx32_orphaned_tokens_are_transient…` (T4.1) / `…distinct_collapses…` (T4.3) | same | seed-2 transient counts (4 raw) |
| `test_dx32_a_fuzz_run_reaches_at_least_one_served_row` (T6.3) | same | the reached partition is asserted **EXACTLY** |
| `heuristic_pools_emptied_is_pinned` (T3.3) | `crates/simulator/tests/sim5_bot_cast_discipline.rs` | `MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED = 1` |
| S8 scripted playthrough | `crates/simulator/tests/local_game_playthrough.rs` | five seeds, "a rejection means the offer was wrong" |

**Rule for the runner: a moved pin is a FINDING first and a re-pin second.** Diagnose why it
moved (did a dredge offer actually appear in that seed's deal?) before touching a number, and
record the diagnosis. The play-server's `DeckSource::Fixed` pins must **not** move at all.

**R2 — `OOS-DX2-3` widening.** The single largest correctness risk in the batch, fully
traced in §3 Q3 and guarded by T3.3. If the runner takes the unconditional-flip alternative
instead, it owes a test pinning the two-entry state, an `OOS-DX2-3` row update, and an
explicit note in the handoff.

**R3 — the decline-re-defer loop.** Guarded structurally by Q2's suppression rule and by
T4.2. If a future batch adds `LegalAction::OrderReplacements`, this guard must be revisited
together with it.

**R4 — a bot now spends a priority action answering a dredge offer.** `advance()`'s
`consecutive_passes` counter is not incremented for a non-pass command, so an offer answered
every turn extends games slightly and consumes `max_commands`. Bounded (§3 Q3's termination
argument), but the fuzz `--max-turns`/`max_commands` headroom should be re-measured, not
assumed.

**R5 — `reset_loop_detection` on every `ChooseDredge`.** `engine.rs:556-557` resets CR 104.4b
loop detection for every dredge answer. A bot answering an offer every turn therefore resets
it every turn. This is pre-existing and CR-correct (dredge IS a meaningful choice), but it is
now *reachable*, so a CR 726 mandatory-loop fixture involving draws could behave differently.
Watch the golden corpus.

**R6 — `queries.rs` is a public API surface.** `dredge_options` becomes callable from
`tools/`. Its doc must state that it is advisory (the module header at `queries.rs:1-16`
already says so) and that the engine re-validates at `handle_choose_dredge`.

**R7 — Architecture Invariant 7.** No new disclosure. The offer names GRAVEYARD objects
(public, CR 400.1) and the `mill` count, which is printed on the card. The **library** is
touched only by the mill that follows an accepted answer, and no library id crosses the wire.
`view.rs`'s label path is `NameIndex` (`action_label`'s `card` closure), not
`question_card_label`, and not `library_look_cards` — so the UI-6 raw-read gate
(`test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`) must stay at its
pinned needle set. **If it moves, stop**: that gate has already caught this class twice
(MR-M11-01, then UI-6).

**R8 — `#[non_exhaustive]` is NOT on `LegalAction`**, so every exhaustive match is a compile
error until arm'd. That is the design. The **danger** is the four sites that are *not*
compile-forced (§4 Stage 5's second list) — in particular `blocking_decision_view`'s
`_ => None`, which will silently do the right thing here and would silently do the *wrong*
thing for a future variant that IS a blocking decision.

**R9 — clippy.** The new `perform_remaining_draws` / `resolve_declined_pending_draw`
signatures push both toward `clippy::too_many_arguments`; `perform_one_draw` already carries
six. Follow the existing `#[allow(clippy::too_many_arguments)]` precedent
(`local_game.rs:432`) with a stated reason rather than bundling parameters into a struct that
nothing else needs.

---

## §7. Seeds to file

| id | statement |
|---|---|
| `OOS-DX2-5` | **CLOSE** — both clients can answer. |
| `OOS-DX2-2` | **CLOSE** — the tail of an interrupted sequence is dredge-offerable at the two same-draw-exempt sites' expense; CR 121.2 / 614.11a / 121.6b cited in the commit, with the PB-DP5 §3.3 distinction stated. |
| `OOS-DX2-7` | **RECORD, not close** — the auto-discharge is now an AUTO-CHOSEN row in `docs/audits/decision-point-audit.md` §3.1, a fresh `OOS-DP10-9` instance. It is still an engine-made decision. |
| `OOS-DX2-3` | **STAYS REOPENED.** Not re-closed, and not re-closed on a structural argument. T3.3 pins the narrower invariant that IS preserved; the two push-site comments are corrected. |
| `OOS-DP5-2` | unchanged — still no deadline on `pending_draws`; this batch makes the offer *answerable*, not *bounded*. |
| **`OOS-DX23-1`** (new) | A `PendingDraw` for a player who does not currently hold priority is not surfaced until they next receive it (`legal_actions.rs:508-510` + `advance()`'s single-acting-seat resolution). CR 117.3d makes that a deferral, never a loss — but it means an offer's *moment* is the engine's, not the player's. |
| **`OOS-DX23-2`** (new) | A `NeedsChoice`-origin `PendingDraw` remains unanswerable through any simulator channel: there is no `LegalAction::OrderReplacements`, and Q2 deliberately withholds the bare decline to avoid the re-defer loop. Zero corpus reach (0 `ReplacementTrigger::WouldDraw` defs). |
| **`OOS-DX23-3`** (new) | The TUI gets no dredge channel — `tools/tui/src/play/input.rs` hand-builds commands via `matches!`/`if let` and never routes through `params.rs`. Same family as `OOS-UI2-5` + `OOS-DX6-5`, which are already merged into PB-DX33; this is a third member. |
| **`OOS-DX23-4`** (new) | `HeuristicBot`'s dredge policy is a 2× library margin with no board evaluation — it never dredges *for value* (to fill a graveyard for a Golgari deck) and never declines *strategically*. Bot play quality, not correctness. |
| **`OOS-DX23-5`** (new, conditional) | File only if a `pb_dx32_fuzz_output.rs` ratchet or a `local_game_playthrough.rs` seed moves: record which, by how much, and the diagnosed mechanism (R1). |

---

## §8. Verification checklist

- [ ] Stage 0 baseline measured on this branch **before any edit** and recorded (4,398 / 0 / 5,
      PROTOCOL 35, HASH 73, play-server 78/0, golden corpus, pre-fix A1/A2/A3 literals)
- [ ] TODO sweep recorded: **0 cards added**, `thrasios_triton_hero` explicitly excluded
- [ ] `rules::queries::dredge_options` exists and `check_would_draw_replacement` calls it
- [ ] `LegalAction::ChooseDredge { card, mill }` emitted, mapped, scored, labelled
- [ ] Q2 suppression rule implemented (no offer when nothing is eligible)
- [ ] Tail flip: `perform_remaining_draws` parameterised; `:1130` and `:1671` unchanged;
      `perform_one_draw:949` passes `false`
- [ ] Commit message states why PB-DP5 §3.3 does not extend to the tail (criterion 3)
- [ ] `OOS-DX2-7` row added to `decision-point-audit.md` §3.1 (criterion 4)
- [ ] `pb_dx2_command_gates.rs:1272` green and **unedited**; both push-site comments
      corrected; `OOS-DX2-3` **not** re-closed (criterion 5)
- [ ] `events.rs:873-875`'s now-false second reason struck
- [ ] Every new test cites its CR (Architecture Invariant 8)
- [ ] Every new gate proven red by an **executed** revert with the rebuild confirmed
- [ ] HASH 73 / PROTOCOL 35 **gate-executed** and unmoved (criterion 6)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` clean (SR-35)
- [ ] Coverage regenerated byte-identical: **1,133/1,803 = 62.8%**
- [ ] `git diff main..HEAD --numstat -- crates/card-defs/` is **comment-only**
- [ ] Execution notes written to `memory/primitives/pb-DX23-execution-notes.md` (revert
      matrix, every measurement, every moved pin with its diagnosis)
