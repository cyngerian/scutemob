# Primitive Batch Review: PB-DP5 — the `WouldDraw` multi-replacement prompt is unanswerable

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-153` · branch `feat/pb-dp5-woulddraw-multi-replacement-prompt-is-unanswerable-th`
**CR Rules**: 616.1 / 616.1a / 616.1e / 616.1f · 614.5 · 614.10 · 614.11 / 614.11a · 121.1 / 121.2 / 121.3 · 104.3b · 702.52a
**Engine files reviewed**:
`crates/card-types/src/state/replacement_effect.rs`,
`crates/engine/src/state/{mod.rs,builder.rs,hash.rs}`,
`crates/engine/src/rules/{replacement.rs,turn_actions.rs,loop_detection.rs,engine.rs,protocol.rs}`,
`crates/engine/src/effects/mod.rs`, `crates/engine/src/lib.rs`
**Test files reviewed**:
`crates/engine/tests/primitives/{main.rs,pb_dp5_pending_draw_choice.rs}`,
`crates/engine/tests/rules/replacement_effects.rs`,
`crates/engine/tests/core/{hash_schema.rs,bare_lookup_ratchet.rs}`
**Card defs reviewed**: **0 modified — correct.** Independently verified: `rg WouldDraw crates/card-defs/src/defs/`
returns exactly one hit, a `Completeness::inert` *note* in `out_of_the_tombs.rs:32`. No corpus card
registers a `WouldDraw` replacement, so DP-5 is unreachable from a legal deck and the plan's
"0 card-def edits" prediction (§11) holds.

## Verdict: **needs-fix** (ship-blocking on Finding 2 only)

The engine change is sound and does what the plan says: `GameState.pending_draws` is a
`pub(crate)` `Vector<PendingDraw>` with builder init, a read accessor, a complete `HashInto`
(all four fields), a `public_state_hash` feed and a `loop_detection` mirror; all **three** emit
sites (including the third, `draw_card_skipping_dredge`, which the original audit never named)
now record pending state; `handle_order_replacements` grows a correctly-guarded second arm; and
the CR 614.11a sequence semantics are real — `draw_cards_for_player` owns the loop, breaks on
deferral, stashes `remaining`, and `resolve_pending_draw` honours it on resume. All four
`effects/mod.rs` call sites were converted correctly (no double-draw, no lost draw). Gates are
clean: HASH 63→64 with an appended (not edited) v64 `HASH_SCHEMA_HISTORY` row carrying both
freshly-computed fingerprints, `FROZEN_HISTORY_PREFIX_DIGEST` genuinely re-pinned
(`392afa…` → `b7dfea…`, checked against the pre-batch checkout), PROTOCOL unmoved at 27, SR-9a
`mod` line present, SR-25 ratchet moved 111→110 in `effects/mod.rs` with a per-site justification
and the other two swept files unmoved. Nothing gates priority, SBAs or step advancement on
`pending_draws` — verified by exhaustive grep — so hard constraint 3 holds and an unanswered
entry leaves the game exactly as playable as before. Every out-of-scope item was left alone
(`crates/simulator` and `tools/` contain zero `OrderReplacements` / `pending_draws` occurrences).

Two things need fixing before collect. **Finding 2 is the ship-blocker**: acceptance criterion
5534's "audit DP-5 row + PB-DP5 row updated" is simply not done — `docs/audits/decision-point-audit.md`
is byte-unchanged and `rg OOS-DP5 docs/` returns **zero** hits, so all eight seeds the plan
enumerated, including **OOS-DP5-7 (a live, reachable free-card exploit on `Command::ChooseDredge`)**,
would be lost at collect. **Finding 1** is the one the coordinator asked to be stress-tested and
it is a real, if narrow, defect: the runner's "already terminal per call" argument for dropping
the CR 616.1f loop is **false** in the CR 616.1a single-self-replacement branch of
`determine_action`, though it is correct for every non-self chain of any length, is not a
regression, and is unreachable from the corpus today. The remaining findings are one dead
`pub` escape hatch, one genuine test hole (no test ever builds a `PendingDraw` with a non-empty
`already_applied`), and seven LOWs.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `rules/replacement.rs:711-714`, `:767-775`, `:1302-1310` | **The single re-check is NOT equivalent to the CR 616.1f loop when a self-replacement is involved.** `determine_action` returns `AutoApply` with 2+ effects still applicable (CR 616.1a); if that effect is non-`SkipDraw` the draw is performed and every remaining applicable replacement is dropped. **Fix:** restore the loop, or correct the two doc comments that assert the equivalence and seed the hole. |
| 2 | **HIGH** | `docs/audits/decision-point-audit.md` (unchanged) | **Acceptance criterion 5534 unmet: the audit was never updated and zero OOS-DP5 seeds were filed.** Loses OOS-DP5-7, a live exploit. **Fix:** apply the plan's §11 audit edits and file OOS-DP5-1..8 in §8.1 before collect. |
| 3 | MEDIUM | `state/mod.rs:730-733` | **`pending_draws_mut()` is dead** — zero consumers in engine or tests. Widens the SR-3 seal for nothing. **Fix:** delete it, or add the test that needs it. |
| 4 | MEDIUM | `tests/primitives/pb_dp5_pending_draw_choice.rs` (all tests) | **No test ever exercises a `PendingDraw` with a non-empty `already_applied`.** The CR 616.1f multi-round branch and the SR-9b sort at `:796-797` are untested. **Fix:** add a 3-effect chain test. |
| 5 | LOW | `rules/replacement.rs:1326` | Resume guard omits `LostToEmptyLibrary`, so a second `PlayerLost` can be emitted. **Fix:** add it to the `matches!`. |
| 6 | LOW | `rules/replacement.rs:807-808` | Doc claims the eliminated/conceded guard runs at "every call site"; `draw_cards_for_player` has none. **Fix:** correct the comment (or add the guard). |
| 7 | LOW | `card-types/.../replacement_effect.rs:401`, `rules/replacement.rs:765` | Docs still name `effects::draw_one_card`, renamed to `draw_cards_for_player`. **Fix:** update both. |
| 8 | LOW | `tests/core/hash_schema.rs:182-187` | `FROZEN_HISTORY_PREFIX_DIGEST` correctly re-pinned but the comment still credits "PB-OS11 … 62→63". **Fix:** append a PB-DP5 63→64 line. |
| 9 | LOW | `effects/mod.rs:8569-8574` | New `break` on `LostToEmptyLibrary` changes "draw 3 from an empty library" from 3 attempts to 1 (CR 121.3). Outcome-identical. **Fix:** record in the DP-5 audit row; also confirm the commit message records `draw_card`'s `?` → `expect_*` error-surface change (plan risk 2). |
| 10 | LOW | `tests/primitives/pb_dp5_pending_draw_choice.rs:606-735` (T10) | The load-bearing fall-through case — draw ids submitted **while a zone change is also pending** — is not covered. **Fix:** answer the draw first in T10. |
| 11 | LOW | `rules/replacement.rs` (`pending_draws` lifecycle) | Entries are never cleaned up (turn end, player loss, source removal). Not exploitable, but they feed the hash/loop fingerprint forever and hand the drawing player unlimited timing control over an owed draw. **Fix:** seed alongside OOS-DP5-2/5; no code change in this PB. |
| 12 | LOW | `rules/replacement.rs:205` | FIFO selection means a player with two outstanding pending draws cannot choose which to answer, and an answer aimed at the newer entry is silently applied to the older one. Card-neutral. **Fix:** note in the plan's §2.2 selection rule and seed. |

### Finding Details

#### Finding 1: the CR 616.1f re-check is not equivalent to a loop when a self-replacement is applicable

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:711-714` (the `else → Proceed` arm),
`:767-775` (`perform_one_draw` doc), `:1302-1310` (`resolve_pending_draw` doc)
**CR Rule**: 616.1a — *"If any of the replacement and/or prevention effects are self-replacement
effects (see rule 614.15), one of them must be chosen."*; 616.1f — *"Once the chosen effect has
been applied, this process is repeated (taking into account only replacement or prevention
effects that would now be applicable) until there are no more left to apply."*

