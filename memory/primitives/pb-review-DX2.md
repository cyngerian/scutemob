# Primitive Batch Review: PB-DX2 — gate the resolution-time commands nothing gates

**Date**: 2026-08-01
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-162` · branch `feat/pb-dx2-gate-the-resolution-time-commands-nothing-gates-oos-d`
**Seeds**: OOS-DP5-7 (headline), OOS-DP7-2, riders OOS-DP2-1 / OOS-DP9-14
**CR Rules verified via MCP**: 702.52 / 702.52a / 702.52b, 614.11 / 614.11a / 614.11b,
616.1 / 616.1a–g, 103.5 / 103.5c, 504.1, 104.4b, 726 (all seven subrules)
**Engine files reviewed**: `crates/engine/src/rules/replacement.rs`,
`crates/engine/src/rules/commander.rs`, `crates/engine/src/rules/resolution.rs`,
`crates/engine/src/rules/events.rs`, `crates/engine/src/rules/engine.rs`,
`crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/miracle.rs`,
`crates/engine/src/rules/loop_detection.rs`, `crates/engine/src/effects/mod.rs`,
`crates/engine/src/state/mod.rs`, `crates/card-types/src/state/replacement_effect.rs`,
`crates/engine/src/testing/script_schema.rs`, `crates/engine/src/testing/replay_harness.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_dx2_command_gates.rs` (T1–T13, T16),
`crates/engine/src/rules/resolution.rs::dx2_pending_effect_choice_reap_tests` (T14, T15),
`crates/engine/tests/mechanics_a_d/dredge.rs`,
`crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs`,
`test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json`
**Card defs reviewed**: 1 (`golgari_grave_troll.rs`) — 0 edits, 0 flips, as predicted

## Verdict: needs-fix

**1 HIGH / 7 MEDIUM / 7 LOW.** The engineering core of the batch is right and I could not
break it: the gate is total in the directions that matter (an entry is required, only the
sender's own entry can be consumed, and it is genuinely consumed), the fold's arithmetic
conserves the draw count exactly in all four traced paths, `draw_card_skipping_dredge`'s
deletion actually *improves* the decline path (it now threads the entry's real
`already_applied` / `remaining` / `sets_has_drawn_for_turn` instead of hardcoding
`(∅, 0, true)`), nothing gates progress on `pending_draws` so no hang is introduced, the
`handle_keep_hand` guard validates before mutating and T12 asserts on the message rather
than `is_err()`, the OOS-DP9-14 reap is narrow and T15 proves the `debug_assert!` kept its
teeth, and PROTOCOL 32 / HASH 69 are genuinely unmoved with no declaration changed. **The
golden-script rewrite is legitimate and is the opposite of a weakened assertion** — I
verified the runner's account from `script_schema.rs:576-599` and
`replay_harness.rs:1056-1064` myself (all `TurnBasedAction` entries are informational and
dispatch no `Command`), and the rewrite *adds* a real CR 504.1 Upkeep→Draw transition while
keeping all nine original assertions unchanged.

The HIGH is the residual the fold creates and the batch's own new doc denies: an unanswered
dredge offer is not "the draw simply never happens" — it is a **bank** that accumulates one
owed draw per turn indefinitely and can be cashed in a single command at any moment, with no
priority and no timing check. Six of the seven MEDIUMs are doc-vs-code, and that matters more
than usual here because OOS-DP7-2 *is* the doc-vs-code seed this batch closes: the five sites
the plan enumerated were reconciled correctly, but `PendingDraw`'s own declaration doc, the
`GameState.pending_draws` field doc, `handle_order_replacements`' routing doc and
`memory/gotchas-rules.md` were all left asserting things this batch made false. The seventh
MEDIUM is a genuine coverage loss: the batch's sharpest documented risk (the undiscriminated
queue, plan §3.3's four-case table) has **zero** tests, and `dredge.rs` test 9 now passes for
a different reason, leaving the `Some` arm's graveyard/keyword/library validations uncovered.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `replacement.rs:857-879`, `events.rs:875-876` | **The fold turns an unanswered dredge offer into an unbounded, cashable-at-any-time draw bank, and the new doc says the opposite.** `remaining` accrues one per turn forever; `ChooseDredge` has no priority/timing gate. **Fix:** cap or expire the obligation, or state the true semantics and seed the timing gate. |
| 2 | MEDIUM | `card-types/src/state/replacement_effect.rs:380-405`, `state/mod.rs:141-142` | **`PendingDraw`'s declaration doc and the `GameState` field doc are now false.** They say the entry means "2+ WouldDraw replacements apply" and is "Resolved by `OrderReplacements`". **Fix:** name the dredge push and `handle_choose_dredge`. |
| 3 | MEDIUM | `replacement.rs:124-157` | **`handle_order_replacements`' doc block was not updated** despite plan §4.5 mandating it. Still claims "provably disjoint" candidate sets with no mention of dredge entries or FIFO. **Fix:** add §3.3's four-case table and the FIFO note. |
| 4 | MEDIUM | `tests/primitives/pb_dx2_command_gates.rs` (absent) | **No test pins cross-player rejection, and none of §3.3's four cross-kind cases is tested.** **Fix:** add T17 (p2 cannot consume p1's entry) and T18/T19 (cross-kind consumption). |
| 5 | MEDIUM | `tests/mechanics_a_d/dredge.rs:675-718` | **Test 9 now passes for a different reason; the `Some` arm's three validations lost all coverage.** **Fix:** reach a real offer first, then name a card not in the graveyard. |
| 6 | MEDIUM | `replacement.rs:1357-1394` | **`resolve_pending_draw`'s doc comment was captured by the newly inserted `perform_remaining_draws`.** The helper now advertises behaviour it does not have; `resolve_pending_draw` has no doc at all. **Fix:** re-separate. |
| 7 | MEDIUM | `memory/gotchas-rules.md:30-31` | **Documents the deleted `draw_card_skipping_dredge` as a live helper**, in a file CLAUDE.md mandates reading before touching `rules/`. **Fix:** rewrite for the gated handler. |
| 8 | MEDIUM | `replacement.rs:857-867` (comment) | *(counted under LOW 1 — see below)* |
| 9 | LOW | `replacement.rs:858-867` | **The fold's stated CR 104.4b justification is wrong**: `remaining` is itself hashed into the mandatory-state fingerprint. **Fix:** correct the comment; PB-DP9's exclusion is the precedent. |
| 10 | LOW | `replacement.rs:3079-3090` | **The decline is not sticky.** After a decline re-defers into a `NeedsChoice` entry, `ChooseDredge { Some }` can still dredge the same draw. **Fix:** document or seed. |
| 11 | LOW | `test-data/.../014_...json:205,216,6` | **Surviving false prose in the rewritten script**: "The engine is paused waiting for ChooseDredge", a stale 2026-02-26 DISPUTE note, and a description that still says the state starts in the draw step. **Fix:** reconcile. |
| 12 | LOW | `effects/mod.rs:9279-9284` | Doc still names only `resolve_pending_draw` / `OrderReplacements` as the discharge path. **Fix:** add `handle_choose_dredge`. |
| 13 | LOW | `replacement.rs:868-872` | **Silent no-op**: `if let Some(entry) = get_mut(i)` after a successful `position(i)`; an impossible `None` destroys a draw with no diagnostic. **Fix:** `expect_*`-class diagnostic or index directly. |
| 14 | LOW | `replacement.rs:855-871` | The fold **discards** the folded draw's `already_applied` and `sets_has_drawn_for_turn`; safe only because every `offer_dredge: true` caller passes an empty set — unenforced. **Fix:** assert or document. |
| 15 | LOW | `resolution.rs:8497-8514`, `decision-point-audit.md:967`, `dredge.rs:150` | T15 asserts nothing in release builds; OOS-DX2-2's cite already drifted (`:1402` → `:1394`); `dredge.rs:150` still says "draw is paused". **Fix:** note/repair. |

## Card Definition Findings

None. The roster derivation is correct: exactly one `Complete` def
(`crates/card-defs/src/defs/golgari_grave_troll.rs`, `Dredge(6)`), 0 edits, 0 completeness
flips, `crates/card-defs/src` untouched. The plan's predicted yield was exact.

---

### Finding Details

#### Finding 1: The fold turns an unanswered dredge offer into an unbounded draw bank, and the batch's own new doc denies it

**Severity**: HIGH
**File**: `crates/engine/src/rules/replacement.rs:857-879` (the fold);
`crates/engine/src/rules/events.rs:875-876` (the doc);
`crates/engine/src/rules/engine.rs:534-544` (no timing gate)
**CR Rule**: 504.1 — *"First, the active player draws a card. This turn-based action doesn't
use the stack."*; CR 121.2; CR 614.11a
**Doc the batch wrote**: `events.rs:875-876` — *"An unanswered offer means the draw simply
never happens."*

**Issue.** The `DredgeAvailable` arm folds a second offer into an existing entry with
`entry.remaining += 1 + remaining_after`. Nothing clears `pending_draws` at end of turn,
end of step, or cleanup — I grepped: the field is referenced only in `replacement.rs`,
`state/builder.rs`, `state/hash.rs`, `state/mod.rs` and `loop_detection.rs`, and there is no
reaping site outside `handle_choose_dredge`'s dead-player discharge. And `Command::ChooseDredge`
is admitted with **`validate_player_exists` only** (`engine.rs:538`) — no priority check, no
active-player check, no step check.

Concrete failure scenario, entirely within legal command shapes, on a `Complete`, deck-legal
card:

1. Turn 4: P1's Golgari Grave-Troll hits their graveyard. Library ≥ 6.
2. Turns 5–11, P1's draw step: `draw_for_turn` → `draw_card` → `perform_one_draw(offer_dredge: true)`
   → `DredgeAvailable`. First one pushes `PendingDraw { remaining: 0 }`; each subsequent one
   folds `+= 1 + 0`. After turn 11 the single entry carries `remaining: 6`.
   T8 already proves the engine happily advances turns with the entry outstanding.
3. Turn 12, during **P4's** declare-blockers step, with a combat trick on the stack and P1
   holding no priority, P1 sends `ChooseDredge { player: p1, card: None }`.
4. The gate finds the entry, consumes it, performs one draw plus
   `perform_remaining_draws(state, p1, 6, …)` — **seven cards drawn in one command**, out of
   priority, in another player's combat.

That is an illegal game state under CR 504.1 (the draw-step draw is a turn-based action of
*that* step) and CR 117. It is strictly better than the pre-batch behaviour (unlimited free
cards) and the *conservation* is CR 614.11a-correct, so this is a residual of a closed
exploit rather than a new exploit — but it is reachable, unbounded in magnitude, and the doc
the batch wrote specifically to close a doc-honesty seed asserts the opposite of what the
code does. Neither `OOS-DX2-3` nor any other new seed records it: DX2-3 is about *two entries*
for one player, which is the case the fold prevents; the accumulation *inside* one entry is
the case the fold creates, and it is unrecorded.

Note the single-entry version of the timing hole (deferring one draw to an arbitrary later
moment) is inherent to the deadline design argued in plan §3.4 and is not new. What is new is
that it accumulates without bound, and that a doc now denies it.

**Fix:** three acceptable dispositions, in preference order.
1. Discharge the obligation at a deadline the CR supplies: reap or auto-decline a `PendingDraw`
   whose owner's draw step has passed (the CR 118.12a-style "auto-decline deadline" pattern
   PB-DP4 already uses for echo/cumulative-upkeep/recover). This requires no new stored state.
2. If (1) is out of scope, at minimum **do not fold across a turn boundary** — bound the entry
   so the exploit is capped at one sequence.
3. Whatever is chosen, **correct `events.rs:875-876`**: "an unanswered offer means the draw
   simply never happens" is false; write that the obligation persists, accumulates, and is
   discharged in full whenever the answer eventually arrives, and file a seed for the missing
   timing gate on `Command::ChooseDredge`.

#### Finding 2: `PendingDraw`'s declaration doc and the `GameState.pending_draws` field doc are now false

**Severity**: MEDIUM
**File**: `crates/card-types/src/state/replacement_effect.rs:380-405`;
`crates/engine/src/state/mod.rs:141-142`
**Issue**: The type the entire gate is built on still documents itself as a CR 616.1-only
structure:

- `:383-385` — *"When 2+ `WouldDraw` replacements apply to one draw, the draw does not happen
  and this entry records everything the resume needs. Resolved by `Command::OrderReplacements`;
  see `resolve_pending_draw`."* After PB-DX2 the entry is **also** pushed for a dredge offer
  (where exactly *one* replacement applies) and is **also** resolved by
  `Command::ChooseDredge` / `handle_choose_dredge`.
- `:399-404` — *"`true` for `turn_actions::draw_card` and `replacement::handle_choose_dredge`'s
  decline arm (both set `PlayerState::has_drawn_for_turn`)"*. **PB-DX2 made this false**: the
  decline arm now passes `pending.sets_has_drawn_for_turn` (`replacement.rs:3087`), which is
  `false` for an effect-draw-originated entry. This is a genuine behaviour change the batch
  made (and an improvement), but the doc still describes the deleted
  `draw_card_skipping_dredge`'s hardcoded `true`.
- `state/mod.rs:142` — *"Resolved by `OrderReplacements`."*

This is the OOS-DP7-2 class at the declaration site of the type the batch's fix depends on,
and it is the first place any future reader looks to answer "what pushes this and what
consumes it". The plan's §5 table enumerated five doc sites; these two are the sixth and
seventh and were not in it.
**Fix:** rewrite both doc blocks to name **both** producers (`perform_one_draw`'s
`NeedsChoice` and `DredgeAvailable` arms) and **both** consumers (`resolve_pending_draw` via
`OrderReplacements`, `handle_choose_dredge` via `ChooseDredge`), and correct
`sets_has_drawn_for_turn`'s description to "the flag of the path that raised *this* entry;
the decline arm now replays it rather than forcing `true`."

#### Finding 3: `handle_order_replacements`' doc block was not updated, contrary to plan §4.5

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:124-157`
**Plan**: §4.5 — *"`handle_order_replacements` logic is untouched — only its doc block gains
the dredge case and the FIFO note (§3.3)."*
**Issue**: The doc block is byte-identical to its pre-batch state. It still reasons only about
zone-change-vs-draw routing and concludes *"the two candidate sets are provably disjoint"* —
true for that axis, and the review brief's warning is exactly right that the argument does not
extend to dredge-entry-vs-`NeedsChoice`-entry now that both share the queue. The four-case
soundness table lives only in `pb-plan-DX2.md` §3.3 and (by reference) in
`handle_choose_dredge`'s doc at `:3011-3016`; a reader of `handle_order_replacements` gets no
warning that `position(|p| p.player == player)` at `:205` can now land on a dredge-originated
entry.

