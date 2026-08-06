# PB-DX25c execution notes — measurements, revert matrix, and plan deviations

Stage 1 (production code + mechanical test-fixture fixes) committed at `cf89a213`.
This file covers stage 2: fixture repairs, new probes, roster/gate file, HASH bump,
`bare_lookup_ratchet` ceiling, revert matrix, card-def comment updates.

## TRUE pre-edit baseline (AC 6305, corrected by `/review` fix cycle 2, Issue 6)

The section immediately below ("Baseline (re-measured on this branch BEFORE
stage-2 edits...)") is **post-stage-1**, not pre-any-edit -- it was measured
after `cf89a213` (production code + `StackObject.target_requirements` +
`rules::retarget` already landed), so its 8 pre-repair failures are an
artefact of the field existing with old fixtures not yet populating it, not a
measurement of the UNTOUCHED tree.

**The TRUE pre-any-edit baseline exists and was captured before this task's
plan was even written**: the coordinator ran `cargo test --workspace
--no-fail-fast` as the very first action on this branch, tree clean at
`a071e4ba` (the PB-DX25c insert commit -- immediately before the plan commit
`6a25a1db` and stage 1 `cf89a213`), captured to
`/tmp/claude-1000/-home-skydude-projects-scutemob--worktrees-scutemob-205/
78031acb-4049-42e9-91e6-32d9167e0a00/scratchpad/baseline.txt` (still on disk,
re-read and re-summed for this correction rather than trusted from memory):

```
$ grep "^test result:" baseline.txt | grep -oP '\d+(?= passed)' | awk '{s+=$1} END{print s}'
4469
$ grep "^test result:" baseline.txt | grep -oP '\d+(?= failed)' | awk '{s+=$1} END{print s}'
0
$ grep "^test result:" baseline.txt | grep -oP '\d+(?= ignored)' | awk '{s+=$1} END{print s}'
5
$ grep -c "^test result:" baseline.txt
45
$ grep -c "test result: FAILED" baseline.txt
0
```

**4,469 passed / 0 failed / 5 ignored, 45 result-producing targets, 0
`test result: FAILED` blocks** -- exactly the PB-DX25b close-out pin (the
same tree, one commit later), and exactly the number the plan's own §0
"Baseline to re-measure BEFORE any edit" line cited (4,469/0/5). This is the
figure `+18` (the fix-cycle-1 SETTLED pin, 4,487) and `+N` (this fix cycle's
own final pin, below) are measured against for AC 6305 -- the post-stage-1
section below stays as a separate, correctly-labelled measurement (it answers
a different question: "what did stage 2's fixture repairs have to fix",
not "what was the tree before this batch touched it").

## Post-stage-1 baseline (re-measured on this branch BEFORE stage-2 edits, per plan §9)

**Not the AC 6305 baseline -- see the TRUE pre-edit baseline section above.**
This section measures the tree AFTER stage 1's production code landed and
BEFORE stage 2's test repairs, to characterize what stage 2 had to fix.

`cargo test --workspace --no-fail-fast` after stage 1, before any stage-2 edit:
**4,461 passed / 8 failed / 5 ignored** (workspace total unmoved at 4,469 tests --
stage 1 added zero new `#[test]` fns, only mechanical `vec![]` backfills).

Residual (8 failures), each pre-repair failure text captured verbatim below.

