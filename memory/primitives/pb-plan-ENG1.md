# Primitive Batch Plan: ENG-1 — effect-driven discard is a real player choice

**Generated**: 2026-08-02
**Task**: `scutemob-191`
**Primitive**: a fourth `EffectChoiceQuestion` / `EffectChoiceAnswer` variant — `Discard` —
plus the deferral branch in `Effect::DiscardCards` that raises it. The first HAND-zone
question in the CR 608.2d machinery.
**CR Rules**: 701.9 (701.9a/b/c), 608.2d, 601.2c, 702.35a, 514.1, 605.1b/605.4a, 400.7,
608.2f / 404.3, 121.3
**Cards affected**: 13 `Complete` defs become CR 701.9b-correct in a human game
(0 card-def edits — every def is correct and innocent; the defect is 100% engine-side)
**Dependencies**: PB-DP9 (the `EffectChoice` machinery), PB-DP7 (the `Subset` picker shape),
UI-1 (`BlockingDecisionView`), UI-4 (`plainClone`)
**Deferred items from prior PBs**: `OOS-UI5-4` (no frontend harness — the tier-1 recipe in
`memory/workstream-state.md`'s UI-5 handoff applies to this batch's `DiscardPicker` change and
should be used rather than re-invented)

> **Do not implement anything from this file until the premise re-verification in §0 has been
> executed on the branch.** Every line number below was read off HEAD on 2026-08-02 in the
> `scutemob-191` worktree, but the PB-DX2 lesson stands: cite by symbol, verify by grep.

---

## 0. Premise — re-verified against HEAD, with the drift the dispatch brief carried

Every claim in the dispatch brief was checked against
`/home/skydude/projects/scutemob/.worktrees/scutemob-191`. **All of it holds**; four line
citations had drifted by 1–5 lines and one framing claim is materially wrong. Corrections:

| Brief said | HEAD says | Impact |
|---|---|---|
| `discard_cards` at `:9387` | fn at `:9387`, doc at `:9380` | ✅ correct (the *triage doc*'s `:9368-9376` is the stale one) |
| `AnswerShapeView` `:406-470` | doc `:401-407`, enum `:408-475` | cosmetic |
| `blocking_decision_view` `:1871` | signature at `:1870` | cosmetic |
| `question_cards` `:1852` | doc `:1851`, fn `:1853` | cosmetic |
| `HashInto for EffectChoiceAnswer` `:3230-3247` | `:3230-3249` | cosmetic |
| *"the short-circuit is what keeps `WheelHand` from ever suspending"* | **FALSE under this plan's placement.** See §3.3 | **material** — a whole paragraph of the brief's reasoning is replaced |

Verified exactly as briefed (no drift): `effects/mod.rs:1202-1208` (the subject arm),
`:3606`/`:3695`/`:3771` (the three ask sites), `:327`/`:352`/`:371`/`:386` (the four default
helpers), `:477` (`ask_or_consume_effect_choice`), `:584`
(`MAX_EFFECT_CHOICES_PER_RESOLUTION`), `:593` + `:622-634` + `:642-677`
(`handle_answer_effect_choice` and its checks 4 and 5), `:733` (`validate_partition`), `:779`
(`execute_effect_answering`), the four `discard_cards` callers at `:1206`/`:1236`/`:1266`/`:9276`,
`stubs.rs:902-904`/`:906-922`/`:929-948`/`:962`/`:988`, `hash.rs:646-660`/`:1105`/`:3208`,
`replay_harness.rs:1117-1148`, `events.rs:1497-1507`, `protocol.rs:302-314`/`:601-607`,
`legal_actions.rs:338-343`/`:464-480`, `params.rs:102`/`:280`/`:682-691`, `view.rs:1843`/`:1853`/
`:1870`/`:1881-1899`/`:1903-1968`, `api.rs:512-560`/`:885-892`, `tui/src/play/app.rs:639-648`.

### 0.1 The four things the brief did NOT know about, all load-bearing

1. **`crates/engine/tests/core/decision_gate.rs` will go red in four places** and needs a
   deliberate, arithmetic-exact edit. It carries a frozen `BASELINE` of
   `(def name, AutoChosen row set, reason)` triples and a hard-equality ratchet
   `MAX_AUTO_CHOSEN_COMPLETE_UNION = 91` (`:503`). **13 `BASELINE` entries name
   `"discard_cards"`** (`:393`, `:399`, `:401`, `:415`, `:417`, `:421`, `:422`, `:426`, `:433`,
   `:454`, `:457`, `:467`). Flipping the `discard_cards` row from `AutoChosen` to `Served` makes
   12 of them empty (delete) and shrinks one to a single row (`Izzet Charm`, `:431-435`, keeps
   `counter_unless_pays`). **This is the batch's yield measurement** — see §7.
2. **`decision_site_walk.rs:317-326`** is where the `discard_cards` row is classified, and its
   `DecisionClass::AutoChosen { why_not_flagged_is_wrong: "CR 701.9b: the affected player chooses
   which card, by default; the engine picks the lowest ObjectId" }` is a *verbatim statement of
   this defect*, sitting green in the suite since 2026-07-27. **The audit found it and the
   classification recorded it as expected.** That is the corpus-scale form of the same
   comment-debt failure §10 is about.
3. **A live arithmetic defect in the arm being edited.** `effects/mod.rs:1203` is
   `let n = resolve_amount(state, count, ctx) as usize;` — **no `.max(0)`**, unlike the
   `Effect::DrawCards` arm two lines above (`:1195`, `MR-M7-05`). A negative amount wraps to
   ~1.8e19 and `discard_cards`' `for _ in 0..n` loop has **no break on an empty hand**, so it
   scans `state.objects` ~1.8e19 times: an effective hang, in release, from a legal
   `EffectAmount`. Fixed in this batch because the short-circuit's arithmetic depends on `n`
   being sane. `Effect::MillCards` (`:1305`) is the only sibling with the same omission — seeded,
   not fixed (§9).
4. **The golden corpus does not need editing.** `replay_harness.rs:397-409`'s
   `auto_answer_blocking_decisions` pump already answers *any* `BlockingDecision::EffectChoice`
   with `default_effect_choice_answer`. Since this batch's default reproduces the pre-batch pick
   byte-for-byte (§6), `stack/016_pull_from_tomorrow_x_draw.json` and
   `stack/083_fiery_temper_madness_cast.json` — the only two approved scripts that reach an
   effect-driven discard — should pass **unchanged**. Do not pre-emptively edit them; run the
   corpus and report. (If either reddens, that is a *finding*, not a chore.)

---

## 1. CR rule text (MCP-verified this session, verbatim)

**701.9. Discard**

- **701.9a** — "To discard a card, move it from its owner's hand to that player's graveyard."
- **701.9b** — "By default, effects that cause a player to discard a card allow the affected
  player to choose which card to discard. Some effects, however, require a random discard or
  allow another player to choose which card is discarded."
- **701.9c** — "If a card is discarded, but an effect causes it to be put into a hidden zone
  instead of into its owner's graveyard without being revealed, all values of that card's
  characteristics are considered to be undefined. If a card is discarded this way to pay a cost
  that specifies a characteristic about the discarded card, that cost payment is illegal; the
  game returns to the moment before the cost was paid (see rule 732)."

**608.2d** — "If an effect of a spell or ability offers any choices other than choices already
made as part of casting the spell, activating the ability, or otherwise putting the spell or
ability on the stack, the player announces these while applying the effect. The player can't
choose an option that's illegal or impossible, with the exception that having a library with no
cards in it doesn't make drawing a card an impossible action (see rule 121.3). …"

**601.2c** (the "determined announcement" principle the search arm already leans on at
`effects/mod.rs:3599-3604`) — "The player announces their choice of an appropriate object or
player for each target the spell requires. … Once the number of targets the spell has is
determined, that number doesn't change …"

**702.35a Madness** — "'Madness [cost]' means 'If a player would discard this card, that player
discards it, but exiles it instead of putting it into their graveyard' and 'When this card is
exiled this way, its owner may cast it by paying [cost] rather than paying its mana cost. If that
player doesn't, they put this card into their graveyard.'"

**514.1** — "First, if the active player's hand contains more cards than their maximum hand size
(normally seven), they discard enough cards to reduce their hand size to that number. This
turn-based action doesn't use the stack."

### 1.1 What the CR requires the engine to do, stated as an obligation

CR 701.9b makes "which card" a **choice belonging to the affected player**, defaulted rather than
optional: the engine may only take it away where the printed effect says "at random" or "an
opponent chooses". **No `Complete` def in the corpus says either** (verified: `gamble.rs` prints
"at random" but is a blocked TODO def carrying no `Effect::DiscardCards`; `grief.rs` prints
"you choose a nonland card" and is likewise blocked). So the CR default covers the **entire live
corpus**, and a `chooser` field is not merely deferrable — there is nothing today it could
express (§9).

CR 608.2d places the announcement **while applying the effect**, which is exactly where PB-DP9's
suspend-and-replay wrapper lives. Nothing new is needed at the rules level.

**Edge case the CR forces us to keep**: CR 702.35a means the chosen card may go to **exile**, not
the graveyard. The answer therefore names *what was chosen*, never *where it goes* — see §2.2.