I verified the four cases by reading and they are all reachable and all produce legal
outcomes:
- `OrderReplacements` on a dredge entry requires a genuinely applicable `WouldDraw`
  replacement (`:214`, `ids.iter().all(|id| applicable.contains(id))`) and `ids` non-empty
  (`:163`), so a bare "consume the dredge entry for free" is impossible — good.
- `ChooseDredge { None }` on a `NeedsChoice` entry re-defers and pushes a fresh entry
  (traced through `pb_dp5_pending_draw_choice.rs:442-530`, which still passes) — no draw lost.
- `ChooseDredge { Some }` on a `NeedsChoice` entry replaces the draw with dredge and abandons
  the CR 616.1 ordering, which CR 616.1e/616.1f permit.

**Fix:** add the four-case table and the FIFO note to `handle_order_replacements`' doc block,
and state explicitly that a `PendingDraw` no longer implies a CR 616.1 multi-replacement
deferral.

#### Finding 4: The batch's sharpest documented risk has zero tests

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx2_command_gates.rs` (absent tests)
**Issue**: Two properties are load-bearing and untested.

1. **Cross-player rejection.** `handle_choose_dredge`'s gate is
   `position(|pd| pd.player == player)` — correct, a player can only consume their own entry.
   But nothing pins it. This is the trust-boundary property of a trust-boundary batch; its
   sibling on `OrderReplacements` *is* pinned
   (`pb_dp5_pending_draw_choice.rs:537`, `test_dp5_order_replacements_rejects_non_affected_player`).
   A future refactor to `position(|pd| …)` that drops the player predicate would be caught by
   nothing in this suite.
2. **The undiscriminated queue.** Plan §3.3 enumerates four cases and declares the design sound
   on their basis. Only case 3 is exercised, and only incidentally, by a pre-existing PB-DP5
   test. Cases 2 and 4 — `OrderReplacements` landing on a dredge entry, and `ChooseDredge { Some }`
   landing on a `NeedsChoice` entry — have no coverage at all, and case 4 is the one with the
   questionable semantics (Finding 10).

**Fix:** add
`test_dx2_choose_dredge_cannot_consume_another_players_entry` (p1 has an entry, p2 sends
`ChooseDredge` → `Err`, p1's entry still present, p2 drew nothing), and two cross-kind tests
built on `pb_dp5_pending_draw_choice.rs`'s two-`SkipDraw` fixture pinning plan §3.3 rows 2
and 4.

#### Finding 5: `dredge.rs` test 9 now passes for a different reason, and the `Some` arm's validations lost all coverage

**Severity**: MEDIUM
**File**: `crates/engine/tests/mechanics_a_d/dredge.rs:675-718`
(`test_dredge_invalid_command_card_not_in_graveyard`)
**Issue**: The fixture has no pending draw, so after PB-DX2 the command is rejected by the
**gate** at `replacement.rs:3062-3073` and never reaches the graveyard-zone check at `:3117`.
The test's name, doc and CR rationale all say "card not in graveyard", and its only assertion
is `result.is_err()`. The plan predicted this ("stays `Err` under the gate; different message;
the assertion is `is_err()` only") and treated it as a non-event, but the consequence is a
coverage hole: after this batch, **no test in the repo exercises `handle_choose_dredge`'s
`Some`-arm validations at all** — not the graveyard-zone check, not the `Dredge(n)` keyword
check, not the CR 702.52b library-count check. Every remaining `Some` test
(`dredge.rs` tests 2, 7, 8, 12, 13; `golgari_grave_troll.rs:359`; T6) names a *valid* card. The
project's own standard applies here — `memory/conventions.md`, "Test-validity MEDIUMs are
fix-phase HIGHs" — and it is the same standard the plan correctly applied to T12's message
assertion (§8.1) but not here.
**Fix:** rewrite test 9 to reach a real draw-step offer first (`build_upkeep_state` +
`pass_all`, as T3 does), *then* send `ChooseDredge { Some(card_in_hand) }`, and assert on the
message (`"not in"` / `"graveyard"`) so it cannot silently degrade into a gate rejection again.
Add sibling probes for the missing-keyword and short-library branches.

#### Finding 6: `resolve_pending_draw`'s doc comment was captured by the newly inserted helper

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:1357-1394`
**Issue**: `perform_remaining_draws` was inserted between `resolve_pending_draw`'s doc block
(`:1357-1381`) and `fn resolve_pending_draw` (`:1423`) without a separating item. Rust
therefore attaches the whole block — old text plus the new helper's own text at `:1382-1393` —
to `fn perform_remaining_draws` at `:1394`, and `resolve_pending_draw` ends up with **no doc
comment at all**. The captured text claims the function *"Applies the chosen replacement
(emitting `ReplacementEffectApplied` for it **before anything else** — this is the order
discriminator …)"* and describes an `already_applied` growth/termination argument. None of that
is true of `perform_remaining_draws`, which applies no replacement, emits no
`ReplacementEffectApplied`, and passes `HashSet::new()` every iteration. This is the exact
failure mode OOS-DP7-2 is about — a doc comment as the only thing asserting a property the
code does not have — introduced by the batch that closes it. The plan said "lift it into a
private helper *above* `resolve_pending_draw`"; the helper landed above the function but below
its documentation.
**Fix:** move `perform_remaining_draws` (and its own `:1382-1393` doc) above `:1357`, so
`resolve_pending_draw` regains `:1357-1381`.

#### Finding 7: `memory/gotchas-rules.md` still documents the deleted helper

**Severity**: MEDIUM
**File**: `memory/gotchas-rules.md:30-31`
**Issue**: *"`draw_card_skipping_dredge` is a helper that bypasses the replacement check to
avoid re-offering the choice after the player declines."* The function was deleted this batch,
and the sentence is wrong twice over: the replacement check is **not** bypassed by the
replacement (the decline arm calls `perform_one_draw` with `offer_dredge: false` but
`already_applied` from the entry, so other `WouldDraw` replacements *are* re-checked — this is
what `pb_dp5_pending_draw_choice.rs:495-515` asserts). CLAUDE.md's "When to Load What" table
makes this file mandatory reading before touching any file in `rules/`, so it is the highest-
traffic stale reference produced by the batch. The plan's §12 "done" checklist scoped the sweep
to `rg … crates/` → 0, which is satisfied (I verified: zero hits under `crates/`) but which by
construction could not catch this.
**Fix:** rewrite the bullet to describe the gated `handle_choose_dredge` — the offer records a
`PendingDraw`, the answer requires-and-consumes it, and the decline resumes the draw with
`offer_dredge: false` while still re-checking other `WouldDraw` replacements.

#### Finding 9: The fold's CR 104.4b justification is incorrect

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:858-867`
**CR Rule**: 104.4b — *"…somehow enters a 'loop' of mandatory actions… the game is a draw.
Loops that contain an optional action don't result in a draw."*
**Issue**: The comment justifies the fold by claiming *"`pending_draws` is in
`loop_detection::compute_mandatory_state_hash`, so an unbounded per-draw push would make two
structurally identical CR 104.4b positions fingerprint differently and could mask a mandatory
loop."* The premise is right and the conclusion does not follow: `loop_detection.rs:161-163`
hashes each `PendingDraw` **in full**, `remaining` included, so the fold converts unbounded
`Vector` growth into unbounded `u32` growth and the fingerprint still diverges on every
iteration. The codebase already knows this hazard and solved it the other way: PB-DP9
deliberately **excluded** `pending_effect_choice` / `effect_choice_answers` /
`next_effect_choice_id` from the mandatory-state hash for precisely this reason
(`loop_detection.rs:111-120`), so the precedent contradicts the comment.

Practical impact is small and arguably nil: the only positions whose fingerprint now drifts
are loops that repeatedly draw for a dredge-holder, and such a loop **contains an optional
action** (the dredge offer), so CR 104.4b's second sentence says it is not a draw anyway. But
the reasoning as written is wrong, and `OOS-DX2-3` repeats the same incomplete claim
("unbounded growth could in principle mask a CR 104.4b mandatory loop") without noting that
the fold does not close that channel.
**Fix:** correct the comment to state the real reason for the fold (conserving the draw and
keeping one obligation per player), record that `remaining` is itself hashed so the
loop-detection channel is *not* closed, and amend OOS-DX2-3 accordingly.

#### Finding 10: The decline is not sticky

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:3079-3090`
**CR Rule**: 616.1e / 616.1f; 702.52a
**Issue**: `handle_choose_dredge`'s `None` arm passes `offer_dredge: false` with the comment
*"the player just declined for THIS draw, so re-offering would loop"*. If that resume hits
`NeedsChoice`, a fresh `PendingDraw` is pushed — and because the queue carries no
discriminator, the player can immediately send `ChooseDredge { Some(the_same_card) }` and
dredge the draw they just declined. The engine is internally inconsistent: `perform_one_draw`
refuses to re-offer dredge after a decline, while `handle_choose_dredge` will apply it on
request. `dredge.rs:722-795` (test 10) asserts the invariant in the event stream only, so it
does not catch this.

CR 616.1f says the process repeats "taking into account only replacement effects that would
now be applicable", and nothing consumed dredge, so the outcome is defensible — plan §3.3 row
4 argues exactly this. But the divergence between the two code paths is undocumented at the
`None` arm, and it is untested (Finding 4).
**Fix:** document the asymmetry at `:3079-3090` — "declining suppresses the automatic re-offer
for this draw; it does not make dredge inapplicable, and CR 616.1e still permits the player to
choose it via an explicit `ChooseDredge { Some }` on the re-deferred entry" — and add the
test from Finding 4 case 4.

#### Finding 11: Surviving false prose in the rewritten golden script

**Severity**: LOW
**File**: `test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json:6, 205, 216`
**Issue**: The rewrite itself is correct and I verified its premise independently (see the
"golden script" section below), but three pieces of prose in the file the batch rewrote were
not reconciled:
- `:205` — *"The engine is paused waiting for ChooseDredge."* This is the literal claim
  OOS-DP7-2 exists to eliminate, surviving in the very file the batch touched to close it.
- `:216` — *"DISPUTE: choose_dredge is not supported in translate_player_action — this action
  is silently skipped by the harness."* Resolved on 2026-02-26 per the file's own first
  dispute entry; a reader of the note would conclude the script is broken.
- `:6` — the metadata description still says *"It is P1's draw step"*, but the rewritten
  `initial_state` starts at Upkeep.