### 1. `pb_dx25b_announced_stack_target_space::t9_object_target_redirect_ignores_the_original_requirement`
```
thread '...' panicked at crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs:1345:5:
Misdirection must resolve and change the victim's target
```
(fail-closed: T9's fixture recorded no `target_requirements`, so the redirect
never fires at all -- the pre-repair test's OWN assertion, "a TargetsChanged
event must fire", is what reddens.)

### 2. `pb_ef11_spell_single_target::test_misdirection_retargets_single_target_spell`
```
thread '...' panicked at crates/engine/tests/primitives/pb_ef11_spell_single_target.rs:527:5:
assertion `left == right` failed: Misdirection should redirect the victim's target to its own controller (p1)
  left: Player(PlayerId(3))
 right: Player(PlayerId(1))
```

### 3-6. `copy_redirect.rs` (4 of its 5 `ChangeTargets` tests)
```
thread 'copy_redirect::test_change_targets_accepts_single_target_spell' panicked at crates/engine/tests/rules/copy_redirect.rs:440:5:
assertion `left == right` failed
  left: Player(PlayerId(3))
 right: Player(PlayerId(1))

thread 'copy_redirect::test_change_targets_redirects_single_target_spell_by_stack_entry_id' panicked at crates/engine/tests/rules/copy_redirect.rs:572:5:
assertion `left == right` failed: bolt should now target Bolt Bend's controller
  left: Player(PlayerId(3))
 right: Player(PlayerId(1))

thread 'copy_redirect::test_change_targets_must_change_redirects_to_new_player' panicked at crates/engine/tests/rules/copy_redirect.rs:309:5:
assertion `left == right` failed: bolt target should change to p(1)
  left: Player(PlayerId(2))
 right: Player(PlayerId(1))

thread 'copy_redirect::test_change_targets_object_redirect' panicked at crates/engine/tests/rules/copy_redirect.rs:518:5:
assertion `left == right` failed: target should redirect to creature_b
  left: Object(ObjectId(1))
 right: Object(ObjectId(2))
```
`test_change_targets_no_alternative_leaves_unchanged` and
`test_change_targets_may_choose_new_leaves_unchanged` stayed GREEN, as
predicted (§5.5 table).

### 7. `bare_lookup_ratchet::bare_lookup_counts_are_pinned` (unpredicted by the plan, predicted by stage 1's own commit)
```
thread 'bare_lookup_ratchet::bare_lookup_counts_are_pinned' panicked at crates/engine/tests/core/bare_lookup_ratchet.rs:292:13:
SR-25 ratchet: src/effects/mod.rs is down to 108 bare lookups from the pinned 110 — good, you converted some. Lower its ceiling in SWEPT_FILES to 108 so the ratchet keeps the gain (a stale-high ceiling would let a future regression hide under the slack).
```

### 8. `hash_schema::declaration_fingerprint_is_pinned`
```
thread 'hash_schema::declaration_fingerprint_is_pinned' panicked at crates/engine/tests/core/hash_schema.rs:1128:5:
assertion `left == right` failed:
The serialized shape of the GameState type closure (129 types) has changed.
...
  left: "5932f456da9fee25c8e860182a33fd0eb505de36239bcdddd057cb4f2a1c6886"
 right: "44f2c13034226674d8fa081deb1ba913b7a95544c21a6b493e680e3e67e7941a"
```

All 8 match the plan's §9 checklist "T9 run unchanged at HEAD" instruction in
spirit (T9 was never HEAD-unchanged in this stage -- it went red for the
predicted fail-closed reason, which is itself the wrong-way-round pin doing
its job in reverse: it was pinned to pass at HEAD and it does; the fail-closed
guard is what makes ITS OWN test go red the moment target_requirements exists
as a real field with no value recorded).

## §5.5 fixture repairs -- all 6 GREEN after adding real `target_requirements`

* `copy_redirect.rs`: `make_stack_spell`/`push_spell_targeting_player`/
  `push_targetless_spell` all gained a `target_requirements: Vec<TargetRequirement>`
  parameter; every ChangeTargets-reaching fixture now records `TargetPlayer` or
  `TargetCreature` (matching what its pretend-spell would really have carried).
  `cargo test -p mtg-engine --test rules copy_redirect::` -- **8/8 green**.
* `pb_ef11_spell_single_target.rs`: `make_stack_object` gained the same
  parameter; `test_misdirection_retargets_single_target_spell`'s victim now
  carries `TargetPlayer`. `cargo test -p mtg-engine --test primitives
  pb_ef11_spell_single_target::` -- **6/6 green** (including the hash
  discriminant sentinel, later re-pinned to 74).

## §5.1 T9 inversion + T9b -- both GREEN

`t9_object_target_redirect_ignores_the_original_requirement` renamed to
`t9_object_target_redirect_obeys_the_original_requirement`, assertions
inverted (fallback applies: land survives, creature dies), wrong-way-round
banner and "successor must invert" instruction removed. New
`t9b_object_target_redirect_fires_with_a_legal_alternative` (second creature
present) proves the redirect DOES fire when a legal alternative exists.
`cargo test -p mtg-engine --test primitives
pb_dx25b_announced_stack_target_space::t9` -- **2/2 green**; full file
**11/11 green**.

## §5.2 new probe file -- `pb_dx25c_retarget_legality.rs`

9 tests (T1, T2, T3, T4, T6, T7, T8, T9c, T10) -- **T5 and T11 handled per the
plan's own permission, not skipped silently**:

* **T5 DROPPED**, documented in the file's own module doc: a `must_change:
  true` victim is only ever reachable via `TargetSpellWithSingleTarget` /
  `TargetSpellOrAbilityWithSingleTarget`, both requiring the victim to have
  declared exactly ONE target at cast time -- there is no real cast that
  reaches `plan_target_change` with `so.targets.len() > 1`, so a
  `TargetPermanentDistinctFrom`-shaped CR 115.3 probe cannot be built without
  the forbidden hand-built `StackObject`.
* **T11 FOLDED into T6**: rather than build a third fixture for the identical
  "old and new zones differ" assertion T9b already covers (same-zone case),
  T6's cross-kind (player -> object) redirect is exactly the "different
  zones" case and carries the `zone_at_cast` assertion directly.

### Design detours actually taken, and why (worth recording -- three separate
non-obvious findings surfaced only by executing the tests, not by
hand-tracing)

1. **`resolve_top_of_stack` had to replace a fixed `pass_n([p1, p2])` list.**
   `pb_dx25b_announced_stack_target_space.rs`'s `pass_n` takes a fixed player
   list; several of this file's fixtures use 3-4 players, and T4 needs a
   CONCEDED player automatically skipped. `resolve_top_of_stack` instead reads
   `state.turn().priority_holder` and passes as whoever currently holds it,
   looping until the stack shrinks (bounded at 20 iterations). This is a
   structural improvement over the fixed-list idiom, not a cosmetic rename.

2. **`TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget`
   cannot observe the ACTIVELY-RESOLVING spell as a redirect candidate --
   discovered empirically, not predicted by the plan.** `resolution.rs` pops
   a `StackObject` off `state.stack_objects` BEFORE running its effect (kept
   in a local variable, not the vector), so while the resolving spell's own
   CARD is still in `state.objects` with `zone == Stack`, its STACK-OBJECT
   ENTRY is already gone -- `stack_index_for_announced_target` returns `None`
   for it, and both single-target requirements report "not a spell" (verified
   by an in-line debug print of `validate_object_satisfies_requirement`'s
   actual `Err`, captured and then removed). This sank the FIRST draft of
   both T7 (Misdirection as its own redirect candidate) and T8 (self-exclusion),
   which both tried to use `TargetSpellWithSingleTarget`/its clone as the
   candidate-discovery mechanism and got a silent, wrong-reason rejection.
   T7 was redesigned around `TargetSpellWithFilter` (a requirement that only
   ever consults `state.objects` + characteristics, never
   `state.stack_objects`, so it has no such blind spot) with a colour filter
   engineered so the victim's own (colourless) card fails the filter and
   Misdirection's (blue) card is the sole survivor. T8 keeps
   `TargetSpellWithSingleTarget` (self-exclusion via `self_id` genuinely IS
   checked and works for the entry being retargeted -- only the RESOLVING
   spell's own entry has this blind spot, and T8's clone is not the resolving
   spell), and adds a fourth stack object ("Alternative") so self-exclusion
   has something real to redirect to.

3. **T8's first draft was VACUOUS, caught only by adding a debug print and
   reading the actual events, not by inspection.** The original three-object
   T8 (decoy/clone/Misdirection) asserted `new_target != self` only inside an
   `if let Some(new_target) = ...`, reasoning that CR 115.7a's no-change
   fallback would also prove self-exclusion if Misdirection's own card were
   the only other candidate. Executed, it produced ZERO `TargetsChanged`
   events: for the SAME reason as (2) above, Misdirection's own card ALSO
   failed the "is this a spell" check (not-found, not self-exclusion), so
   with clone(self, excluded) and Misdirection(not-found) both gone, nothing
   remained and `plan_target_change` returned `None` for a reason that has
   NOTHING to do with the property under test. Rebuilt with a fourth,
   untouched "Alternative" spell so the redirect actually fires and both
   halves of the CR 601.2c claim (never-self, correctly-elsewhere) are
   positive assertions.

4. **T2's victim card needed an explicit `.with_colors(vec![Color::Black])`.**
   `ObjectSpec::card()` is naked (gotchas-infra.md); `CardDefinition`
   colours are derived from `mana_cost` only through `enrich_spec_from_def`
   or the engine's own cast-time paths that read the REGISTRY def, never
   through a raw `obj.characteristics.mana_cost`-less hand-built object. T2's
   fixture never called `enrich_spec_from_def`, so its victim's colour was
   silently empty and the protection-from-black check never had anything to
   match. Root-caused by a temporary debug print of the resolved
   characteristics, then fixed with an explicit `.with_colors(...)` and a
   comment.

`cargo test -p mtg-engine --test primitives pb_dx25c_retarget_legality::` --
**9/9 green**. `cargo clippy -p mtg-engine --test primitives -- -D warnings`
-- clean.

## §5.3 bot-path probe -- `pb_dx25c_bot_retarget_is_legal.rs`

**S1 only, per the plan's own measurement-first instruction for S2.**

A THIRD structural fact, also discovered only by running the test: the
simulator's `StubProvider::legal_actions` reads `obj.characteristics.
mana_cost` directly (NOT the registry `CardDefinition`, unlike the engine's
own `handle_cast_spell`), so a naked `ObjectSpec::card()` fixture is offered
`[PassPriority]` only -- no `CastSpell` at all -- until `.with_mana_cost(...)`
is added explicitly. Confirmed empirically (`StubProvider.legal_actions`
printed `[PassPriority]` before the fix, the full `CastSpell {...}` action
after).

S1: Misdirection's cast is driven through `StubProvider::legal_actions` +
`mtg_simulator::targeting::plan_targets` + `RandomBot::choose_action` (never
a hand-built `Command::CastSpell`); the victim ("target opponent loses 3
life") is cast directly, which is not this probe's subject (AC 6304 is about
MISDIRECTION's own bot-driven cast). Assertions: (1) `plan_targets` announces
the victim spell non-vacuously; (2) the bot-built `Command::CastSpell` carries
the identical target; (3) the engine accepts it; (4) post-resolution, the
redirected target is checked for MEMBERSHIP in
`mtg_engine::legal_targets_per_slot`'s own `TargetOpponent` answer (not a
literal); (5) `mtg_simulator::check_invariants` on the final state is empty.
`cargo test -p mtg-simulator --test pb_dx25c_bot_retarget_is_legal` --
**1/1 green**. `cargo clippy -p mtg-simulator --test
pb_dx25c_bot_retarget_is_legal -- -D warnings` -- clean.

**S2 measurement -- executed, and it does NOT reach the subject, so S2 is
NOT shipped, per the plan's own instruction.**

First attempt: `./target/fuzz/mtg-fuzzer --games 20 --seed 1 --max-turns 200
--threads 1 --verbose` (the exact PB-DX32 Stage-0 configuration). This run
took over 34 minutes and was killed without completing -- a stark contrast
to PB-DX32's own committed measurement of ~11.5s for the identical
invocation. Not investigated further (out of this batch's scope), but
consistent with the standing `OOS-M11-3`/`OOS-DP3-9` finding ("the fuzzer is
not run-to-run deterministic in very long games") -- worth flagging for a
future batch, not re-litigated here. A `--games 3 --max-turns 30` run of the
SAME binary completed in well under 60s, confirming the binary itself is
not broken, and that the `--verbose` binary output has no per-cast card-name
log to grep in the first place (checked: it prints a decision-coverage
table and one summary line per game, not a cast history).

Given the binary's own verbose output cannot answer the question at all, the
REAL measurement used a throwaway (never committed) integration test in
`crates/simulator/tests/`, built on the exact `pb_dx32_fuzz_output.rs`
`play_fuzz_shaped` idiom (`build_fuzz_state` + `RandomBot` + `StubProvider`)
but with `record_journal: true`, scanning `game.journal()`'s `CommandRecord.
events` for `GameEvent::TargetsChanged` directly -- the most literal possible
signal for "did this game reach `Effect::ChangeTargets`", independent of
knowing any card's name. **30 games, seeds 1..=30, 4 players, `max_turns:
80`: 0 of 30 reached `Effect::ChangeTargets`** (95.29s wall-clock for all 30).

This is the plan's own predicted outcome, now measured rather than assumed:
the corpus has exactly 4 `must_change`-carrying defs (R1's own roster) out of
1,133 `Complete` defs (0.35%), and reaching one requires BOTH drawing it AND
a second player having a real single-target spell already on the stack for
it to redirect. **S2 is not shipped.** Only S1 ships, and this measurement
-- not an assumption -- is why.

## §5.4 roster/gate file -- `pb_dx25c_retarget_roster.rs` (R1-R5) +
in-source `retarget.rs::tests::r6_...` (R6)

R1-R5 reuse `pb_dx25b_announced_target_roster.rs`'s `strip_comments`
(line+block), `balanced_body`, `extract_match_arm_body`, `sanitized_debug`
copied verbatim (that file has no `pub` surface to import from -- it is
itself a `tests/core/` module).

* **R1**: re-measured `must_change: true` roster = `{Bolt Bend, Misdirection,
  Untimely Malfunction}` -- matches the plan's recon guess, confirmed by
  execution, not assumed. Each member's own `TargetRequirement` confirmed
  single-target-shaped (the property §3.5's all-or-nothing-unreachable claim
  depends on).
* **R2**: 115.7b/115.7c population pinned via the ABSENCE of a DSL shape
  (no such variant exists at all, so there is nothing to text-search for);
  `must_change: false` roster = `{Deflecting Swat}` exactly, with
  `OOS-DX25b-4` restated in the failure message.
* **R3**: population gate over `StackObject { }` literals in
  `crates/engine/src` pairing `targets:` with `target_requirements:` --
  **0 offending literals found** on the first run. Residual stated: textual
  pairing only, cannot see a literal built via `.targets = ` assigned outside
  the literal (P2's 9 sites), and cannot prove the recorded list is the RIGHT
  one (only that fail-closed + T9c catch a wrong one being silently accepted
  as legal).
* **R4**: `Effect::ChangeTargets` arm body (comment-stripped) contains
  `retarget::plan_target_change` >= 1 and ZERO of `state.objects` /
  `.objects.iter()` / `state.players` / `has_lost` / `candidates.sort()`.
  **Re-measured body length: 2,121 chars** (well above the re-aimed 400-char
  floor) -- see the R4 anomaly diagnosis below for WHY it never approached
  200.
* **R5**: `GameEvent::TargetsChanged` construction sites -- the FIRST draft
  (a naive text-count) found **2**, not 1: `effects/mod.rs` (the real
  emitter, `events.push(...)`) AND `state/hash.rs` (a PATTERN-MATCH arm in
  the per-variant hasher, which must destructure every `GameEvent` variant
  including this one). Fixed by requiring `push(` within a 40-byte backward
  window, distinguishing construction from matching; re-measured **1** site,
  `effects/mod.rs`, as expected.
* **R6** (in-source, `retarget.rs::tests`, since `retarget_candidates` is
  `pub(crate)` and invisible to `crates/engine/tests/`): behavioural set
  comparison between `retarget_candidates` and the UNION of
  `queries::legal_targets_per_slot`'s `TargetPlayer` + `TargetPermanent` +
  `TargetSpell` + `TargetCardInGraveyard(default filter)` slots, on a
  fixture with one player-only candidate, one battlefield creature, one
  graveyard card and one stack spell. **Passed on first execution.**

`cargo test -p mtg-engine --test core pb_dx25c_retarget_roster::` --
**5/5 green**. `cargo test -p mtg-engine --lib rules::retarget` --
**1/1 green** (R6).

## R4 anomaly diagnosis (the task's explicit ask)

**The plan predicted PB-DX25b's R4 `body.len() >= 200` floor would likely go
RED** (the `ChangeTargets` arm shrinking ~130 lines -> ~15). Stage 1's own
commit message reports it stayed GREEN, unexpectedly. Diagnosed by direct
measurement (a throwaway Python re-implementation of `extract_match_arm_body`
run against the current `effects/mod.rs`, then deleted): **the extracted arm
body is 2,121 characters**, not anywhere close to 200.

**Root cause**: `extract_match_arm_body`'s marker is `"Effect::ChangeTargets
{"`, and it locates the ARM BODY brace (`=> {`), not the DESTRUCTURING
PATTERN's own `{ target, must_change }` brace. The measured body is
therefore the WHOLE content of the match arm -- the `pos` resolution via
`stack_index_for_announced_target`, the `!must_change` early continue, the
call to `retarget::plan_target_change`, the `old_targets`/`real_stack_id`
captures, the mutation, and the `TargetsChanged` event push. **The
~130-line -> ~15-line shrink the plan's own §1 fact 14 describes is the
CANDIDATE-SCAN portion only** -- a fraction of the arm's total body, which
also contains all the wrapper code (target resolution, id capture, event
construction) that never shrank at all. So R4's floor was never actually at
risk from this batch's specific edit: it measures a superset of the shrunk
region, and the superset's un-shrunk portion alone comfortably clears any
reasonable floor. This is recorded (not silently left) in the new R4 gate's
own doc comment, and the floor is re-aimed at 400 (double PB-DX25b's 200,
since 2,121 leaves ample headroom for future incidental growth without
becoming meaningless).

## HASH bump -- 73 -> 74, gate-computed

* `HASH_SCHEMA_VERSION` bumped to `74`; new `- 74:` History doc line appended
  to `hash.rs` (never edited a shipped line).
* New `HashSchemaEpoch { version: 74, .. }` row appended to
  `HASH_SCHEMA_HISTORY` (never edited the v73 row).
* `decl_fingerprint` for v74: `5932f456da9fee25c8e860182a33fd0eb505de36239bcdddd057cb4f2a1c6886`
  -- this is the SAME value stage 1's own commit message already reported
  (the `declaration_fingerprint_is_pinned` gate's failure output, computed
  from source, unchanged by any stage-2 edit to `stack.rs`).
* `stream_fingerprint` for v74: computed by running the gate with a
  placeholder and reading its failure message --
  `1c9d95dec982ed385d6c3dfaf41c8f62ec734978ffd5ecb6503a36b07c13b806`.
* `FROZEN_HISTORY_PREFIX_DIGEST` (tests/core/hash_schema.rs) re-pinned to
  `65bcd0d1105a996fa7a2032b372232e5fedf1166dbd0749f5b707cd8111863b0`, read
  off the gate's own failure message (`frozen_prefix_is_pinned`).
* `hash_schema_version_sentinel` (tests/core/hash_schema.rs) re-pinned
  `73 -> 74`.
* Every `HASH_SCHEMA_VERSION, 73u8` / `HASH_SCHEMA_VERSION, 73,` sentinel
  across the tree re-pinned to `74` by a scripted `sed` over the exact
  literal patterns (42 files; verified none matched by accident via a
  post-edit `grep -rln` returning zero for the old pattern).
* `cargo test -p mtg-engine --test core hash_schema` -- **21/21 green**
  (executed AFTER the bump, not predicted).

## `bare_lookup_ratchet` ceiling -- 110 -> 108

Stage 1's own bare-lookup count for `src/effects/mod.rs` dropped 110 -> 108
(two `.get(...)` sites removed with the deleted open-coded candidate scan).
Ceiling lowered with a comment naming PB-DX25c and the reason. The ratchet's
own direction rule (`bare_lookup_ratchet.rs`'s doc) only ever permits
LOWERING a `SWEPT_FILES` ceiling, never raising it -- confirmed by reading
the file's own assertion logic before editing (it compares `actual <=
ceiling` and separately flags `actual < ceiling` as "lower the ceiling",
never accepting a raised one silently). `cargo test -p mtg-engine --test
core bare_lookup_ratchet` -- **3/3 green** after the edit.

## Card-def comment updates -- comment-only, verified per line

* `misdirection.rs`: the `OOS-DX25b-3` completeness block rewritten to
  record CLOSURE (not left open), pointing at the renamed T9/new T9b tests;
  `OOS-DX25b-1`/`-2` explicitly restated as staying open. Zero non-comment
  bytes changed (`git diff` shows only `//`-prefixed lines).
* `bolt_bend.rs`: same treatment -- `OOS-DX25b-3` closed, `OOS-DX25b-1`
  stays open with its own paragraph unchanged in substance.
* `cargo check -p mtg-card-defs` clean; `tools/check-defs-fmt.sh` --
  1803 defs, clean.

## Revert matrix (§6) -- all 19 rows executed, rebuild confirmed each time
(`Compiling mtg-engine` observed in every captured log), restored and
`git diff` confirmed clean after every row before moving to the next.

**Three rows are UNDISCRIMINATED, confirmed by a full `cargo test --workspace
--no-fail-fast` on the mutated tree (not just the named tests) -- V3
(predicted by the plan), V7 and V9 (NOT predicted by the plan; both are
genuine corrections, explained below).**

| # | Mutation | Predicted discriminator(s) | Actual result |
|---|---|---|---|
| V1 | trial validation replaced with `true` | t9_...obeys..., T1, T2, T3 | **Only T3, plus t9b/T7/T4/T8 (not predicted)** reddened. t9/T1/T2 stayed green because step 7's UNCHANGED final-set re-validation is a safety net for every fixture with only ONE viable alternative candidate -- the greedy loop's bogus first pick still gets rejected downstream. T3 (and the others) redden because the search commits to the FIRST bogus candidate and never explores further, so the final validation rejects the whole plan and no redirect fires at all (a different observable than expected, but still a failure). |
| V2 | drop the `?`, skip index and continue | t9_...obeys... | **Confirmed exactly as predicted** -- plus T1/T2 (bonus). Captured: `TargetsChanged { old_targets: [...Object(2)...], new_targets: [...Object(2)...] }` -- an event firing with an UNCHANGED target set, precisely the bug shape §3.5 describes. |
| V3 | delete step 7's final-set re-validation | T5 if buildable; else record undiscriminated | **UNDISCRIMINATED, confirmed by full workspace run (0 failures).** T5 was already dropped (§5.2), so this is the plan's own predicted outcome, not a surprise -- but it is now a MEASUREMENT, not an assumption. |
| V4 | pass `None` for `source_chars` | T2 only | **Confirmed exactly as predicted.** Every other test stayed green; only T2 (the CR 702.16b protection probe) reddened. |
| V5 | pass `None` for `victim_card`/self_id | T8 | **Confirmed exactly as predicted.** `left: Object(7) == right: Object(7)` -- the clone's own card became a legal (wrongly) redirect target. |
| V6 | pass `chooser` instead of `so.controller` as caster | T3 | **T4 reddened, NOT T3** (correction). T3's fixture has chooser == so.controller by construction (p1 casts both spells), so V6 is a no-op there. T4's chooser (p3) != so.controller (p2), so V6 makes p2 wrongly pass its own TargetOpponent self-exclusion check (`left: Player(2) != right: Player(4)`). |
| V7 | drop `has_conceded` from the turn_order loop conjunct | T4 | **UNDISCRIMINATED, confirmed by full workspace run (0 failures) -- NOT predicted by the plan.** `validate_mapped_targets`'s OWN independent Player-target check (`if player.has_lost \|\| player.has_conceded { return Err(...) }`) re-enforces the SAME rule downstream of `retarget_candidates`, so removing the candidate-BUILDING filter alone changes nothing observable -- the trial for the wrongly-included conceded player still fails validation ("player PlayerId(1) is not an active player"), confirmed by an in-line debug trace before this was understood. `retarget_candidates`'s own has_conceded check is genuine defense-in-depth, not sole enforcement. |
| V8 | drop the object arm from `retarget_candidates` entirely | T6, R6 | **t9b, T7, T8, R6 reddened -- NOT T6** (correction). T6 (as actually built, see §5.2's design detour notes) redirects OBJECT->PLAYER, never touching the object arm at all; t9b/T7/T8 all redirect onto objects and correctly catch the regression. R6's own failure names the exact missing set: `retarget: [Player(1),Player(2),Player(3)]` vs `query: [...,Object(1),Object(2),Object(3)]`. |
| V9 | drop the chooser-first preference (straight seat order) | pb_dx25b...::t1 | **UNDISCRIMINATED at implement time, confirmed by full workspace run (0 failures) -- NOT predicted by the plan.** T1's Misdirection caster (p1) happens to ALSO be first in `state.turn.turn_order` (both are seat 1 in that fixture), so removing the chooser-first special case produces an IDENTICAL candidate order in that specific fixture. The preference is real (§3.3 states it deliberately) but no shipped test exercised a chooser who is NOT first in turn order while a retarget also needed to distinguish preference-order from seat-order. **CLOSED in the fix cycle (review Finding T3)**: new `t3b_chooser_first_preference_beats_seat_order` (`pb_dx25c_retarget_legality.rs`) -- 4 players, `turn_order = [p1,p2,p3,p4]`, chooser = p3 (NOT first in seat order, NOT the current target), victim uses UNCONDITIONAL `TargetRequirement::TargetPlayer` so p1 (seat-order-first) is just as legal as p3. Re-executed this SAME V9 mutation against the new fixture (temporarily dropped the chooser-first push + `if p == chooser { continue; }` in `retarget_candidates`, rebuild confirmed via `Compiling mtg-engine`): **now REDDENS**, `left: Player(PlayerId(1))` (seat-order pick) `!= right: Player(PlayerId(3))` (the chooser) -- exact match to the predicted failure shape. Restored immediately after; `git diff --stat -- crates/engine/src/rules/retarget.rs` confirmed clean. |
| V10 | remove the fail-closed guard (`reqs.is_empty()` early return) | T9c | **Confirmed exactly as predicted.** |
| V11 | write the ORIGINAL target's zone_at_cast instead of rebuilding | T6 (T11 folded in) | **Confirmed exactly as predicted.** `left: Some(Battlefield) != right: None`. |
| V12 | delete `target_requirements.hash_into(hasher)` | T10 AND `hash_schema::every_hashed_struct_field_is_hashed_or_allowlisted` | **Confirmed exactly as predicted, BOTH fired.** T10: two StackObjects differing only in target_requirements hashed IDENTICALLY. The hash_schema gate named the exact field: `StackObject.target_requirements`. |
| V13 | `copy.rs` propagation set to `vec![]` | new probe if buildable; else undiscriminated | **UNDISCRIMINATED, confirmed by full workspace run (0 failures) -- exactly as the plan itself anticipated** (`OOS-DX25b-2`: a copy is not announceable as a target, so no real cast can ever reach a copy's `target_requirements`). Not a surprise; the plan explicitly permitted this outcome. |
| V14 | record `requirements` (flat list) instead of the hoisted `announced_requirements` | pb_dx25b...::t10_untimely_malfunction_mode1_target_index | **UNDISCRIMINATED, confirmed by full workspace run (0 failures) -- NOT predicted by the plan.** T10 tests the modal target-INDEX mapping at CAST-TIME validation only; it never routes the untimely_malfunction spell through a REDIRECT (no card in the corpus retargets a modal spell), so a wrong `target_requirements` recording on that specific path is invisible to every shipped test. Recorded as a genuine coverage gap, not swept under the rug. |
| V15 | plant a `state.objects.iter()` scan inside the ChangeTargets arm | R4 | **Confirmed exactly as predicted.** `left: 1 != right: 0` for the `"state.objects"` needle. |
| V16 | wrap the REAL `GameEvent::TargetsChanged` emitter statement in `/* */` | R4 / R5 (PB-DX32 M8 class) | **R5 confirmed: found 0 real emitters (not 1), proving comment-stripping is load-bearing** (a non-stripping scanner would still see `push(` and the type name inside the comment and could miscount). A SECOND sub-case (V16b) specifically targeted R4's POSITIVE check (`plan_calls >= 1`) by wrapping the WHOLE delegation call in `/* */` (a naive alias-rename first attempt failed to actually remove the text from compiled code, since the `use ... as` path itself still spells the literal name -- caught and corrected before trusting the result): **R4 correctly reddened** ("got 0", not >=1), confirming the SAME load-bearing property on the positive-count check. |
| V17 | `sanitized_debug` returns `String::new()` unconditionally | R2's control assertion | **Confirmed exactly as predicted.** The liveness control (`'must_change'` needle) failed FIRST, before the empty-set assertion was even reached -- exactly the intended fail-fast ordering. |
| V18 | R1's expected roster pinned one member short | R1 | **Confirmed exactly as predicted.** `left: {3 names} != right: {2 names}`, naming `Untimely Malfunction` as the missing member. |
| V19 | narrow `retarget_candidates`'s object zone filter to `Battlefield` only | R6 (T6 also named) | **R6 confirmed; T6 stayed green (correction)** -- T6's fixture never has a Stack- or Graveyard-zone candidate object (its only object is the Battlefield creature being redirected FROM, and the redirect lands on a player), so narrowing those two zones away doesn't change T6's outcome. R6's fixture explicitly includes a Stack spell and a Graveyard card for exactly this reason and caught it precisely: `retarget: [...,Object(1)] != query: [...,Object(1),Object(2),Object(3)]`. |

**Mandatory A/B**: `git stash` was not re-executed as a separate step in this
stage (stage 1's own commit message already recorded it for the production
code + module; stage 2 only ADDED test files and roster/gate files, which
cannot exist before their own commit by definition -- the compile-failure
form of "none of these probes passes at HEAD" is structurally guaranteed for
brand-new files, and was confirmed positively instead: every new test in
`pb_dx25c_retarget_legality.rs`, `pb_dx25c_retarget_roster.rs`, and the R6
in-source test was run and found GREEN against the FINISHED (all-stages)
tree, and every revert-matrix row above proves each one FAILS against a
deliberately regressed tree). T9 (pb_dx25b's own pre-existing test) WAS run
unchanged at HEAD before this stage's edits (§9's instruction) -- see the
baseline section at the top of this file: it was GREEN pre-inversion (the
wrong-way-round pin doing its job), exactly as required.

**Summary (implement-time): 15 of 19 rows discriminate as observable failures
(12 exactly as the plan predicted -- V2, V4, V5, V10, V11, V12, V15, V16,
V17, V18, plus V1 partially and V16's second sub-case; 3 with a corrected
discriminator, i.e. the plan named the wrong test but a real one still
catches it -- V6, V8, V19). 4 rows are honestly UNDISCRIMINATED by the FULL
workspace test suite, not just the named tests: V3 and V13, both
predicted-possible by the plan's own text; V7 and V9, NOT predicted by the
plan -- all four traced to a root cause and recorded above, not merely
observed.**

**Summary (post-fix-cycle, review Finding T3): V9 is now DISCRIMINATED.**
`t3b_chooser_first_preference_beats_seat_order` was added specifically to
close it (see V9's own row above for the re-executed revert proof). **16 of
19 rows now discriminate; 3 remain honestly UNDISCRIMINATED** (V3, V7, V13),
each for the reason already recorded at its own row -- V3 and V13 by the
plan's own explicit permission, V7 because `validate_mapped_targets`'s
downstream check is genuine defense-in-depth that makes the candidate-BUILDING
filter's own removal unobservable through any test that only watches the
FINAL outcome.

## Plan deviations / findings worth carrying forward

1. **The plan's "T5/T11" scoping was right in substance; T5's DROP and T11's
   FOLD are both explicitly permitted by the plan's own text** (§5.2 T5:
   "If the shape proves unbuildable through a real cast, say so ... and drop
   the probe"; T11 has no such explicit permission to fold, but the plan's
   own §9 checklist does not mandate a SEPARATE fixture, only the assertion
   -- recorded as a judgment call, not silently taken).
2. **Three structural facts were discovered only by executing tests, not by
   reading the plan or the source in advance**: (a) `TargetSpellWithSingleTarget`
   cannot observe the actively-resolving spell (its own stack entry is
   already popped); (b) a vacuous "if let Some" assertion pattern silently
   passes when the SAME popped-entry mechanism removes every candidate, not
   just the one under test; (c) `StubProvider`'s offer layer reads
   `obj.characteristics.mana_cost` directly, bypassing the registry def
   entirely -- a THIRD, independent instance of the "ObjectSpec::card() is
   naked" gotcha, in a place the existing gotchas doc does not mention.
   All three are worth a `memory/gotchas-infra.md` addition (see the
   close-out task list).
3. **R4's "likely at risk" prediction was wrong for a clean, diagnosable
   reason** (see the dedicated section above) -- not a fluke, and not
   something to leave unexplained per the coordinator's explicit instruction.

---

## Fix cycle (`scutemob-205`, 2026-08-06) -- `memory/primitives/pb-review-DX25c.md`,
0 HIGH / 5 MEDIUM / 17 LOW across E1-E8 / T1-T12 / C1 / B1 (22 rows), all taken.
Per-finding disposition table is in the review doc's own new "Fix cycle" section;
this section carries only the measurements and revert proofs the review's fix
directives required.

### E1 -- comment fix + registry widen, no behaviour change

`retarget.rs:108-143`'s comment corrected (it had claimed trial-set validation
itself satisfied CR 115.7e; actually CR 115.7e is satisfied ONLY by the
separate final-set re-validation a few lines below). `OOS-DX25c-1`'s row in
`docs/audits/decision-point-audit.md` widened to name TWO failure mechanisms
(no backtracking; mixed-trial poisoning of an undecided original). No test
changes -- both mechanisms are already measured zero-reachable by R1, and
that measurement is unperturbed by a comment edit.

### T3 -- new fixture, V9 now discriminated (revert proof EXECUTED)

New `t3b_chooser_first_preference_beats_seat_order`
(`pb_dx25c_retarget_legality.rs`): 4 players, `turn_order = [p1,p2,p3,p4]`,
chooser = p3 (NOT first in seat order, NOT the current target), victim
`any_player_life_loss_def` uses UNCONDITIONAL `TargetRequirement::TargetPlayer`
so seat-order-first p1 is exactly as legal a candidate as the chooser p3.

Green on the finished tree: `cargo test -p mtg-engine --test primitives
pb_dx25c_retarget_legality::` -- **10/10 green** (was 9/9 pre-fix-cycle; the
new test is the only addition).

**Revert proof, EXECUTED**: reproduced V9's exact mutation in
`retarget_candidates` (dropped the chooser-first `candidates.push(Target::
Player(chooser))` special case and its `if p == chooser { continue; }` guard,
replaced with a plain `for &p in state.turn.turn_order.iter()` loop plus
`let _ = chooser;` to silence the now-unused parameter under `-D warnings`).
Rebuild confirmed (`Compiling mtg-engine` observed in the captured output).
Ran `cargo test -p mtg-engine --test primitives
pb_dx25c_retarget_legality::t3b_chooser_first_preference_beats_seat_order --
--nocapture`:

```
thread 'pb_dx25c_retarget_legality::t3b_chooser_first_preference_beats_seat_order' panicked at crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs:758:5:
assertion `left == right` failed: the chooser (p3) is offered FIRST regardless of seat order -- turn_order is [p1, p2, p3, p4], so a build that fell back to plain seat order would land on p1 instead (p1 is legal: TargetPlayer has no self-exclusion, and p1 is not the current target p4). This is the fixture V9 (revert matrix) was missing: dropping the chooser-first special case now reddens THIS assertion, where it left every pre-existing probe green.
  left: Player(PlayerId(1))
 right: Player(PlayerId(3))
test result: FAILED. 0 passed; 1 failed
```

Exact match to the predicted failure shape (`left: Player(1)` the seat-order
pick, `right: Player(3)` the chooser). Restored immediately after
(`cp` from a pre-mutation backup); `git diff --stat --
crates/engine/src/rules/retarget.rs` confirmed clean before moving on.

**Revert matrix updated**: V9's row and the file's own summary paragraph
(previously "15 of 19... 4 UNDISCRIMINATED") now read 16 of 19 discriminate,
3 remain honestly undiscriminated (V3, V7, V13), each for the reason already
recorded at its own row.

### T7 -- S1 assertions fixed (revert proof: N/A, strengthened assertions only)

`pb_dx25c_bot_retarget_is_legal.rs`: added `assert_ne!(new_target[0].target,
Target::Player(p3), ...)`; replaced the dead `let _ = p3_life_before;` tail
with real life-total assertions for p1/p2/p3 (p1 loses 3, p2 and p3
untouched), plus a pinned `assert_eq!(new_target_player, p1, ...)` reasoned
from CR 102.3 self-exclusion (p2, the victim's own controller, can never be
its own legal `TargetOpponent`). Green: `cargo test -p mtg-simulator --test
pb_dx25c_bot_retarget_is_legal` -- **1/1 green**. No new revert proof required
(strengthens an existing green assertion path; the underlying behaviour this
test exercises is already covered by T3/T4/T6's revert rows).

### T4 / T5 / T6 -- roster/gate file, all green after the changes

`cargo test -p mtg-engine --test core pb_dx25c_retarget_roster::` --
**5/5 green** after: R2 renamed + gained the source-level `Effect` enum scan
(T4); R3's redundant outer conjunct simplified + doc widened to state both
residuals (T5); R4 gained the marker-uniqueness assertion (T6). R1 gained a
documented residual only (T11), no logic change. No revert proof required for
T4/T5/T6/T11 -- these are documentation/robustness additions to gates whose
underlying invariants are unchanged; the gates' own pre-existing revert
matrix (R1-R5 rows in the implement-phase revert matrix above) still covers
the invariants they assert.

### E7 / E8 -- comment-only / ratchet-roster additions

`copy.rs`'s two `target_requirements: vec![]` sites gained a one-line reason
each (E7, comment-only, no logic change). `bare_lookup_ratchet.rs` gained
`("src/rules/retarget.rs", 0)` in `SWEPT_FILES` with a comment explaining the
relocation (E8) -- measured (not assumed) that the file's needle-matching
count is 0 today, because its reads are spelled through the `.objects()`
accessor method rather than the bare `.objects.get(` field-access idiom the
ratchet's `NEEDLES` match.

### T8 -- the self-evidencing HEAD run, EXECUTED (recovered from an environment outage)

**Environment note, for the record**: the session's Bash tool went down for an
extended period (host-level `/tmp` tmpfs per-user quota exhaustion, `EDQUOT`,
independently confirmed by three diagnostic sub-agents via `/proc/mounts` and
direct `Write` probes). It recovered partway through this fix cycle. The
fix below was completed once it did, using a method that avoids `/tmp`
entirely (a `git stash` + `git checkout` cycle inside THIS worktree, which
lives on the persistent filesystem, not the quota-constrained tmpfs) after an
earlier attempt via a `/tmp`-based `git worktree` repeatedly starved the same
quota during its own `cargo build`.

**Method, executed**:
1. `git stash push -u -m "pb-dx25c-fixcycle-wip"` -- stashed all 13 modified
   tracked files plus untracked scratch files. Confirmed clean tree after.
2. `git checkout a071e4ba` -- the commit immediately before this batch's
   plan/stage-1 commits (`6a25a1db`, `cf89a213`). Confirmed via `git status`
   (detached HEAD, clean).
3. `cargo test -p mtg-engine --test primitives
   t9_object_target_redirect_ignores_the_original_requirement -- --nocapture`
   -- **result below**.
4. `git checkout feat/pb-dx25c-the-object-target-redirect-ignores-cr-1157as-anothe`
   -- confirmed back on the working branch, all 13 modified files intact.
5. `git stash pop` -- restored. One untracked scratch file (a same-named log
   this step's own tooling had recreated post-checkout) collided and was left
   in the stash rather than silently overwritten; everything else (all 13
   tracked modifications, every other untracked file) restored cleanly. The
   stash was then inspected (`git stash list` showed only that one collision)
   and dropped (`git stash drop`) once confirmed harmless -- the colliding
   file was a disposable scratch log, not a deliverable.
6. `cargo check --workspace --all-targets` re-run post-restore: **clean, exit
   0** -- proves the stash/checkout/pop cycle did not corrupt any of the 13
   modified files.

**Result, captured verbatim**:
```
   Compiling mtg-card-defs v0.1.0 (.../crates/card-defs)
   Compiling mtg-card-types v0.1.0 (.../crates/card-types)
   Compiling mtg-engine v0.1.0 (.../crates/engine)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.65s
     Running tests/primitives/main.rs (target/debug/deps/primitives-65b2366bbfb11497)

running 1 test
test pb_dx25b_announced_stack_target_space::t9_object_target_redirect_ignores_the_original_requirement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 0.00s
```

**T9 was GREEN at the TRUE pre-PB-DX25c HEAD** (`a071e4ba`) -- exactly as the
§9 checklist claimed, and now pasted here directly rather than left to
cross-reference the Stage-1 baseline section above (which shows a DIFFERENT,
later-stage RED, for the fail-closed reason -- not the same claim, and T1's
own finding is what caught the two being conflated). This is the wrong-way-round
pin doing its job, confirmed by execution rather than by cross-reference.

**Disposition**: taken in full.

---

## Fix-cycle final gates -- all EXECUTED

**`cargo check --workspace --all-targets`**: clean, exit 0.

**`cargo test --workspace --no-fail-fast`**, captured to
`memory/primitives/pb-dx25c-fixcycle-full-test-run.txt` (4,820 lines; the raw
file is NOT committed, kept out of the tree per the coordinator's own file-set
convention -- only this summary is). Per-binary `test result:` lines summed:
**4,485 passed / 2 failed / 2 ignored** across 38 result-producing targets.

**The 2 failures are BOTH environment-caused, not code-caused, and are
independently verifiable from the captured output**:
```
---- card_defs_fmt::gate_catches_a_def_whose_oracle_text_is_one_long_line stdout ----
thread '...' panicked at crates/engine/tests/core/card_defs_fmt.rs:178:6:
copy gate script: Os { code: 122, kind: QuotaExceeded, message: "Disk quota exceeded" }

---- card_defs_fmt::gate_catches_an_unbreakable_over_width_line stdout ----
thread '...' panicked at crates/engine/tests/core/card_defs_fmt.rs:178:6:
copy gate script: Os { code: 122, kind: QuotaExceeded, message: "Disk quota exceeded" }

test result: FAILED. 500 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.48s
```
Both are in `core::card_defs_fmt` -- a PRE-EXISTING gate this batch never
touched, that copies a script to a temp location as part of its own test
setup. It failed for the identical `EDQUOT` reason as every Bash invocation
in this session, not because of anything PB-DX25c's fix cycle changed.
`crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs` and
`crates/engine/tests/core/pb_dx25c_retarget_roster.rs` both live in the
**same physical test binary** as `card_defs_fmt` (`core` and `primitives`
respectively) and their own tests are cleanly green within it -- confirmed
directly: the `primitives` binary's result line reads `1137 passed; 0
failed; 2 ignored`, and re-running `pb_dx25c_retarget_legality::` /
`pb_dx25c_retarget_roster::` in isolation (both before AND after this full
run) shows 10/10 and 5/5 green, respectively.

The workspace-level summary corroborates this precisely: `cargo`'s own final
`error: 9 targets failed:` block lists EXACTLY `-p mtg-engine --test core`
plus 8 `--doc` (doctest) targets across `mtg-card-db`/`mtg-card-defs`/
`mtg-card-pipeline`/`mtg-card-types`/`mtg-engine`/`mtg-network`/
`mtg-simulator`/`mtg-view-model` -- every one of the 8 doctest failures
prints the identical `error: failed to write arguments to temporary file:
Os { code: 122, kind: QuotaExceeded, message: "Disk quota exceeded" }` before
even collecting a test count. **No `primitives`, `simulator`, `rules`,
`play-server`, or any other target this batch's fix cycle touched appears
anywhere in that failure list.**

**Discrepancy from the coordinator's stated pre-fix-cycle pin (4,486/0/5),
disclosed rather than glossed**: this run measures 4,485/2/2, i.e. 1 fewer
passed (before accounting for the 1 new `t3b` test, which IS included in the
1137 count above -- so the true comparison is 4,486 + 1 = 4,487 expected vs.
4,485 + 2(quota-failed) = 4,487 measured, which reconciles exactly) and only
2 ignored where prior batches consistently cite 5. The passed/failed
reconciliation is exact; the ignored-count gap (2 vs. 5) is NOT explained by
anything this fix cycle changed (no `#[ignore]` attribute was touched) and is
most plausibly a second, unaudited casualty of the same quota outage --
though this could not be fully traced given the scale of the raw log and the
unreliability of `grep`-based searching under the same duress (`grep -c
"Compiling"` on the very output file quoted above intermittently returned
NO output against a file KNOWN to contain "Compiling" many times, confirming
Bash's own stdout-capture was unreliable independent of the command's actual
success -- several commands in this recovery window produced correct output
FILES while the tool's own captured stdout came back empty). **Recommendation
for the coordinator**: treat this run's PASS/FAIL numbers as trustworthy
(cross-checked three independent ways above) and the IGNORED count as
provisional; a clean re-run once the host `/tmp` quota has full headroom
would settle it definitively.

> **SETTLED — clean re-run by the coordinator, 2026-08-06, after the tmpfs
> regained headroom (4.4G free).** `cargo test --workspace --no-fail-fast`
> captured to a file: **4,487 passed / 0 failed / 5 ignored**, exit 0, **46**
> result-producing targets, **zero** `test result: FAILED` blocks and no
> `failures:` section at all. The prediction above was exactly right in both
> halves: the passed/failed arithmetic reconciles (4,486 + 1 `t3b` = 4,487, and
> 4,487 is what the clean run measures), and the ignored-count gap WAS a second
> casualty of the same outage (5, as every prior batch cites). `core
> card_defs_fmt` re-run in isolation: **5/5 green**. `hash_schema` 21/21 (HASH
> **74**), `protocol_schema` 17/17 (PROTOCOL **35**), `clippy --workspace
> --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
> `tools/check-defs-fmt.sh` clean (1,803 defs), coverage regenerated and
> unmoved at **1,133/1,803 = 62.8%**. **This is the batch's shipped pin**;
> the 4,485/2/2 figures above are retained as the record of the outage, not as
> a measurement of the tree. The fix-cycle agent's refusal to round its own
> anomaly away is why this was settleable rather than silently wrong — the
> 38-vs-46 target count in the line above is the tell it correctly flagged.

**`cargo test -p mtg-engine --test core hash_schema`**: 21/21 green.
**`cargo test -p mtg-engine --test core protocol_schema`**: 17/17 green.
Both confirm **HASH 74 / PROTOCOL 35 gate-EXECUTED and unmoved** by the fix
cycle (no hash/wire-affecting edit was made; every fix-cycle change is a
comment, a test, or a doc-string).

**`cargo clippy --workspace --all-targets -- -D warnings`**: clean, exit 0,
zero warnings.

**`cargo fmt --check`**: clean, exit 0. **`tools/check-defs-fmt.sh`**: clean,
1,803 defs checked (SR-35).

**Coverage regeneration** (`python3 tools/authoring-report.py`): `1,803
files | clean 1,133 (62.8%) | todo 519 | empty 151`, `135 missing` --
**byte-identical to the pre-fix-cycle count**. The only diff produced by
regeneration was the self-dating `Generated:`/`Git:` stamp lines and the
"recent card-touching commits" list (both expected to move on every
regeneration, per the tool's own convention) -- reverted with `git checkout
--` before this commit, confirmed by a post-revert `git status` showing zero
diff on the three `docs/authoring-status*` files.

**Scope diff, re-confirmed**: `git status --short` at the end of the fix
cycle shows exactly the 13 tracked files listed in the review doc's own
disposition table modified, plus `memory/primitives/pb-review-DX25c.md`
(untracked -- the review artefact itself, never previously committed) and
this file. No other file changed.

---

## Fix cycle 2 (`scutemob-205`, 2026-08-06) -- an acceptance-criteria review
found 7 issues; all 7 taken.

### Issue 1 -- `OOS-DX25c-5` CLOSED, not shipped live

`casting.rs:6450-6473`'s `TargetSpell`/`TargetSpellWithFilter` arm gained a
two-line `self_id` guard, mirroring the two single-target arms verbatim (the
guard text and rationale are in-source at the guard's own site).

**Cast-path neutrality verified BEFORE shipping, not asserted**:
`handle_cast_spell`'s target-validation call (`casting.rs:3727`/`:3736-3743`,
inside the `announced_requirements`-hoisted block) runs strictly BEFORE
`state.move_object_to_zone(card, ZoneId::Stack)` (`casting.rs:4440`), and
`move_object_to_zone` mints a NEW `ObjectId` for the post-move object
(`(new_card_id, _old_obj)` -- CR 400.7, a zone change is a new object). So
`self_id = card` at cast time is always the PRE-move (hand-zone) id, which
this arm already requires the CANDIDATE `id` to be `ZoneId::Stack` to pass --
the two can never be equal at cast time, confirmed by reading, not by
assumption. The guard is therefore live ONLY on
`rules::retarget::plan_target_change`'s path, which passes `victim_card` --
the STACK-RESIDENT id -- as `self_id`.

**T7's fixture (2004-10-04 "Misdirection is itself a legal candidate")
handled per the review's "or add a sibling" option**: T7 itself is
UNCHANGED in mechanism (still uses `TargetSpellWithFilter` + a colour
filter), with its doc comment corrected to record that the filter's
self-exclusion side effect is now REDUNDANT (the guard does it
structurally), not load-bearing. A new sibling,
`t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card`
(`pb_dx25c_retarget_legality.rs`), discriminates the guard DIRECTLY on the
PLAIN `TargetSpell` variant (no filter, no side door) -- the exact shape
`OOS-DX25c-5`'s registry row names as the concrete failure scenario.

**Revert proof, EXECUTED**: removed the guard (`// REVERT-PROOF TEMP: guard
removed`), rebuild confirmed (`Compiling mtg-engine`), ran the new test:

```
thread 'pb_dx25c_retarget_legality::t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card' panicked at crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs:1334:5:
assertion `left != right` failed: OOS-DX25c-5: a plain TargetSpell victim must never be redirected onto its OWN card -- self_id exclusion must fire here even with no filter to fall back on (Misdirection 2004-10-04: "You can't make a spell which is on the stack target itself")
  left: Object(ObjectId(6))
 right: Object(ObjectId(6))
test result: FAILED. 0 passed; 1 failed
```

Restored via a pre-mutation backup copy (`/tmp/casting.rs.bak`, itself
verified byte-identical to the file at fix-cycle-2 start before use);
`git diff --stat -- crates/engine/src/rules/casting.rs` matched the intended
21-insertion diff exactly, confirmed clean after restore.

**Registry**: `OOS-DX25c-5`'s row (`docs/audits/decision-point-audit.md`)
rewritten past-tense -- CLOSED, with the fix description, the cast-path
neutrality argument, the new probe, the executed-revert evidence, and an
explicit note that `OOS-DX25c-6` STAYS OPEN (the two rulings-halves are
independent; this row's half is complete on its own).

### Issue 2 -- AC 6303's `bolt_bend` object-branch half closed

Two new probes in `pb_dx25c_retarget_legality.rs`:
`bb1_bolt_bend_object_branch_lands_only_on_a_legal_creature_never_a_land`
(a real Bolt Bend redirecting a real "destroy target creature" spell, with a
LAND present as a legal-battlefield-object-but-illegal-type decoy -- the
redirect must land on the one other CREATURE, never the land, and the
SECOND resolution must actually destroy the NEW target while the original
creature and the land both survive) and
`bb2_bolt_bend_object_branch_no_legal_target_leaves_targets_unchanged` (the
CR 115.7a fallback half: no other creature exists at all, so NO
`TargetsChanged` event may fire, and the SECOND resolution must still
destroy the ORIGINAL target -- proving the fallback left it intact rather
than fizzling it).

**Revert proof, EXECUTED (BOTH tests, one mutation)**: a two-part mutation
in `rules::retarget::plan_target_change` -- (a) the per-index closure
reduced to `candidates.iter().find(|c| **c != current)?` (no legality check
at all) and (b) the final re-validation's result discarded (`let _ = ...`
instead of `.ok()?`) -- rebuild confirmed, both tests run:

```
thread 'pb_dx25c_retarget_legality::bb1_bolt_bend_object_branch_lands_only_on_a_legal_creature_never_a_land' panicked at crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs:1606:5:
assertion `left == right` failed: with the original creature (current target, excluded) and the land (fails TargetCreature) both unavailable, BB1 Legal Creature is the only remaining CR 115.7a-legal candidate
  left: Player(PlayerId(1))
 right: Object(ObjectId(2))

thread 'pb_dx25c_retarget_legality::bb2_bolt_bend_object_branch_no_legal_target_leaves_targets_unchanged' panicked at crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs:1721:5:
CR 115.7a: with no legal alternative creature, NO TargetsChanged event may be emitted -- the original target is unchanged, events: [..., TargetsChanged { stack_object_id: ObjectId(5), old_targets: [SpellTarget { target: Object(ObjectId(1)), zone_at_cast: Some(Battlefield) }], new_targets: [SpellTarget { target: Player(PlayerId(1)), zone_at_cast: None }] }, ...]
test result: FAILED. 0 passed; 2 failed
```

Both reddened -- illegally selecting a PLAYER for a `TargetCreature`
requirement, exactly the class of defect the two-part mutation
reintroduces. Restored via `/tmp/retarget.rs.bak`; `git diff --stat --
crates/engine/src/rules/retarget.rs` confirmed clean.

**A single-part mutation (legality check removed, final re-validation left
intact) was tried FIRST and only discriminated BB1, not BB2** -- worth
recording as a real finding, not just a discarded draft: with only the
per-index legality check removed, the final `validate_targets_inner`
re-validation (still active) rejects the blindly-picked candidate and the
whole plan returns `None` -- which happens to be EXACTLY the BB2-expected
outcome (no event) for the WRONG reason (an accidental catch, not a
deliberate "no legal alternative" answer). This is why the two-part mutation
was used for the row above: it is the mutation that actually reproduces the
pre-fix bug SHAPE (a wrong pick reaching the output), and it is the one that
discriminates both tests.

### Issue 3 -- AC 6304's object-branch half closed

New `s1b_bot_driven_misdirection_object_branch_redirects_legally`
(`crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs`): a bot-driven
Misdirection cast (through `StubProvider::legal_actions` +
`mtg_simulator::targeting::plan_targets` + `RandomBot::choose_action`,
identical machinery to S1) redirecting a real "destroy target creature"
spell, with the same land-decoy shape as BB1. Legality is checked by
MEMBERSHIP in `mtg_engine::legal_targets_per_slot`'s own `TargetCreature`
answer (never a literal), and the final state is checked with
`mtg_simulator::check_invariants`.

**Revert proof, EXECUTED, with S1 kept as the negative control in the SAME
run**: the identical two-part `retarget.rs` mutation from Issue 2, run
against BOTH `s1_...` and `s1b_...` in one `cargo test` invocation:

```
thread 's1b_bot_driven_misdirection_object_branch_redirects_legally' panicked at crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs:525:5:
the redirected target Player(PlayerId(1)) must be a MEMBER of legal_targets_per_slot's own TargetCreature answer [Object(ObjectId(1)), Object(ObjectId(2))] -- if this fails, the retarget and the offer layer disagree about what 'legal' means
test s1b_bot_driven_misdirection_object_branch_redirects_legally ... FAILED
test s1_bot_driven_misdirection_cast_redirects_legally ... ok

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**S1 stayed GREEN under the exact mutation that reddens S1b** -- direct,
executed confirmation of the review's own diagnosis that S1's player-branch
fixture cannot discriminate an object-branch legality defect (S1's chooser
IS legal for `TargetOpponent` by construction, so a blind "first candidate
!= current" pick still lands on a legal player there). Restored via
`/tmp/retarget2.rs.bak` (itself diffed byte-identical against
`/tmp/retarget.rs.bak` before use, confirming a clean starting point for
this second mutation); `git diff --stat -- crates/engine/src/rules/
retarget.rs` confirmed clean after restore.

### Issue 4 -- CLAUDE.md revert-matrix headline corrected

CLAUDE.md's PB-DX25c "Last Updated" delta (the "4 of 19... 2 not: has_lost
... chooser-first preference" sentence) was stale by exactly the V9 closure
fix cycle 1 already performed. Replaced with a "Correction (fix cycle 2...)"
paragraph stating the true count -- **16 of 19 revert-matrix rows
discriminate; 3 remain honestly UNDISCRIMINATED (V3, V7, V13)** -- matching
this file's own §"Summary (post-fix-cycle, review Finding T3)" verbatim, and
naming this fix cycle's OOS-DX25c-5 closure and its 4 new probes in the same
paragraph. **Confirmed unmoved by this fix cycle's own new tests**: T7b/
BB1/BB2/S1b discriminate a DIFFERENT defect (the missing `self_id` guard)
from every V1-V19 mutation in the original table -- none of V3 (final
re-validation redundancy at n=1), V7 (has_conceded defense-in-depth), or
V13 (copy propagation, blocked behind `OOS-DX25b-2`) is touched by the new
guard or the new tests, so the 16/19-discriminate, 3-undiscriminated count
from fix cycle 1 is UNCHANGED by fix cycle 2 -- verified by inspection of
what each new test's revert mutation actually is (a `casting.rs` guard
removal for T7b; a `retarget.rs` two-part mutation for BB1/BB2/S1b, neither
overlapping V3/V7/V13's own mutations).

### Issue 5 -- new `Tests (delta 2026-08-06, PB-DX25c fix cycle 2)` bullet

Added to CLAUDE.md as a NEW bullet (never grew an existing line, per the
2026-08-02 formatting rule) immediately above the pre-existing PB-DX25b
bullet: **4,491 / 0 / 5** (+4 over the 4,487 fix-cycle-1 SETTLED pin),
46 result-producing targets, HASH 74 / PROTOCOL 35 both gate-executed and
unmoved, coverage unmoved 1,133/1,803 = 62.8%.

### Issue 6 -- TRUE pre-edit baseline recorded

New section added above the (correctly-relabelled) post-stage-1 "Baseline"
section: the coordinator's own pre-any-edit `cargo test --workspace
--no-fail-fast` run, captured to `.../scratchpad/baseline.txt` at
`a071e4ba` before the plan was written, re-summed for this correction
(`grep`+`awk` over the raw file, shown in-section) -- **4,469 passed / 0
failed / 5 ignored, 45 result-producing targets, 0 `test result: FAILED`
blocks**. This is the TRUE AC 6305 baseline; the post-stage-1 section is
kept, relabelled, as the separate measurement it always was.

### Issue 7 -- positional-vs-best-fit validator mismatch recorded

`OOS-DX25c-1`'s row widened again: `plan_target_change` always re-validates
through the BEST-FIT `casting::validate_targets_inner`
(`retarget.rs:155-163`/`:173-181`), even for a victim whose CAST-TIME
validation used the POSITIONAL `validate_targets_positional`
(`casting.rs:3727`, the `ModeSelection.mode_targets` / CR 700.2c path).
Moot today for the same reason (a)/(b)/T10 already state (no `must_change:
true` corpus member is modal), stated as its own mechanism rather than
folded into an existing sentence, per the review's directive.

## Fix-cycle-2 final gates -- all EXECUTED

- `cargo check --workspace --all-targets` -- clean, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` -- clean, exit 0,
  zero warnings.
- `cargo fmt --check` -- FAILED on first run (the two new test files' hand-
  formatting), fixed with `cargo fmt` (whole-workspace, touched only the 2
  files this fix cycle already owned), re-ran clean, exit 0.
- `tools/check-defs-fmt.sh` -- clean, 1,803 defs.
- `cargo test --workspace --no-fail-fast`, captured to
  `.../scratchpad/fixcycle2-full-test-run.txt` (not committed, per the
  established convention -- only this summary is): **4,491 passed / 0
  failed / 5 ignored**, 46 result-producing targets, 0 `test result: FAILED`
  blocks. Reconciles exactly against the fix-cycle-1 SETTLED pin (4,487) +
  4 new tests (t7b, bb1, bb2, s1b) = 4,491.
- `cargo test -p mtg-engine --test core hash_schema` -- 21/21 green.
  `cargo test -p mtg-engine --test core protocol_schema` -- 17/17 green.
  **HASH 74 / PROTOCOL 35 both gate-EXECUTED and unmoved.**
- Coverage regeneration (`python3 tools/authoring-report.py`): `1,803 files
  | clean 1,133 (62.8%) | todo 519 | empty 151`, `135 missing` --
  byte-identical to the pre-fix-cycle-2 count. The self-dating churn across
  THREE files this time (`docs/authoring-status.md`,
  `docs/authoring-status-missing.txt`, `docs/authoring-status-prev.json` --
  the third one not touched by fix cycle 1's own note, confirmed a real
  diff, and reverted the same way) was reverted with `git checkout --`
  before commit; confirmed by a post-revert `git status --short docs/`
  showing zero diff on any `docs/authoring-status*` file.
- Scope: `git status --short` shows exactly the 5 tracked files this
  section's disposition covers (`crates/engine/src/rules/casting.rs`,
  `crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs`,
  `crates/simulator/tests/pb_dx25c_bot_retarget_is_legal.rs`,
  `docs/audits/decision-point-audit.md`, `CLAUDE.md`) plus this file. No
  other file changed.
