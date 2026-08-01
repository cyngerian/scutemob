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