**The runner's claim, tested.** The runner's stated deviation is that `resolve_pending_draw`
makes a **single** call to `perform_one_draw` instead of the plan's explicit CR 616.1f loop,
justified by *"`check_would_draw_replacement`'s own `AutoApply` dispatch is already terminal
per call (it only ever fires when exactly one replacement remains applicable, so there is
nothing left to loop over afterward)"*.

The parenthetical is **false**. `determine_action` (`:82-123`) returns `AutoApply` in **two**
situations, not one:

1. `applicable.len() == 1` (`:91-93`) — genuinely terminal.
2. `applicable.len() >= 2` **and exactly one of them is a self-replacement** (`:105-108`,
   CR 616.1a) — **not** terminal: after the self-replacement is applied, CR 616.1f requires
   repeating the process over the still-applicable remainder.

In case 2, `check_would_draw_replacement` takes the auto-applied id, finds its modification is
not `SkipDraw`, and falls into the `else` at `:711-714` → `DrawAction::Proceed`. `perform_one_draw`
then performs the draw and returns `Completed`. Every other applicable replacement — **including
a `SkipDraw` that CR 616.1f says must now be applied, which would mean no card is drawn at all** —
is silently discarded, and the auto-applied id is never recorded in `already_applied` either.

Concrete divergence: applicable = `{S = self-replacement, RedirectToZone(Exile); X = non-self,
SkipDraw}`.
- CR: 616.1a forces S first; 616.1f then repeats and finds X applicable; X is a `SkipDraw`; the
  draw is replaced by nothing. **No card drawn.**
- Engine (both pre- and post-PB-DP5): `AutoApply(S)` → non-`SkipDraw` → `Proceed` → **card drawn.**
- The plan's §3.2 loop (`AutoApply(id) → SkipDraw? stop : insert id into already_applied and LOOP`)
  would have produced the CR answer.

**Chains of 3+ that the coordinator asked about, without a self-replacement, ARE equivalent.**
Traced exhaustively: after the player's choice, `already_applied` grows by one and
`find_applicable` excludes it (`:53-56`). If 2+ remain → `NeedsChoice` → a fresh `PendingDraw`
and a second round-trip (correct: CR 616.1f needs another CR 616.1e choice). If exactly 1 remains
→ `AutoApply`; `SkipDraw` stops (correct), non-`SkipDraw` proceeds — and the loop version, having
inserted that last id, would find zero applicable and also proceed. Identical. So the single call
is a legitimate simplification for every chain reachable without a self-replacing draw
replacement.

**Reachability.** `register_permanent_replacement_abilities` (`:2362-2368`) copies
`is_self_replacement: *is_self` verbatim from `AbilityDefinition::Replacement`, so a `WouldDraw`
self-replacement **is** authorable in the DSL. No corpus card registers any `WouldDraw`
replacement at all (verified), so this is unreachable in a real game today, and the same arm
behaved identically pre-PB-DP5 — **this is not a regression**, which is why it is MEDIUM and not
HIGH. But the PB explicitly rests its correctness argument on the equivalence, and the two doc
comments now assert it as fact for future readers.