**Fix:** replace `:205`'s note with the deadline wording (the draw does not occur, a
`PendingDraw` is recorded, the engine does not block), delete the stale DISPUTE sentence at
`:216`, and update `:6` to describe the Upkeep start.

#### Findings 12–15 (LOW, brief)

- **12** — `effects/mod.rs:9279-9284` still says the recorded remaining draws are *"performed
  by `resolve_pending_draw` once the player answers `Command::OrderReplacements`"*. After
  PB-DX2 this path's own deferral is usually a dredge offer discharged by
  `handle_choose_dredge`. **Fix:** name both.
- **13** — `replacement.rs:868-872`: `if let Some(entry) = state.pending_draws.get_mut(i)`
  guards an index just returned by `position(i)`. The `None` branch is unreachable and silently
  drops the draw with no diagnostic — the CR 614.11a bug class the batch fixes, re-introduced
  as an unreachable branch. `replacement.rs` is outside SR-4's enumerated scope
  (`effects/mod.rs` + `rules/resolution.rs`), so this is not a gate violation.
  **Fix:** index directly (`position` guarantees the index) or route through an `expect_*`
  diagnostic.
- **14** — the fold computes `sorted` and then discards it, and likewise discards the folded
  draw's `sets_has_drawn_for_turn`. Both are safe only because every caller that passes
  `offer_dredge: true` also passes `HashSet::new()` (`turn_actions.rs:1252-1259`,
  `effects/mod.rs:9295-9302`) — a convention nothing enforces, and
  `check_would_draw_replacement` checks dredge *before* consulting `already_applied` at all
  (`replacement.rs:665-700`). **Fix:** `debug_assert!(already_applied.is_empty())` in the
  `DredgeAvailable` arm, or fold the sets.