**Edge case CR 701.9c does NOT reach**: it governs a discard into a hidden zone as a *cost*
payment. `Cost::DiscardCard` (`effects/mod.rs:9276`) is out of scope (§9); note that 701.9c is
the reason a cost discard is a harder problem than a resolution discard, not an easier one.

---

## 2. Primitive specification

### 2.1 `EffectChoiceQuestion::Discard` — exact field shape

**File**: `crates/card-types/src/state/stubs.rs`, appended after
`EffectChoiceQuestion::Surveil` (`:921`).

```rust
    /// CR 701.9b: the affected player's whole hand, in ascending `ObjectId`
    /// order, and how many cards they must choose.
    ///
    /// `hand` is the FULL hand, not a pre-trimmed subset: CR 701.9b puts no
    /// restriction on which card may be chosen, so the whole hand IS the legal
    /// answer space. Named `hand` (not `candidates`) to match
    /// `GameEvent::CleanupDiscardChoiceRequired.hand` and
    /// `LegalAction::DiscardToHandSize.hand` -- the two discard channels in this
    /// engine should use one vocabulary, and a reader who knows one should not
    /// have to learn the other.
    ///
    /// `count` is `u32` to match `PendingCleanupDiscard.count` and
    /// `BlockingDecision::CleanupDiscard { count }`, for the same reason.
    ///
    /// Ascending order is REQUIRED, not incidental: the replay's
    /// question-equality check compares this value structurally, and
    /// `default_discard_answer` recovers the pre-batch auto-pick by taking the
    /// first `count` entries.
    Discard {
        hand: Vec<ObjectId>,
        count: u32,
    },
```

**Why the full legal answer space travels on the question**: the type's own doc (`:894-896`)
makes this the contract — "carrying its full legal answer space so a client can render a picker
without a second query". A `Discard` question that carried only `count` would force the client
to re-derive the hand, and `view.rs` would then be a second place that decides what a hand *is*.

### 2.2 `EffectChoiceAnswer::Discard` — exact field shape, and why `chosen`

**File**: same, appended after `EffectChoiceAnswer::Surveil` (`:947`).

```rust
    /// CR 701.9b: the cards the affected player chose. Exactly the question's
    /// `count` of them, no duplicates, every one drawn from the question's
    /// `hand`.
    ///
    /// The ORDER is meaningful and is the player's to choose: it is the order
    /// the cards are discarded, and therefore (CR 608.2f / CR 404.3) the
    /// relative order they enter the graveyard.
    ///
    /// **Named `chosen`, not `discarded`.** The three sibling answers name a
    /// DESTINATION (`found`, `bottom`/`top`, `graveyard`/`top`) because those
    /// questions are about where cards go. This one is not a partition -- the
    /// unchosen cards stay in hand and the effect never touches them -- so a
    /// destination name would overstate what the answer says. Worse, it would
    /// be WRONG: CR 702.35a sends a chosen Madness card to EXILE, so at answer
    /// time nothing has been discarded and the destination is not yet known.
    /// `chosen` names the act CR 701.9b actually gives the player.
    Discard { chosen: Vec<ObjectId> },
```

**Decision recorded**: `chosen: Vec<ObjectId>`. Not `Vec<ObjectId>` under the name `discarded`
(wrong per CR 702.35a, above); not a `BTreeSet`/`OrdSet` (the order is a real payload, CR 404.3);
not `Option<Vec<..>>` (there is no "decline" — CR 701.9b offers no fail-to-discard, unlike CR
701.23b's fail-to-find; a player with fewer cards than `count` is handled by the short-circuit,
not by a decline).

### 2.3 The type doc at `stubs.rs:902-904` — exact rewrite

The current sentence is **falsified** by this batch:

> **Hidden information (Architecture Invariant 7).** Every `ObjectId` in every
> variant names a card in a HIDDEN zone -- the library. That is why
> `GameEvent::EffectChoiceRequired::private_to()` returns `Some(player)`.

Replace with (the conclusion is unchanged; the premise is widened and the *new* premise is named
so a reviewer can check it):

```rust
/// **Hidden information (Architecture Invariant 7).** Every `ObjectId` in every
/// variant names a card in a HIDDEN zone, and the recipient is entitled to see
/// every one of them -- but for two DIFFERENT reasons, and the second one is
/// newer and weaker, so it is stated rather than folded into the first:
///
/// * `SearchLibrary` / `Scry` / `Surveil` name cards in the answerer's LIBRARY.
///   The effect itself is what grants the look (CR 701.23a / 701.22a / 701.25a):
///   the player is entitled to see these ids only because this effect is
///   resolving, and only for as long as it is.
/// * `Discard` (ENG-1, CR 701.9b) names cards in the answerer's own HAND. The
///   entitlement is not granted by the effect at all -- the player already holds
///   those cards. CR 701.9b names "the affected player" as the chooser, and
///   `PendingEffectChoice.player` IS that player, so the ids are only ever sent
///   to the seat that already has them.
///
/// Either way the answerer may see the whole question, which is why
/// `GameEvent::EffectChoiceRequired::private_to()` returns `Some(player)` --
/// unchanged by ENG-1.
///
/// **The premise the hand variant rests on, named so it can be checked.** "The
/// answerer owns the cards" is only true because `entry.player` is enforced in
/// three independent places: `process_command`'s admission gate,
/// `effects::handle_answer_effect_choice` check 2 (the SR-29 trust boundary),
/// and -- on the read side -- the play-server guard pinned by
/// `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`. If any
/// of those three is ever relaxed, this variant leaks a hand and the library
/// variants leak a look; the hand is the one a human will notice.
```

Also update **`crates/engine/src/rules/events.rs:1487-1494`**, whose doc on
`GameEvent::EffectChoiceRequired` asserts "the library candidates that matched a search filter
(CR 401.2) or the top N a player is looking at" — add the hand case with the same two-premise
split.

### 2.4 Placement — the ask goes in the `Effect::DiscardCards` ARM, not in `discard_cards`

**This is the plan's single most load-bearing decision, and it is the one the dispatch brief got
wrong.** The brief reasoned as if the ask lived inside the `discard_cards` helper (hence its
claim that the short-circuit is what protects `Effect::WheelHand`).

Put the ask in the **arm** (`effects/mod.rs:1202-1208`). Consequences, all of them the ones we
want:

* **`Effect::WheelHand` (`:1236`, `:1266`) cannot suspend, by construction** — it calls the
  helper directly and the helper never asks. Not "because of the short-circuit". §3.3.
* **`Cost::DiscardCard` (`:9276`) cannot suspend, by construction.** This is not a nicety: that
  call is inside `pay_optional_cost`, on a **cost-payment path with no resolution wrapper to roll
  back to**. An ask there would record a `pending_effect_choice` that nothing can discharge — the
  trap-state class `OOS-DP9-14` was filed for, and CR 701.9c says a cost discard has extra rules
  of its own. Placement in the arm makes "cost discards do not ask" a *structural* property, not
  a promise.
* The choice is expressed once, at the one site where CR 701.9b's default applies to a real,
  live, resolution-time choice.

**Rejected alternative**, recorded so nobody "unifies" the two paths later: putting the ask
inside `discard_cards` would be a one-line-shorter diff and would silently arm both of the above.
§8 test (d) exists to make that regression red.

### 2.5 `discard_cards` refactor — one implementation of the CR 702.35a body, two entry points

`discard_cards` (`:9387-9448`) currently interleaves *choosing* (the `min_by_key`) with
*performing* (the Madness check, the zone move, the `CardDiscarded` event, the `MadnessTrigger`
push). ENG-1 needs the performing half with an announced list.

Extract, **without changing `discard_cards`' loop**:

```rust
/// CR 701.9a / CR 702.35a / CR 400.7: discard ONE named card from `player`'s
/// hand. The per-card body extracted from `discard_cards` so that the announced
/// path (ENG-1, CR 701.9b) and the auto-pick path share one implementation of
/// the Madness route, the event and the trigger push.
fn discard_one_chosen_card(
    state: &mut GameState,
    player: PlayerId,
    card_id: ObjectId,
    events: &mut Vec<GameEvent>,
) { /* verbatim body of the current `if let Some(card_id) = card_id { .. }` block */ }
```

`discard_cards` keeps its `for _ in 0..n { ...min_by_key... }` shape and calls the new helper.
The announced path is `for id in chosen { discard_one_chosen_card(state, p, *id, events) }`.

**Why keep the loop rather than compute the `n` lowest up front**: the two are equivalent today
(nothing re-enters the hand between iterations — `expect_move_object_to_zone` moves the card out,
and the Madness path pushes to `state.pending_triggers`, which is flushed after the resolution,
CR 603.3), but "equivalent today" is an argument, and preserving the byte-identical loop is a
fact. Do not trade a fact for an argument in the batch whose whole premise is "the default must
not churn".

---

## 3. Engine changes

### Change 1 — the two new variants

**File**: `crates/card-types/src/state/stubs.rs`
**Action**: append `Discard` to `EffectChoiceQuestion` (after `:921`) and to
`EffectChoiceAnswer` (after `:947`), with the docs in §2.1/§2.2; rewrite the type doc per §2.3.
**Pattern**: follow `Surveil`'s shape exactly. **Append — never reorder** (the `HashInto`
discriminants below are positional).