**Fix**: preferred — restore the plan's §3.2 loop inside `check_would_draw_replacement` (or
`perform_one_draw`): on `AutoApply(id)` whose modification is **not** `SkipDraw`, insert `id`
into `already_applied` and re-run `find_applicable`/`determine_action` rather than returning
`Proceed`; the plan's termination proof (`already_applied` strictly grows, `find_applicable`
excludes it, bound = `state.replacement_effects.len()`) applies verbatim and must be a real doc
comment. Acceptable alternative — leave the code as-is, but (a) strike the "already terminal per
call" sentence from `perform_one_draw`'s doc (`:773-775`) and `resolve_pending_draw`'s
(`:1302-1310`), replacing it with the precise statement "*equivalent to the CR 616.1f loop for
every chain of non-self-replacement effects; the CR 616.1a single-self-replacement branch of
`determine_action` still drops the remainder — see OOS-DP5-N*", and (b) file that as a seed
folded into OOS-DP5-6/OOS-DP5-8 (the `WouldDraw`-widening PB), since the widening PB is where a
self-replacing draw replacement would first become authorable.

#### Finding 2: the audit was never updated and **zero** OOS-DP5 seeds were filed

**Severity**: HIGH (process/bookkeeping, not engine correctness — but an explicit acceptance
criterion, and it silently drops a live-exploit finding)
**File**: `docs/audits/decision-point-audit.md` — unchanged
**Acceptance criterion 5534**: *"`cargo test --all`, clippy, `cargo fmt --check` **and**
`tools/check-defs-fmt.sh` clean; **audit DP-5 row + PB-DP5 row updated**."*
**Plan §11 checklist**: *"`docs/audits/decision-point-audit.md` updated: §4 L383 (third site),
§5 DP-5 row (SHIPPED + the W1/W5 corrections + the §5.3 deviation), §8 PB-DP5 row (prediction
confirmed/falsified), §8.1 (OOS-DP5-1..8)."*

**Issue**: verified independently —
- `docs/audits/decision-point-audit.md:432` (the DP-5 row) still reads *"The `WouldDraw`
  multi-replacement prompt is unanswerable and the draw is destroyed … Reachable with any two
  `WouldDraw` replacements"* with no SHIPPED marker, no third-emit-site correction, and none of
  the plan's §9 W1/W5 corrections (W1: DP-5 is **not** reachable from a legal deck; W5: the
  effect path emitted N prompts and drew zero, not "the draw is eaten" singular).
- `:383` still names only two emit sites.
- `:581` (the §8 PB-DP5 row) is unchanged; the HASH-bump prediction is neither confirmed nor
  recorded as confirmed.
- `rg OOS-DP5 docs/` → **0 matches across 0 files.** None of OOS-DP5-1..8 exists anywhere
  outside the plan file.

The most serious consequence is **OOS-DP5-7**, which the plan itself describes as *"a live,
reachable exploit … arguably a higher-severity finding than DP-5 itself"*: `rules/engine.rs:302-306`
gates `Command::ChooseDredge` with `validate_player_exists` only, and `handle_choose_dredge`
(`replacement.rs:2819`) validates the *card* but never that a draw is pending, so any player at
any time can send `ChooseDredge { card: None }` and take a free card via
`draw_card_skipping_dredge`. Confirmed still true on this branch. If it is not written into
§8.1 it disappears at collect along with the plan file's currency.

**Fix**: before signalling collect, make the four audit edits the plan's §11 enumerates
(§4 L383, §5 DP-5 row, §8 PB-DP5 row, §8.1 OOS-DP5-1..8) and update the doc's
`<!-- last_updated: -->` stamp. Add to that seed list the two items this review adds:
Finding 1's CR 616.1a hole and Finding 11's `pending_draws` lifecycle/timing note.

#### Finding 3: `pending_draws_mut()` has no consumer

**Severity**: MEDIUM
**File**: `crates/engine/src/state/mod.rs:730-733`
**Invariant**: Architecture Invariant #3 / SR-3 — `GameState` is sealed `pub(crate)`; the only
mutation path is a `Command` through `process_command`. Coordinator hard constraint 4: *"a `_mut`
escape hatch **only if genuinely needed**, gated the way its siblings are."*
**Issue**: `rg pending_draws_mut` across the whole worktree returns the declaration and two plan
mentions — nothing else. Engine code mutates the field directly (`replacement.rs:798`, `:1292`,
in-crate); the tests use `pending_zone_changes_mut` and `replacement_effects_mut` but never
`pending_draws_mut`. The plan justified it on two grounds (§2.1): `effects/mod.rs` needing to
"decrement / write `remaining` on the entry it just pushed", and tests needing to construct
pending states directly. Neither materialized — `remaining` is passed *into* `perform_one_draw`
as `remaining_after` and written at push time, and every test builds its pending draw by actually
calling `draw_card`. The result is a `pub` method that lets any downstream crate mutate the queue
outside the Command path, for zero benefit. Clippy will not flag it because it is `pub`.
**Fix**: delete `pending_draws_mut()`. If you would rather keep it, add the test that needs it —
the natural one is Finding 11's stale-entry probe (construct an ancient `PendingDraw` directly and
assert an `OrderReplacements` naming a no-longer-applicable id is rejected).

#### Finding 4: nothing tests a `PendingDraw` with a non-empty `already_applied`

**Severity**: MEDIUM (per `memory/conventions.md`, "test-validity MEDIUMs are fix-phase HIGHs")
**File**: `crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs` (whole file)
**CR Rule**: 614.5 (an effect applies at most once per event) · 616.1f (repeat until none left)
**Issue**: every test registers exactly **two** `WouldDraw` effects. After the first choice,
`find_applicable` therefore always returns exactly one id, `determine_action` returns `AutoApply`,
and the chain terminates in that same call. Consequently the branch at `replacement.rs:787-805`
that pushes a **second** `PendingDraw` carrying the *grown* `already_applied` — the whole point
of the CR 614.5 threading, and the only place the SR-9b determinism sort at `:796-797` can
matter — is **never executed by any test**. T6 does produce successive deferrals, but they come
from the CR 614.11a *resume loop* (`:1327-1344`), which passes `HashSet::new()`, so
`already_applied` is empty there too. `pending_draws()[0].already_applied` is asserted exactly
once in the suite (T0, `replacement_effects.rs:3034-3037`) and the assertion is
`is_empty()`.