- **15** — three small ones: (a) `resolution.rs:8498`'s `#[cfg_attr(debug_assertions,
  should_panic)]` means T15 executes with **no assertion whatsoever** in a release test run —
  correctly documented at `:8503-8506`, worth noting since T15 is the batch's only proof that
  the reap is narrow; (b) `OOS-DX2-2` already cites `resolve_pending_draw:1402` for
  `perform_remaining_draws`, which is at `:1394` — a fresh instance of the OOS-DP6-8
  documentation-rot class inside the closure paragraph that boasts about correcting two
  others; (c) `dredge.rs:150` still comments "draw is paused waiting for ChooseDredge".

---

## Focus-Area Verdicts

### 1. Is the gate actually a gate? — YES, and I could not break it

- **Require**: `replacement.rs:3062-3073`. No entry ⇒ `Err(InvalidCommand)`. Entries are pushed
  only by `perform_one_draw`'s two arms (`:857-879`, `:883-900`), each of which represents
  exactly one un-performed draw, so an entry can never be conjured without an owed draw.
- **Consume**: both arms clone-then-`remove(idx)` (`:3077-3078`, `:3156-3157`). T3 pins it.
- **Own player only**: `position(|pd| pd.player == player)`. Correct; untested (Finding 4).
- **`NeedsChoice` entry consumed by `ChooseDredge`**: reachable, produces a legal outcome in
  both `None` (re-defers, nothing lost — pinned incidentally by
  `pb_dp5_pending_draw_choice.rs:442-530`) and `Some` (dredge replaces the draw, CR 616.1e).
  See Finding 10 for the one questionable sub-case.
- **Dredge entry consumed by `handle_order_replacements`**: reachable **only** when a genuinely
  applicable `WouldDraw` replacement exists, because `ids` must be non-empty (`:163-167`) and
  every id must pass `find_applicable` against the entry's `already_applied` (`:213-214`). A
  bare consume-for-free is impossible. The DP-5 disjointness argument does **not** survive as
  stated, but the property it was protecting does, by a different mechanism (applicability of
  the *named ids*, not of the entry kind). This needs to be written down — Finding 3.

### 2. The fold — conserves, does not lose or resurrect a count

Arithmetic verified in all four paths. Entry represents `1 + remaining` owed draws; fold adds
`1 + remaining_after` (the folded draw plus its own tail); every consumer performs
`1 + pending.remaining` draws (or `dredge + pending.remaining`). Traced:
draw-step(1) + effect(2) ⇒ `remaining` 0 → 2 ⇒ 3 draws on decline — exactly T7's assertion.
`u32` overflow is not reachable in practice. It **can** carry a stale entry across turns —
that is Finding 1, and it is a semantics problem, not an arithmetic one.

### 3. `draw_card_skipping_dredge` deletion — correctly re-routed, and improved

Zero references remain under `crates/` (only `memory/` — Finding 7). The decline arm
(`:3083-3090`) discharges all three documented contract clauses and improves one:
(a) other `WouldDraw` replacements **are** re-checked (`perform_one_draw` →
`check_would_draw_replacement` with the entry's `already_applied`), pinned by
`pb_dp5_pending_draw_choice.rs:503-510`; (b) dredge is **not** re-offered (`offer_dredge: false`),
pinned by `dredge.rs:773-781`; (c) `sets_has_drawn_for_turn` is now the *entry's* flag rather
than a hardcoded `true` — a deliberate improvement (the old helper wrongly set
`has_drawn_for_turn` on effect-draw declines), but it falsifies `PendingDraw`'s doc (Finding 2).
The dead-player guard is preserved and widened at `:3042-3058`.

### 4. No hang — CONFIRMED

`pending_draws` is referenced in exactly five source files (`replacement.rs`,
`state/builder.rs`, `state/hash.rs`, `state/mod.rs`, `loop_detection.rs`). **Zero** references
in `rules/engine.rs` — so `blocking_decision`, the admission gate, `enter_step`,
`handle_all_passed` and the SBA loop are all untouched. No `BlockingDecision` variant was
added. T8 exercises six full priority rounds with an entry outstanding and asserts
`blocking_decision().is_none()` throughout; `pb_dp5_pending_draw_choice.rs:1199-1221` still
passes. A fuzzer/bot game with a dredge card in a graveyard completes exactly as before, losing
the draw (now recorded rather than destroyed — correctly seeded as OOS-DX2-5).

### 5. The golden-script rewrite — legitimate, and it strengthens the script

I verified the runner's account independently rather than accepting it:
- `script_schema.rs:594-599` — *"No replay driver (`replay_harness`, `script_replay`,
  replay-viewer) currently reads this field; all `TurnBasedAction` entries are treated as
  informational and dispatch no engine `Command`."*
- `replay_harness.rs:1056-1064` — the same contract stated from the harness side.

So the account is exactly right: the old script's `turn_based_action: draw_card` label never
caused a draw, the script's `initial_state` began *inside* the draw step with no step-entry
transition, and its `choose_dredge` succeeded purely on the ungated path PB-DX2 closes. The
script was a live reproduction of the exploit.

**SR-9c check: no assertion was weakened or deleted.** Old: 6 individual checks in the first
three actions + 3 in the post-dredge assert = 9. New: 3 + 3 + 3 = 9, with the same paths and
the same expected values (`hand.p1.count` 2 / 2 / 3, GGT `includes` / `includes` / `excludes`,
`stack.is_empty`). What changed is the *mechanism* reaching the offer: `initial_state` now
starts at `phase: "upkeep"` (`parse_step` at `replay_harness.rs:2341` maps it to `Step::Upkeep`)
and a leading `priority_round { all_pass }` drives the real Upkeep→Draw transition and its
CR 504.1 turn-based action. That is strictly more coverage than before. The append-only dispute
entry is accurate on every point I could check, and the original 2026-02-26 entry is preserved.
The only defect is surviving prose (Finding 11).

### 6. The OOS-DP9-14 reap — correct, narrow, and T15 is not vacuous

`resolution.rs:105-114` clears **only** when `expect_player(entry.player)` reports
`has_lost || has_conceded` (and, defensively, when the player is absent). It sits above the
`debug_assert!` at `:115-119`, and above `let restart_point = state.clone()` at `:120` — so
the wholesale rollback at `:142` cannot resurrect a reaped entry. `effect_choice_answers` is
cleared alongside, which is right at a resolution entry point.

T15 (`:8497-8514`) is not vacuous **for its stated purpose**: it builds the identical fixture
with a *live* owner and asserts the `debug_assert!` still panics, which is exactly the property
that would break if someone replaced the liveness predicate with an unconditional clear. It
passes before and after by design. Its only weakness is that it asserts nothing in release
builds (Finding 15a). T14 (`:8461-8495`) is non-vacuous: it fails pre-fix with the assert
panic, and post-fix asserts not only that the field is cleared but that
`GameEvent::AbilityResolved` was emitted — i.e. that resolution actually proceeded rather than
returning `Ok(vec![])`.

### 7. The `handle_keep_hand` guard — correct

`commander.rs:911-947`. All validation precedes all mutation: the count check (`:902-910`), then
the scoped hand-membership + duplicate block (`:921-947`), then the move loop (`:948-952`). A
rejected command leaves the state untouched. The duplicate check is real and T12
(`pb_dx2_command_gates.rs:585-629`) asserts `msg.contains("twice")`, which is the non-vacuous
form the plan demanded — an `is_err()` probe would indeed have passed pre-fix via
`ObjectNotFound` after already bottoming one card (CR 400.7). `bare_lookup_ratchet`'s ceilings
are unmoved (`tests/core/bare_lookup_ratchet.rs:112` resolution.rs 100, `:137` replacement.rs
24, `:180` commander.rs 6) and the guard adds no bare lookup (`expect_zone`, the NONSWALLOW
helper). CR 103.5's *"puts a number of **those cards**"* is correctly read as "the cards of the
hand", MCP-verified. The residual (no pregame phase gate; `TakeMulligan` untouched and worse)
is correctly out of scope and correctly seeded as OOS-DX2-4 — I verified `engine.rs:477-489`
does check only `validate_player_exists` for both.
Minor: `handle_keep_hand`'s own doc block (`:878-890`) does not mention the new validation.

### 8. The doc reconciliation (OOS-DP7-2) — five sites done, four more in the same family left

Read in shipped state:
- `replacement.rs:617-625` (`DrawAction::DredgeAvailable`) — **honest**.
- `events.rs:856-878` (`DredgeChoiceRequired`) — honest *except* the final sentence
  ("the draw simply never happens"), which Finding 1 shows is false.
- `replacement.rs:761-782` (`DrawStepOutcome::DredgeOffered`) — **honest**, correctly reversed
  ("the caller MUST STOP").
- `events.rs:1378-1389` (`CleanupDiscardChoiceRequired`) — **honest**; the block-vs-deadline
  contrast is preserved and the "not implemented" claim is correctly dropped.
- `events.rs:832-855` (`MiracleRevealChoiceRequired`) — **honest, and better than required**:
  it does not merely drop the false pause, it states the verified gate gap and points at
  OOS-DX2-1. I confirmed the underlying claim by reading `miracle.rs:44-106`: the handler
  validates hand-zone, the Miracle keyword and `cards_drawn_this_turn == 1`, and never checks
  that `card` is the object just drawn.

**But the seed's own criterion — "a doc comment is the only thing asserting the property" —
is not fully met.** Four more sites in the same family still assert things the shipped code
does not do, three of them made false *by this batch*: `PendingDraw`'s declaration doc and the
`GameState.pending_draws` field doc (Finding 2), `handle_order_replacements`' routing doc
(Finding 3), `memory/gotchas-rules.md:30` (Finding 7), plus `effects/mod.rs:9282` and the
golden script's note (Findings 12, 11). I would call OOS-DP7-2 **closed with residual** rather
than closed.

### 9. Test non-vacuity

| test | fail-before? | what would make it fail | verdict |
|---|---|---|---|
| T1 | yes (runner log) | removing the gate | non-vacuous; panics with a state-delta message, not a bare `is_err()` |
| T2 | yes | removing the gate | non-vacuous; asserts hand/graveyard/library deltas in the failure text |
| T3 | added after fix | not removing the entry in the `None` arm | **non-vacuous** — pre-fix the second answer drew a second card; it also asserts the first answer emitted `CardDrawn` and emptied the queue |
| T4 | new | not pushing on `DredgeAvailable` | non-vacuous; pins `player`, `remaining == 0`, empty `already_applied` |
| T5 | yes | removing `DredgeOffered` from `draw_cards_for_player`'s break set (offer count → 3) or breaking the resume (total → 1) | strongest test in the file |
| T6 | added after fix | dropping `perform_remaining_draws` from the `Some` arm (drawn → 0) | **non-vacuous** |
| T7 | new | changing the fold to a push (len → 2) or getting the arithmetic wrong (`remaining ≠ 2`, total ≠ 3) | non-vacuous; pins Finding-2-adjacent conservation |
| T8 | passes both | adding any `pending_draws` progress gate | intentionally a guard test; its value is post-fix |
| T9 | added after fix | removing step 0's discharge (entry survives / command errors) | **non-vacuous** |
| T10/T11 | yes | removing the hand-zone guard | non-vacuous; assert the object did not move as well as the `Err` |
| T12 | yes, **on the message** | reordering validation after the move loop | non-vacuous; the plan's vacuity trap was correctly avoided |
| T13 | passes both | over-tightening the guard | genuine non-regression |
| T14 | yes (assert panic) | removing the reap | non-vacuous; also asserts `AbilityResolved` |
| T15 | passes both | making the reap unconditional | non-vacuous for its purpose; no assertion in release |
| T16 | passes both by design | any wire bump | the AC pin |

The four the runner added post-fix (T3/T6/T8/T9) all hold up: T3, T6 and T9 each assert a
post-condition that the pre-fix code demonstrably violates, and T8 is a guard test that is
supposed to pass in both directions. **The gaps are Findings 4 and 5**, not vacuity.

### 10. Invariants

| Invariant | Status | Evidence |
|---|---|---|
| **SR-3** (`GameState` sealed) | ✅ | `pending_draws` stays `pub(crate)` (`state/mod.rs:144`); the only accessor is read-only (`:489`); T14/T15 are in-src precisely to avoid widening the seal, per plan §9.3. No test-only setter added. |
| **SR-8** (wire closure) | ✅ | `PROTOCOL_VERSION = 32` (`protocol.rs:335`), `HASH_SCHEMA_VERSION = 69` (`hash.rs:679`), both unchanged; no struct, field, enum variant or field type changed anywhere (`PendingDraw`, `Command::ChooseDredge`, `GameEvent::DredgeChoiceRequired` all byte-identical in declaration). T16 pins both. |
| **SR-9b** (determinism) | ✅ | `already_applied` is sorted by `ReplacementId` before storage at both push sites (`:855-856`, `:892-893`); entries are appended with `push_back` and located with `position` (first match); the fold mutates in place. Nothing iterates a `HashMap`/`HashSet` to an outcome on any new path. |
| **SR-9c** (no silent script skip / no weakened assertion) | ✅ | Script rewrite strengthens; assertion count and values unchanged; append-only dispute entry added with CR citations. |
| **SR-25** (`bare_lookup_ratchet`) | ✅ | Ceilings unmoved; new code uses `expect_player` / `expect_zone` / `expect_move_object_to_zone`. |
| **SR-4** (silent failures classified) | ⚠️ | The one new unclassified silent no-op (`get_mut` at `:868-872`) is in `replacement.rs`, outside SR-4's enumerated scope. Finding 13. |
| **Arch. Invariant 4** (no phantom events) | ⚠️ | `DredgeChoiceRequired` is emitted on a fold even though no new entry is created. Defensible — the offer is genuinely answerable via the folded entry, and T7 pins the 1-event/1-entry pairing — but worth stating in the doc. Sub-case of Finding 1. |
| **Arch. Invariant 8** (tests cite rules) | ✅ | Every new test carries a CR citation. |
| **§14 collision surface** (`scutemob-163`) | ✅ | No edits to `lib.rs`, `rules/mod.rs`, `casting.rs` or `tests/rules/main.rs`; the new test lives in the `primitives` target. |

## CR Coverage Check

| CR Rule | Verified verbatim | Implemented? | Tested? | Notes |
|---------|-------------------|--------------|---------|-------|
| 702.52a | ✅ MCP | Yes | T1, T2, T3, T6 | "if you would draw a card" is now a precondition, not decoration |
| 702.52b | ✅ MCP | Yes | T2 (indirectly) | library ≥ n re-validated at answer time (`:3141-3153`) |
| 614.11 | ✅ MCP | Yes | — | unchanged |
| 614.11a | ✅ MCP | Yes | T5, T6, T7 | the sequence now stops at the offer and resumes after; P3's bug closed |
| 614.11b | ✅ MCP | n/a | — | not in scope |
| 616.1 / 616.1e / 616.1f | ✅ MCP | Partially | — | the four-case soundness argument is untested (Finding 4) and undocumented at `handle_order_replacements` (Finding 3) |
| 121.2 / 121.2c | ✅ | Yes | T5, T7 | one-at-a-time draws preserved |
| 103.5 / 103.5c | ✅ MCP | Yes | T10, T11, T12, T13 | "those cards" correctly read as the hand |
| 504.1 | ✅ MCP | **Partially** | golden 014 | the draw-step draw can be deferred and banked indefinitely — Finding 1 |
| 608.2d | ✅ | Yes (reap) | T14, T15 | |
| 104.4b | ✅ MCP | Unchanged | — | the fold's justification is wrong (Finding 9); CR 726 correctly identified as "Restarting the Game" — OOS-DX2-6 verified accurate, all seven subrules are restart procedure |
| 400.7 | ✅ | Yes | T12, T13 | duplicate-id classification |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `golgari_grave_troll` | Yes (`Dredge(6)`, `Complete`) | 0 | Yes | Untouched. Roster derivation confirmed the plan's prediction exactly: 1 distinct `Complete` def carrying `KeywordAbility::Dredge(_)`, 0 flips, 0 def edits, `crates/card-defs/src` diff empty. |

## Seed Assessment

### Closure claims — verified against shipped code, not banners

| Seed | Claim | Verdict |
|---|---|---|
| **OOS-DP5-7** | closed by require-and-consume | **TRUE.** Gate at `replacement.rs:3062-3073`, consume at `:3077-3078` / `:3156-3157`, own-player predicate correct. `card: None` is no longer a free card and `card: Some` no longer dredges at will. **Residual: Finding 1** (a bounded, bankable, out-of-priority multi-draw remains) — the closure should say so. |
| **OOS-DP7-2** | closed by reconciling five doc sites | **TRUE for the five sites named**, all read and confirmed honest, including the third (`events.rs:1378-1389`) the seed did not name and the miracle site where the batch verified rather than guessed. **CLOSED WITH RESIDUAL** — four more sites in the same family remain, three of them falsified *by this batch* (Findings 2, 3, 7, plus 11/12). |
| **OOS-DP2-1** | closed by per-entry hand guard + duplicate check | **TRUE** for the cross-zone/cross-player half, which was the seed's content. Validation precedes mutation; T12 asserts the message. The phase-gate half is correctly carved out and re-seeded as OOS-DX2-4. Stale cite `:877-885` → `:891` correctly applied. |
| **OOS-DP9-14** | closed defensively and narrowly | **TRUE.** `resolution.rs:105-114`, above the assert, above the `restart_point` clone, dead-owner only, pinned in both directions. The cite correction (`drop_departed_trigger_flush`'s `engine.rs:2664` is inside `handle_concede`, not the top of `resolve_top_of_stack`) is accurate — I confirmed `discharge_effect_choice_on_concede`'s only caller is `handle_concede`. |

### New seeds OOS-DX2-1..6 — all real, none a restatement

| Seed | Real? | Verification |
|---|---|---|
| **OOS-DX2-1** (miracle not gated on the offer) | ✅ | Read `miracle.rs:44-106`: hand-zone + Miracle keyword + `cards_drawn_this_turn == 1`, and **no** check that `card` is the just-drawn object. CR 702.94a's "as you draw it" is genuinely unenforced. Correctly classified live + HASH-bumping. |
| **OOS-DX2-2** (tail draws never re-offer dredge) | ✅ | `perform_remaining_draws:1406` passes `offer_dredge: false` unconditionally. Cite has already drifted (`:1402` → `:1394`) — Finding 15b. |
| **OOS-DX2-3** (two entries per player; FIFO routing) | ✅ but **incomplete** | The `NeedsChoice` arm at `:894` still pushes unconditionally, so two entries per player remain reachable. The CR 104.4b clause is however wrong for the reason in Finding 9, and the seed does not record the *intra-entry* `remaining` growth channel the fold creates. |
| **OOS-DX2-4** (no pregame gate on `KeepHand`/`TakeMulligan`) | ✅ | Confirmed at `engine.rs:477-489`: `validate_player_exists` only for both. `TakeMulligan` is indeed the worse of the two. |
| **OOS-DX2-5** (bots never dredge) | ✅ | Consistent with plan P4 and with the observed absence of any `ChooseDredge` producer outside `crates/engine`. |
| **OOS-DX2-6** (CR 726 is the wrong cite) | ✅ | MCP-verified: CR 726 is "Restarting the Game" (Karn Liberated), all seven subrules restart procedure; CR 104.4b is the mandatory-loop rule, verbatim as quoted. `loop_detection.rs:1` still carries the wrong cite, correctly left to the sweep. |

**Missing seed**: Finding 1's accumulation-and-timing residual is recorded nowhere. It should
be filed (or fixed) rather than left in the gap between OOS-DX2-3 (two entries) and OOS-DX2-5
(bots don't answer).

## Previous Findings

Not applicable — first review of PB-DX2.

---

## Fix cycle (2026-08-01, `scutemob-162`, same day)

All 15 findings dispositioned. Nothing was argued down without a code change — the closest to
that is Findings 9/13/14, which the Finding 1 rewrite made moot (the code they described no
longer exists), and Finding 15a, which needed no action because it was already correctly
documented.

| # | Sev | Disposition | Where |
|---|---|---|---|
| 1 | HIGH | **FIXED — design changed from fold to discharge**, not any of the review's three listed options verbatim, argued below. | `replacement.rs::perform_one_draw`, new `resolve_declined_pending_draw` |
| 2 | MED | FIXED — `PendingDraw` doc + `GameState.pending_draws` field doc rewritten to name both producers and both consumers, and the `sets_has_drawn_for_turn` divergence corrected. | `card-types/src/state/replacement_effect.rs`, `engine/src/state/mod.rs` |
| 3 | MED | FIXED — four-case table + FIFO note added to `handle_order_replacements`'s doc, plus a note that `OOS-DX2-3`'s FIFO concern is now vacuous (Finding 1's fix). | `replacement.rs::handle_order_replacements` |
| 4 | MED | FIXED — 3 new tests: `test_dx2_choose_dredge_cannot_consume_another_players_entry` (case 1), `test_dx2_order_replacements_can_answer_a_dredge_originated_entry` (case 2), `test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry` (case 4, doubles as Finding 10's pin). Case 3 was already covered incidentally. | `pb_dx2_command_gates.rs` T17/T18/T19 |
| 5 | MED | FIXED — test 9 rewritten to reach a real draw-step offer first (`build_upkeep_state` + `pass_all`), then names a hand card as the dredge target, and asserts on the message (`.contains("graveyard")`) rather than `is_err()`. | `dredge.rs::test_dredge_invalid_command_card_not_in_graveyard` |
| 6 | MED | FIXED — `perform_remaining_draws` relocated ABOVE `resolve_pending_draw`'s doc block, restoring `resolve_pending_draw`'s own doc and giving `perform_remaining_draws` its own. A note explaining the Rust doc-attachment hazard is left in place so this class of bug is not reintroduced blind. | `replacement.rs` |
| 7 | MED | FIXED — bullet rewritten for the gated handler; explicitly corrects the "bypasses the replacement check" claim the review flagged as double-wrong. | `memory/gotchas-rules.md:26-36` |
| 8 | (dup of 9) | No separate action — the review's own table marks this "counted under LOW 1", i.e. LOW Finding 9. Resolved together with 9. | — |
| 9 | LOW | MOOT, not merely fixed — the fold code the wrong CR 104.4b justification was attached to no longer exists after Finding 1's rewrite. Verified by grep: no surviving `fold` comment or CR 104.4b mandatory-loop justification anywhere in `replacement.rs`. | `replacement.rs` |
| 10 | LOW | FIXED (documented, not changed — the review's own recommendation) — explicit CR 616.1e-cited note added to `handle_choose_dredge`'s function doc explaining the decline is not sticky, and pinned by new test T19 (Finding 4/case 4). | `replacement.rs::handle_choose_dredge` doc; T19 |
| 11 | LOW | FIXED — all three stale prose sites in the golden script rewritten (`:6` upkeep-start description, `:205` "paused" claim, `:216` resolved DISPUTE deleted from the note text — the append-only dispute LOG entries themselves are untouched, only the still-live `note` field that repeated the stale claim). | `replacement/014_golgari_grave_troll_dredge.json` |
| 12 | LOW | FIXED — doc now names both `resolve_pending_draw`/`OrderReplacements` AND `handle_choose_dredge`/`ChooseDredge`. | `effects/mod.rs::draw_cards_for_player` |
| 13 | LOW | MOOT — the `get_mut(i)` silent no-op no longer exists; `position()` → discharge → unconditional `push_back` replaced the fold's `get_mut` mutation entirely. Verified: `grep -n get_mut crates/engine/src/rules/replacement.rs` → 0 hits. | `replacement.rs` |
| 14 | LOW | MOOT — the fold that discarded `sorted`/`already_applied`/`sets_has_drawn_for_turn` no longer exists; the new code always uses what it computes (either the discharge reads the OLD entry's own fields, or the fresh push uses the CURRENT call's fields — nothing is ever computed and thrown away). | `replacement.rs` |
| 15a | LOW | No action — already correctly documented (`resolution.rs:8499-8506`) that T15 asserts nothing in release builds. Verified the doc is still accurate post-fix-cycle (unrelated code). | `resolution.rs` |
| 15b | LOW | FIXED — `OOS-DX2-2`'s cite corrected `resolve_pending_draw:1402` → `perform_remaining_draws:1495` (the function's line moved again during the fix cycle's Finding 6 relocation; re-verified against the actual post-edit line, not copied from the implement-phase number). | `docs/audits/decision-point-audit.md` |
| 15c | LOW | FIXED — `dredge.rs:150`'s comment and its test's doc comment (line ~108) both corrected to drop "paused". | `dredge.rs` |

### Finding 1 — the design decision, argued

The review offered three dispositions in preference order: (1) reap/auto-decline at a
CR-defensible deadline (draw step passed), (2) do not fold across a turn boundary (bound the
entry), (3) document the true semantics and seed the gap. **What shipped is closest to (1) and
(2) combined, realised as a single mechanism** — discharge-on-next-draw-event — rather than
either verbatim:

- It is **not** a time-based deadline (no "has this player's draw step passed" check, no new
  stored turn/step marker) because that would need to record *when* the entry was created —
  new state on `PendingDraw` — which is a HASH bump and violates the hard constraint (PROTOCOL
  32 / HASH 69 pinned).
- It is **not** simply "cap the fold's arithmetic at 1" (a literal reading of option 2) because
  that would still fold — still let TWO logically distinct draws share one entry with muddled
  bookkeeping — rather than resolving the earlier one honestly. Discharge-then-push keeps every
  entry's fields describing exactly one offer.
- It **is** data-driven off the same event that made the old fold trigger in the first place
  (a second draw for a player who already owes an answer), which is exactly what option 1 asked
  for ("a deadline the CR supplies") — the deadline here is "your own next draw", which needs no
  new state to detect because `perform_one_draw` is already the single call site every draw of
  every kind passes through.
- **Why unconditional at the top of the function, not nested inside the `DredgeAvailable` arm**
  (the narrower reading of "cap the fold"): gating the discharge on the CURRENT draw also
  turning out to be a dredge offer would leave a gap if the dredge card left the graveyard
  between offers (exiled by another effect, discarded via some other mechanism) — the code
  would then never re-enter that arm on a later draw, and the stale entry would sit forever,
  reproducing the exact "never cleaned up" complaint `OOS-DP5-2` already files. Placing it
  unconditionally closes that gap too, and, found while implementing rather than prescribed by
  the review, this also makes it structurally impossible for two entries to ever coexist for
  one player (**OOS-DX2-3 CLOSED** — every `pending_draws.push_back` site is downstream of the
  discharge, and there are only two such sites, both inside this one function).
- **What is deliberately NOT fixed, and why that is correct rather than a gap**: the review's
  own §3.4 argument (quoted in the runner's brief) already concedes the single-entry version of
  "answerable at an arbitrary later moment, out of priority" is *inherent to the deadline
  design, not new* — blocking on `ChooseDredge` was rejected for concrete reasons (deadlocks
  every bot/fuzzer game, since `crates/simulator` constructs no `ChooseDredge`). This residual
  is already filed as `OOS-DP5-2` ("no deadline for `pending_draws`... hands the drawing player
  unbounded timing control over an owed draw") — the runner did NOT file a new seed for it
  (the review flagged this as a possible gap: "It should be filed (or fixed)"), because filing a
  second seed for the same residual `OOS-DP5-2` already names verbatim would be the
  `OOS-DP7-1`-class duplication this program's seed hygiene exists to avoid. `OOS-DP5-2`'s row
  was instead amended to note the fix cycle narrowed (not closed) it.

### Verification

- `git diff --stat -- crates/engine/src/rules/protocol.rs crates/engine/src/state/hash.rs` →
  empty. `cargo test -p mtg-engine --test core -- protocol_schema hash_schema` → 38 passed, 0
  failed, both fingerprints unmoved (`PROTOCOL_VERSION == 32`, `HASH_SCHEMA_VERSION == 69`,
  pinned by T16 as before).
- `cargo test --all` → **3,974 passing, 0 failing** (baseline for this fix cycle was 3,971;
  +3 net from T17/T18/T19; T7 rewritten in place, not a net-new test).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --check` → clean (one `cargo fmt` pass was needed mid-cycle for the restructured
  `perform_one_draw` match arms and a wrapped `CardRegistry::new` call in T17; re-verified
  clean after).