### Change 2 — the deferral branch in `Effect::DiscardCards`

**File**: `crates/engine/src/effects/mod.rs`, arm at `:1202-1208`
**CR**: 701.9b (the choice), 608.2d (when it is announced), 601.2c's principle (the
short-circuit)
**Pattern**: mirror the `Effect::SearchLibrary` arm at `:3595-3625` structurally, **including
its `None => return` comment**.

Target shape:

```rust
        // CR 701.9b (ENG-1): "By default, effects that cause a player to
        // discard a card allow the affected player to choose which card to
        // discard." Before ENG-1 this arm called `discard_cards` straight
        // through, which takes the lowest `ObjectId` -- the human's
        // leftmost/oldest card -- and the affected player was never asked.
        // No def in the corpus prints "at random" or "another player chooses",
        // so the CR default covers the whole live corpus (see the missing
        // `chooser` field, OOS-ENG1-3).
        Effect::DiscardCards { player, count } => {
            // MR-M7-05, applied here for the first time (see the DrawCards arm
            // above): a negative amount cast straight to `usize` wraps to ~1.8e19
            // and `discard_cards`' loop has no empty-hand break, so it is an
            // effective hang in release. The short-circuit below also needs `n`
            // to be a real count.
            let n = resolve_amount(state, count, ctx).max(0) as usize;
            let players = resolve_player_target_list(state, player, ctx);
            for p in players {
                // CR 701.9b: the whole hand is the legal answer space, ascending
                // (`state.objects` is an `imbl::OrdMap`, and `retain`/`filter`
                // preserve its order) -- the same order the pre-ENG-1
                // `min_by_key` scanned, so `hand[..n]` IS the old auto-pick.
                let hand_zone = ZoneId::Hand(p);
                let hand: Vec<ObjectId> = state
                    .objects
                    .iter()
                    .filter(|(_, obj)| obj.zone == hand_zone)
                    .map(|(&id, _)| id)
                    .collect();
                debug_assert!(
                    hand.windows(2).all(|w| w[0].0 < w[1].0),
                    "CR 608.2d: the discard question's hand must be in ascending \
                     ObjectId order -- the replay's question-equality check and \
                     `default_discard_answer` both depend on it"
                );
                // CR 601.2c's principle (the same argument the search arm makes
                // at the `!may_fail_to_find && candidates.len() == 1` branch):
                // when the answer space admits exactly ONE legal answer the
                // announcement is DETERMINED, so there is nothing to announce.
                // `n == 0` -> the empty set is the only answer; `n >= hand.len()`
                // -> the whole hand is (and an empty hand is that case too).
                // Skipping the question here is what keeps a full-hand discard
                // from costing a round trip and from perturbing a fuzz seed.
                if n == 0 || n >= hand.len() {
                    discard_cards(state, p, n, events);
                    continue;
                }
                let question = EffectChoiceQuestion::Discard {
                    hand: hand.clone(),
                    count: n as u32,
                };
                let chosen = match ask_or_consume_effect_choice(state, ctx, p, question) {
                    Some(EffectChoiceAnswer::Discard { chosen }) => chosen,
                    Some(other) => {
                        debug_assert!(
                            false,
                            "CR 608.2d: variant mismatch answering a discard: {other:?}"
                        );
                        hand.iter().take(n).copied().collect()
                    }
                    // Suspended. Apply NOTHING -- the wrapper rolls the whole
                    // resolution back. The `for p in players` loop must not
                    // continue either: every later player's question is
                    // re-derived by the replay.
                    None => return,
                };
                for id in &chosen {
                    discard_one_chosen_card(state, p, *id, events);
                }
            }
        }
```

**Note the two exits are different**: `continue` for the determined case (later players still get
asked), `return` for the suspension (the whole pass is discarded). Getting these backwards is the
single easiest way to break this arm, and neither is caught by a compile error.

**SR-4 classification**: the `Some(other)` arm is a `debug_assert!` + deterministic fallback,
matching the scry/surveil arms at `:3706-3712` / `:3780-3786` verbatim in form. It is
**engine-bug side** (check 4 in `handle_answer_effect_choice` established variant agreement
before this code can run), and the fallback is the old auto-pick so release behaviour degrades to
pre-batch rather than to nothing.

### Change 3 — one question for all `n` cards (decision + justification)

**Decided: ONE question, `count: n`.** The engine asks "choose exactly `n` of these" once per
affected player, not `n` times.

Reasons, in descending weight:

1. **Nothing between the picks can change the answer space.** No player receives priority during
   a resolution (CR 608.2), and a chosen Madness card's trigger goes on the stack *after* the
   resolution (CR 603.3 — `discard_cards` pushes to `state.pending_triggers`, flushed later). So
   `n` sequential questions would each be asked against the identical hand minus the earlier
   picks: strictly less information, `n×` the round trips.
2. **It is what CR 514.1's cleanup discard already does.** `PendingCleanupDiscard { count }` /
   `LegalAction::DiscardToHandSize { count, hand }` ask for the whole subset at once. Two discard
   channels shaped alike is worth real money to every client.
3. **`DiscardPicker.svelte` already renders exactly this shape** and its own doc predicts the
   reuse: *"a second 'choose exactly N of these cards' question would reuse this component with
   no new client code."*
4. **Bank size.** `Effect::DiscardCards` under `PlayerTarget::EachOpponent` already multiplies by
   seat count; `n` questions per seat would multiply again against
   `MAX_EFFECT_CHOICES_PER_RESOLUTION`.

**What it forfeits, stated**: an effect whose `k`-th pick may depend on the outcome of the
`(k-1)`-th. `Effect::DiscardCards` has no such semantics (nothing is revealed between picks).
Seeded as `OOS-ENG1-4` so a future "discard a card, then discard a card" printing does not
silently inherit the wrong shape.

### 3.3 The short-circuit — what it does and does NOT protect

**Does**: keeps a determined announcement (`n == 0`, or `n >= hand.len()`, which includes the
empty hand) from costing a round trip, and therefore keeps every bot-only game and every fuzz
seed whose discards are full-hand discards on their pre-batch command trace.

**Does NOT**: protect `Effect::WheelHand`. Both wheel call sites (`:1236`, `:1266`) pass a
pre-snapshotted `hand_size` and would satisfy `n >= hand.len()` — but they never reach this
branch at all, because they call `discard_cards` **directly** and the ask lives in the
`Effect::DiscardCards` arm (§2.4). The brief's reasoning here is replaced.

That is a *structural* guarantee and structural guarantees rot silently, so **§8 test (d) is
mandatory** and asserts the structure, not the arithmetic: a `WheelHand` resolution must discard
the whole hand exactly once and leave `state.pending_effect_choice()` `None` throughout. If a
later batch "simplifies" by moving the ask into `discard_cards`, that test goes red and the
double-count risk the brief worried about is caught before it ships.

### Change 4 — `default_discard_answer` + the dispatcher arm

**File**: `crates/engine/src/effects/mod.rs`, after `default_surveil_answer` (`:379`)

```rust
/// CR 608.2d / CR 701.9b (ENG-1): the deterministic default answer for an
/// effect-driven discard -- the `count` LOWEST `ObjectId`s, byte-identical to
/// the pre-ENG-1 `discard_cards` auto-pick (`min_by_key(|id| id.0)`, taken `n`
/// times against an ascending hand). So the discard half of ENG-1 is zero-churn
/// for bot-only games and the fuzzer: the same cards are discarded, in the same
/// order; only the COMMAND TRACE grows an `AnswerEffectChoice`.
///
/// **This is the OPPOSITE end of the sorted hand from
/// `rules::turn_actions::default_cleanup_discard`**, which takes the `count`
/// HIGHEST ids. Both helpers are doing the same job -- reproduce the auto-pick
/// their site used to make -- and the two auto-picks genuinely differed
/// (CR 514.1's took `obj_ids.last()`; CR 701.9b's took `min_by_key`). Do not
/// "unify" them; pinned in one place by
/// `test_eng1_defaults_reproduce_both_pre_batch_picks`.
///
/// Called by nobody in the engine; see [`default_search_answer`].
pub fn default_discard_answer(q: &EffectChoiceQuestion) -> EffectChoiceAnswer {
    match q {
        EffectChoiceQuestion::Discard { hand, count } => EffectChoiceAnswer::Discard {
            chosen: hand.iter().take(*count as usize).copied().collect(),
        },
        _ => default_effect_choice_answer(q),
    }
}
```

and the **exhaustive** dispatcher at `:386-402` gains the identical arm. *This is the compile
error the runner will hit first.* Add the matching cross-reference line to
`default_cleanup_discard`'s doc (`rules/turn_actions.rs:1395-1403`) pointing back here.

### Change 5 — `handle_answer_effect_choice` checks 4 and 5

**File**: `crates/engine/src/effects/mod.rs`

* **Check 4** (`:622-634`), variant agreement: add
  `| (EffectChoiceQuestion::Discard { .. }, EffectChoiceAnswer::Discard { .. })` to the
  `matches!`. **Not a compile error** — a `matches!` is not exhaustive. If it is missed, every
  discard answer is rejected with "does not answer question", which at least fails loudly.