A wrong-but-plausible implementation that dropped `already_applied` entirely on the re-defer
(pushing `PendingDraw { already_applied: vec![], .. }`) would pass all 14 tests, and would then
spin the player through the same choice forever, re-offering an effect CR 614.5 forbids.

**Fix**: add T14 with **three** `WouldDraw` effects for the same player — e.g. two
`RedirectToZone(Exile)` (draw no-ops) plus one `SkipDraw` — and assert, after the first
`OrderReplacements`:
- a **second** `ReplacementChoiceRequired` was emitted and its `choices` **excludes** the chosen id;
- `pending_draws().len() == 1` and `pending_draws()[0].already_applied == vec![chosen_id]`;
- re-submitting the already-applied id is **rejected** with the applicability error (CR 614.5);
- answering the second prompt with the `SkipDraw` id terminates the chain with no `CardDrawn`.

#### Finding 5: the CR 614.11a resume runs after the deferred draw itself emptied the library

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1326`
**CR Rule**: 104.3b / 121.3
**Issue**: the guard is `if !matches!(outcome, DrawStepOutcome::Deferred) && pending.remaining > 0`.
`perform_one_draw` can also return `LostToEmptyLibrary`, in which case the resume loop runs, its
first iteration hits the empty library again, emits a **second** `GameEvent::PlayerLost`, and
breaks. `draw_cards_for_player` (`effects/mod.rs:8569-8574`) breaks on both outcomes; the two
loops should agree.
**Fix**: `if !matches!(outcome, DrawStepOutcome::Deferred | DrawStepOutcome::LostToEmptyLibrary)
&& pending.remaining > 0`.

#### Finding 6: `perform_one_draw`'s eliminated/conceded claim is false for one of its four callers

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:807-808` — *"The eliminated/conceded guard runs
in every call site before this is reached."*
**Issue**: verified per caller —
`turn_actions::draw_card:1176-1180` ✅ · `draw_card_skipping_dredge:2934-2938` ✅ ·
`resolve_pending_draw` ✅ *indirectly* (`engine.rs:224` runs `validate_player_active`, and the draw
arm only routes when `pending.player == sender`, so the drawing player is the validated one) ·
`effects::draw_cards_for_player:8555` ❌ **no guard**. This is not a regression — the pre-batch
`draw_one_card` (`effects/mod.rs:8547` on `main`) had none either — but the comment is now wrong.
**Fix**: either narrow the comment to name the three sites that do guard and record the effect
path's gap, or add the `has_lost || has_conceded` early-return to `draw_cards_for_player`
(behaviour change: seed it rather than doing it as a drive-by).

#### Finding 7: doc comments name a function that no longer exists

**Severity**: LOW
**File**: `crates/card-types/src/state/replacement_effect.rs:401`; `crates/engine/src/rules/replacement.rs:765`
**Issue**: `PendingDraw.sets_has_drawn_for_turn`'s doc says *"`false` for `effects::draw_one_card`
(which does not)"* and `perform_one_draw`'s doc says the same. `draw_one_card` was renamed
`draw_cards_for_player` in this very batch.
**Fix**: rename both references.

#### Finding 8: `FROZEN_HISTORY_PREFIX_DIGEST` re-pin is unattributed

**Severity**: LOW
**File**: `crates/engine/tests/core/hash_schema.rs:182-187`
**Issue**: the digest itself is correct — verified `392afa3c…` (pre-batch checkout) → `b7dfea87…`
(this branch), i.e. it was genuinely re-pinned because the v63 row joined the frozen prefix, not
hand-forced. But the explanatory comment still reads *"PB-OS11 (2026-07-19): re-pinned on the
62→63 bump"*. The file's own convention (`:1181-1186`) is that the re-pin is annotated with the
bump that caused it, so a future reader will mis-attribute this value.
**Fix**: append *"PB-DP5 (2026-07-26): re-pinned on the 63→64 bump — version 63 became a
superseded row and joined the frozen prefix."*

#### Finding 9: undocumented behaviour change on the empty-library draw sequence

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:8569-8574`
**CR Rule**: 121.3 — *"A player who attempts to draw a card from a library with no cards in it
loses the game the next time a player would receive priority."*
**Issue**: pre-batch, `Effect::DrawCards { count: 3 }` against an empty library called
`draw_one_card` three times and emitted three `PlayerLost` events (three *attempts*). The new
`break` on `LostToEmptyLibrary` emits one. `has_lost` is set either way so the game outcome is
identical, and the new shape is cleaner event-stream-wise, but arguably CR 121.3 describes each
draw as an attempt. Not listed among the plan's deviations.
**Fix**: no code change required; record it in the DP-5 audit row alongside the §5.3 deviation.
While there, confirm the Phase 2 commit message records the other predicted behaviour change
(plan risk 2): `turn_actions::draw_card` moved from `state.zone(..)?` / `move_object_to_zone(..)?`
to the `expect_*` forms, so a corrupted-state lookup now `debug_assert!`s and swallows instead of
returning `Err`.

#### Finding 10: the routing fall-through's load-bearing case is untested

**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs:606-735` (T10)
**Issue**: the plan's §4 argument is that a draw answer submitted **while a zone change is also
pending** falls through the zone-change arm's applicability check into the draw arm. T10 submits
the zone-change ids first; by the time it submits the draw ids the zone change is already
resolved (`state.pending_zone_changes().is_empty()` is asserted at `:708-711`), so the
fall-through is never exercised. The source is correct — `handle_order_replacements:197-203`
returns only when `ids.iter().all(|id| applicable.contains(id))`, otherwise falls through to
`:205` — but nothing pins it.
**Fix**: reverse T10's order (answer the **draw** first while both are pending, assert
`pending_zone_changes()` is untouched, then answer the zone change), or add a T10b that does.