- `tools/check-defs-fmt.sh` → clean, 1,804 defs.
- `cargo build --workspace` → clean (simulator / network / tui / replay-viewer).
- 211/211 golden scripts unaffected by this fix cycle (only `replacement/014`'s prose changed,
  no assertion touched); not independently re-run beyond the `cargo test --all` pass, which
  includes the golden-script harness test target.

### What the runner found that the review did not

Two more "the engine pauses" doc sites in the same family, not named by any of the 15 findings:
`check_would_draw_replacement`'s own doc comment (`replacement.rs`, "the engine pauses for the
player's choice") and — new, not a correction of an existing lie — a "Per-player invariant"
section added to `perform_one_draw`'s doc explaining the discharge design and its relationship
to `OOS-DX2-3`/`OOS-DP5-2`, since no doc anywhere stated the per-player uniqueness invariant the
fix now depends on. Both fixed in the same pass as Findings 2/3/7/12 since they are the same
class of drift.

---

## Re-review (fix cycle) — 2026-08-01, `scutemob-162`

**Reviewer**: primitive-impl-reviewer (Opus), read-only
**Scope**: the fix-cycle diff only (`0fbf5d94..HEAD`), reviewed as new work
**CR re-verified via MCP this pass**: 702.52 / 702.52a / 702.52b, 614.11 / 614.11a / 614.11b,
616.1 / 616.1a–g, 121.1, 104.3 (all subrules), 104.4b

### Verdict: needs-fix — **1 HIGH / 3 MEDIUM / 5 LOW**

**The original HIGH is genuinely resolved.** The unbounded, cashable-at-any-time draw bank is
gone: `remaining` can no longer accumulate across turns, because `perform_one_draw` discharges
the stale entry rather than folding into it, and the entry that survives always carries at most
the *current* sequence's own remainder. T7 was rewritten to pin exactly that (`remaining == 1`,
not `2`; `discharge_drawn == 1`; conservation total `3`), and it is strongly non-vacuous — the
fold design fails three of its assertions. The residual (a *single* entry answerable at an
arbitrary later moment with no priority/step check) is now stated plainly in
`events.rs:877-892`, is correctly attributed to the pre-existing `OOS-DP5-2` rather than
duplicated as a new seed, and `OOS-DP5-2`'s row was amended honestly — it says what the
discharge does close ("player keeps playing, keeps drawing" self-heals) and what it does not
(loss, source-leaves-battlefield, never-draws-again, and the timing half). That amendment is the
right call and is better than filing a duplicate.

