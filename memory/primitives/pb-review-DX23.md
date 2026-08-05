# Primitive Batch Review: PB-DX23 — dredge has no answer channel for anyone

**Date**: 2026-08-05
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-201` · worktree `/home/skydude/projects/scutemob/.worktrees/scutemob-201`
**Plan**: `memory/primitives/pb-plan-DX23.md` · **Execution record**:
`memory/primitives/pb-DX23-execution-notes.md`
**CR rules verified independently via MCP**: 702.52a, 702.52b, 121.2, 121.6a/b/c, 614.11,
614.11a/b, 616.1e/f, 400.1, 400.7, 103.8a, 104.3c, 117.3d, 514.1/514.3
**Engine files reviewed**: `crates/engine/src/rules/queries.rs`,
`crates/engine/src/rules/replacement.rs`, `crates/engine/src/rules/events.rs`,
`crates/engine/src/rules/engine.rs` (read-only, admission gate),
`crates/engine/src/state/keyword_registry.rs`, `crates/engine/src/effects/mod.rs` (read-only),
`crates/engine/src/rules/turn_actions.rs` (read-only),
`crates/card-types/src/state/replacement_effect.rs` (read-only)
**Simulator / tools files reviewed**: `crates/simulator/src/legal_actions.rs`,
`crates/simulator/src/params.rs`, `crates/simulator/src/heuristic_bot.rs`,
`crates/simulator/src/random_bot.rs` (read-only), `crates/simulator/src/targeting.rs` (read-only),
`tools/play-server/src/view.rs`, `tools/play-server/src/main.rs`,
`tools/play-server/frontend/src/lib/ActionBar.svelte` (read-only),
`tools/play-server/README.md`
**Tests reviewed**: `crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs` (7),
`crates/simulator/tests/pb_dx23_dredge_answer_channel.rs` (6),
`tools/play-server/src/main.rs::test_dx23_browser_can_answer_a_dredge_offer` (1),
`crates/engine/tests/primitives/pb_dx2_command_gates.rs` (2 rewritten + 1 protected pin),
`crates/simulator/tests/local_game_playthrough.rs` (1 arm)
**Card defs reviewed**: `crates/card-defs/src/defs/golgari_grave_troll.rs` (1 — comment only)

---

## Verdict: needs-fix

**No HIGH.** The engine change is CR-correct where it matters and I could not construct an
SR-38 counterexample: the tail flip draws the "same draw vs. different draw" line exactly where
CR 121.2 + CR 614.11a/121.6b draw it, `dredge_options`' two consumers are provably one
derivation, the `OOS-DX2-3` two-entry trace the batch threads a flag to prevent is **real** (I
re-derived it from source independently, and the flag does prevent it), the two rewritten
`pb_dx2_command_gates.rs` tests are genuinely *strengthened* rather than weakened (3 decline
rounds for a draw-three is CR-correct and no draw is lost or double-counted), the protected pin
`test_dx2_needschoice_redefer_grows_the_queue` is byte-unedited and `OOS-DX2-3` is nowhere
re-closed, and Architecture Invariant 7 is untouched (`view.rs`'s three new arms route through
`NameIndex`; the UI-6 raw-read gate's seven-needle pin set is unmoved).

**Four MEDIUM.** Two are stale docs in `replacement.rs` that the batch's own Stage-2 sweep
missed and that now assert the *opposite* of what this batch shipped, on the very enum variant
and the very function whose parameter changed meaning. The other two are one defect and its
seed row: **the Q2 suppression rule does not remove the re-defer loop it claims to remove
"structurally."** The guard is keyed on the *graveyard* (`dredge_options` non-empty) while the
entry the engine FIFO-answers may be `NeedsChoice`-origin — and the state where both hold
simultaneously is the exact end state that `test_dx2_needschoice_redefer_grows_the_queue`
already builds and asserts. In that state `HeuristicBot` declines forever with no state
progress. Latent (0 `ReplacementTrigger::WouldDraw` card defs — I re-ran the grep myself), same
reach as the case the guard *does* cover, so it is not a HIGH; but the shipped doc and
`OOS-DX23-2` both overstate what was achieved.

**Nine LOW**, mostly evidence-hygiene: the one revert the plan singled out as mandatory (T1.1)
was substituted rather than executed, the pre-edit baseline was inherited rather than
re-measured (and the batch's own Stage 0 proved an inherited pin stale in the same breath), the
`OOS-DX2-3` guard test's named assertion is not the one that reddens, and no browser-level
verification was executed for the human channel.

**Q6 divergence: ACCEPTABLE.** See the adjudication section below.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | MEDIUM | `crates/engine/src/rules/replacement.rs:786-789` | **`DrawStepOutcome::DredgeOffered`'s doc asserts the invariant this batch deleted.** "Dredge is only ever offered with `offer_dredge: true`, i.e. never mid-resume … the resume paths pass `false`." `perform_remaining_draws` IS a resume path and now passes `true` in three of four caller configurations; T3.1 asserts a `DredgeOffered` arising there. **Fix:** rewrite in the two-axis vocabulary (same draw vs. different draw) and point at `perform_remaining_draws`' own caller table. |
| E2 | MEDIUM | `crates/engine/src/rules/replacement.rs:682-684` | **`check_would_draw_replacement`'s doc: "`offer_dredge` is `false` on a resume (PB-DP5 §3.3)."** Same class as E1, on the function whose parameter this batch re-scoped. **Fix:** amend to "`false` on a SAME-DRAW resume; a TAIL resume passes the caller's flag (PB-DX23 §3 Q3)." |
| E3 | LOW | `crates/engine/src/rules/queries.rs:359-388` | **`dredge_options` reads raw `obj.characteristics.keywords`, not layer-resolved characteristics.** Consistent with `handle_choose_dredge`'s own validation, so there is no offer-vs-engine divergence — but the function is now a PUBLIC query and PB-DX19's durable lesson makes this axis a standing hazard (an effect such as "cards in graveyards lose all abilities" would be invisible to both). **Fix:** one-line note at the query stating the read is raw, that this matches the answer-time validator, and that closing it means changing both together. |
| E4 | LOW | `crates/engine/src/rules/replacement.rs:1775-1786` vs `:926-933` | **Asymmetry the batch introduces but does not record:** a tail auto-declined whole by the implicit discharge (`tail_offers_dredge: false`) becomes dredge-offerable again if one of its draws hits `NeedsChoice` and is later resumed through `resolve_pending_draw` (`true`). Zero corpus reach. **Fix:** one sentence in `resolve_pending_draw`'s `true` comment acknowledging it, or fold into the E5 seed. |
| E5 | LOW | seed registry (`docs/audits/decision-point-audit.md` §8.1) | **The newly-narrowed CR 702.52a deviation has no seed row.** §3 Q3 owns it in prose ("an auto-discharged sequence's tail is auto-declined whole, not just at its head") and `resolve_declined_pending_draw`'s doc states it, but `OOS-DX2-7`'s row is about the *head* of the discharge, and no row covers the tail. **Fix:** extend `OOS-DX2-7`'s text to name the tail explicitly, or file `OOS-DX23-8`. |

## Simulator / Client Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| S1 | **MEDIUM** | `crates/simulator/src/legal_actions.rs:296-306` + `:671-685` | **The Q2 suppression rule does not remove the re-defer loop, and its doc says it does.** The guard is a property of the graveyard; the entry answered is FIFO. With a dredge-eligible graveyard AND a `NeedsChoice`-origin entry queued first, the provider offers, the engine answers the `NeedsChoice` entry, `None` re-defers, and the bot declines forever. **Fix:** see Finding Details S1 — either bound it with a `RepeatKey::ChooseDredge` (cap 1) in `heuristic_bot.rs`, or narrow the doc claim and file a seed. |
| S2 | MEDIUM | `docs/audits/decision-point-audit.md:1340` (`OOS-DX23-2`) | **The seed's statement is over-broad and therefore wrong.** "A `NeedsChoice`-origin `PendingDraw` remains unanswerable through any simulator channel" is false whenever any dredge card is eligible — then it *is* answerable (FIFO), and answering it re-defers. **Fix:** reword to "unanswerable whenever no dredge card is eligible; answerable-but-re-deferring when one is — see S1." |
| S3 | LOW | `crates/simulator/src/heuristic_bot.rs:347-356` | **"…so the action stays choosable when it is all there is" is unreachable for this variant.** `PassPriority` (score 1) is pushed unconditionally at `legal_actions.rs:552` before the dredge block runs, so a `Some` scored `0` can never be the top score. Inherited idiom (`TapForMana` has the same shape), so this is a comment accuracy issue, not a policy bug. **Fix:** add "(inherited idiom; for `ChooseDredge` specifically `PassPriority` is always present, so the 0 arm is effectively 'never')". |
| S4 | LOW | `tools/play-server/README.md:1079-1096` | Item **29** is inserted between items 26 and 27. Pre-existing disorder in this list (25 precedes 24), so this is continuation, not regression. **Fix:** renumber the whole list once, or leave and note. |

## Test / Evidence Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| T1 | LOW | `memory/primitives/pb-DX23-execution-notes.md:455` | **The one revert the plan singled out as mandatory was not executed.** §5 T1.1: "delete the `ChooseDredge` push block from `StubProvider::legal_actions` — A1 and A2 must both redden, rebuild confirmed." The matrix records "N/A". **Fix:** execute it (30 seconds), or state the equivalence argument explicitly. |
| T2 | LOW | `memory/primitives/pb-DX23-execution-notes.md:16-23` | **The pre-edit baseline was inherited, not re-measured** — while §0.2, two paragraphs later, proves an inherited pin (play-server "78") was stale by exactly this mechanism. **Fix:** run the full suite at the merge-base next time, or record the inheritance as a named deviation. |
| T3 | LOW | `crates/engine/tests/primitives/pb_dx23_dredge_tail_and_query.rs:485-491` | **T3.3's headline `OOS-DX2-3` guard is not the assertion the revert reddens.** The earlier `outer_offer_count == 1` (`:460`) fires first — as the execution notes' own "left: 2, right: 1" record shows. **Fix:** move the `pending_draws().len()` assertion above the offer-count assertion, or state which assertion is the discriminator. |
| T4 | LOW | (whole batch) | **No browser-level verification was executed** for the human channel. **Fix:** one headless-Chromium pass at `T5_DX23_SEED`. |
| T5 | LOW | `docs/audits/decision-point-audit.md:905` | Says "Four new rows **OOS-DX23-1..4** are appended"; six rows were appended (1, 2, 3, 4, 6, 7). **Fix:** correct to "six rows, `OOS-DX23-1..4` + `-6`/`-7`; `-5` deliberately not filed." |

---

## Finding Details

### S1 (MEDIUM): the Q2 suppression rule is graveyard-keyed, the entry it answers is FIFO-keyed

**File**: `crates/simulator/src/legal_actions.rs:671-685` (the guard) and `:296-306` (the claim)
**CR**: 616.1e/f, 702.52a
**Also**: `crates/engine/src/rules/replacement.rs:3371-3382` (`handle_choose_dredge`'s FIFO
`position()`), `crates/engine/tests/primitives/pb_dx2_command_gates.rs:1339-1409` (the fixture
that reaches the state)

The shipped doc says, at `legal_actions.rs:296-306`:

> **Suppression rule (plan §3 Q2), load-bearing, not defensive**: this action is offered at ALL
> only when `dredge_options(state, player)` is non-empty … Without this, a `NeedsChoice`-origin
> `PendingDraw` … would get a bare decline offer whose resume RE-DEFERS … A bot scoring
> `ChooseDredge` above `PassPriority` would then decline forever and burn `max_commands`.

The plan (§3 Q2) states it more strongly: *"Suppressing the offer when no dredge card is
eligible removes that loop **structurally**, at zero CR cost."*

**It does not.** The guard tests a property of the *graveyard*. The entry the engine answers is
`state.pending_draws.iter().position(|pd| pd.player == player)` — FIFO, oldest first, with no
discriminator between a dredge-origin and a `NeedsChoice`-origin entry (`PendingDraw` has no
such field, and cannot gain one without a PROTOCOL bump). So the loop survives whenever a
dredge card *is* eligible **and** a `NeedsChoice`-origin entry sits ahead of the dredge entry
in the queue.

That state is not hypothetical: it is exactly the end state of the batch's own protected pin.
`test_dx2_needschoice_redefer_grows_the_queue` (`pb_dx2_command_gates.rs:1339`) builds a fixture
with a `Dredge(3)` card in `p1`'s graveyard, four library cards, and two `SkipDraw` `WouldDraw`
replacements, and asserts the queue ends at **two** entries. I traced the ordering from source:

1. `draw_card` → `perform_one_draw(true)`; dredge is checked first, so entry 1 is dredge-origin.
2. `ChooseDredge { None }` consumes it; `resolve_declined_pending_draw`'s
   `perform_one_draw(false)` hits `NeedsChoice` and pushes `N1`. Queue = `[N1]`.
3. Second `draw_card` → discharge of `N1` → its resume re-defers → pushes `N1'`; then the outer
   call's own `check_would_draw_replacement(true)` pushes the dredge entry `D2`.
   **Queue = `[N1', D2]`** — the `NeedsChoice` entry is FIRST.