#### Finding 11: `pending_draws` entries are never cleaned up

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs` (lifecycle); `state/hash.rs:7739`;
`rules/loop_detection.rs:146-150`
**Issue**: an entry is removed only by `resolve_pending_draw`. There is no end-of-turn sweep, no
removal when the drawing player loses or concedes, and no removal when the replacement effects
that raised the prompt stop being applicable (source leaves the battlefield). Assessed against
the coordinator's exploit question:
- **Free cards: no.** `handle_order_replacements:214` requires every submitted id to be in
  `find_applicable(...)` *now*, not merely registered, so an entry whose replacements have
  expired is unanswerable and inert. Nothing else consumes the entry.
- **Answering after losing: no.** `engine.rs:224` `validate_player_active` rejects a
  `has_lost || has_conceded` sender before `handle_order_replacements` is reached. (Note that
  `resolve_pending_draw` itself has no such guard — it is safe only because of that dispatch
  check plus the `pending.player == sender` routing condition. Worth a comment.)
- **Real residue**: the entry keeps feeding `public_state_hash` and the CR 104.4b loop-detection
  fingerprint for the rest of the game, and — the more interesting one — the drawing player has
  **unbounded timing control over an owed draw**: `OrderReplacements` requires no priority, so
  they may answer during an opponent's turn, mid-combat, after seeing information, and pull down
  `remaining + 1` cards at that moment. That is a strict improvement on the status quo (the
  draws used to be destroyed) and is inherent to the §5.1 "recorded, non-blocking obligation"
  semantics, but it is a deviation the plan does not name and belongs next to OOS-DP5-5.
**Fix**: no code change in PB-DP5. Add the timing-control deviation to the DP-5 audit row and
fold the lifecycle question into OOS-DP5-2 (the deadline sweep), which is the natural place to
also decide "drop the entry when the player is eliminated".

#### Finding 12: FIFO selection can apply an answer to the wrong pending draw

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:205`
**Issue**: `state.pending_draws.iter().position(|p| p.player == player)` takes the oldest entry.
With two outstanding entries A (older, `already_applied = []`) and B (newer,
`already_applied = [X]`), `find_applicable` for A is a **superset** of that for B, so any
well-formed answer aimed at B also passes A's applicability check and is applied to A. The player
cannot address B while A is outstanding. No card is gained or lost (both draws are owed and both
entries eventually resolve), and this matches the plan's declared §2.2 FIFO rule and the
zone-change arm's existing `.position(..)` shape, so it is not a defect — but it is an
undocumented user-visible consequence of the `Vector` choice.
**Fix**: none in this PB. Note it in the plan's §2.2 selection-rule paragraph and seed it with
OOS-DP5-1 (the `LegalAction`), which is where a per-entry discriminator would naturally be
designed.

## Point-by-point response to the review brief