**The cure did not introduce anything worse than the disease, but it did introduce one HIGH.**
The batch now asserts, in **seven** places including two engine doc blocks the code's own FIFO
and termination arguments lean on, that at most one `PendingDraw` entry can exist per player —
and marks `OOS-DX2-3` **CLOSED** on that basis. The invariant is false. The runner's stated
argument is the exact fallacy the review brief predicted: it reasons about *push sites* ("both
`push_back` sites live inside `perform_one_draw`, both downstream of the discharge") when the
question is about *push order*. `resolve_declined_pending_draw` re-enters `perform_one_draw`,
and that inner call's own `NeedsChoice` push happens **between** the discharge and the outer
push. Exposure in a real game today is **zero** (no card def registers a `WouldDraw`
replacement — the audit's own DP-5 and OOS-DP5-3 rows say so, and I re-derived it: the single
corpus hit is an `inert` completeness *note* in `out_of_the_tombs.rs`), so nothing is live-wrong;
but a seed removed from the queue on a wrong structural proof does not come back, and the
failure mode it hides is the same class the HIGH cured (unbounded growth), with unbounded
recursion depth added.

Everything else in the fix cycle checks out. All five doc-vs-code MEDIUMs (2, 3, 6, 7, 12) are
genuinely fixed by reading, not by grep: `perform_remaining_draws` is now placed **above**
`resolve_pending_draw`'s doc block with an explicit note about the Rust doc-attachment hazard;
`handle_order_replacements` gained the four-case table; `PendingDraw`'s declaration and the
`GameState` field doc name both producers and both consumers; `memory/gotchas-rules.md` is
rewritten and explicitly corrects the "bypasses the replacement check" claim. Findings **9, 13,
14 are genuinely moot** — verified by reading, not by grep: the fold arm no longer exists, so the
wrong CR 104.4b justification, the `get_mut` silent no-op and the discarded `sorted` /
`sets_has_drawn_for_turn` all went with it, and both push sites now use every value they compute.
`dredge.rs` test 9's rewrite is correct and the `.contains("graveyard")` assertion is
discriminating (the gate's own message does not contain the word). T17/T18/T19 are all
non-vacuous. Wire gates hold. No hang.

### New findings

| # | Sev | File:Line | Description |
|---|-----|-----------|-------------|
| R1 | **HIGH** | `replacement.rs:905-910`, `:951-956`, `:1051-1055`; audit `:968` | **The per-player uniqueness invariant is false, and `OOS-DX2-3` is marked CLOSED on a wrong argument.** A `NeedsChoice`-origin stale entry re-defers *inside* the discharge, so the outer push produces a second entry; each further draw adds one more, and recursion depth grows with the count. **Fix:** reopen `OOS-DX2-3`, correct all seven doc sites, and re-derive `resolve_declined_pending_draw`'s termination bound from the true premise. |
| R2 | MEDIUM | `replacement.rs:895-910`; `docs/audits/decision-point-audit.md` §3.1 / §4.10 / §8.1 | **The discharge is a new engine-made choice and it is recorded nowhere in the decision-point ledger.** It auto-declines an offer the player could still legitimately answer. **Fix:** add an AUTO-CHOSEN row and amend the §8.1 banner, which still describes only the implement-phase design. |
| R3 | MEDIUM | `tests/primitives/pb_dx2_command_gates.rs` (absent) | **Nothing pins the restructured control flow.** No test reaches "discharge produced events AND the current draw took the `Proceed` path" — the only shape in which the early-`return` defect the runner caught by hand is observable. **Fix:** add that probe. |
| R4 | MEDIUM | `tests/mechanics_a_d/dredge.rs` (absent) | **Finding 5's fix is half-applied**: the graveyard-zone branch regained coverage, the `Dredge(n)` keyword check and the answer-time CR 702.52b library check still have zero. **Fix:** add the two sibling probes the original directive named. |
| R5 | LOW | `replacement.rs:982-997` | **Duplicate `PlayerLost` when the discharge decks the player out** — the outer draw repeats the loss with no state change (Arch. Invariant 4). **Fix:** short-circuit on `has_lost` after the discharge. |
| R6 | LOW | `replacement.rs:810`, `:980`, `:984` (+3 more) | **The empty-library loss is cited as CR 104.3b; the rule is CR 104.3c.** MCP-verified. The batch rewrote `:980`/`:984` this cycle and kept the wrong cite. **Fix:** correct the two rewritten lines; fold the rest into `OOS-DX2-6`. |
| R7 | LOW | golden `014:191`; `dredge.rs:905`; `pb_dx2_command_gates.rs:903-908` | **Three surviving "paused" prose sites in files this cycle touched**, plus T8's now-false assertion message. **Fix:** reconcile. |
| R8 | LOW | audit `OOS-DX2-2`, `OOS-DP2-1` rows | **Two fresh cite drifts inside the batch's own cite corrections**: `perform_remaining_draws` is at `:1497`, not `:1495`; `handle_keep_hand` is at `commander.rs:894`, not `:891`. **Fix:** re-derive both. |
| R9 | LOW | `replacement_effect.rs:414-418`; audit §9.3, §4.10 DP-5 row | **Residual doc inconsistencies**: `sets_has_drawn_for_turn`'s doc still opens by claiming the decline arm forces `true`; §9.3 still says only `pending_zone_changes` gates anything; §4.10's DP-5 row still names the deleted `draw_card_skipping_dredge`. **Fix:** reconcile. |

---

### Finding R1: the per-player uniqueness invariant is false, and `OOS-DX2-3` is closed on a wrong proof

**Severity**: HIGH
**Files**: `crates/engine/src/rules/replacement.rs:905-910` (the discharge), `:930-935` and
`:951-956` (the two unconditional `push_back`s), `:1056-1087`
(`resolve_declined_pending_draw`), `:849-872` and `:1051-1055` (the invariant / termination
docs), `:172-177` and `:3146-3150` (the two FIFO arguments that lean on it),
`crates/card-types/src/state/replacement_effect.rs:388-392`,
`crates/engine/src/state/mod.rs:143-144`, `crates/engine/src/rules/events.rs:882-887`,
`memory/gotchas-rules.md:33-37`, `docs/audits/decision-point-audit.md:968`
**CR**: 616.1 / 616.1e / 616.1f (MCP-verified), 614.11a

**The trace.** `perform_one_draw` is:

```
let mut events = Vec::new();
if let Some(i) = state.pending_draws.iter().position(|p| p.player == player) {
    let stale = state.pending_draws[i].clone();
    state.pending_draws.remove(i);
    events.extend(resolve_declined_pending_draw(state, player, stale));   // ← re-enters perform_one_draw
}
let (draw_events, outcome) = match check_would_draw_replacement(...) {
    DredgeAvailable(..) => { ... state.pending_draws.push_back(...); }    // unconditional
    NeedsChoice(..)     => { ... state.pending_draws.push_back(...); }    // unconditional
    ...
```

Let the stale entry be `NeedsChoice`-origin (2+ applicable `WouldDraw` replacements).
`resolve_declined_pending_draw` calls `perform_one_draw(offer_dredge: false,
already_applied: stale.already_applied, remaining_after: stale.remaining)`. That inner call's
own discharge check finds `pending_draws` empty for the player (the caller removed the entry),
so it skips; `check_would_draw_replacement` finds the same 2+ replacements still applicable
(nothing consumed them — CR 616.1f only removes what was *applied*), returns `NeedsChoice`, and
**pushes a fresh entry**. It returns `Deferred`, so `resolve_declined_pending_draw` skips
`perform_remaining_draws` and returns. Control returns to the *outer* `perform_one_draw`, which
now runs its own `check_would_draw_replacement` and pushes a **second** entry.

This is not speculative composition: **T19 proves the inner half empirically.**
`test_dx2_choose_dredge_some_can_answer_a_needschoice_originated_entry`
(`pb_dx2_command_gates.rs:1131-1222`) drives `ChooseDredge { None }` → the *same*
`resolve_declined_pending_draw` → and asserts a `ReplacementChoiceRequired` was emitted and
`pending_draws().len() == 1`. Extend that fixture by one line —
`turn_actions::draw_card(&mut state, p1)` after the decline — and `pending_draws().len()` is
**2**.

**Growth and recursion.** Let `g(k)` be the entry count after one `perform_one_draw` on a state
with `k` entries for the player, assuming the replacements stay applicable: `g(0) = 1`, and
`g(k) = g(k-1) + 1`, so `g(k) = k + 1`. Every draw adds exactly one entry, without bound, and
the discharge recursion depth is `k + 1`, so it grows linearly with the number of draws — i.e.
eventual stack exhaustion in the pathological case. `pending_draws` is hashed in full into both
`public_state_hash` and `loop_detection::compute_mandatory_state_hash`, so the growth also
re-opens the fingerprint-divergence channel the original review's Finding 9 discussed.

**What it does *not* do — the brief's specific HIGH test is answered NO.** The discharge cannot
**destroy** a CR 616.1 obligation: the re-created entry is field-identical (`already_applied`,
`remaining`, `sets_has_drawn_for_turn` are all threaded through unchanged), so no draw is lost.
It cannot **auto-answer** a live CR 616.1 ordering choice either: `resolve_declined_pending_draw`
applies no replacement and emits no `ReplacementEffectApplied`; it only re-runs
`check_would_draw_replacement`. If the applicable set has since dropped to one, the outcome is
whatever CR 616.1 would give with one applicable effect (no choice exists), which is legal. So on
the axis the brief named, this is not a HIGH.

**Why it is a HIGH anyway.** Three reasons, in order of weight.

1. **`OOS-DX2-3` is marked CLOSED on a demonstrably wrong argument** (audit `:968`): *"since
   `rg -n 'pending_draws\.push_back' crates/engine/src` shows exactly two call sites and BOTH
   live inside `perform_one_draw`, both now downstream of that discharge, it is structurally
   impossible for two entries to coexist for one player — not bounded, but literally zero."*
   That is a claim about **where** the pushes are, not about **when** they run relative to the
   discharge, which is exactly the failure mode the review brief flagged in advance. A seed
   struck from the ledger on a false proof does not get re-derived.
2. **The false invariant is load-bearing in shipped source, not just in prose.**
   `handle_order_replacements`' doc (`:172-177`) and `handle_choose_dredge`'s doc (`:3146-3150`)
   both dismiss the FIFO ambiguity on the grounds that *"there is never a second candidate"*, and
   `resolve_declined_pending_draw`'s **termination argument** (`:1051-1055`) is derived from it
   verbatim — *"only ever finds `pending_draws` empty for `player` at this point"* — which is
   false whenever `k > 1` and is precisely the case in which the recursion is deepest. A
   termination proof that assumes the thing it needs to prove is not a proof.
3. **The seven assertion sites** — `perform_one_draw`'s "Per-player invariant" section,
   `handle_order_replacements`, `handle_choose_dredge`, `PendingDraw`'s declaration,
   `GameState.pending_draws`, `GameEvent::DredgeChoiceRequired`, `memory/gotchas-rules.md` —
   make this the largest single instance of "a doc comment is the only thing asserting a property
   the code does not have" in the batch, in the batch whose second seed (`OOS-DP7-2`) is exactly
   that class.

**Mitigation, stated plainly so the fix can be sized correctly.** Corpus exposure is **zero**:
no card definition registers a `ReplacementTrigger::WouldDraw` replacement, so a
`NeedsChoice`-origin `PendingDraw` cannot arise from any legal deck. I re-derived this rather
than taking it on trust (the only `WouldDraw` string in `crates/card-defs/src` is inside
`out_of_the_tombs.rs`'s `Completeness::inert` note, on a def with `abilities: vec![]`), and the
audit's own DP-5 row and `OOS-DP5-3` say the same. So nothing is live-wrong; this is a latent
defect plus a bookkeeping failure.

**Fix** (all wire-neutral):
1. **Reopen `OOS-DX2-3`** — amend the row rather than deleting the narrative, and replace the
   structural claim with the true one: at most one **dredge-originated** entry per player, but a
   CR 616.1 re-defer raised *inside* a discharge can coexist with the new offer's entry, so `k`
   can grow by one per draw in the `NeedsChoice` regime. Record the zero-corpus-exposure
   mitigation in the same row so the ranking is honest.
2. **Correct all seven doc sites** to that statement, and in particular stop using the invariant
   to dismiss FIFO in `handle_order_replacements` and `handle_choose_dredge` — FIFO is real again.
3. **Re-derive `resolve_declined_pending_draw`'s termination bound** from the true premise:
   depth is bounded by the number of entries for `player` at entry (each level removes exactly
   one), and that number is itself unbounded across draws — which is the argument for treating
   (1) as a real open seed rather than a footnote.
4. **Pin the actual behaviour with a test** on T19's fixture (decline, then one more
   `draw_card`), asserting whatever count is decided to be correct. Do not "fix" it by clearing
   the player's entries again immediately before each `push_back` — that would silently destroy
   the re-deferred draw, which is worse.

---

### Finding R2: the discharge is a new engine-made choice and no decision-point row records it

**Severity**: MEDIUM
**Files**: `crates/engine/src/rules/replacement.rs:895-910`;
`docs/audits/decision-point-audit.md` §3.1, §4.10 (`:482`), §8.1 (`:800-821`)
**CR**: 702.52a — *"you **may** instead"*

The engine now decides, on the player's behalf and at a moment neither the CR nor the player
chose, that an outstanding dredge offer is **declined**. That is a legal outcome (declining is
always legal) and it is honestly documented in source
(`events.rs:882-887`, `replacement.rs:849-872`), and it strictly improves the pre-batch
behaviour and the bot path. But it is an AUTO-CHOSEN decision site in exactly the sense
`docs/audits/decision-point-audit.md` §3.1 classifies, and it is recorded in **no** row of that
ledger. Two consequences worth separating:

- **The PB-DP10 gates do not and should not change.** `decision_site_walk.rs` and
  `decision_gate.rs` walk *card-def DSL variants* (`Effect` / `Condition` names over
  `all_cards()`); dredge is a `KeywordAbility`, not an `Effect`, so no row of either gate can see
  it and no `BASELINE` entry moves. That is the correct answer to the brief's question 1 — but it
  is also a fresh, concrete instance of **`OOS-DP10-9`** (the gate can only see a decision the
  DSL encoded), and the strongest one yet, because here the auto-choice was *added* by an engine
  batch after the gate shipped.
- **The auto-decline is not outcome-neutral.** A discharged draw takes the top of the library
  **now**, not at the moment of the offer; if the library was shuffled or reordered in between,
  the card differs from what an immediate answer would have produced. And a player who intended
  to dredge simply loses the option. Pre-fix-cycle (the fold) the option survived. That is a real
  cost of the cure and is not stated anywhere.

Separately, **§8.1's PB-DX2 banner (`:800-821`) never mentions the fix cycle at all**: it
describes the shipped mechanism as "requiring-and-consuming a `PendingDraw` entry, design (b)"
and says nothing about discharge-on-next-draw, so a reader of the audit gets the implement-phase
design.

**Fix:** add a row to §3.1 (and/or §4.10) classifying `ChooseDredge`'s implicit decline as
AUTO-CHOSEN, naming `replacement.rs::perform_one_draw`'s discharge as the engine site and CR
702.52a as the rule that makes it legal; state the library-reorder non-neutrality; and amend the
§8.1 banner so it records the mechanism that actually shipped.

---

### Finding R3: nothing pins the restructured control flow

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_dx2_command_gates.rs` (absent test)

The runner reports — correctly, and this is the batch's best piece of self-catching — that the
two early `return`s in `perform_one_draw`'s `Proceed` arm had to become nested `match` tail
expressions, because a bare `return` skips `events.extend(draw_events)` and silently drops the
discharge's events. I audited every exit path: **the function now has zero `return` statements**
(`:887-1035`), a single exit at `:1033-1034`, and both the CR 104.3c empty-library arm
(`:983-997`) and the `expect_move_object_to_zone` `None` arm (`:1002`) are match tails. So the
current code is right.

But **no test would catch a regression**, and the runner says so itself ("caught during
implementation, not by a test"). The defect is only observable when the discharge produces
events *and* the current draw takes the `Proceed` path — and no test in the suite reaches that
shape. T7's outer call ends in `DredgeAvailable` (the dredge card never leaves the graveyard, so
every draw is offered), so a `return` in the `Proceed` arm would not affect it; T9 is the
dead-player no-op; T3/T5/T6 all answer immediately. This is exactly the PB-DP8 recurring bug
class ("a guard that returns early inherits the obligation of the statements it skipped") left
with only the author's care behind it.

**Fix:** add a probe — outstanding entry for `p1`, then remove the dredge card from the graveyard
(or drop the library below `N`), then `turn_actions::draw_card(p1)` — asserting **two**
`CardDrawn` events for `p1` from that single call, in discharge-first order, and
`pending_draws().is_empty()` afterwards. That reddens on either early `return` and also pins the
event ordering of Finding R5's neighbourhood.

---

### Finding R4: Finding 5's fix restored one of the three `Some`-arm validations

**Severity**: MEDIUM
**File**: `crates/engine/tests/mechanics_a_d/dredge.rs` (absent tests)

Test 9's rewrite is good: it reaches a real offer via `build_upkeep_state` + `pass_all`, names a
hand card, and asserts `msg.contains("graveyard")` — which is discriminating, because the gate's
own rejection message (`replacement.rs:3213-3217`) does not contain that word. The
graveyard-zone branch is properly covered again.

The original Fix directive also said *"Add sibling probes for the missing-keyword and
short-library branches."* Neither was added; enumerating the file's 13 tests confirms it. The
missing-keyword branch matters most: it is a trust-boundary check, and if it were dropped,
`ChooseDredge { Some(any card in your own graveyard) }` would return that card to hand for free
while satisfying the gate. `test_dredge_insufficient_library_not_offered` covers the *offer*
side of CR 702.52b, not `handle_choose_dredge`'s answer-time re-validation
(`replacement.rs:3271-3276`), which is the one that matters once the library can change between
offer and answer — and after the fix cycle it can, because the discharge makes intervening draws
legal.

**Fix:** two probes on test 9's fixture — (a) name a non-dredge card in `p1`'s own graveyard,
assert the message names the keyword; (b) reach the offer, mill/exile the library below `N`, then
answer, assert `"cannot dredge"`.

---

### Finding R5: duplicate `PlayerLost` when the discharge decks the player out

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:982-997`
**CR**: 104.3c (MCP-verified), Architecture Invariant 4

The discharge's inner `perform_one_draw` can hit an empty library, set `has_lost = true` and emit
`PlayerLost`. Control then returns to the outer call, which has no liveness check, runs
`check_would_draw_replacement` (dredge is not offered — `library_count` is 0, so no option passes
`(*n as usize) <= library_count`), takes `Proceed`, finds the library empty again, sets
`has_lost = true` a second time and emits a **second** `PlayerLost`. Two identical events, the
second with no corresponding state change.

Reachable in the deck this batch exists for: the offer is made with `library == N`, an opponent
mills the player out while the offer stands, and any later draw fires both. Pre-fix-cycle the
stale entry simply sat there and only one `PlayerLost` was emitted. Mechanically harmless (the
SBA path reads `has_lost`), but it is the Architecture Invariant 4 shape and it is new.

**Fix:** after the discharge, if `state.expect_player(player)` reports `has_lost`, return
`(events, DrawStepOutcome::LostToEmptyLibrary)` immediately rather than attempting the current
draw — and cite CR 104.3c, not 104.3b (Finding R6).

---

### Finding R6: the empty-library loss is cited as CR 104.3b throughout; the rule is CR 104.3c

**Severity**: LOW
**Files**: `replacement.rs:810`, `:980`, `:984`, `:1620`; `rules/events.rs:39`;
`rules/turn_actions.rs:1236`
**CR (MCP, verbatim)**: **104.3b** — *"If a player's **life total is 0 or less**, that player
loses the game the next time a player would receive priority."*; **104.3c** — *"If a player is
**required to draw more cards than are left in their library**, they draw the remaining cards and
then lose the game the next time a player would receive priority."*

Six sites cite 104.3b for the empty-library draw loss. All are pre-existing (PB-DP5 era), so this
is not a regression — but the fix cycle **rewrote `:980` and `:984`** when it converted the
`Proceed` arm's early `return`s into match tails, and carried the wrong cite through. This is the
same class as the batch's own `OOS-DX2-6` (wrong CR cite propagated self-consistently until
someone checks).

**Fix:** correct the two lines this batch rewrote; fold the other four into `OOS-DX2-6`'s sweep
ticket, which already exists and already covers exactly this pattern.

---

### Finding R7: three surviving "paused" prose sites in files the fix cycle touched

**Severity**: LOW

- `test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json:191` —
  `"After draw step turn-based action fires: hand still 2 (draw paused for dredge choice), …"`.
  This sits **one line above** the `note` at `:205` that the fix cycle rewrote specifically to say
  *"The engine does NOT pause or block (PB-DX2)"*. Findings 11's three named sites (`:6`, the old
  `:205`, the old `:216`) are all correctly fixed; this is a fourth instance of the same phrase in
  the same file, missed because the sweep was by line number rather than by the phrase.