Now run the provider on that state. `pending_draws` contains `p1` ✓; `dredge_options(state, p1)`
returns `[(dredge_card, 3)]` (library 4 ≥ 3) ✓ — so the guard passes and the offer is emitted.
`HeuristicBot` scores `Some` at `0` (library 4 < `2 * 3 = 6`) and `None` at `2`, above
`PassPriority`'s `1`, so it declines. `handle_choose_dredge`'s FIFO `position()` answers `N1'`,
not `D2`. The decline's resume re-defers, and its own re-entrant discharge of `D2` re-defers too:
two entries removed, two pushed, **zero cards drawn, no library change, no progress**. The bot
declines again next window. That is the exact livelock the guard exists to prevent, reached with
the guard fully in force.

**Severity**: MEDIUM, not HIGH. Reach is zero today —
`grep -rn "ReplacementTrigger::WouldDraw" crates/card-defs/src/defs/` returns 0 hits (I re-ran
it), so no legal deck can produce a `NeedsChoice`-origin `PendingDraw`. That is precisely the
same zero-reach argument that justifies the guard's *covered* case, so the two are equally
latent. What makes this a finding rather than a note is that the shipped in-source doc, the
plan, and `OOS-DX23-2` all assert a property the code does not have.

**Fix (pick one, (b) is cheaper and also useful):**
- **(a)** Correct the claim: `legal_actions.rs:296-306` should say the guard removes the loop
  *only when the graveyard has no eligible dredge card*, and that a mixed queue (a
  `NeedsChoice`-origin entry ahead of a dredge entry, with a dredge card still eligible) is
  still loop-reachable because `PendingDraw` carries no origin discriminator; file the seed.