| # | Question | Answer |
|---|----------|--------|
| 1 | Is the single re-check equivalent to a loop for 3+ replacements? | **For non-self chains, yes** — traced exhaustively; after each choice either 2+ remain (new `NeedsChoice`, correct per CR 616.1f) or exactly 1 remains (`AutoApply`; applying it leaves 0, same as the loop). **For the CR 616.1a single-self-replacement branch, no** — see Finding 1. Real defect, narrow, pre-existing, unreachable from the corpus. |
| 2 | CR 614.11a citation, `remaining` on resume, 4 call sites | **Citation verified verbatim** via the CR server: 614.11a is exactly *"If an effect replaces a draw within a sequence of card draws, all actions required by the replacement are completed, if possible, before resuming the sequence."* `remaining` is written at push (`:801`), read at `:1317`/`:1326-1328`, and decremented correctly (`pending.remaining - 1 - i`). All four `effects/mod.rs` sites converted: `:658` (`Effect::DrawCards`, outer `for p in players` retained — per-seat sequences stay independent), `:713` (`WheelDraw::GreatestDiscarded`), `:746` (`ThatMany \| Fixed`), `:4754` (Connive — the `for _ in 0..n` at `:4761` is the *discard* loop, correctly left alone). **No double-draw, no lost draw.** |
| 3 | Is the applicability routing total? Do both SR-29 checks survive? | **Totality confirmed in source.** `trigger_matches` (`:297-410`) pattern-matches on `(effect_trigger, event_trigger)` pairs of the *same* variant and ends in `_ => false` (`:409`), so a `WouldChangeZone` id can never appear in `find_applicable(WouldDraw{..})` and vice versa — the candidate sets are provably disjoint and "zone change first" is a tie-break that cannot fire. **Both security checks survive**: affected-chooser (`:205`, `p.player == player`) and currently-applicable (`:213-214`, `find_applicable` + `all`, not mere registration). A hostile client submitting a registered-but-inapplicable id gets the `:226-233` error — pinned by T9. |
| 4 | No hang / no new failure mode | **Confirmed.** Exhaustive grep: `pending_draws` appears only in `state/{mod,builder,hash}.rs`, `rules/loop_detection.rs` and `rules/replacement.rs`. Nothing in `sba.rs`, `engine.rs` (priority/`handle_all_passed`/`force_resolve_overdue_payments`), `turn_structure.rs`, `crates/simulator` or `tools/`. Unlike `pending_zone_changes` (which the SBA loop skips), `pending_draws` gates nothing. T12 is the regression guard. Unanswered ⇒ draw lost ⇒ exactly the pre-batch outcome. |
| 5 | Stale-entry hazards | Not exploitable for free cards; unanswerable after the source leaves; unanswerable after loss/concede (dispatch guard). Never cleaned up. Real residues are hash/fingerprint residue and unbounded player-chosen timing on an owed draw. Findings 11 + 12. |
| 6 | Test vacuity / CR citations / fail-before honesty | **T2/T3 are non-vacuous** — same setup, orders `[601,600]` vs `[600,601]`, asserting **different** first `ReplacementEffectApplied.effect_id`; 601 is the second-registered id, so `applicable[0]`, `choices.first()` and registration order are all ruled out. **T11 is stronger still** (`applied_ids == vec![1101, 1100]`, exact sequence). **T0 satisfies 5532's "replacement applied" branch** and asserts the chosen id 601, not merely that *some* order applied; **T4 satisfies the "card in hand" branch** (hand 1, library 0, `cards_drawn_this_turn` 1, `CardDrawn`, `pending_draws()` empty). **T6 is the strongest test in the batch** (1 prompt not 3, `remaining` 2→1→0, hand 1→2→3). CR citations spot-checked against the rules server: 616.1/616.1a/616.1e/616.1f, 614.5/614.10/614.11a, 121.1/121.2, 702.52a all match the doc comments' use. **The one real hole is Finding 4.** The fail-before table is honest about its method (a probe file using only pre-fix-stable signatures) but its "FAILS" rows conflate probe behaviour with test behaviour — T1/T4/T5/T6/T7/T10/T11 all call `pending_draws()` and would not compile pre-fix either, same as T0. LOW, cosmetic. |
| 7 | Gates | **All green.** HASH 63→64 (`state/hash.rs:591`), `- 64:` History line (`:578-590`), appended v64 `HashSchemaEpoch` with both fresh fingerprints (`:892-900`) and **no existing row edited**; `FROZEN_HISTORY_PREFIX_DIGEST` genuinely re-pinned (verified against the pre-batch checkout) though unattributed (Finding 8); local sentinel at 64 (`tests/core/hash_schema.rs:1194`). PROTOCOL **27** unmoved (`protocol.rs:260`), `PendingDraw` reachable only from `GameState`. SR-3 seal not widened *in principle* — field is `pub(crate)`, read accessor added — but the unused `pub` `_mut` hatch is Finding 3. SR-9a `mod pb_dp5_pending_draw_choice;` present (`tests/primitives/main.rs:25`). SR-19: `HashInto for PendingDraw` (`hash.rs:2966-2976`) reads all four fields. SR-25 ratchet: `effects/mod.rs` 111→110 with a per-site justification comment (`bare_lookup_ratchet.rs:94-98`) — the three consolidated bare lookups in `draw_one_card` (`state.zones.get(&lib_id)` and friends) are genuinely gone into `expect_*` forms; `rules/replacement.rs` (24) and `rules/turn_actions.rs` (7) correctly unmoved. |
| — | Out-of-scope items | **All untouched, as designed.** No `LegalAction` for `OrderReplacements` (`crates/simulator`: 0 occurrences of `OrderReplacements` **and** 0 of `pending_draws`). No widening beyond `SkipDraw` (`check_would_draw_replacement:711-714` still falls through to `Proceed`). No `advance()`/`AwaitingHuman` change. OOS-DP5-7's `ChooseDredge` gate not touched — correct scope discipline, but see Finding 2: the seed must be *filed*, not just avoided. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 616.1 (affected player chooses) | Yes | Yes | `PendingDraw.player`; T1/T8 |
| 616.1a (self-replacement first) | **Partial** | No | `determine_action:94-116` orders correctly, but the non-`SkipDraw` `AutoApply` outcome drops the remainder — Finding 1 |
| 616.1e (any applicable may be chosen) | Yes | Yes | T2/T3/T11 |
| 616.1f (repeat with now-applicable) | **Partial** | Partial | Correct for non-self chains; the multi-round branch is untested (Finding 4); the 616.1a case diverges (Finding 1) |
| 614.5 (at most once per event) | Yes | **No** | `already_applied` threaded end-to-end but never observed non-empty by a test — Finding 4 |
| 614.10 (SkipDraw replaces with nothing) | Yes | Yes | T11 both orders |
| 614.11 / 614.11a (sequence resumes after the replacement) | Yes | Yes | `PendingDraw.remaining`, `draw_cards_for_player` break, `resolve_pending_draw:1326-1345`; T6 |
| 121.1 (`cards_drawn_this_turn`) | Yes | Yes | T4 |
| 121.2 (each draw is its own event) | Yes | Yes | T6 — one prompt per draw, not one per sequence |
| 121.3 / 104.3b (empty library) | Yes | No | `LostToEmptyLibrary`; Findings 5 + 9 |
| 702.52a (dredge) | Preserved | Yes | `offer_dredge` flag; T7 covers the third emit site. Resume never re-offers — stated deviation, seed OOS-DP5-3 (**unfiled**, Finding 2) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| *(none)* | n/a | n/a | n/a | 0 card defs modified — verified correct. The corpus registers zero `WouldDraw` replacements (`rg WouldDraw crates/card-defs/src/defs/` → 1 hit, an `inert` *note* in `out_of_the_tombs.rs:32`), so DP-5 is unreachable from a legal deck and there was nothing to fix. `laboratory_maniac.rs` / `teferis_ageless_insight.rs` / `out_of_the_tombs.rs` remain `inert` on the `SkipDraw`-only limitation, correctly deferred to OOS-DP5-6. |

## Required before collect

1. **Finding 2** (ship-blocker) — update `docs/audits/decision-point-audit.md` §4 L383, §5 DP-5
   row, §8 PB-DP5 row and file **OOS-DP5-1..8** in §8.1, plus the two seeds this review adds.