- `crates/engine/tests/mechanics_a_d/dredge.rs:905` — `// No CardDrawn yet — the draw is paused
  for the player's choice.` Finding 15c fixed `:150` and the doc at `~:110`; this third one
  survives.
- `crates/engine/tests/primitives/pb_dx2_command_gates.rs:903-908` (T8) — the assertion message
  *"the unanswered entry must still be present — nothing else resolves it for the player"* is now
  **false in general**: the player's own next draw resolves it. The test still passes only because
  six `pass_all` rounds from Upkeep do not reach `p1`'s next draw step, and `len() == 1` cannot
  distinguish "the same entry survived" from "it was discharged and replaced". The test's *value*
  (no `BlockingDecision`, no hang) is intact; only the final assertion's claim is stale.

**Fix:** reconcile the three strings; for T8, either capture the entry before the loop and assert
identity, or narrow the message to what the test actually proves.

---

### Finding R8: two fresh cite drifts inside the batch's own cite corrections

**Severity**: LOW
**File**: `docs/audits/decision-point-audit.md`, rows `OOS-DX2-2` (`:967`) and `OOS-DP2-1`

- `OOS-DX2-2` now cites `perform_remaining_draws` at `rules/replacement.rs:1495`, and the row's
  own prose says the number was *"re-verified against the actual post-edit line, not copied from
  the implement-phase number"*. `fn perform_remaining_draws(` is at **`:1497`**; `:1495` lands
  inside its doc comment.
- `OOS-DP2-1`'s closure corrected the cite from `commander.rs:877-885` to `:891`. After the fix
  cycle's doc additions, `pub fn handle_keep_hand(` is at **`:894`**.

Both are trivial in isolation; together they are the third and fourth instances of the
`OOS-DP6-8` documentation-rot class produced *by* a batch that fixes two of them and says so in
its banner. Worth a one-line convention note (cite the symbol, or cite `file::symbol` rather than
`file:line`) more than worth the two edits.

---

### Finding R9: residual doc inconsistencies

**Severity**: LOW

- `crates/card-types/src/state/replacement_effect.rs:414-418` —
  `sets_has_drawn_for_turn`'s doc still opens *"`true` for `turn_actions::draw_card` and an
  EXPLICIT decline via `replacement::handle_choose_dredge`'s `None` arm (both set
  `PlayerState::has_drawn_for_turn`)"* and only corrects itself two sentences later. The `None`
  arm now passes `pending.sets_has_drawn_for_turn`, so the opening clause is still the false
  statement Finding 2 flagged.
- `docs/audits/decision-point-audit.md` §9.3 (`:1050`) — *"only `pending_zone_changes` actually
  gates anything. The other five pending vectors are inert queues that nothing consults."*
  `pending_draws` now gates `Command::ChooseDredge`; that is the batch's headline.
- `docs/audits/decision-point-audit.md` §4.10 (`:482`) and the DP-5 row (`:554`) still name
  `replacement.rs::draw_card_skipping_dredge` as a live emit site / touched symbol. It was deleted
  in the implement phase.

---

## Gate confirmations (independently verified this pass)

| Gate | Status | Evidence |
|---|---|---|
| **PROTOCOL 32 unmoved** | ✅ | `rules/protocol.rs:335` — `pub const PROTOCOL_VERSION: u32 = 32;`. No `Command`/`GameEvent` declaration changed; `Command::ChooseDredge` and `GameEvent::DredgeChoiceRequired` are doc-only edits. T16 pins it. |
| **HASH 69 unmoved** | ✅ | `state/hash.rs:679` — `pub const HASH_SCHEMA_VERSION: u8 = 69;`. `PendingDraw` still has exactly its four fields (`replacement_effect.rs:398-427`); `GameState` gained no field (`state/mod.rs:141-147`); the fix cycle is control flow + docs. T16 pins it. |
| **No hang (simulator / fuzzer)** | ✅ | `pending_draws` is referenced in six source files — `replacement.rs`, `state/{mod,hash,builder}.rs`, `loop_detection.rs`, `events.rs` (doc only) — and **zero** references in `rules/engine.rs`, so `blocking_decision`, the admission gate, `enter_step`, `handle_all_passed` and the SBA loop are all untouched. No `BlockingDecision` variant added. T8 still passes. **The fix cycle strictly improves the bot path**: a bot that never answers now has its stale entry discharged (drawn) on its next draw instead of leaving it outstanding forever — `OOS-DX2-5`'s "every simulated game silently loses the draw" is now "loses at most the last one". |
| **SR-3** (sealed `GameState`) | ✅ | `pending_draws` stays `pub(crate)` (`state/mod.rs:147`); the only accessor is read-only (`:492`); no test-only setter added; T14/T15 remain in-src. |
| **SR-4** (silent failures classified) | ✅ | The one new unclassified silent no-op the original review flagged (`get_mut` after `position`) is gone — `grep get_mut` in `replacement.rs` → 0 hits. New code uses `expect_player` / `expect_zone` / `expect_move_object_to_zone` throughout. |
| **SR-9b** (determinism) | ✅ | The discharge uses `position` (first match) + `remove(i)`; both push sites still sort `already_applied` by `ReplacementId` before storing; `pending.already_applied.iter().copied().collect()` into a `HashSet` is only ever consumed by `find_applicable`, never iterated to an outcome. |
| **SR-9c** (no weakened/skipped script) | ✅ | Golden `replacement/014`: assertion count and values unchanged at 3 + 3 + 3 = 9 (`hand.p1.count` 2/2/3, GGT `includes`/`includes`/`excludes`, `stack.is_empty` ×3); the fix cycle changed only prose in the metadata description and two `note` fields, and left both append-only dispute log entries intact. One stale phrase survives — Finding R7. |
| **SR-25** (`bare_lookup_ratchet`) | ✅ | `replacement.rs` ceiling 24 (`bare_lookup_ratchet.rs:137`); the fix cycle's new code (`resolve_declined_pending_draw`, the restructured `Proceed` arm) adds no `.objects.get(` / `.players.get(`. |
| **Arch. Invariant 4** (no phantom events) | ⚠️ | One new violation: duplicate `PlayerLost` on the empty-library discharge path — Finding R5. Event *ordering* is otherwise coherent: discharge events precede the current draw's events at the single exit (`replacement.rs:1033`), every one corresponds to a real state change, and `check_and_flush_triggers` sees them in emission order at the command boundary (`ChooseDredge`'s arm in `engine.rs` is untouched). |

## Previous findings

| # | Sev | Previous status | Current status | Notes |
|---|-----|-----------------|----------------|-------|
| 1 | HIGH | OPEN | **RESOLVED** | The bank is gone; `remaining` never accumulates. Fixed by a fourth option (discharge, not fold) rather than any of the three offered — a bigger change than proposed and, on balance, the better one. **But the cure carries a new HIGH (R1)** and a new engine-made choice (R2). |
| 2 | MED | OPEN | **RESOLVED** | Both doc blocks name both producers and both consumers; `sets_has_drawn_for_turn`'s divergence corrected — with one stale opening clause left (R9). |
| 3 | MED | OPEN | **RESOLVED** | Four-case table + FIFO note added at `replacement.rs:153-177`. The FIFO paragraph now repeats the false invariant (R1). |
| 4 | MED | OPEN | **RESOLVED** | T17 (cross-player), T18 (§3.3 row 2), T19 (§3.3 row 4). All three non-vacuous — see below. |
| 5 | MED | OPEN | **PARTIAL** | Graveyard-zone branch restored and asserted on the message. The two sibling probes the directive named were not added — R4. |
| 6 | MED | OPEN | **RESOLVED** | `perform_remaining_draws` relocated above `resolve_pending_draw`'s doc block (`:1478-1525` / `:1526-1551`), with a note recording the Rust doc-attachment hazard. |
| 7 | MED | OPEN | **RESOLVED** | `memory/gotchas-rules.md:26-39` rewritten; explicitly corrects the "bypasses the replacement check" claim. |
| 9 | LOW | OPEN | **MOOT (verified)** | Read `replacement.rs:913-937`: the fold arm and its CR 104.4b justification no longer exist. |
| 10 | LOW | OPEN | **RESOLVED** | CR 616.1e-cited "decline is not sticky" note at `:3152-3162`, pinned by T19. |
| 11 | LOW | OPEN | **RESOLVED (3 of 4)** | `:6`, the old `:205` and the old `:216` all fixed; a fourth instance of the same phrase survives at `:191` — R7. |
| 12 | LOW | OPEN | **RESOLVED** | `effects/mod.rs:9279-9287` names both consumers. |
| 13 | LOW | OPEN | **MOOT (verified)** | `grep get_mut crates/engine/src/rules/replacement.rs` → 0. The `position` → discharge → unconditional `push_back` shape has no `get_mut`. |
| 14 | LOW | OPEN | **MOOT (verified)** | Both push sites consume `sorted`; `resolve_declined_pending_draw` reads the stale entry's own `already_applied` / `remaining` / `sets_has_drawn_for_turn`. Nothing is computed and discarded. |
| 15a | LOW | OPEN (no action) | **CORRECT** | `resolution.rs:8499-8506` still documents that T15 asserts nothing in release builds; the reap block (`:104-119`) and `dx2_pending_effect_choice_reap_tests` (`:8345`) are untouched by the fix cycle. |
| 15b | LOW | OPEN | **PARTIAL** | Cite updated but to `:1495`; the `fn` is at `:1497` — R8. |
| 15c | LOW | OPEN | **PARTIAL** | `dredge.rs:150` and the enclosing doc fixed; `:905` missed — R7. |

