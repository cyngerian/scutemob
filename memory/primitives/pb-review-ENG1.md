# Primitive Batch Review: ENG-1 — effect-driven discard is a real player choice

**Date**: 2026-08-02
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-191` · **Branch**: `feat/eng-1-effect-driven-discard-is-a-real-player-choice-effectch`
**CR Rules verified**: 701.9a/b/c, 608.2d, 608.2f, 601.2c, 702.35a, 514.1, 404.3, 400.7, 605.1b/605.4a, 104.3a/800.4
**Engine files reviewed**: `crates/card-types/src/state/stubs.rs`, `crates/engine/src/effects/mod.rs`,
`crates/engine/src/state/hash.rs`, `crates/engine/src/rules/protocol.rs`,
`crates/engine/src/rules/events.rs`, `crates/engine/src/rules/resolution.rs`,
`crates/engine/src/rules/mana.rs`, `crates/engine/src/testing/replay_harness.rs`,
`crates/engine/src/testing/script_schema.rs`, `crates/simulator/src/params.rs`
**Client files reviewed**: `tools/play-server/src/view.rs`, `tools/play-server/src/api.rs`,
`tools/play-server/src/main.rs`, `tools/play-server/frontend/src/lib/DiscardPicker.svelte`,
`tools/play-server/frontend/src/lib/ActionBar.svelte`, `tools/tui/src/play/app.rs`
**Test files reviewed**: `crates/engine/tests/primitives/pb_eng1_effect_discard_choice.rs` (10 tests),
`crates/engine/tests/core/decision_gate.rs`, `crates/engine/tests/core/decision_site_walk.rs`,
`crates/engine/tests/casting/x_cost_spells.rs`, `crates/engine/tests/primitives/pbp_power_of_sacrificed_creature.rs`,
`crates/engine/tests/primitives/pb_dp9_effect_choice.rs`, the two HTTP probes in `main.rs`
**Card defs reviewed**: **0 changed** (verified: no `ENG-1`/`ENG1` reference anywhere under `crates/card-defs`;
the 21 defs carrying `Effect::DiscardCards` were read for the roster claim only)

## Verdict: needs-fix

**No HIGH findings, and I looked hard for them.** The CR 701.9b/608.2d implementation is correct;
the suspend/replay contract's determinism premise genuinely holds for the new variant; the
`continue`/`return` split is right in both directions and is executed by a test in one resolution;
all three "silent" plumbing sites (check 4's `matches!`, check 5's arm before `unreachable!()`,
`api.rs::validate_decision_params`'s `_ =>` catch-all) were extended and agree with each other and
with the engine; both `HashInto` arms use discriminant `3u8` and feed every field with a consistent
`u32` route; Architecture Invariant 7 holds (`private_to()` unchanged, `entry.player` enforced at all
three gates, hand labels routed through `NameIndex` rather than `question_card_label`); PROTOCOL 33→34
and HASH 70→71 are gate-computed with append-only history rows. The ten engine tests are
non-vacuous — several would have caught the shipped defect, and (g) genuinely discriminates a shared
`discard_one_chosen_card` body from a copy-paste. The three repaired pre-existing fixtures were
repaired by *answering* rather than by weakening, and the event-merge helper is correct because
`resolve_top_of_stack` returns **only** the question event on the aborted pass (`resolution.rs:159`),
so nothing is double-counted.

Findings are **2 MEDIUM and 8 LOW**. The MEDIUMs are: (1) neither new `HashInto` arm is exercised by
any gate in the workspace — the canonical fixture carries no `pending_effect_choice` at all, so
dropping a field feed from either arm ships green, and the *cheapest moment to close this is inside
v71, before it lands on main*; (2) `OOS-ENG1-9`'s placeholder makes two same-resolution-drawn
candidates render **identically**, which is a fixable ambiguity distinct from the deferred general fix.
On focus item 10 my explicit opinion is: **deferring `OOS-ENG1-9` is correct, and the batch is not a
net regression** — see Finding 2.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `crates/engine/tests/core/hash_schema.rs` (canonical fixture) | **Both new `HashInto` arms are unexercised by every gate in the workspace.** A dropped field feed passes `cargo test --workspace` green. **Fix:** populate `canonical_fixture()` with a `Discard`-shaped `pending_effect_choice` + a one-entry `effect_choice_answers`, and re-pin `stream_fingerprint` **in the v71 row this batch is adding**, before it ships. |
| 3 | LOW | `effects/mod.rs:1301-1312` | **The short-circuit's stated justification is false in the CR 608.2f sense.** `n >= hand.len()` determines the *set*, not the *order*, and `EffectChoiceAnswer::Discard::chosen`'s own doc says the order is a real payload. **Fix:** amend the comment to say "exactly one legal SET" and cite `OOS-ENG1-7` for the order half. |
| 4 | LOW | `rules/events.rs:1547-1549`, `:1570-1571` | **Two inline comments on `reveals_hidden_info`/`private_to` still say the ids are library-only.** The batch's own §10 comment-debt class, in the file it updated 60 lines above. **Fix:** widen both to name the hand case. |
| 5 | LOW | `tools/play-server/src/view.rs:1926-1928` | **The arm-level comment asserts "the CANDIDATES are library cards and are not [in `NameIndex`]".** False for `Discard`, and it sits directly above the arm whose whole point is the opposite. **Fix:** rewrite to say the candidate source is per-question. |
| 6 | LOW | `crates/engine/tests/primitives/pb_eng1_effect_discard_choice.rs` | **The `Cost::DiscardCard` structural guarantee has no regression guard**, although §2.4 calls it the more dangerous of the two. **Fix:** add a test asserting `pending_effect_choice()` is `None` after paying a `Cost::DiscardCard` from a hand of >1. |
| 7 | LOW | `tools/play-server/src/main.rs:9017-9020` | **Gate (j) needles only the `"candidates"` key, not the foreign hand's card names.** **Fix:** add a raw-body assertion that at least one of seat 1's known hand card names is absent. |

## Card Definition Findings

**None.** Zero card defs changed, and that is correct: I independently confirmed `fell_specter.rs`
is `Complete` with `Effect::DiscardCards { player: DeclaredTarget{0}, count: Fixed(1) }` and
`TargetRequirement::TargetOpponent`, matching oracle text ("When Fell Specter enters, target
opponent discards a card"), and that the defect was 100% engine-side. No def in the corpus prints
"at random" or "an opponent chooses", so `OOS-ENG1-3`'s deferral of a `chooser` field is right.

| # | Severity | Card / doc | Description |
|---|----------|------|-------------|
| 8 | LOW | roster arithmetic (`memory/workstream-state.md:1016-1024`) | **21 def files carry `Effect::DiscardCards`, not 22.** `nezahal_primal_tide.rs:55` mentions the name only in a comment, exactly like the `reforge_the_soul.rs` case the handoff *did* exclude. Derived by grep-minus-exceptions rather than an `all_cards()` enumeration (SR-36). The 12-`Complete` and 7-draw figures are correct — I verified them. **Fix:** correct the denominator to 21 and note the method. |
| 9 | LOW | `memory/workstream-state.md:1009` | **Summary overstates its own measurement.** It says "every candidate renders as the unknown-label placeholder"; :1005 and the audit row both say only the freshly-drawn candidates do (5 of 7 rendered correctly on the Faithless Looting probe). **Fix:** change to "every candidate DRAWN IN THAT RESOLUTION". |
| 10 | LOW | seed register | **`OOS-ENG1-5` is skipped** — the filed set is 1,2,3,4,6,7,8,9. **Fix:** either file the missing seed or note the gap so a future reader does not hunt for it. |

### Finding Details

#### Finding 1: neither new `HashInto` arm is exercised by any gate — MEDIUM

**File**: `crates/engine/src/state/hash.rs:3254-3258` / `:3279-3282`; gate at
`crates/engine/tests/core/hash_schema.rs`
**Verified**: `grep 'pending_effect_choice|effect_choice_answers|EffectChoiceQuestion|EffectChoiceAnswer'`
over `crates/engine/tests/core/hash_schema.rs` returns **zero matches**. The canonical fixture has
never populated `pending_effect_choice` — so this is not new to ENG-1; *all four* arms of both impls
are unexercised, and have been since PB-DP9.

**Issue**: `hash.rs:3228-3234`'s own warning says the SR-19 gate
(`every_hashed_struct_field_is_hashed_or_allowlisted`) scans **structs only**, and that the two enum
impls are "held by review and by `stream_fingerprint`, nothing else". The second half of that
sentence is **not true**: `stream_fingerprint` is computed over `canonical_fixture()`, which never
reaches either impl. The batch observed this correctly at `workstream-state.md:1065-1070` and at
`hash.rs:720-724` ("the new `HashInto` arms' own bytes are not what moves this digest, the
version-sentinel byte is") — but recorded it rather than closing it.

**Concrete failure scenario**: a later refactor rewrites the question arm as
`EffectChoiceQuestion::Discard { hand, .. } => { 3u8.hash_into(h); hand.hash_into(h); }`, dropping
`count`. Two `GameState`s that differ **only** in how many cards a pending discard demands then hash
identically. `cargo test --workspace` stays fully green: the SR-19 struct scan does not see it, the
`stream_fingerprint` pin does not move, and SR-9b's `harness_equivalence` per-step fingerprint
cross-validates green because *both* regimes drop the same field. The harm surfaces as an
undetectable desync in exactly the state the M10 network layer most needs to detect one.

**Why now rather than as a seed**: closing it changes `stream_fingerprint`. Inside this batch that is
a re-pin of the v71 row it is already writing (a row that has not shipped to main). In a successor
batch it is a **PROTOCOL-free HASH 71→72 bump plus a 46-file sentinel re-pin** for a test-fixture
change. The cost differs by roughly an order of magnitude and the correctness is identical.

**Fix**: add a `Discard`-shaped `pending_effect_choice` (non-empty `hand`, `count` ≠ 0 and ≠
`hand.len()`) and a one-entry `effect_choice_answers` bank to `canonical_fixture()`; re-pin
`stream_fingerprint` on the **v71** row to what `stream_fingerprint_is_pinned` prints; add one
sentence to `hash.rs:3228-3234` recording that the arms are now stream-exercised. Do **not** bump
HASH again — v71 is not on main.

#### Finding 2: `OOS-ENG1-9` — the deferral is correct, the placeholder is not — MEDIUM

**Files**: `tools/play-server/src/view.rs:2004-2010` (`names.label(*id)`),
`tools/play-server/src/view.rs:1111-1116` (`UNKNOWN_LABEL` fallback)
**CR**: 608.2d (the roll-back), 400.7 (the drawn card is a new object with a new `ObjectId`)

**The mechanism, independently confirmed**: `resolution.rs:120` snapshots `restart_point`, `:142`
restores it wholesale. `state/mod.rs:1304` mints a fresh `ObjectId` on **every** zone change, so a
card drawn earlier in the same resolution holds an id that does not exist in the restored state at
all. `NameIndex::from_view` walks the redacted view and correctly has no entry, so `label()` returns
`(unknown card)`. The ids are strictly greater than every restored id, so there is **no** risk of a
candidate being mislabelled as the wrong card — the failure is confined to the placeholder.

**My opinion on the deferral, argued as requested.** Deferring is **correct**, and the batch is
**not** a net regression for the affected printings, for three reasons:

1. **The scope is smaller than the dispatch brief states.** The brief says "16 of the 23
   `Effect::DiscardCards` defs draw in the same effect, so this is the dominant printing". The
   batch's own measurement — which I re-derived and agree with — is **7 of the 12 deck-legal
   `Complete` defs** (Chart a Course, Faithless Looting, Frantic Search, Geier Reach Sanitarium,
   Greater Good, Izzet Charm, Pull from Tomorrow), against 5 that do not draw. "Dominant printing"
   is fair; "16 of 23" is not.
2. **Within an affected card, only the drawn subset is unlabelled.** The probe's own evidence: on
   Faithless Looting, **5 of 7** candidates rendered real names and 2 did not. A human asked to
   discard 2 of 7 can still make a fully informed choice from the 5 they can read — which is
   *strictly more agency than the pre-batch silent auto-pick of the two lowest ObjectIds*, which
   gave them none and told them nothing.
3. **The correct fix is genuinely out of scope.** Capturing candidate identity at ask time widens
   `PendingEffectChoice` → `BlockingDecision` → `LegalAction` → the view, i.e. a second wire-adjacent
   surface in a batch already moving PROTOCOL and HASH. Bolting it on here would have been the
   scope-creep the plan's §9 exists to refuse.

**What is nonetheless a defect this batch can cheaply fix**: two same-resolution-drawn candidates
render as **two buttons with identical text**. `DiscardPicker` keys on `card.id` so they are distinct
objects, but to the human they are indistinguishable, and `(unknown card)` reads as a redaction bug
in the seat's own hand. **Input**: Faithless Looting resolved by the human with 5 cards in hand.
**Wrong output**: a picker showing `Mountain | Swamp | Bolt | Brainstorm | Ponder | (unknown card) |
(unknown card)`.

**Fix**: client-side only, zero wire cost — in the `PickN` arm of `blocking_decision_view`
(`view.rs:2004-2010`), replace the bare `names.label(*id)` fallback for a `Discard` candidate absent
from `NameIndex` with a distinguishing label (e.g. `format!("(card drawn this resolution #{n})")`),
and extend the prompt string at `:1990` with a note that same-resolution draws cannot be named yet,
citing `OOS-ENG1-9`. Update `test_eng1_the_browser_renders_a_pickn_discard`'s premise assertion so it
still fails on a *genuine* label regression (it currently asserts `!= UNKNOWN_LABEL`, which a new
placeholder would silently satisfy — assert against the exact drawn-card prefix instead).

#### Finding 3: the short-circuit determines the set, not the order — LOW

**File**: `crates/engine/src/effects/mod.rs:1301-1312`
**CR**: 601.2c's principle (invoked), CR 608.2f / 404.3 (contradicted)
**Issue**: the comment says "when the answer space admits exactly ONE legal answer the announcement
is DETERMINED". For `n >= hand.len()` the *set* is determined but the *order* is not, and
`EffectChoiceAnswer::Discard::chosen`'s own doc (`stubs.rs:995-997`) states that the order is the
player's choice and is the graveyard order. So the code's justification contradicts the type's doc.
`OOS-ENG1-7` covers the picker's ascending-ids deviation but explicitly not this path.
**Practical impact**: nil today — `check_ids` treats the list as a set and no corpus card reads
graveyard order. This is a doc/consistency defect, not a behaviour defect.
**Fix**: amend the comment to "exactly one legal SET" and add a sentence naming the order half as
part of `OOS-ENG1-7`'s scope.

#### Finding 6: no regression guard on the `Cost::DiscardCard` structural guarantee — LOW

**File**: `crates/engine/tests/primitives/pb_eng1_effect_discard_choice.rs` (absent test)
**Issue**: §2.4 says the cost-site guarantee is "not a nicety" — an ask there records a
`pending_effect_choice` nothing can roll back (`OOS-DP9-14`). Test (d) guards only the `WheelHand`
side, and it guards it imperfectly: a future batch that moved **both** the ask and the short-circuit
into `discard_cards` would leave (d) green (`WheelHand` passes `n == hand_size`, so the short-circuit
fires) while `Cost::DiscardCard` — which passes `n = 1` against a larger hand — would begin recording
undischargeable entries. `optional_cost_and_counter_tax.rs:250/299` exercises the cost path
incidentally and `resolve_top_of_stack:115-119`'s re-entry `debug_assert` is a partial backstop, but
neither is a named guard.
**Fix**: add `test_eng1_a_cost_discard_never_suspends` — pay a `Cost::DiscardCard` from a 3-card hand
and assert `state.pending_effect_choice().is_none()` afterwards, with a comment naming §2.4 and
`OOS-ENG1-1`.

---

## Focus-item disposition (the ten questions asked)

| # | Item | Finding |
|---|------|---------|
| 1 | CR 701.9b / 608.2d / 702.35a / 400.7 | **Clean.** 701.9b default is served at the one resolution-time site; 608.2d placement is the arm, inside the existing wrapper; 702.35a is preserved because both paths share `discard_one_chosen_card` and test (g) discriminates a copy-paste; CR 400.7 is safe — `chosen` ids are collected before any move and each remaining id is untouched, and `discard_one_chosen_card` reads `new_id` back out of `expect_move_object_to_zone` rather than assuming stability. |
| 2 | Suspend/replay determinism | **Clean.** `restart_point = state.clone()` / `*state = restart_point` is a wholesale restore including `next_object_id`, so the replay re-mints identical ids; nothing runs between the abort and the replay (the admission gate admits only the answer and `Concede`, and `discharge_effect_choice_on_concede` abandons the bank). The `hand` derivation is `state.objects` (an `OrdMap`) filtered by zone, which is ascending by construction and additionally `debug_assert`ed. I could not construct a legal command sequence that perturbs hand membership or order between passes. |
| 3 | `continue` vs `return` | **Clean and *proven*.** `test_eng1_multiplayer_discard_exercises_both_loop_exits` drives both exits in one `EachOpponent` resolution and — the part that makes it a real proof — asserts that P2's already-applied determined discard is **rolled back** by P3's suspension and reapplied on the replay. That is exactly the property that makes `return` correct rather than merely working. |
| 4 | The three silent plumbing sites | **Clean.** Check 4 `effects/mod.rs:662-665`; check 5 `:707-739` (before the `_ => unreachable!()`); `api.rs:556-568` (before the `_ =>`). All three agree on: exactly `count`, no duplicates, membership in the **recorded** question's hand. They differ only in check ordering (engine checks count first, `api.rs` membership first), which changes only the error text. |
| 5 | Architecture Invariant 7 | **Clean, with Finding 5/7 as LOW polish.** `private_to()` unchanged at `events.rs:1572`; `entry.player` enforced at `effects/mod.rs:636-641` and at the play-server read guard; `view.rs` uses `names.label` not `question_card_label`, so `test_ui1_view_rs_reads_game_state_in_exactly_the_two_known_places`'s count is unmoved. Gate (j) is **sufficient for the channel it names** — it asserts the decision is entirely absent *and* needles the raw body for `"candidates"` *and* proves the write side 409s — but see Finding 7 for the defence-in-depth gap. |
| 6 | `HashInto` | **Arms are correct; the gate is not.** Discriminant `3u8` in both, append-only, every field fed, `count: u32` via the single `impl HashInto for u32` in both directions (no `usize`/`u32` split). Your stage-D observation is right and is **Finding 1, MEDIUM** — yes, it is worth closing, and worth closing *inside v71*. |
| 7 | The tests | **Non-vacuous, spot-checked by reading.** (b) fails on a revert because it asserts the *lowest*-id card is still in hand — the exact pre-batch outcome. (f)'s five rejections each name a distinct message and each re-asserts `public_state_hash()` **plus** a positive control that an accepted answer *does* move the hash, so the rejections cannot pass for the wrong reason. (g) would fail against a copy-pasted second Madness implementation. (d) is the weakest of the ten — see Finding 6 — but it does catch the naive "move the ask into `discard_cards`" regression it was written for. None is vacuous; none asserts something that was true before the batch. |
| 8 | The three repaired fixtures | **Clean.** No assertion lost meaning — `draw_count == 3`, `net_hand_change == +1`, `drawn == 4` all survive verbatim, because `default_discard_answer` reproduces `min_by_key` exactly. The event merge is correct: `resolve_top_of_stack:152-159` returns **only** the question event on the aborted pass and discards the inner pass's `events`, so `initial_events ++ answer_events` counts each `CardDrawn` once. The Greater Good repair correctly does *not* merge events (it reads final zone counts) and says so. |
| 9 | `decision_gate.rs` | **Arithmetic verified independently.** `BASELINE` now has exactly **80** entries (I counted them), no entry names `discard_cards`, Izzet Charm is `&["counter_unless_pays"]`, `MAX_AUTO_CHOSEN_COMPLETE_UNION = 80` matches the entry count (union is over defs and every entry is a distinct def), `MIN_BASELINE = 50` clears with 30 to spare and was **not** lowered. The 91→80 drop is 11, consistent with 11 solo rows deleted + Izzet Charm retained. The plan said "13 rows / 12 deletions"; the implementation read 12/11 off T9 and **recorded the correction in the constant's doc** — which is the right behaviour, not a defect. T4's fixture swap to `sacrifice_permanents` is correct (that row is still `AutoChosen`, with 12 BASELINE entries hitting it) and the reason is in a comment. T8 gained `("discard_cards", 1)`. |
| 10 | `OOS-ENG1-9` | **Finding 2.** Explicit opinion given: deferral correct, not a net regression, but the placeholder ambiguity should be fixed here. |

## Also-check disposition

| Item | Result |
|---|---|
| **SR-8** (wire closure) | Clean. Both enums entered the closure at v31; only their declared shape moved. PROTOCOL 33→34 with a new `ProtocolEpoch` row and a new `- 34:` history line; no shipped row edited; closure type count claimed unchanged at 96 and stated as gate-read. |
| **SR-9a** (test target layout) | Clean. `mod pb_eng1_effect_discard_choice;` is present at `crates/engine/tests/primitives/main.rs:53`; no new top-level `tests/*.rs`. |
| **SR-4** (silent-failure classification) | Clean. The `Some(other)` arm at `effects/mod.rs:1319-1325` is `debug_assert!(false, ...)` + a deterministic pre-batch-equivalent fallback, structurally identical to the scry/surveil arms, and engine-bug-side is the honest classification (check 4 establishes variant agreement before this code can run). The mana-ability gate's silent default at `:514-527` correctly discharges its obligation onto the widened roster test. |
| **SR-35 / SR-36** | `check-defs-fmt.sh` is moot (0 card-def lines changed, verified). SR-36 is **partly missed** — Finding 8: the roster denominator came from a grep with manual exclusions rather than an `all_cards()` walk, and got 22 where the answer is 21. |
| **Comment debt (§10)** | Mostly honest. `discard_cards`' doc (`effects/mod.rs:9513-9531`) now names the debt, the two remaining no-choice callers, and `OOS-ENG1-1` at the cost caller — accurate and verifiable against the code. `Effect::Connive`'s inline comment (`:5652-5657`) carries `deferred, OOS-ENG1-2` and the CR cite — accurate. **Two sites were missed**: Findings 4 and 5. |
| **Mana-ability roster gate** | Clean. `pb_dp9_effect_choice.rs:2558` now reads `["SearchLibrary", "Scry", "Surveil", "DiscardCards"]` and `rules/mana.rs:878-879` says "four asking effects". |
| **Golden corpus** | Untouched and expected green: `replay_harness.rs:397-409`'s pump answers any `BlockingDecision::EffectChoice` with `default_effect_choice_answer`, which now handles `Discard` and reproduces the pre-batch pick. The new `EffectChoiceScriptAnswer.discard` field is `#[serde(default)]`, backward compatible with every existing script. |
| **Scope: declared out but should have been in** | Nothing. `Cost::DiscardCard` (`OOS-ENG1-1`), Connive (`-2`), `chooser` (`-3`), `MillCards`' missing `.max(0)` (`-6`) are all correctly excluded with stated reasons, and I agree with each. I verified `MillCards` at `effects/mod.rs:1432-1433` is indeed still unclamped, as declared. |
| **Scope: declared in but quietly skipped** | Nothing found. Every row of the plan's §3.9 table is addressed, including the four non-compile-error sites, the `pb_dp9` roster array, the `params.rs` doc, the TUI formatter, and the three `main.rs` picker lists. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.9a | Yes | Yes | `discard_one_chosen_card` hand → graveyard |
| 701.9b (default chooser) | Yes | Yes | tests (a), (b), (i); the affected-player half is the point of (a) |
| 701.9c (cost discard) | N/A — excluded | n/a | `OOS-ENG1-1`, structural exclusion, correct |
| 608.2d (announce while applying) | Yes | Yes | (a), nested test |
| 608.2f / 404.3 (discard order) | Partially | No | order honoured by the engine loop; picker submits ascending (`OOS-ENG1-7`); short-circuit path not covered by that seed — Finding 3 |
| 601.2c principle (determined) | Yes | Yes | (c), four cases |
| 702.35a (Madness → exile) | Yes | Yes | (g), and it discriminates |
| 514.1 (contrast with cleanup) | Yes | Yes | (e), both defaults pinned side by side |
| 400.7 | Yes | Yes | implicitly by (b)/(g); the handoff records the id-427 trap |
| 605.1b / 605.4a (mana gate) | Yes | Yes | roster widened to four effects |
| 104.3a / 800.4 (dead answerer) | Yes (unchanged) | pre-existing | correct for discard with no change |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| — (0 defs changed) | n/a | n/a | n/a | `fell_specter.rs` re-read against oracle and confirmed correct as-is; the 12 deck-legal `Complete` defs carrying `Effect::DiscardCards` all become CR 701.9b-correct with no def edit. `fable_of_the_mirror_breaker.rs` correctly stays `partial` (`OOS-ENG1-8`) — its TODO names this primitive but needs optional + up-to-N + a count-driven draw. |

## Previous Findings

First review of ENG-1. No previous findings table.