- **(b)** Bound it at zero wire cost: add `RepeatKey::ChooseDredge` with `cap() == 1` to
  `crates/simulator/src/heuristic_bot.rs:35-68` and `RepeatKey::of`. One decline per turn is
  ample for every real dredge flow (the tail flip needs at most `remaining` answers per
  sequence, and a genuine dredge is `Some`, not `None`); the cap drops the score to `0`,
  i.e. below `PassPriority`, and the loop terminates. Note in the arm that the cap is a damper
  on the mixed-queue case, not a legality gate.

**The experiment that settles it** (it is a claim from a source trace, not something I could
execute — I have no shell in this session): append to
`crates/simulator/tests/pb_dx23_dredge_answer_channel.rs` a test that builds
`test_dx2_needschoice_redefer_grows_the_queue`'s fixture verbatim, drives it to the two-entry
state through the same `turn_actions::draw_card` / `Command::ChooseDredge` calls that test uses,
sets `priority_holder`, then loops `StubProvider.legal_actions` → `HeuristicBot::choose_action`
→ `process_command` and asserts `state.pending_draws().len()` and `cards_drawn_this_turn` are
unchanged after N iterations. If the loop is real, both stay pinned for every N.

### E1/E2 (MEDIUM): two docs in `replacement.rs` now assert the opposite of what shipped