## Test non-vacuity (new tests only)

| test | what production change makes it fail | verdict |
|---|---|---|
| **T7** (rewritten) | reverting to the fold: `discharge_drawn` → 0 **and** `remaining` → 2 **and** the entry count/conservation totals shift. Removing the discharge without the fold: `pending_draws().len()` → 2. | **Non-vacuous, and the strongest test in the file.** It still pins what it was written to pin (CR 614.11a conservation across a deferral) and now additionally pins the bound — the assertion messages spell out both. |
| **T17** | dropping the `pd.player == player` predicate from `handle_choose_dredge`'s `position` (p2 would consume p1's entry and draw); or removing the gate entirely. Both land in the `Ok(..)` arm, which panics with the p2 hand delta. | Non-vacuous; the trust-boundary property is now pinned, matching its `OrderReplacements` sibling. |
| **T18** | not pushing on `DredgeAvailable` (`draw_idx` → `None` → `Err`); or adding an origin discriminator to `handle_order_replacements`' draw arm. Also asserts the chosen `ReplacementEffectApplied` fires and the dredge card stays in the graveyard, so a "consume for free" regression is caught. | Non-vacuous. It is a characterization test for §3.3 row 2 rather than a bug probe, which is what Finding 4 asked for. |
| **T19** | making the decline sticky (rejecting `Some` on a re-deferred entry), or making dredge inapplicable after a decline. Asserts `Dredged` fires, the queue empties and the card reaches hand. | Non-vacuous. It also happens to be the fixture that falsifies R1's invariant with one extra line. |
| **`dredge.rs` test 9** (rewritten) | reordering the graveyard-zone check behind the gate, or dropping it (→ `Ok`). The `.contains("graveyard")` assertion is discriminating because the gate's own message does not contain that word. | Non-vacuous; the degradation-to-gate-rejection trap that caused the original finding is now closed by construction. |

## Ship recommendation

**Do not ship until R1 is dispositioned.** The engine behaviour is correct for every state
reachable from a legal deck, and the HIGH the review raised is genuinely and well fixed — the
design change is a bigger and better answer than any of the three options offered, and the
runner's reasoning for it (documented at `replacement.rs:895-904` and in the fix-cycle appendix)
holds up on every point except one.

That one point is the blocker, and it is cheap: **`OOS-DX2-3` is struck from the ledger on a
proof that does not hold**, and the false invariant it rests on is now quoted in seven places
including the FIFO and termination arguments of shipped source. The remedy is ~30 lines of doc
and audit correction, one seed row reopened, and one test that pins the real entry count — no
engine behaviour need change, no wire moves. R2 (add the AUTO-CHOSEN row and amend the §8.1
banner), R3 (one control-flow probe) and R4 (two validation probes) belong in the same pass;
R5–R9 are fine to ship as seeds if the coordinator prefers, though R5 and R6 are each a two-line
edit in code the fix cycle already rewrote.

---

## Fix cycle 2 (2026-08-01, same day) — disposition of all 9 re-review findings

**R1 (HIGH) reproduced first, before any fix, per the runner's brief.** Extended T19's exact
fixture by one line (`turn_actions::draw_card(&mut state, p1)` after the decline) in a throwaway
test and ran it: `pending_draws().len() == 2`, confirmed. The reviewer's trace is correct in
every particular — the fallacy is exactly "where the pushes are" vs "when they run relative to
the discharge." **Disposition: reopened, doc-corrected, pinned by a permanent test — no engine
fix applied, per the reviewer's own explicit warning against clearing entries early.**
- `OOS-DX2-3` reopened in `docs/audits/decision-point-audit.md` (row rewritten in place, `~~struck~~`
  preserved for the record rather than deleted) with the corrected invariant (bounded to one
  *dredge-originated* entry, not to one entry total) and the zero-corpus-exposure mitigation
  stated plainly.
- All eight doc sites carrying the false invariant corrected (the review's count of "seven" plus
  an eighth, `PendingDraw`'s field-level `sets_has_drawn_for_turn` doc, folded in under R9 since
  it is the same drift class): `replacement.rs`'s `perform_one_draw` "Per-player invariant"
  section (rewritten to state the true bound and the growth mechanism), `handle_order_replacements`'s
  and `handle_choose_dredge`'s FIFO notes (FIFO is real again), `resolve_declined_pending_draw`'s
  termination doc (re-derived from the true premise: depth bounded by the *entry count at the
  start of the chain*, itself unbounded across draws), `PendingDraw`'s struct-level doc
  (`replacement_effect.rs`), `GameState.pending_draws`'s field doc (`state/mod.rs`),
  `DredgeChoiceRequired`'s event doc (`events.rs`), and `memory/gotchas-rules.md`.
- New test `test_dx2_needschoice_redefer_grows_the_queue` (T20,
  `pb_dx2_command_gates.rs`) pins the TRUE invariant permanently — explicitly documents in its
  own body that a future "fix" which re-clears entries before `push_back` (destroying the
  re-deferred draw) must fail this test, per the reviewer's warning.
- No engine behaviour changed for this finding — confirmed correct by the reviewer's own
  zero-corpus-exposure argument, independently re-derived here (`rg -rn WouldDraw
  crates/card-defs/src` → only `out_of_the_tombs.rs`'s `inert` note).

**R2 (MEDIUM)**: added `OOS-DX2-7` (new seed, AUTO-CHOSEN classification) and a new row to
audit §4.10's table naming the discharge as an AUTO-CHOSEN decision site with the CR 702.52a
legality argument and the library-reorder non-neutrality. Amended the §8.1 banner — the paragraph
describing "design (b), fold guard" is now followed by a correction block naming the discharge as
what actually shipped and cross-referencing the reopened `OOS-DX2-3` and the new `OOS-DX2-7`.
**Disposition: fixed (doc/audit only, as directed — the finding does not ask for an engine
change and the PB-DP10 gates correctly do not move, since dredge is a `KeywordAbility` not a
DSL `Effect`/`Condition`).**

**R3 (MEDIUM)**: added `test_dx2_discharge_then_proceed_both_produce_events_in_one_call` (T21,
`pb_dx2_command_gates.rs`) reaching the exact shape the finding named — dredge card milled below
the Dredge(3) threshold between the offer and a second draw, so BOTH the discharge and the
current draw take `Proceed`. **Verified non-vacuous by injecting the exact regression the runner
described** (temporarily dropping the discharge's accumulated events via `events.clear()` before
`events.extend(draw_events)`, simulating what a bare early `return` would do): the test failed
with `left: 1, right: 2` as predicted, then passed again after reverting. **Disposition: fixed.**

**R4 (MEDIUM)**: added the two sibling probes the original Finding 5 directive named, in
`tests/mechanics_a_d/dredge.rs`: `test_dredge_some_rejects_a_graveyard_card_without_the_keyword`
(names a graveyard card lacking `Dredge(n)`, asserts on `"Dredge keyword"`) and
`test_dredge_some_rejects_when_library_drops_below_threshold_after_the_offer` (mills the library
below the threshold between the offer and the answer — legal only because R1's discharge makes an
intervening draw possible while an offer stands — asserts on `"cannot dredge"`). Both pass.
**Disposition: fixed.**

**R5 (LOW)**: `perform_one_draw`'s empty-library `Proceed` arm now checks `has_lost` before
emitting a second `PlayerLost`, expressed as a tail-value branch (NOT a `return`) so it cannot
reintroduce the exact defect R3 pins — a bare `return` there would skip
`events.extend(draw_events)` for the SAME reason the Proceed arm's other two exits were already
converted in fix cycle 1. **Disposition: fixed.**

**R6 (LOW)**: the two lines the fix cycle rewrote (`replacement.rs`'s empty-library `Proceed` arm,
now carrying R5's fix) corrected from CR 104.3b to CR 104.3c (MCP-verified: 104.3b is the
life-total-≤0 loss, 104.3c is the required-to-draw-more-than-remain loss). The other four
pre-existing sites (`replacement.rs:814`/`:1686` roughly, `turn_actions.rs`, `events.rs`) are left
for `OOS-DX2-6`'s existing sweep ticket, per the finding's own fix directive. **Disposition:
fixed (the two rewritten lines); the rest deliberately deferred to the existing ticket.**

**R7 (LOW)**: all three named prose sites reconciled — golden `replacement/014`'s `:191`
description ("draw paused for dredge choice" → "draw replaced by a recorded dredge offer, not
paused"), `dredge.rs`'s `// the draw is paused for the player's choice` comment (clarified: no
draw in *this* call, not a claim the game is blocked), and T8's assertion message and doc comment
(no longer claims "nothing else resolves it for the player" in general — narrowed to what the
fixture actually proves, PLUS strengthened from a count-only assertion to an identity assertion
by capturing and comparing the whole `PendingDraw` entry, which is strictly more informative than
the "narrow the message" option the finding offered). **Disposition: fixed, and strengthened
beyond the minimum the finding asked for.**

**R8 (LOW)**: `OOS-DX2-2`'s cite switched from a line number (already stale a second time,
`:1495` vs the actual `:1497`) to `replacement::perform_remaining_draws` by symbol, with a note
explaining why (the number had already drifted twice within one batch and this very fix pass
would have drifted it a third time by adding doc content above it). `OOS-DP2-1`'s cite corrected
to `commander.rs::handle_keep_hand`, confirmed at `:894` by direct read (this one was NOT touched
by this pass's edits, so a line number is safe here, but symbol-first is used for consistency).
**Disposition: fixed, both cites re-verified by reading the actual current file, not copied from
the finding.**

**R9 (LOW)**: all three named sites fixed — `replacement_effect.rs`'s `sets_has_drawn_for_turn`
doc (the opening clause no longer claims `handle_choose_dredge`'s `None` arm forces `true`; it
now correctly states that arm only ever *propagates* the resolved entry's own stored value),
`docs/audits/decision-point-audit.md` §9.3 (a new correction block added, not a silent edit, since
the paragraph was already once corrected by a prior PB-DP7 update-in-place and a second silent
edit would have hidden the history), and §4.10's table row plus the DP-5 row's "Touched Symbols"
list (both corrected from `draw_card_skipping_dredge`, which fix cycle 1 deleted, to
`replacement::resolve_declined_pending_draw`). **Disposition: fixed.**

### Verification (fix cycle 2)

- `git diff --stat -- crates/engine/src/rules/protocol.rs crates/engine/src/state/hash.rs` →
  empty. PROTOCOL 32 / HASH 69 confirmed unmoved by `test_dx2_wire_version_sentinels` (T16, still
  present, still passing) and by direct read of both constants.
- `cargo build --workspace` → clean.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo fmt --check` → clean.
- `tools/check-defs-fmt.sh` → clean, 1,804 defs.
- `cargo test --all` → **3,978 passing, 0 failing** (fix-cycle-1 baseline was 3,974; +4 net: T20,
  T21, and the two `dredge.rs` sibling probes).
- R1's fix was independently verified non-vacuous twice: once by reproducing the defect BEFORE
  any fix (matching the runner brief's explicit instruction), and once by confirming the new T20
  test asserts the TRUE count (2) rather than silently re-asserting the false one (1).
- R3's fix (T21) was independently verified non-vacuous by injecting the exact regression class
  the finding describes and observing the test fail with the predicted counts, then reverting and
  confirming it passes clean.

### What this pass did NOT do, and why

No engine behaviour changed for R1 — the reviewer's own mitigation argument (zero corpus
exposure, and an engine fix risks silently destroying a re-deferred draw) is correct and was not
second-guessed. No new `GameState` field, no wire change, for any of the nine findings — all are
either pure control-flow additions inside an existing function (R5), doc/audit corrections (R1's
doc half, R2, R6's two lines, R7, R8, R9), or new tests (R1's pin, R3, R4). PROTOCOL 32 / HASH 69
remain unmoved through two full fix cycles.