* **Check 5** (`:642-677`), per-variant legality: add an arm **before** the
  `_ => unreachable!("variant agreement checked above")` default. **Not a compile error either**
  — the `_` arm swallows it, and a missed arm here `unreachable!()`-panics the engine on the
  first real discard answer. Both of these must be on the runner's checklist as *silent* sites.

```rust
        (
            EffectChoiceQuestion::Discard { hand, count },
            EffectChoiceAnswer::Discard { chosen },
        ) => {
            // CR 701.9b: exactly `count`, no duplicates, every one from the hand
            // the ENGINE recorded. Nothing is re-derived from the board and
            // nothing positional is trusted from the wire.
            if chosen.len() != *count as usize {
                return Err(GameStateError::InvalidCommand(format!(
                    "CR 701.9b: this effect discards exactly {count} card(s); the answer \
                     names {}",
                    chosen.len()
                )));
            }
            let mut seen: Vec<ObjectId> = Vec::with_capacity(chosen.len());
            for id in chosen {
                if seen.contains(id) {
                    return Err(GameStateError::InvalidCommand(format!(
                        "CR 701.9b: {id:?} is named more than once"
                    )));
                }
                if !hand.contains(id) {
                    return Err(GameStateError::InvalidCommand(format!(
                        "CR 701.9b: {id:?} is not in the hand this effect is discarding from"
                    )));
                }
                seen.push(*id);
            }
        }
```

`validate_partition` (`:733`) is **deliberately not reused**: the discard answer is not a
partition (the unchosen cards stay in hand), and its message strings say "every looked-at card
must be placed exactly once", which would be a false diagnosis. Duplicating ~12 lines is the
right call; note it in a comment so a later reviewer does not "deduplicate" it.

### Change 6 — `ask_or_consume_effect_choice`'s three gates, applied to discard

**File**: `crates/engine/src/effects/mod.rs:477-558`. **No code change needed** — the function is
variant-agnostic. What ENG-1 owes is the *discharge* of each gate's obligation for the new
variant:

1. **Mana-ability gate** (`:486-498`, CR 605.1b/605.4a). The branch applies the default silently,
   and its own comment says the obligation is discharged by
   `tests/primitives/pb_dp9_effect_choice.rs::test_dp9_mana_ability_gate`, whose roster assertion
   scans `["SearchLibrary", "Scry", "Surveil"]` at **`:2558`**. **ENG-1 MUST add `"DiscardCards"`
   to that array** and widen the test's doc comment ("the three asking effects" → four). Without
   it, the comment at `:492-496` becomes a claim wearing a gate's authority — exactly the
   OOS-DP7-11 class. *(Expected result: still empty. A mana ability that discards is not a thing
   any real card does; the assertion's value is that nobody can author one silently.)*
   Also widen `rules/mana.rs:876-878`'s comment, which names the same test.
2. **Dead-player gate** (`:499-510`, CR 104.3a/800.4). Correct for discard with no change: a
   player who has left announces nothing, so the default (the lowest `count` ids of a hand that
   is about to be exiled by CR 800.4a anyway) applies. No test owed beyond the existing one.
3. **Already-suspended gate** (`:511-516`). Correct with no change. It is what makes the
   `for p in players` `return` (Change 2) safe: a second player's discard in the same pass
   records nothing.

### Change 7 — `HashInto`

**File**: `crates/engine/src/state/hash.rs`

* `impl HashInto for EffectChoiceQuestion` (`:3208-3229`): new arm, discriminant **`3u8`**,
  feeding `hand` then `count`. Append; never renumber `0`/`1`/`2`.
* `impl HashInto for EffectChoiceAnswer` (`:3230-3249`): new arm, discriminant **`3u8`**, feeding
  `chosen`.
* Both matches are **exhaustive → compile errors**. Good.
* **Read the warning at `:3196-3207` before editing**: the SR-19 gate
  (`every_hashed_struct_field_is_hashed_or_allowlisted`) **scans structs only**, so a dropped
  field feed in an *enum* arm passes every gate green (`OOS-DP9-13`). These two arms are held by
  review and by `stream_fingerprint`, nothing else. Feed every field.
* `count: u32` — hash via the same route `TriggerTargetOption.max: u32` uses; do **not**
  `as usize`/`as u64` inconsistently between the two impls.

### Change 8 — the version-history append sites

* **`crates/engine/src/state/hash.rs`**, the `HASH_SCHEMA_HISTORY` doc block (`:646-660` is the
  PB-DP9 entry to model on) and the `HASH_SCHEMA_HISTORY` table: **append a new `- 71:` line and
  a new table row.** Never edit a shipped row.
* **`crates/engine/src/rules/protocol.rs`**, the History doc (`:302-314` is PB-DP9's entry) and
  `PROTOCOL_HISTORY` (`:614-620` is the v33 row): **append `- 34:` and a v34 row.**
* Both numbers are **gate-computed, never predicted** — §11.

### Change 9 — exhaustive-match sites (the compile-error checklist)

Produced by running the brief's mandated grep on HEAD:

```
grep -rn 'EffectChoiceQuestion::\|EffectChoiceAnswer::' --include=*.rs . | grep -v target
```

**121 occurrences across 16 files.** Classified below. `cargo build --workspace` (SR-3's seal
gate) plus `cargo build --workspace --tests` is what closes this list.

| File | Match expression | Line | Exhaustive? | Action |
|---|---|---|---|---|
| `crates/card-types/src/state/stubs.rs` | enum decls | 906, 929 | — | **Add both variants + rewrite the type doc (§2.3)** |
| `crates/engine/src/effects/mod.rs` | `default_effect_choice_answer` | 386-402 | **YES → compile error** | Add `Discard` arm |
| `crates/engine/src/effects/mod.rs` | `default_search/scry/surveil_answer` | 327/352/371 | no (`_ =>`) | no change; add `default_discard_answer` beside them |
| `crates/engine/src/effects/mod.rs` | check 4 `matches!` | 622-634 | **NO — silent** | Add the pair. Miss ⇒ every discard answer 400s |
| `crates/engine/src/effects/mod.rs` | check 5 `match` | 642-677 | **NO — `_ => unreachable!()`** | Add arm. Miss ⇒ panic on first real answer |
| `crates/engine/src/effects/mod.rs` | `Effect::DiscardCards` arm | 1202-1208 | — | Change 2 |
| `crates/engine/src/effects/mod.rs` | `discard_cards` | 9387 | — | Change 2.5 + §10 doc |
| `crates/engine/src/state/hash.rs` | `HashInto for EffectChoiceQuestion` | 3208 | **YES → compile error** | `3u8` arm |
| `crates/engine/src/state/hash.rs` | `HashInto for EffectChoiceAnswer` | 3230 | **YES → compile error** | `3u8` arm |
| `crates/engine/src/state/hash.rs` | history doc + table | 646, tail | — | Append `- 71:` (Change 8) |
| `crates/engine/src/rules/protocol.rs` | history doc + `PROTOCOL_HISTORY` | 302, 614 | — | Append `- 34:` (Change 8) |
| `crates/engine/src/rules/events.rs` | `EffectChoiceRequired` doc | 1487-1507 | — | Widen the hidden-info doc (§2.3) |
| `crates/engine/src/rules/resolution.rs` | wrapper | 90-160 | variant-agnostic | **no change** (verified) |
| `crates/engine/src/rules/engine.rs` | `BlockingDecision` | 146-166 | variant-agnostic | **no change** (verified) |
| `crates/engine/src/testing/replay_harness.rs` | `"answer_effect_choice"` | 1116-1148 | **YES → compile error** | Add `Discard` arm (§3.9a) |
| `crates/engine/src/testing/script_schema.rs` | `EffectChoiceScriptAnswer` | 631-652 | — | Add `discard: Vec<String>` (§3.9a) |
| `crates/simulator/src/legal_actions.rs` | `LegalAction::AnswerEffectChoice` build | 464-480 | variant-agnostic | **no change** (verified) |
| `crates/simulator/src/params.rs` | `AnswerEffectChoice` arm | 682-691 | variant-agnostic | **no change**; widen the `:88-91` doc ("a library search, a scry or a surveil") |
| `crates/simulator/src/heuristic_bot.rs` | score | 327 | variant-agnostic | **no change** |
| `crates/simulator/src/local_game.rs` | `DecisionKind` | 129, 418 | variant-agnostic | **no change** |
| `tools/play-server/src/view.rs` | `blocking_decision_view` question match | 1910-1961 | **YES → compile error** | Add `Discard` arm + new shape (§4) |
| `tools/play-server/src/view.rs` | `AnswerShapeView` | 408-475 | — | Add `PickN` variant (§4) |
| `tools/play-server/src/api.rs` | `validate_decision_params` `(question, answer)` | 516-559 | no (`_ =>` catch-all) | **Add arm — a miss is a silent 400 on every discard** |
| `tools/play-server/src/api.rs` | `question_kind` | 885-892 | **YES → compile error** | `"discard"` |
| `tools/tui/src/play/app.rs` | event formatter | 642-646 | **YES → compile error** | `"discard"`; keep the no-ids rule (`:634-638`) |
| `crates/engine/tests/core/decision_site_walk.rs` | `discard_cards` row | 317-326 | — | **`AutoChosen` → `Served { by: "ENG-1", residual: [..] }`** (§7) |
| `crates/engine/tests/core/decision_gate.rs` | `BASELINE`, `MAX_AUTO_CHOSEN_COMPLETE_UNION`, T8 | 383-503, 925-930 | — | **§7 — arithmetic-exact edit** |
| `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` | mana-gate roster array | 2558 | — | Add `"DiscardCards"` (Change 6) |
| `crates/engine/tests/primitives/pb_dx3_stale_blocker_notes.rs` | 3 matches | 863, 891, 999 | no (`other =>`) | no change |
| `crates/engine/tests/mechanics_m_z/{surveil,library_ordering}.rs` | constructs | — | no | no change |
| `crates/simulator/tests/local_game.rs` | 1 match | 1191-1195 | no (`other =>`) | no change |
| `tools/play-server/src/main.rs` | shape probes | 3698/3810/3964/4092 | — | Add a `PickN` probe (§8 i) |
| `tools/play-server/src/main.rs` | `plainClone` picker list | 6779-6783 | — | **Add `DiscardPicker.svelte`** (§4) |

#### 3.9a The replay-harness arm

`crates/engine/src/testing/replay_harness.rs:1116-1148` matches **exhaustively** on
`entry.question`. Add, following the `Scry` arm's shape:

```rust
                crate::state::EffectChoiceQuestion::Discard { hand, count } => {
                    if spec.discard.is_empty() {
                        crate::effects::default_effect_choice_answer(&entry.question)
                    } else {
                        let mut remaining = hand.clone();
                        let mut chosen = Vec::new();
                        for name in &spec.discard {
                            let id = find_named_among(state, &remaining, name)?;
                            remaining.retain(|x| *x != id);
                            chosen.push(id);
                        }
                        let _ = count; // legality is the engine's judgment, not ours
                        crate::state::EffectChoiceAnswer::Discard { chosen }
                    }
                }
```

and add to `EffectChoiceScriptAnswer` (`script_schema.rs:631-652`, which is
`#[serde(deny_unknown_fields)]` — a new `#[serde(default)]` field is backward compatible, every
existing script omits it):

```rust
    /// CR 701.9b (ENG-1): for an effect-driven discard, the cards to discard,
    /// **in discard order** (CR 608.2f / 404.3 — the order they enter the
    /// graveyard). Must name exactly the question's `count` cards, all in hand.
    /// Unlike `bottom`/`graveyard`, there is no complementary half to derive:
    /// the unchosen cards stay in hand.
    #[serde(default)]
    pub discard: Vec<String>,
```

**Do not add a golden script for it in this batch** (SR-9c: the corpus is a triaged partition,
208 approved / 63 retired, and adding to it is bookkeeping this batch has not budgeted). Pin the
new arm from Rust instead — §8 test (h).

---

## 4. Play-server: the `AnswerShapeView` decision, made and justified

**The problem, restated precisely.** `DiscardPicker.svelte` renders the `Subset` shape and emits
`{[answerField]: [id, ...]}` — a bare id array into `ActionParamsDto::discard_cards`. An
`effect_choice_answer`, by the `template`-clone contract documented on
`AnswerShapeView::Partition::template` (`view.rs:450-460`) and restated in
`SearchPicker.svelte:33-41`, must be a **clone of the engine's own serialized
`EffectChoiceAnswer`** with one key overwritten — *the client must never spell the variant name
itself*. `Subset` carries no `template`. So `DiscardPicker` cannot answer an effect-choice
discard as written.

**I read `ActionBar.svelte`'s shape dispatch (`:825-897`) before deciding.** It is a
four-way `{#if currentShape?.shape === …}` chain with an `{:else}` "This client does not know how
to answer a … decision" fallback that is *reachable and cancellable*, and whose own comment says
the dispatch is on SHAPE precisely so "a fifth question reusing an existing shape must need no
change here."

### Option A — optional `template` + `chosen_key` on `Subset`

`Subset` grows `template: Option<EffectChoiceAnswer>` and `chosen_key: Option<String>`;
`DiscardPicker` grows two nullable props and branches at runtime on `template === null`.

**Rejected.** Not because of the branch (Option B has the same branch), but because of what
happens when the data is *wrong*: a `Subset` payload whose `template` is missing or malformed
degrades to **posting a bare id array into `effect_choice_answer`**, which the server 400s with a
deserialization error the human cannot act on. That is a new silent-wrong path, in the exact
class UI-4 (`scutemob-185`) was dispatched to remove.

### Option B — a new `AnswerShapeView::PickN` variant — **CHOSEN**

```rust
    /// CR 701.9b (ENG-1): choose **exactly** `count` of `candidates`, answered
    /// through a template. The answer goes into
    /// `ActionParamsDto::effect_choice_answer`.
    ///
    /// `PickOne` and `PickN` are the two cardinal-choice shapes that answer via
    /// `effect_choice_answer` and therefore carry a `template` + a key;
    /// [`Self::Subset`] is the one that answers via its own `discard_cards`
    /// field and carries no template. Splitting on that, rather than making
    /// `Subset`'s template optional, means a stale or malformed payload lands in
    /// `ActionBar`'s visible "unknown shape" fallback instead of posting a body
    /// the server will 400 (the UI-4 lesson).
    PickN {
        candidates: Vec<CardOptionView>,
        count: usize,
        /// The key inside `template`'s single variant object that the chosen ids
        /// go in (`"chosen"`). See [`Self::Partition::template`].
        chosen_key: String,
        /// See [`Self::Partition::template`] — serialized verbatim, cloned by the
        /// client, never re-spelled.
        template: EffectChoiceAnswer,
        /// The engine's own default subset (`default_discard_answer`: the `count`
        /// LOWEST `ObjectId`s -- note that is the opposite end of the hand from
        /// `Subset`'s CR 514.1 default). Sent so "use the default" is one click
        /// and so a test can assert the human drove something else.
        default: Vec<u64>,
    },
```

**The `blocking_decision_view` arm** (`view.rs`, inside the `LegalAction::AnswerEffectChoice`
match at `:1910-1961`):

```rust
                EffectChoiceQuestion::Discard { hand, count } => (
                    "Discard",
                    format!(
                        "{src}: discard {count} card{} — you choose (CR 701.9b)",
                        plural(*count as usize)
                    ),
                    AnswerShapeView::PickN {
                        // CR 701.9b: these are the ANSWERER'S OWN hand cards, so
                        // they are already in the seat-redacted view and their
                        // labels come through `NameIndex` -- exactly as the
                        // CR 514.1 arm above does it, and deliberately NOT
                        // through `question_card_label`. That channel exists for
                        // LIBRARY cards the effect has granted a look at, and
                        // `test_ui1_view_rs_reads_game_state_in_exactly_the_two_
                        // known_places` pins its size; routing an owned-hand
                        // question through it would enlarge a channel for no
                        // reason and blur what that gate is counting.
                        candidates: hand
                            .iter()
                            .map(|id| CardOptionView { id: id.0, label: names.label(*id) })
                            .collect(),
                        count: *count as usize,
                        chosen_key: "chosen".to_string(),
                        template: answer.clone(),
                        default: match answer {
                            EffectChoiceAnswer::Discard { chosen } => {
                                chosen.iter().map(|id| id.0).collect()
                            }
                            _ => Vec::new(),
                        },
                    },
                ),
```

> **Verification the browser probe owes**: if `names.label` returns the `UNKNOWN_LABEL`
> placeholder for these ids, the premise "the decision belongs to the viewer, so its hand ids are
> in the viewer's redacted view" is false and the guard is not what we think it is. §8 (i) must
> show **real card names**, not placeholders, or stop and report.

### Frontend

**`tools/play-server/frontend/src/lib/DiscardPicker.svelte`** — three new props, all defaulted so
the `Subset` call site is unchanged:

* `template = null`, `chosenKey = 'chosen'`, `onError = null`
* `confirm()` branches **explicitly and documentedly**:
  * `template === null` → today's `onConfirm?.({ [answerField]: [...selected].sort(...) })`
    (the `Subset` / `discard_cards` path — unchanged bytes)
  * otherwise → `import { plainClone }` and the **exact** `SearchPicker.emit` body
    (`SearchPicker.svelte:117-146`): `plainClone(template)` inside a `try`, take
    `Object.keys(answer)[0]`, guard `variant === undefined || typeof answer[variant] !== 'object'`
    with an `onError?.(...)`, write `answer[variant][chosenKey] = [...selected].sort((a,b)=>a-b)`,
    `catch` → `onError?.(...)`. **Never `structuredClone`** — the UI-4 defect.
* Update the component doc: its "the only question that arrives in this shape is the CR 514.1
  cleanup discard" sentence is now false; and its "Ascending ids, not click order" section is
  **wrong for `PickN`** in principle (CR 608.2f/404.3 make discard order a real player payload) —
  but ship ascending anyway for now, with the deviation named and seeded (`OOS-ENG1-7`), because
  `check_ids` treats the list as a set and no card in the corpus reads graveyard order.

**`ActionBar.svelte`** — a fifth arm, immediately after the `Subset` arm:

```svelte
      {:else if currentShape?.shape === 'PickN'}
        <DiscardPicker
          prompt={currentDecision.prompt}
          candidates={currentShape.candidates}
          count={currentShape.count}
          defaults={currentShape.default}
          template={currentShape.template}
          chosenKey={currentShape.chosen_key}
          answerField={currentDecision.answer_field}
          disabled={loading}
          onConfirm={onDecisionConfirm}
          onCancel={cancelChain}
          onError={onPickerError}
        />