`replacement.rs:782-790`, on `DrawStepOutcome::DredgeOffered`:

> Dredge is only ever offered with `offer_dredge: true`, i.e. **never mid-resume** (PB-DP5 plan
> §3.3) — **the resume paths pass `false`** and thread the entry's own
> `already_applied`/`remaining` instead.

`replacement.rs:680-684`, on `check_would_draw_replacement`:

> `offer_dredge` is **`false` on a resume** (PB-DP5 §3.3): re-offering dredge mid-chain would
> restart a CR 616.1 application the player already began …

Both are now false for the tail. `perform_remaining_draws` is a resume path and receives `true`
from `handle_choose_dredge`'s `Some` arm (`:3490`), from `resolve_pending_draw` (`:1785`), and
from `resolve_declined_pending_draw` when its caller is the explicit-decline arm (`:3396`).
`test_dx23_tail_of_an_answered_multi_draw_offers_dredge_again` asserts, in so many words, that
a `DredgeOffered` arises inside a resume. E1 is the worse of the two: it sits on the enum
variant that *is* the mechanism, and a reader reasoning from it would conclude T3.1's assertion
is impossible.

The batch's Stage 2 correctly rewrote `perform_remaining_draws`' doc, both push-site comments,
`resolve_declined_pending_draw`'s doc, `events.rs`'s struck reason, and
`memory/gotchas-rules.md`. These two were missed because the plan's Stage-2 checklist did not
name them.

**Fix:** at `:786-789`, replace with the two-axis statement: dredge is never re-offered for the
SAME draw event (`resolve_declined_pending_draw`'s and `resolve_pending_draw`'s own
`perform_one_draw` calls, both unconditionally `false`), but the TAIL of a sequence is a
different draw event under CR 121.2 and IS offerable — see `perform_remaining_draws`' caller
table. At `:682-684`, the same amendment in one sentence.

---

## What I checked and found sound (the sceptic's list, item by item)

**1. The tail flip's CR line.** Correct, and drawn where the CR draws it. CR 121.2 ("Cards may
only be drawn one at a time … that player performs that many individual card draws") and CR
614.11a / 121.6b (the replacement completes, *then* the sequence resumes) together make each
resumed draw a fresh "would draw" event; CR 702.52a's "if you would draw a card, you may
instead" applies to each. All six `perform_one_draw` call sites are set correctly:

| site | `offer_dredge` | verdict |
|---|---|---|
| `turn_actions.rs:1252` (draw step) | `true` | correct, CR 121.1 |
| `effects/mod.rs:9500` (fresh sequence) | `true` per iteration | correct, CR 121.2 |
| `replacement.rs:1160-1174` (`resolve_declined_pending_draw`, THIS draw) | `false` unconditional | correct — same draw event; re-offering is an infinite choice on one event |
| `replacement.rs:1746-1753` (`resolve_pending_draw`, CR 616.1f re-check) | `false` | correct — **verified it really is the same draw**: `handle_order_replacements` routes here with the entry's own `already_applied` grown by `chosen_id`, and this call completes the draw that entry replaced |
| `replacement.rs:1650-1657` (`perform_remaining_draws` loop) | caller's flag | correct — **verified it really is the tail**: `remaining_after = remaining - 1 - i` and the entry it pushes carries that verbatim (`:971`), so `handle_choose_dredge` resumes from the right place |

`perform_remaining_draws`' three callers pass `true`/`true`/forwarded-flag as the plan
prescribes.