2. **Finding 1** — either restore the CR 616.1f loop or correct both doc comments and seed the
   CR 616.1a hole.
3. **Finding 3** — delete `pending_draws_mut()` (or give it a consumer).
4. **Finding 4** — add the 3-effect / non-empty-`already_applied` test.
5. Findings 5–12 are LOW and may be dispositioned in the close-out rather than fixed, except
   Finding 5 which is a two-token change and should just be made.

## Fix cycle (runner)

**Scope note**: per coordinator instruction, **Finding 2 was explicitly SKIPPED in this cycle**
(the audit-doc update and OOS-DP5-1..8 seed filing are being done separately, in parallel, by
the coordinator — do not read its absence below as an oversight). Findings 1 and 3–12 were
worked. `docs/audits/decision-point-audit.md` and `CLAUDE.md` were not touched.

| # | Severity | Disposition | Notes |
|---|----------|-------------|-------|
| 1 | MEDIUM | **FIXED** (code, not just doc) | See "Finding 1 — detailed resolution" below. |
| 2 | HIGH | **SKIPPED per coordinator instruction** | Being done separately by the coordinator in parallel; do not collect on this branch's absence of audit edits as evidence the fix cycle missed it. |
| 3 | MEDIUM | **FIXED** — deleted | `pending_draws_mut()` removed from `crates/engine/src/state/mod.rs`. Confirmed zero consumers workspace-wide (`rg pending_draws_mut` → only the declaration, pre-fix) and Finding 4's new test does not need it (built via real `Command`/effect execution, not direct state construction). |
| 4 | MEDIUM | **FIXED** | Added `test_dp5_third_effect_forces_second_choice_with_nonempty_already_applied` (T14) — 3 non-self `WouldDraw` replacements (two `RedirectToZone(Exile)`, one `SkipDraw`); choosing the first leaves 2 applicable, forcing a genuine second `NeedsChoice` whose re-pushed `PendingDraw` carries `already_applied = [chosen_id]`. Asserts: the second prompt excludes the chosen id; `pending_draws()[0].already_applied == vec![chosen_id]`; re-submitting the already-applied id is rejected (CR 614.5); answering the second prompt with the `SkipDraw` id ends the chain with no `CardDrawn`. **Verified non-vacuous**: temporarily changed the re-defer push to `already_applied: vec![]`, re-ran — test failed with `left: [] / right: [ReplacementId(1200)]` exactly as expected — then restored the line byte-for-byte and reran to confirm 15/15 pass again. |
| 5 | LOW | **FIXED** | `resolve_pending_draw`'s resume guard now excludes `DrawStepOutcome::LostToEmptyLibrary` in addition to `Deferred`, matching `draw_cards_for_player`'s own break condition — prevents a second `PlayerLost` on an already-empty library. |
| 6 | LOW | **FIXED** (comment) | The "eliminated/conceded guard runs in every call site" claim in `perform_one_draw`'s `Proceed` arm was false for `effects::draw_cards_for_player` (no guard, pre-existing, not a regression). Corrected to name the three call sites that do guard and state the one that doesn't. |
| 7 | LOW | **FIXED** | Both stale `draw_one_card` references (`card-types/src/state/replacement_effect.rs:401`, `rules/replacement.rs:634`) updated to name `effects::draw_cards_for_player` (with "renamed from `draw_one_card` in PB-DP5" for searchability). Left the two comments that reference the OLD name *historically and correctly* (`replacement.rs`: "the sequence loop, formerly `draw_one_card`" and "the pre-PB-DP5 `draw_one_card` had none either") untouched — they are accurate past-tense references, not present-tense claims about a function that exists. |
| 8 | LOW | **FIXED** | Appended a `PB-DP5 (2026-07-26): re-pinned on the 63→64 bump...` line to the `FROZEN_HISTORY_PREFIX_DIGEST` comment in `tests/core/hash_schema.rs`, alongside the existing PB-OS11 line, per the file's own "annotate with the bump that caused it" convention. Digest value itself was already correct (untouched). |
| 9 | LOW | **DECLINED — no code change; recorded here instead of the audit row it was destined for** | No code change required, as the review itself specifies. The `LostToEmptyLibrary`-single-attempt behavior change is outcome-identical and was already the intended shape of Phase 2 (the `break` was deliberate, not a bug). Checked whether the Phase 2 commit message (`b3e8e435`) records the `draw_card`'s `?` → `expect_*` error-surface change (plan risk 2): it does not explicitly call this out, though the sibling SR-25 note ("the three consolidated bodies already used `expect_*` forms") is adjacent. Not amending a landed commit message per git-hygiene protocol; both items belong in the DP-5 audit row, which Finding 2's parallel track owns. |
| 10 | LOW | **FIXED** | Added `test_dp5_precedence_draw_first_falls_through_to_draw_arm` (T10b) — submits the DRAW answer FIRST while a zone change is ALSO pending, so `handle_order_replacements`' arm 1 (zone change) must reject the ids on applicability and fall through to arm 2 (draw), which the original T10 (zone-change-first) never exercised. Asserts the draw resolves, the zone change stays untouched, and the zone change can still be answered afterward. |
| 11 | LOW | **DECLINED — no code change, per the review's own "Fix"** | Review explicitly says "no code change in PB-DP5. Add the timing-control deviation to the DP-5 audit row and fold the lifecycle question into OOS-DP5-2." That's Finding 2's parallel track. No action taken here. |
| 12 | LOW | **DECLINED — no code change, per the review's own "Fix"** | Review explicitly says "none in this PB. Note it in the plan's §2.2 selection-rule paragraph and seed it with OOS-DP5-1." That's Finding 2's parallel track / a plan-file annotation, not an engine change. No action taken here. |

### Finding 1 — detailed resolution