```

Update the doc at `:52-55` ("four pickers chosen by `decision.answer.shape`") and `:149-152`
("one of exactly four shapes") — **five and five**. Both are prose claims that would become
false, which is the OOS-DP7-11 class.

**`tools/play-server/src/main.rs:6779-6793`** — the `(b)` non-vacuity arm asserts that "the three
pickers really route through the sanctioned helper". `DiscardPicker.svelte` now builds an answer
by copying a template prop, so **add it to that list** (`"DiscardPicker.svelte"`) and update the
arm's comment from "three pickers" to four. Missing this leaves a picker that can reintroduce the
UI-4 `DataCloneError` with the gate green.

---

## 5. Card definition fixes

**None. Zero card-def lines change in this batch.**

This is worth stating as a positive assertion rather than an omission. `fell_specter.rs` is
`Complete`, its `Effect::DiscardCards { player: DeclaredTarget{0}, count: Fixed(1) }` and its
`TargetRequirement::TargetOpponent` are both correct (the latter was already repaired by PB-EF6),
and the defect was 100% engine-side. The same holds for the other 12.

**Corpus roster, enumerated (SR-36: from the type, not from a grep of prose).** 23 def *files*
carry `Effect::DiscardCards`; **13 are effectively `Complete`** and therefore deck-legal, and are
exactly the 13 named in `decision_gate.rs`'s `BASELINE`:

Burglar Rat · Chart a Course · Consign // Oblivion · Faithless Looting · **Fell Specter** ·
Frantic Search · Geier Reach Sanitarium · Greater Good · Izzet Charm · Pull from Tomorrow ·
Raiders' Wake · Sword of Feast and Famine · *(+ the 13th `BASELINE` row — read it off `T9`'s
printed output, do not count from this list; the whole point of `T9` is that this number has been
mis-copied before)*

The remaining 10 files are non-`Complete` and get the fix for free when they are repaired.

**Coverage is expected to be UNMOVED at 1,133/1,803 = 62.8%.** Prove it by regenerating
`tools/authoring-report.py` to a byte-identical body — **not** by an empty `git diff` over
`crates/card-defs`, per the PB-DX19 lesson.

### 5.1 MANDATORY — pre-existing TODO sweep (roster-recall gate)

Run and record. Executed this session:

```
grep -rniE 'TODO.*[Dd]iscard|discard.*choice|choose.*discard|EffectChoice' crates/card-defs/src/defs/
```

**27 hits, 1 of which names this primitive**, and it is a **NOT-a-forced-add** with a stated
reason — which is the finding, not the absence of one:

* **`fable_of_the_mirror_breaker.rs:174-191`** (`Completeness::partial`) says, verbatim:
  *"no DSL primitive for a bounded optional discard whose count drives a matching draw
  (**DiscardCards has no player-choice bound**; WheelHand only disposes of the whole hand)."*
  **This is NOT closed by ENG-1 and must not be claimed.** Chapter II is "You **may** discard
  **up to two** cards. If you do, draw that many" — three separate gaps: (a) *optional*
  (a `may`, CR 601.2b-style decline), (b) *up to N* rather than exactly N, and (c) the discarded
  count feeding a draw. ENG-1's question asks for **exactly** `count`. Fable stays `partial`.
  Recorded as `OOS-ENG1-8`: with `EffectChoiceQuestion::Discard` in the tree, closing it is now a
  `min`/`max` widening plus an `EffectAmount` source, not a new primitive.

The other 26 hits are a different primitive each (`Cost::DiscardCard` filters,
`AdditionalCost::DiscardCard` on spells, `WheneverOpponentDiscards` triggers, `DiscardAtRandom`,
reflexive "when you do" triggers). None is a forced add. **Positive assertion: the TODO sweep
found 0 cards that ENG-1 unblocks and 1 card whose TODO names the primitive but is not closed by
it.**

---

## 6. The bot / fuzz default, and which fixtures move

**The engine's default reproduces the pre-batch pick exactly**, so no game OUTCOME changes in any
bot-only game. What changes is the **command trace**: one extra `Command::AnswerEffectChoice` per
asked discard.

`default_discard_answer` takes the `count` **lowest** ids from an ascending `hand`, which is
exactly `min_by_key(|id| id.0)` applied `n` times to a hand nothing else touches between
iterations (§2.5). **This is the OPPOSITE of `default_cleanup_discard`** (`count` **highest**,
`turn_actions.rs:1404-1420`). Both are faithful reproductions of two auto-picks that genuinely
differed. §8 test (e) pins both in one place so the next reader cannot confuse them.

**Anything that could move, and how to enumerate it — do not guess, measure:**

1. Run `cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/eng1-baseline.txt` **before any
   edit** and again after. Never pipe to `tail` (2026-08-02: a tail pipe hid a compile failure and
   faked a green run).
2. **Expected movers, ranked by likelihood:**
   * `tools/play-server/src/main.rs` probes asserting a `command_count` delta across a turn that
     includes a discard.
   * `crates/simulator/tests/*` seeded fixtures — `UI1_SEED`, `UI2_SEED`, `SIM1_SEED` and the
     `sim5_bot_cast_discipline.rs` A/B seeds (0/7/42). These assert on rejections, taps and
     journal lines rather than trace length, so most should hold; any that reddens is telling you
     a bot now spends an action answering.
   * `crates/engine/tests/scripts/harness_equivalence.rs` — SR-9b's per-step fingerprint across
     the JSON and direct-`Command` regimes. Both regimes route through
     `auto_answer_blocking_decisions`, so this should hold; if it does not, the two regimes are
     reaching the pump differently and that is a finding.
   * The two golden scripts (§0.1 item 4). **Expected green unchanged.**
3. **Not expected to move**: fuzz outcomes (`OOS-UI2-1` — the fuzzer has never cast a spell, so
   it has never reached an effect-driven discard at all; the honest prediction is "no fuzz
   change, because there is no fuzz coverage here", not "no fuzz change, because it is
   zero-churn").

---

## 7. `decision_gate.rs` / `decision_site_walk.rs` — the batch's own yield measurement

This is the largest single edit in the batch and it is **arithmetic**, not judgment. Do it last,
after the engine compiles, and read every number off a failing gate's own output.

1. **`decision_site_walk.rs:317-326`** — flip the `discard_cards` row:

```rust
    Row {
        id: "discard_cards",
        cr: "701.9 / 701.9b",
        site: "effects/mod.rs::execute_effect (DiscardCards) -> EffectChoiceQuestion::Discard",
        class: DecisionClass::Served {
            by: "ENG-1",
            residual: &["OOS-ENG1-1", "OOS-ENG1-2", "OOS-ENG1-3", "OOS-ENG1-4"],
        },
        predicate: p_discard_cards,
    },
```

   Leave the `connive` row (`:455-464`) and the `wheel_hand` row (`:327-338`) **untouched** — see
   §9. `wheel_hand`'s `NoDecision` justification is still exactly right.

2. **`decision_gate.rs` `BASELINE`** (`:383-...`) — delete the 12 entries whose row set is
   `&["discard_cards"]` alone, and shrink `Izzet Charm` (`:431-435`) to
   `("Izzet Charm", &["counter_unless_pays"], None)`. **T5**
   (`every_baseline_entry_is_live_and_necessary`) is what tells you if you got this wrong, by
   name.
3. **`MAX_AUTO_CHOSEN_COMPLETE_UNION`** (`:503`, currently `91`) — **T6**
   (`auto_chosen_complete_union_is_ratcheted`) is a hard *equality*, and its failure message
   prints the new number. **Set it to the number T6 prints.** Do not compute `91 - 12`: the union
   is over *defs*, not over `(def, row)` pairs, and `Izzet Charm` stays in it (it still hits
   `counter_unless_pays`). Extend the constant's doc with an ENG-1 sentence — its existing doc is
   an essay about why this number moved three times inside PB-DX4 and must be read off the gate.
4. **`MIN_BASELINE = 50`** (`:506`) — after deleting 12, `BASELINE.len()` must still clear 50.
   Check, don't assume. If it does not, **stop and report**; do not lower the floor.
5. **T8** (`served_rows_still_have_their_hooks`, `:925-930`) — add `("discard_cards", 1)` to the
   `[(id, min)]` array so the new `Served` row gets a non-zero roster floor like its three
   siblings. (Floor will be 13; assert `>= 1` for consistency with the others.)
6. **T4's synthetic fixture at `:668-671`** uses `["proliferate", "discard_cards"]` as a
   *mismatched* baseline row set for a synthetic def that only hits `proliferate`. Since
   `discard_cards` is no longer an `AutoChosen` row, `auto_chosen_row_hits` can never return it —
   the assertion still passes (the recorded set is still a strict superset), but the fixture now
   demonstrates the superset arm with a row that no def can ever hit, which is a weaker probe.
   **Swap `"discard_cards"` for another live `AutoChosen` row id** (e.g. `"sacrifice_permanents"`)
   and say why in a comment.

**The yield number this produces is the batch's headline**: the still-auto-chosen `Complete`
union drops from **91** to whatever T6 prints. Report both.

---

## 8. Tests

**SR-9a**: 9 test targets, `crates/engine/tests/<group>/`, never a new top-level `tests/*.rs`.
New engine file: **`crates/engine/tests/primitives/pb_eng1_effect_discard_choice.rs`**, with
**`mod pb_eng1_effect_discard_choice;`** added to `crates/engine/tests/primitives/main.rs`
(alphabetical-ish, near `mod pb_dx19_characteristics_recursion;` at `:32`). *A dropped `mod` line
silently deletes the whole file's coverage and the SR-9a gate catches it — add it in the same
commit.*

| # | Test | Target file | CR | What it proves |
|---|---|---|---|---|
| a | `test_eng1_effect_discard_suspends_for_the_affected_player` | `primitives/pb_eng1_…` | 701.9b, 608.2d | Cast/resolve a real `Complete` **`fell_specter`** ETB targeting an opponent with a 3-card hand: the resolution suspends, `state.pending_effect_choice()` is `Some`, its `player` is the **TARGETED OPPONENT** (not `ctx.controller`), and its question is `Discard { hand: <3 ids ascending>, count: 1 }`. The player-identity half is the point — CR 701.9b says *affected*, and `PendingEffectChoice`'s own doc calls out that this is not the controller. |
| b | `test_eng1_a_non_default_answer_discards_the_chosen_card` | same | 701.9b | From (a)'s block, answer with `chosen: vec![hand[2]]` (the **highest** id). Assert that card is in the graveyard and `hand[0]` (the pre-batch auto-pick) is **still in hand**. *This is the test that would have caught the shipped defect.* |
| c | `test_eng1_a_determined_discard_does_not_suspend` | same | 601.2c principle, 608.2d | Three cases, one test: `count > hand.len()`, `count == hand.len()`, and an **empty hand** — each resolves to completion with `pending_effect_choice()` `None` and the right cards in the graveyard. Plus `count == 0` against a non-empty hand (no suspension, nothing discarded). |
| d | `test_eng1_wheel_hand_discards_the_whole_hand_exactly_once_and_never_suspends` | same | 701.9, 121.1 | Resolve a `WheelHand { disposal: Discard, draw: ThatMany }` with a 4-card hand: 4 cards in the graveyard (**not 8** — the double-count a suspend/replay would cause), 4 drawn, and `pending_effect_choice()` `None` at every step. **Assert the structure, not the arithmetic** (§3.3): this is the regression guard against a later batch moving the ask into `discard_cards`. Also cover `WheelDraw::GreatestDiscarded` (the two-pass branch at `:1221`). |
| e | `test_eng1_defaults_reproduce_both_pre_batch_picks` | same | 701.9b, 514.1 | In ONE test: `default_discard_answer` on a 5-card hand with `count: 2` returns `hand[0..2]` (**lowest**), and `default_cleanup_discard` on the same hand with a `count: 2` cleanup entry returns `hand[3..5]` (**highest**). Comment says these are deliberately opposite and why. |
| f | `test_eng1_illegal_discard_answers_are_refused_and_leave_the_state_untouched` | same | 701.9b, 608.2d | Four rejections off one block, each a distinct message, each asserting the block **survives** and the hand is unchanged (`process_command` takes `GameState` by value, so "untouched on rejection" is by construction — assert it anyway): (1) a `Scry` answer to a `Discard` question (check 4); (2) an id not in `hand` (check 5); (3) the wrong number of ids; (4) a duplicated id. Plus (5) the SR-29 half: a **different seat** submits a legal-looking answer and is refused by check 2. |
| g | `test_eng1_a_chosen_madness_card_still_routes_to_exile` | same | 702.35a, 701.9b | Hand contains `fiery_temper` (Madness) plus two others. Answer `chosen: [fiery_temper_id]` **explicitly** (not by default). Assert: `CardDiscarded` still fires; the card is in **exile**, not the graveyard; a `PendingTriggerKind::Madness` is queued with the right cost. The point is that the announced path shares `discard_one_chosen_card`'s body — a copy-pasted second implementation would pass every other test here and fail this one. |
| h | `test_eng1_the_script_harness_can_drive_a_named_discard` | same | 608.2d | Call `mtg_engine::testing::replay_harness::translate_player_action("answer_effect_choice", …)` with an `EffectChoiceScriptAnswer { discard: vec!["Mountain".into()], ..Default::default() }` against a live block; assert the produced `Command::AnswerEffectChoice` names the Mountain's id. Pins §3.9a without touching the SR-9c golden partition. |
| i | `test_eng1_the_browser_renders_a_pickn_discard` | `tools/play-server/src/main.rs` (add beside the four shape probes at `:3698`/`:3810`/`:3964`/`:4092`) | 701.9b | HTTP probe: drive a game to an effect-driven discard for the human seat, `GET` the seat payload, assert `answer["shape"] == "PickN"`, `answer_field == "effect_choice_answer"`, `candidates` carry **real card names** (§4's premise check), `chosen_key == "chosen"`, `template` is `{"Discard":{"chosen":[...]}}`, and a `POST` of a **non-default** `effect_choice_answer` is accepted and moves the game. |
| j | `test_eng1_a_foreign_seats_discard_question_never_reaches_this_payload` | `tools/play-server/src/main.rs` | Invariant 7 | The hand-zone analogue of `test_ui1_a_foreign_seats_effect_choice_never_reaches_this_payload`. Drive a discard block for seat A, move `PlaySession::human` to seat B, assert the payload loses the decision **and** the `candidates` key. **This is the new-channel gate**: the shipped `GameSummary.seed` HIGH is precisely what happens when a redaction gate checks the channel it was written for and a new channel is invisible to it. A hand is a new channel. |

**Non-vacuity, non-negotiable**: every one of (a)–(j) must be **proven red by executing a
revert**, not by argument. For (b), (f) and (j) the revert is the deferral branch itself; for (d)
the revert is moving the ask into `discard_cards`; for (i) it is deleting the `PickN` arm.

**Fixture pattern**: follow `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` (the closest
analogue end to end) and `pb_dp7_cleanup_discard.rs`'s `build_oversized_hand` /
`advance_to_cleanup_block` for hand construction. Remember `enrich_spec_from_def()` — a naked
`ObjectSpec::card()` has no Madness keyword and test (g) will pass for the wrong reason.

---

## 9. Out of scope — stated as such, with the reason each is excluded

1. **A `chooser` field on `Effect::DiscardCards`.** `card_definition.rs` declares
   `DiscardCards { player, count }` and nothing more. **Do NOT add it.** 23 def files carry the
   effect, 13 are `Complete`, and **zero** print "at random" or "another player chooses". The two
   corpus cards that would need it (`gamble.rs` — "discard a card at random"; `grief.rs` — "you
   choose a nonland card from it") are both blocked TODO defs that carry **no**
   `Effect::DiscardCards` at all, so adding the field today would ship a field with no reader.
   → **`OOS-ENG1-3`**.
2. **`Effect::SacrificePermanents` (`effects/mod.rs:4222`).** CR 701.21a, the same complaint class
   (`OOS-UI2-5`), and named by the triage as the cheapest follow-on once the discard question
   exists. Genuinely cheaper now — but it is a different rule, a different answer space
   (battlefield, **public**, so a different hidden-info argument) and a different roster. It gets
   its own batch.
3. **`Cost::DiscardCard` (`effects/mod.rs:9276`, inside `pay_optional_cost`).** CR 701.9b covers a
   cost discard too, and this site still auto-picks the lowest id. Excluded **structurally**, not
   by preference: a cost is paid outside any resolution wrapper, so an ask there records a
   `pending_effect_choice` nothing can roll back or discharge (the `OOS-DP9-14` trap-state class),
   and CR 701.9c adds cost-specific rules an announcement would have to respect. → **`OOS-ENG1-1`**.
4. **`Effect::Connive`'s inlined discard (`effects/mod.rs:5518-5560`).** CR 701.50a + 701.9b, its
   own `AutoChosen` row in `decision_site_walk.rs:455-464`, and it duplicates the `min_by_key`
   because it needs per-card nonland accounting. Now trivially closable — but the nonland counter
   must survive a suspend/replay, which is a real design question and not a rename.
   → **`OOS-ENG1-2`**.
5. **`Effect::MillCards` (`:1305`)** — the only sibling of §0.1 item 3's missing `.max(0)`. Not
   fixed here (this batch touches the discard arm; a drive-by fix in an adjacent arm is how
   review scope-creep starts). Check whether `mill_cards` has an empty-library break before
   deciding severity. → **`OOS-ENG1-6`**.
6. **The golden-script corpus.** §0.1 item 4 / §3.9a. Run it, report it, do not edit it.

---

## 10. Comment debt — the item this whole batch is a lesson about

`discard_cards`' doc (`effects/mod.rs:9380`) reads:

> `/// Discard n cards from a player's hand (first by ObjectId, deterministic).`

It states the placeholder **as a design property**. Every one of the ~13 sibling auto-pick sites
the triage census found (`:4222` `SacrificePermanents`, `:3157` Bolster, `:3274` Amass, `:3450`
top-N zone move, `:4467`/`:4479`, `:6027`, `:7324`/`:7386`, `:2822`, `:5904`/`:5523`/`:8940`,
`:4143`, `:2943`) carries a `deferred to M10+` comment. **This one did not — which is exactly why
the PB-DP decision-point audit's own greps missed it and a human found it in a live game.**

The generalisable rule, which every comment this batch touches must obey:

> **A deliberate placeholder that documents its MECHANISM instead of its DEBT is invisible to
> every audit that greps for the debt.**

**Required edits:**

* `discard_cards`' doc → say plainly that its `min_by_key` is the **auto-pick path only**, that
  CR 701.9b's choice is served by the `Effect::DiscardCards` arm, and that the two callers who
  still reach it without a choice (`Effect::WheelHand`, `Cost::DiscardCard`) do so for the
  reasons in §2.4/§9 — with `OOS-ENG1-1` named at the cost caller.
* `Effect::Connive`'s inline comment at `:5524` ("Deterministic: discard the card with the
  smallest ObjectId") → add `deferred, OOS-ENG1-2` and the CR cite. **This is a one-line edit and
  it is not optional**: it is the last remaining copy of the exact comment shape that hid this
  bug for a year.
* File **`OOS-G3-2`** as the census seed: *the surviving sibling sites all carry `deferred to
  M10+`; the census in `memory/playtest-triage-2026-08-02b.md` §G3 has never been machine-checked,
  and `decision_site_walk.rs`'s `AutoChosen` rows are the machine-checkable version of it. Reconcile
  the two lists and make the source comments derive from the table rather than the other way round.*

---

## 11. PROTOCOL (SR-8) and HASH — gate-computed, never predicted

**Both are expected to move.** `EffectChoiceQuestion` and `EffectChoiceAnswer` are in the SR-8
wire closure (`protocol.rs`'s `- 31:` history line says so explicitly) and both are fed to
`HashInto` from `GameState`, so adding a variant moves the declared shape in both digests.
**Expected: PROTOCOL 33 → 34, HASH 70 → 71, closure type count unchanged (96) — expected, not
asserted.**

**The exact commands that produce each value** (run them; copy the digest out of the *failure
message*, never compute it):

```bash
cd /home/skydude/projects/scutemob/.worktrees/scutemob-191
~/.cargo/bin/cargo test -p mtg-engine --test core protocol_schema -- --nocapture
~/.cargo/bin/cargo test -p mtg-engine --test core hash_schema -- --nocapture
```

`protocol_schema_fingerprint_is_pinned` (`tests/core/protocol_schema.rs:846`) and
`declaration_fingerprint_is_pinned` / `stream_fingerprint_is_pinned`
(`tests/core/hash_schema.rs:1086`, `:1111`) each print the value to set.

**History rule**: **append a new row, never edit a shipped row.** Two histories each, both of
which must move together with their constant:

* `crates/engine/src/rules/protocol.rs` — the `- 34:` doc line (model on `- 31:`, `:302-314`) and
  a `ProtocolEpoch { version: 34, fingerprint: "<from the gate>" }` row after `:614-620`.
* `crates/engine/src/state/hash.rs` — the `- 71:` doc line (model on `- 68:`, `:636-659`) and a
  `HASH_SCHEMA_HISTORY` row. `hash_schema.rs:1139-1157` asserts the last row's version equals
  `HASH_SCHEMA_VERSION`; `:1181-1204` asserts the frozen prefix digest, which will also need
  re-pinning **to the value it prints**.

### Sentinel re-pin procedure

**Re-pin by SYMBOL, not by number.** There are **60 sentinel assertions across 48 files**
(`assert_eq!(HASH_SCHEMA_VERSION, 70…)` / `assert_eq!(PROTOCOL_VERSION, 33…)`), and some are
written multi-line so a `70u8`-literal grep misses them (PB-DX5 shipped with two multi-line
survivors for exactly this reason).

```bash
grep -rn 'HASH_SCHEMA_VERSION' --include=*.rs crates tools | grep -v 'src/state/hash.rs'
grep -rn 'PROTOCOL_VERSION'    --include=*.rs crates tools | grep -v 'src/rules/protocol.rs'
```

Then **confirm by execution, not by inspection**:

```bash
~/.cargo/bin/cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/eng1-sentinels.txt
```

**Report the residual list** (the set of sentinel sites that were still red after the by-symbol
pass) even when it is empty — PB-DX6's was empty and PB-DX5's had two, and that difference is a
fact about the sentinel population, not about the procedure. **Never `| tail`.**

---

## 12. Verification checklist

- [ ] §0 premise re-verified on branch; any further drift recorded in this file before coding
- [ ] `~/.cargo/bin/cargo build --workspace` (SR-3 seal gate) and `--workspace --tests` both clean
- [ ] Every row of §3.9's table addressed — **including the four NON-compile-error sites**
      (check 4, check 5, `api.rs` validate, the `pb_dp9` mana roster array)
- [ ] `"DiscardCards"` added to `pb_dp9_effect_choice.rs:2558`, and its doc + `mana.rs:876-878`
      widened from three to four
- [ ] §7's `decision_gate` edit done with every number read off a failing gate
- [ ] Card defs untouched; `tools/authoring-report.py` regenerates byte-identically
- [ ] `tools/check-defs-fmt.sh` **and** `cargo fmt --check` (SR-35 — `cargo fmt` checks none of the
      1,798 defs and still exits 0)
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace --no-fail-fast` to a **file**, full output read
- [ ] Golden corpus green; the two discard scripts reported as changed-or-not
- [ ] PROTOCOL + HASH bumped from gate output, both histories appended, 60 sentinels re-pinned,
      residual list reported
- [ ] Every new test proven red by an executed revert
- [ ] Browser verification of (i) with a **non-default** answer (UI-4's standard) and real names
- [ ] Frontend bundle rebuilt; `main.rs`'s `plainClone` picker list includes `DiscardPicker.svelte`
- [ ] Seeds `OOS-ENG1-1..8` + `OOS-G3-2` filed in `memory/workstream-state.md`

---

## 13. Risks & edge cases

1. **The two silent plumbing sites.** Check 4's `matches!` and check 5's `_ => unreachable!()`
   compile fine when missed. Miss check 4 ⇒ every discard answer is refused; miss check 5 ⇒ the
   engine **panics** on the first real one. Both are in §3.9's table flagged NON-compile-error;
   both must be exercised by §8 (b) and (f).
2. **`decision_gate.rs` is a ratchet with a hard equality.** A "close enough" number fails, and
   the failure message is the fix. Do not compute; read.
3. **Hidden information — the new channel.** The hand variant is safe *only* because the answerer
   owns the cards and three separate guards enforce `entry.player`. §8 (j) is the gate; the
   `GameSummary.seed` HIGH is the precedent for what "a new channel is invisible to the old gate"
   costs.
4. **The `continue` vs `return` distinction** in the `for p in players` loop (Change 2). Neither
   is a compile error. `return` on suspend is what makes the replay correct for a multi-player
   `Effect::DiscardCards`; `continue` on the short-circuit is what keeps later players asked.
   §8 (a) should be extended with a 2-opponent case, or a sibling test added, so both exits are
   executed.
5. **`Effect::DiscardCards` nested in `ForEach` / `Sequence` / `Conditional`.** The sub-context
   builders at `:4075` and `:4125` inherit `effect_choice_gate_closed` and the `Sequence`/`ForEach`
   suspension checks already exist for the three sibling effects (`:4007`, `:4078`, `:4128`).
   Confirm by test that a discard nested one level deep suspends and replays; do not assume the
   inherited machinery covers a fourth variant just because it covered three.
6. **Execution outside `resolve_top_of_stack`.** Verified this session: the non-resolution
   `execute_effect` callers are `rules/mana.rs:880` (gated) and `rules/replacement.rs:2257`
   (a hard-coded `Effect::CreateToken`, unreachable by `DiscardCards`). So the discard arm's
   exposure is a **subset** of the three existing arms'. If that grep changes, re-derive it.
7. **The bank and `MAX_EFFECT_CHOICES_PER_RESOLUTION = 64`.** `Effect::DiscardCards` under
   `PlayerTarget::EachOpponent` in a 4-player game banks 3 answers; nested in a `ForEach` it could
   bank more. 64 is comfortable, but the one-question decision (§3) is part of why — say so if
   the ceiling is ever revisited.
8. **`count: u32` vs `usize`.** Cross the boundary in exactly one place per direction and keep the
   `HashInto` feed consistent between the two impls. A `usize`-vs-`u32` inconsistency here does not
   fail to compile; it changes the hash stream on a 32-bit target and nowhere else.
9. **No frontend harness (`OOS-UI5-4`).** The `DiscardPicker` template branch is unreachable by any
   automated test in this repo. Use the tier-1 recipe from the UI-5 handoff (~15 min, proven twice,
   thrown away twice) rather than shipping a fourth untested picker path.