**2. The `OOS-DX2-3` two-entry trace is REAL.** I re-derived it from source without reading the
plan's version: with an unconditional `true` at `:1653`, the implicit discharge's own tail
(`perform_one_draw(true)` inside `perform_remaining_draws`) finds `pending_draws` already empty
for the player (the outer call removed the stale entry at `:923-925`), so its `DredgeAvailable`
arm pushes `E2` and `break`s; control unwinds to the OUTER `perform_one_draw`, whose
`check_would_draw_replacement(…, true)` at `:936` then pushes `E3`. Two dredge-originated
entries for one player. Threading `tail_offers_dredge: false` at `:933` is the correct minimal
prevention, and I confirmed the "at most one dredge-originated entry" invariant now holds
globally: a `DredgeAvailable` push requires an `offer_dredge: true` frame, and every `true` tail
frame is reached only from a caller that has no outer `perform_one_draw` still to push
(`handle_choose_dredge`, `resolve_pending_draw`, `handle_choose_dredge`'s `None` arm), while the
one caller that *does* have an outer push (`perform_one_draw`'s discharge) passes `false`.

`test_dx23_implicit_discharge_does_not_mint_a_second_dredge_entry` does reproduce the trace
(two `DredgeChoiceRequired` under the revert ⇒ two entries) — see T3 for the caveat that the
*named* assertion is not the discriminating one.

**Nothing re-closes `OOS-DX2-3`.** I grepped every occurrence tree-wide: every live mention says
REOPENED. `test_dx2_needschoice_redefer_grows_the_queue` (`:1340-1412`) is byte-unedited (its
body still asserts `pending_draws().len() == 2` and carries its original R1 doc), and both
push-site comments (`:944-960`, `:983-989`) now carry the corrected narrower claim with a
pointer to the reopened row.

**3. The two rewritten `pb_dx2_command_gates.rs` tests are strengthened, not weakened.**
`test_dx2_multi_draw_sequence_stops_at_the_dredge_offer`: `total_drawn == 3` (unchanged — no
draw lost, none double-counted) **plus a new `rounds == 3`** assertion that is itself the
regression guard against re-introducing tail immunity. Three decline rounds for a draw-three is
CR-correct given the fixture never removes the dredge card from the graveyard: each of the three
draws is a separate "would draw" event and the card stays eligible (library 10 → 7, always ≥ 3).
`test_dx2_second_dredge_offer_discharges_the_first_and_conserves_draws`: the end-to-end
conservation assertion (`discharge_drawn + decline_drawn == 3`) is preserved verbatim, and the
`decline_rounds == 2` assertion is added. The runner's deviation note (the dispatch said "do not
edit this file") is correctly reasoned: leaving those two red is incompatible with the mandatory
"residual list empty" gate, and the only alternatives were reverting the tail flip the plan
mandates. Both edits are assertion-and-doc only; same fixtures, same cards, same registry.

**4. SR-38: I could not construct a counterexample.** The guard is `pending_draws` contains
`player` AND `dredge_options(state, player)` non-empty, evaluated after the priority check at
`:545`, after the `blocking_decision()` early return at `:420-524`, and after the commander-zone
and mulligan early returns. Against `handle_choose_dredge`'s own rejection paths:
- *dead / conceded player* — step 0 returns `Ok(vec![])` (never `Err`), and such a player never
  holds priority anyway;
- *entry belonging to a different player* — the guard filters on `p.player == player`;
- *`Some(id)` whose card left the graveyard, or library below `n`* — impossible at the instant
  of the offer, because both conjuncts of `handle_choose_dredge`'s `Some` validation
  (`:3406-3442`) are the same two `dredge_options` computes (`queries.rs:359-388`); across a
  gap, `LocalGame::advance()` consumes the list immediately and the browser is `seq`-guarded;
- *two entries queued* — FIFO answers the oldest, and `handle_choose_dredge`'s doc argues (CR
  616.1e) that `Some` against a `NeedsChoice` entry is a feature; either way it is `Ok`, so no
  SR-38 violation. (The *loop* that follows is S1, a different property.)
- *blocking decision standing* — `engine.rs:304-314`'s allow-list excludes `ChooseDredge`, and
  the provider's own early return at `:524` fires first. `test_dx23_provider_is_silent_while_a_
  blocking_decision_stands` pins it with a non-vacuity floor (`DiscardToHandSize` still offered).
- *params* — `ChooseDredge` is correctly outside `params.rs:271-286`'s nine-arm allowlist, and
  `random_bot::action_to_command`'s `plan_targets` returns `NotTargeted` for it
  (`targeting.rs:121-131`'s `_ => None`), so no `UnsupportedParam` can arise from the bot path.

**5. Q2's zero-corpus premise — grep re-run by me.**
`grep -rn "ReplacementTrigger::WouldDraw" crates/card-defs/src/defs/` → **0 hits**. Confirmed.
`grep -rn "Dredge(" crates/card-defs/src/defs/` → 1 hit (`golgari_grave_troll.rs:91`). Confirmed.
On the follow-up question — *can a dredge-origin entry become non-eligible after being pushed?*
Yes (card exiled, or library milled below `n`), and the provider then withholds the offer
entirely, including the decline, so the entry is undischargeable-by-choice. The consequence is
bounded, though: once `dredge_options` is empty, `check_would_draw_replacement` will not offer
dredge on the *next* draw either, so that draw's own discharge completes the stale entry and the
new draw proceeds normally. One-time lag, self-healing — not the permanent one-behind loop this
batch closed. Worth a sentence in `OOS-DX23-2`, not a separate finding.

**6. The bot's `2 * mill` margin.** Reads `state.zones().get(&ZoneId::Library(player))` — the
right zone; `2 * (*mill as usize)` cannot overflow on a 64-bit target and cannot underflow;
`.unwrap_or(0)` on a missing zone is safe and errs toward declining. No `RepeatKey` for
`ChooseDredge`, so `is_capped_repeat` is inert for it (`RepeatKey::of` returns `None`).
**Termination**: a bot that dredges every turn terminates — dredging moves the card out of the
graveyard (CR 400.7, `:3464-3471`), ending the offer chain, and each dredge mills `mill` cards;
a bot that declines every turn also terminates, because each decline completes a real draw and
shrinks the library. The one non-terminating case is S1, which is a *decline* loop that draws
nothing. `RandomBot` picks uniformly by index and reaches the same `params.rs` arm, so both arms
stay exercised. The `0` score is choosable-but-last in principle; in practice unreachable — S3.

**7. Architecture Invariant 7 is satisfied.** The offer names graveyard objects (public, CR
400.1) and a `mill` count printed on the card. No library `ObjectId` crosses into the view model:
`view.rs::action_object` returns the offer's own `Option<ObjectId>` (`:1267`) and
`action_label` renders through `names.label(id)` — the `NameIndex` path, **not**
`question_card_label` and **not** `library_look_cards`. The mill's library reads happen inside
`handle_choose_dredge` after the answer, and emit only `CardMilled { new_id }` for cards now in
a public zone. `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places`
(`main.rs:4973-5048`) is unmoved at its full seven-needle pin set (`.objects()` = 2,
`.zone(` = 1, five zero-pins).

**8. Q6 — adjudicated ACCEPTABLE.** See the dedicated section below.

**9. `rules::queries::dredge_options`.** Structurally the same scan
(`zone == Graveyard(player)` → `find_map` on `KeywordAbility::Dredge(n)` → `n as usize <=
library_count` → `sort_by_key(ObjectId)`), and the strongest evidence of byte-equivalence is
behavioural: `cargo test -p mtg-engine --test mechanics_a_d dredge` is 15/15 green **with no test
edited**, including `test_dredge_exact_library_count_is_eligible` (the `<=` boundary), and the
golden script `replacement/014_golgari_grave_troll_dredge.json` — which drives
`Command::ChooseDredge` through the harness and would see any option-ordering change — is
byte-identical to its Stage-0 baseline. The sort is `sort_by_key` on `ObjectId`, total and
stable, so `DredgeChoiceRequired.options` is deterministic (SR-9b). **T2.1 and T2.2 do
discriminate independently of T2.3**: T2.1's fixture plants a battlefield card with the identical
keyword and asserts it is absent (the zone filter, not the consumer agreement); T2.2 plants an
exact-count card (`N == library_count == 3`) and an over-floor card (`N == 4`) and asserts
exactly one survives (CR 702.52b's floor, not the consumer agreement). Both reverts were executed
and named the right failure. PB-DX20's lesson is honoured, and the T2.3 doc says so in its own
words. **SR-5 caught a real site**: `keyword_registry.rs:211-221` now declares `queries.rs`
alongside `replacement.rs` for `Dredge` — found by the gate, not by the plan.

**10. Test quality — two spot-checks against vacuity.**
- `test_dx23_every_offered_action_is_engine_accepted` (T4.4) **avoids the PB-DX21 hazard
  correctly**: it calls `process_command(state.clone(), …)` and asserts only on
  `result.is_ok()`, never reading a `GameState` back out of a failing call. Non-vacuity floor
  (`!dredge_actions.is_empty()`) present.
- `test_dx23_heuristic_bot_declines_rather_than_milling_itself_out` (T4.5) is genuinely
  discriminating: the fixture is built at `library_count == 7` with `mill == 6`, so the `Some`
  option is *legally offered* (asserted first, as a non-vacuity floor) and only the bot's own
  survival margin refuses it. The revert (drop the `2 *`) makes the bot choose `Some`.
- The T1.1 mandatory probe's A2 arithmetic survives a real trap the runner found and recorded:
  a naive `TurnStarted` count returns 3, not 2, because `advance()`'s cap is checked at the top
  of the next iteration and turn 7's `TurnStarted` is journalled by the last command of turn 6.
  Uncorrected, the RHS would have over-counted by exactly the amount the LHS was short and the
  probe would have **passed vacuously pre-fix** (`1 == 1`). The fix is in the test's own filter,
  no production code touched. This is exactly the class of self-catch the review process exists
  to find, and the runner found it first.

**11. The card-def edit.** `golgari_grave_troll.rs` — comment only, `git diff --stat --
crates/card-defs/` empty per the notes and no DSL line differs on inspection. What the new
comment asserts is **true**: the machinery existed, nothing could reach it, and the channel now
runs provider → `heuristic_bot` → `params.rs` → play-server. It correctly leaves `OOS-DX2-7`
open. I re-verified the def against Scryfall oracle text via MCP: mana cost `{4}{G}`, type
`Creature — Troll Skeleton`, `0/0`, all three abilities present (ETB counters replacement,
`{1}` + remove-counter regenerate, `Dredge 6` keyword marker) — exact match.

---

## Q6 adjudication: is the divergence from acceptance criterion 1 acceptable?

**Yes, and the CR argument is sound.**

The criterion's wording is "play-server **blocking-decision UI** surfaces it." The batch routes
`ChooseDredge` through the ordinary action list instead — no new `AnswerShapeView` variant, no
new `ActionParams`/`ActionParamsDto` field, no picker, zero frontend production lines.

**The CR argument holds.** CR 702.52a is "you **may** instead"; declining is always legal, so
"no answer" has a well-defined meaning, which is the block-vs-deadline test PB-DP4/PB-DP7 use.
`BlockingDecision` (`engine.rs:145-166`) has three variants and `engine.rs:304-314`'s admission
gate makes each of them *exclusive* — while one stands, every other command is refused with
`BlockedByPendingDecision`. That exclusivity would be CR-wrong here: while a dredge offer stands,
`PassPriority` is legal, every cast is legal, every activation is legal, and the engine
demonstrably continues (priority, SBAs, step advancement — `events.rs:860-875`). Adopting the
blocking shape would also *retire* CR 702.52a's own "no answer is a decline", and would move
`state/hash.rs::public_state_hash` (HASH 73 → 74) for a decision the CR marks optional.

**The substance of the criterion is met.** `test_dx23_browser_can_answer_a_dredge_offer`
(`main.rs:10205-10277`) drives a real game over the real HTTP router — `session::new_game` with
`DeckSource::Fixed`, the Troll cast on curve and dying to CR 704.5f as a 0/0 — until an option
with `kind == "ChooseDredge"` and a non-null `object_id` appears, then POSTs it and verifies the
effect out of band from the engine's own journal (`GameEvent::Dredged`, `milled == 6`, the new
CR 400.7 id present in hand). That is the **non-default** answer (`Some(troll)`, not the
decline), so game state distinguishes the human's choice from any fallback — the UI-4/SIM-6
standard, met. The `option["decision"].is_null()` assertion is non-vacuous:
`ActionOptionView.decision` is a real, populated field (`view.rs:221`, filled by
`blocking_decision_view` at `:2468`), and it *is* non-null for `DiscardToHandSize` and the other
two blocking kinds — so the assertion pins the Q6 shape rather than a missing key.

**Residual risk, recorded as T4 (LOW), not as a blocker.** The claim that the *browser* renders
and submits it rests on the payload plus a reading of `ActionBar.svelte`: any kind not in
`controlKinds`/`relocatedKinds` lands in the `plays` group as a plain labelled button
(`:261-263`, `:710-734`), and `beginChain` → `advanceChain` → `pickerNeeded(...) === null` →
`onAct(index, {})` submits immediately (`:464-490`) — the live `PayEcho`/`PayRecover` path.
That path is exercised by shipped browser flows, so the risk is genuinely low; but UI-4 is this
project's own precedent for a case where every server-side test was green and the browser
handler threw. One headless pass at `T5_DX23_SEED` would close it.

---

## Acceptance criteria

| # | Criterion | Met? | Notes |
|---|---|---|---|
| 1 | `ChooseDredge` end-to-end: provider, `params.rs`, `heuristic_bot`, play-server UI — bot AND human channels live | **YES**, with the Q6 divergence **ACCEPTABLE** | Provider `legal_actions.rs:671-685`; `params.rs:644-647`; `heuristic_bot.rs:346-371`; `view.rs` three arms. Not routed through the blocking-decision UI, and correctly so (adjudication above). Human half proven by T5.1 over real HTTP with a non-default answer; browser-level verification not executed (T4, LOW). |
| 2 | Mandatory probe: real game, no state pokes, Troll in graveyard, draw count asserted after three turns, watched failing by revert | **YES**, with one caveat | `pb_dx23_dredge_answer_channel.rs::test_dx23_real_game_with_a_grave_troll_keeps_its_draw_cadence`. No post-`start` mutation, every command from `advance()`'s bot path, A3 non-vacuity floor, CR 103.8a stated explicitly. Committed RED at Stage 0 with literal pre-fix values (`pending=1`, `a2 = 1 vs 2`, `offers = 2`). **Caveat (T1, LOW)**: the plan's named revert was substituted by the Stage-0/Stage-2 red observations rather than executed. |
| 3 | `OOS-DX2-2` rider: multi-draw tail dredge-offerable per CR 121.2, commit stating why PB-DP5 §3.3 does not extend to the tail | **YES on substance**; commit text not verifiable from this session | The distinction is stated verbatim at `replacement.rs:1593-1625`, at `resolve_declined_pending_draw`'s doc `:1102-1127`, in `memory/gotchas-rules.md:45-58`, and in `docs/audits/decision-point-audit.md:881-886`. I have no shell in this session and could not read the commit message; **verify `git log` before close-out**. |
| 4 | `OOS-DX2-7` rider: stale-entry auto-discharge recorded as an AUTO-CHOSEN row in the decision-point audit §3.1 | **YES** | `docs/audits/decision-point-audit.md:242-277`. Filed as a prose block rather than a table row, with an explicit, correct argument for why it cannot be a row (the walk is over card-def `Effect`/`Condition` DSL variants; dredge is a `KeywordAbility`) and why §3.2 is also wrong for it. `Complete` defs reachable = 1. Recorded, not closed — and the "still open after PB-DX23" paragraph is accurate. |
| 5 | `OOS-DX2-3` watch item: no structural re-closure; `test_dx2_needschoice_redefer_grows_the_queue` respected; two push-site comments corrected | **YES** | Tree-wide grep: every live mention says REOPENED; nothing asserts "structurally impossible". The pin at `pb_dx2_command_gates.rs:1340` is byte-unedited and still asserts `len() == 2`. Both push-site comments (`:944-960`, `:983-989`) corrected to the narrower dredge-origin invariant with a pointer to the reopened row. |
| 6 | Wire gate-computed not assumed; baseline re-measured pre-edit; full suite to a file, 0 failures; seeds dispositioned; coverage honest | **MOSTLY** | HASH 73 / PROTOCOL 35 **gate-executed** (21/21 and 17/17), unmoved — and correctly so, since no state field and no wire type changed. Full suite **4,412 / 0 / 5**, captured to two files, residual list empty, arithmetic reconciling exactly (4,398 + 1 + 7 + 6). Seeds dispositioned: `OOS-DX2-5`/`-2` CLOSED, `-7` RECORDED, `-3` REOPENED, `OOS-DP5-2` unchanged, `OOS-DX23-1..4` + `-6`/`-7` filed, `-5` correctly not filed (its condition did not fire; every named R1 ratchet checked and diagnosed). Coverage **1,133/1,803 = 62.8%** unmoved, card-def diff comment-only. **Shortfall (T2, LOW)**: the pre-edit baseline was inherited, not re-measured — in the same Stage 0 that proved an inherited pin stale. |

---

## CR Coverage Check

| CR rule | Implemented? | Tested? | Notes |
|---|---|---|---|
| 702.52a (dredge is "you may instead", graveyard-only) | Yes | Yes | `dredge_options` zone filter; T2.1; T4.1; T5.1 |
| 702.52b (library floor `>= N`) | Yes | Yes | `n as usize <= library_count`; T2.2 (exact-count boundary, both directions) |
| 121.2 (draw N = N individual draws) | Yes | Yes | tail flip; T3.1, rewritten `pb_dx2` T5/T7 round counts |
| 121.6b / 614.11a (replacement completes, then sequence resumes) | Yes | Yes | T3.1, T3.4 (`remaining` bookkeeping through a tail deferral) |
| 616.1e / 616.1f (choose among applicable; repeat until none) | Unchanged | Yes | T3.2 (no re-offer on the same draw); `pb_dx2` T19/T20 unedited |
| 400.1 (graveyard is public) | Yes | Yes | offer names graveyard objects only; UI-6 gate unmoved; T5.1 |
| 400.7 (new object on zone change) | Unchanged | Yes | T5.1 reads `card_new_id` from `Dredged`, not the offer's graveyard id |
| 104.3c (mill-out loss) | Bot policy only | Yes | `2 * mill` survival margin; T4.5 |
| 103.8a (first player skips their first draw) | N/A (engine, pre-existing) | Yes | T1.1's A2 RHS derivation, stated in the test doc |
| 514.1 / 514.3 (admission gate exclusivity) | Respected | Yes | T4.3 (provider silent under a blocking decision, with a non-vacuity floor) |
| 117.3d (everyone gets priority before the step advances) | Reasoned, not tested | No | `OOS-DX23-1` — deferral, never loss; acceptable to leave untested |

## Card Def Summary

| Card | Oracle match | TODOs remaining | Game state correct | Notes |
|---|---|---|---|---|
| `golgari_grave_troll` | **Yes** (verified against Scryfall via MCP: `{4}{G}`, Creature — Troll Skeleton, 0/0, all three abilities) | 0 | Yes | Comment-only edit; zero DSL lines; the new comment's claims are accurate and it correctly leaves `OOS-DX2-7` open. The pre-existing 2018-12-07 reanimate-ordering caveat is untouched and out of scope. |

## Previous Findings

Not a re-review. `memory/primitives/pb-review-DX23.md` did not exist before this pass.