**Question posed by the coordinator**: does restoring an explicit loop in `resolve_pending_draw`
(around its calls to `perform_one_draw`) actually close the hole, or does the hole live inside
`check_would_draw_replacement`'s `AutoApply` dispatch, making that loop orthogonal?

**Answer: the hole lives inside `check_would_draw_replacement`, and a loop at the
`resolve_pending_draw` layer would be orthogonal — it would never fire.** Traced concretely:
when `determine_action` hits the CR 616.1a branch (exactly one self-replacement, 2+ applicable
overall), it returns `AutoApply(S)`. If `S`'s modification is not `SkipDraw`,
`check_would_draw_replacement`'s pre-fix `else` arm returned `DrawAction::Proceed` immediately —
which `perform_one_draw` maps straight to `DrawStepOutcome::Completed` (the draw happens), not
`Deferred`. A loop wrapped around `perform_one_draw` calls in `resolve_pending_draw` only ever
re-invokes on `Deferred`; `Completed` never reaches it. So looping at that outer layer would not
have re-examined anything — the remaining applicable replacement (including a `SkipDraw` that CR
616.1f says must now apply) is dropped before `resolve_pending_draw` is ever involved, and in
fact before any player choice is even offered (the self-replacement case never emits a
`NeedsChoice` at all when `self_ids.len() == 1`).

**Corroborating evidence from the sibling code path**: `check_zone_change_replacement` (the
`WouldChangeZone` analogue, `rules/replacement.rs:~984-1046`) already implements the CR 616.1f
re-check as a `loop { ... }` **around its own `determine_action` call**, not around its caller —
confirming that the correct location for this loop is inside the "determine and apply one
replacement" function, not one layer up.

**Fix applied**: `check_would_draw_replacement` now runs the identical `loop` shape internally.
On every `AutoApply(id)` whose modification is not `SkipDraw`, `id` is inserted into a local
`applied: HashSet<ReplacementId>` (seeded from the caller's `already_applied`) and the function
re-runs `find_applicable`/`determine_action` rather than returning `Proceed`. Termination: bounded
by `state.replacement_effects.len()` (`applied` strictly grows, `find_applicable` excludes its
members) — same proof shape the zone-change loop and the plan's §3.2 pseudocode both rely on.
This closes the hole completely, including for the genuinely-new-today divergence the review
constructed (`{S: self-replacement RedirectToZone(Exile), X: non-self SkipDraw}` — CR 616.1a
forces S, CR 616.1f then finds X applicable and stops the draw; pre-fix the engine auto-applied S
and proceeded to draw a card anyway).

Two doc comments that had asserted the now-corrected false equivalence
(`perform_one_draw`'s and `resolve_pending_draw`'s) were rewritten to state precisely what is and
isn't true: `perform_one_draw` and `resolve_pending_draw` still make a single call each to the
next layer down (no loop needed *at those layers*, for the same "future round-trip" reason
`resolve_pending_zone_change` doesn't loop either), but the CR 616.1f re-check they rely on now
genuinely happens *inside* `check_would_draw_replacement`, not by virtue of "AutoApply always
being terminal" (which was the false claim).

**New regression test**: `test_dp5_self_replacement_autoapply_still_rechecks_remainder` (T15) —
builds exactly the `{S, X}` scenario above and asserts no `CardDrawn` is ever emitted and the
card stays in the library. **Fail-before verified**: reverted `check_would_draw_replacement` to
its pre-fix single-dispatch form (a scratch patch, not a git checkout of the whole file — a git
checkout accidentally reverted the entire uncommitted fix cycle mid-session and had to be redone
from the plan/review text; the restore below is the byte-identical final state, confirmed via
`cargo test` passing 15/15 immediately after), ran the test, observed the predicted failure
(`CardDrawn { player: PlayerId(1), .. }` emitted, i.e. the card was wrongly drawn), then restored
the fixed code and reran to confirm the full 16-test suite in this file passes.

**No residual seed needed for Finding 1 itself** — the hole is closed by an actual code fix, not
a caveat. The disposition table above still routes the seeds Finding 1's own "Fix" text asked for
(OOS-DP5-6/8, the `WouldDraw`-widening PB) through the coordinator's Finding-2 track, since those
seeds were about the *pre-existing* `SkipDraw`-only limitation, which Finding 1's fix does not
touch or worsen.

### Gates (post-fix-cycle, all green)

- `cargo build --workspace`: clean.
- `cargo test --all`: **3,797 passing, 0 failing** (3,794 baseline + T14 + T15 + T10b).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.
- `tools/check-defs-fmt.sh`: clean, 1,804 defs.
- `cargo test -p mtg-engine --test scripts run_all_scripts`: 8/8, including the dredge golden
  script.
- `HASH_SCHEMA_VERSION` (`state/hash.rs:591`): **64**, unchanged from the implement phase (no new
  hashed field was added in this fix cycle).
- `PROTOCOL_VERSION` (`rules/protocol.rs:260`): **27**, unchanged.

### Files touched in this fix cycle

- `crates/engine/src/rules/replacement.rs` — Finding 1 fix (internal CR 616.1f loop in
  `check_would_draw_replacement`), Finding 5 (resume guard), Finding 6 (comment), Finding 7
  (comment, `:634`), Finding 1's two doc-comment corrections.
- `crates/engine/src/state/mod.rs` — Finding 3 (`pending_draws_mut()` deleted).
- `crates/card-types/src/state/replacement_effect.rs` — Finding 7 (comment, `:401`).
- `crates/engine/tests/core/hash_schema.rs` — Finding 8 (`FROZEN_HISTORY_PREFIX_DIGEST` comment).
- `crates/engine/tests/primitives/pb_dp5_pending_draw_choice.rs` — Finding 4 (T14), Finding 10
  (T10b), and a new regression test for Finding 1 (T15). 16 tests in this file, up from 13.
