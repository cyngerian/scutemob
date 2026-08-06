# PB-DX25b Execution Notes

**Branch**: `feat/pb-dx25b-validatetargetrequirement-spell-target-id-space-con` (ESM `scutemob-204`)
**Plan**: `memory/primitives/pb-plan-DX25b.md`

## Pre-edit baseline (measured, not trusted from the plan)

```
cargo test --workspace --no-fail-fast > baseline.txt 2>&1
```
Result: **4,452 passed / 0 failed / 5 ignored** (45 test binaries). Matches the plan's stated
baseline exactly — confirmed independently on this branch before any edit.

---

## §1 Production changes (all in ONE logical commit set)

1. **New primitive**: `crates/engine/src/state/stack_registry.rs` —
   `pub fn stack_index_for_announced_target(stack_objects: &imbl::Vector<StackObject>, announced:
   ObjectId) -> Option<usize>`, body: `so.id == announced || (!so.is_copy &&
   card_in_stack_zone(&so.kind) == Some(announced))`.
2. **C1** (`casting.rs`, `TargetSpellOrAbilityWithSingleTarget` arm): lookup routed through the
   helper.
3. **C2** (`casting.rs`, `TargetSpellWithSingleTarget` arm): lookup routed through the helper; the
   `is_spell` classification `matches!` block is kept verbatim (per plan §3.3 — it is CR-correct
   and only reachable through the direct-id clause today, `OOS-DX25b-1`); its surrounding comment
   extended.
4. **C3** (`effects/mod.rs`, `Effect::ChangeTargets`): the index is resolved ONCE at the top of the
   loop body (`let Some(pos) = stack_index_for_announced_target(...) else { continue };`); both the
   read (`state.stack_objects[pos]`) and the write (`state.stack_objects.get_mut(pos)`) use that
   same `pos`. `GameEvent::TargetsChanged.stack_object_id` now names the STACK-ENTRY id
   (`real_stack_id = stack_obj.id`, captured before mutation), not the announced card id — this is
   a behaviour correction on a path that never fired at HEAD, not a new field (no HASH/PROTOCOL
   impact, confirmed by gate execution, §4 below).
5. **C4** (`effects/mod.rs`, `Effect::CopySpellOnStack`): same index resolution; the REAL
   stack-entry id (`state.stack_objects[pos].id`) is passed into `copy::copy_spell_on_stack`, not
   the announced id. `copy.rs` itself is untouched.
6. **`Effect::CounterSpell`** (`effects/mod.rs`, PB-DX25's own consumer): the open-coded
   `position(...)` closure replaced with a call to the shared helper. Comment extended to point at
   the helper for the rule and keep the CR 702.21a/707.10 prose.
7. **CR 115.10 mis-citation corrected** (comment-only, §4.4 of the plan): `casting.rs`'s in-source
   test doc comment and `pb_ef11_spell_single_target.rs`'s module + Test 4 doc comments now cite
   CR 601.2a/601.2c/115.7a instead of the unrelated CR 115.10 (affects-vs-targets rule).

**Already-correct sites left byte-unchanged** (confirmed via `git diff`): `effects/mod.rs:7690-7692`
Ward's `DeclaredTarget` fallback (`exists_in_objects || exists_on_stack`), `PlayerTarget::ControllerOf`'s
objects-then-stack fallback, `resolution.rs::counter_stack_object` (its `so.id ==` lookup; its
classification already routes through `card_in_stack_zone` from PB-DX25), `copy.rs`'s three other
callers, `abilities.rs:6747`'s `targeting_stack_id` `matches!`.

**Zero simulator/view-model/card-types/tools lines**:
```
git diff main..HEAD --numstat -- crates/simulator/ crates/view-model/ crates/card-types/ tools/
```
Output: **empty**. Confirmed.

**Zero card-def lines**:
```
git status --short -- crates/card-defs/
```
Output: **empty**. Confirmed (matches the plan's "0 completeness flips" prediction — Misdirection
and Bolt Bend were already `Complete`; this batch makes the marker true, not changed).

---

## §2 Test files

### New

* `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs` — T1-T8 (8 tests).
  `mod` line added to `crates/engine/tests/primitives/main.rs`.
* `crates/engine/tests/core/pb_dx25b_announced_target_roster.rs` — R1-R5 (5 tests). `mod` line
  added to `crates/engine/tests/core/main.rs`.

### Modified

* `crates/engine/src/rules/casting.rs`'s in-source `#[cfg(test)] mod tests`:
  * `make_test_stack_spell` signature widened to `(id, source_object, controller, targets)` — the
    two existing callers previously passed ONE value for both, collapsing the id spaces.
  * `test_target_spell_single_target_self_targeting_prevented`: now mints a distinct `entry_id`
    (non-vacuity anchor asserted with `assert_ne!`).
  * `test_target_spell_with_single_target_self_and_kind_check`: now three sub-cases — (i) distinct
    ids (spell half, discriminates C2's lookup), (ii) COLLAPSED ids kept deliberately (the only
    remaining configuration that reaches the `is_spell` guard with a FOUND non-spell object), (iii)
    NEW — an ActivatedAbility with a DISTINCT id (the production shape), asserting the same
    "is not a spell" message text via the NOT-FOUND path, explicitly documented as NOT
    discriminating the `is_spell` guard.
* `crates/engine/tests/primitives/pb_ef11_spell_single_target.rs`:
  * `build_base_state` now mints a separate `entry_id` (via `test_util::next_object_id`) distinct
    from `other_id`; returns a 4-tuple. All 4 callers updated.
  * `test_spell_single_target_accepts_single_target_spell`'s non-vacuity assertion re-aimed at
    `entry_id` instead of `other_id`.
  * `test_spell_single_target_rejects_activated_ability`'s doc rewritten: **correction** — with
    distinct ids this no longer discriminates the `is_spell` guard at all (the lookup returns
    `None` outright before the guard is ever reached); points at `casting.rs`'s sub-case (iii) for
    the real discriminator.
  * `test_misdirection_retargets_single_target_spell` rebuilt: a real victim CARD object is placed
    in `ZoneId::Stack`, the `StackObject` entry gets a distinct id, and the announced target is the
    CARD id — replacing the old version, which announced the `StackObject`'s own id directly into
    `execute_effect`, a path no real cast can ever produce.
* `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`: G2 re-aimed (see §5 below).

---

## §3 T1/T2 headline non-vacuity proof (AC 6297/6298) — executed

`t1_misdirection_announces_and_resolves` and `t2_bolt_bend_announces_and_resolves` both:
* build the fixture through `CardRegistry` + `GameStateBuilder`, place the victim spell in HAND,
* cast it via a real `Command::CastSpell` (p2 → p3),
* capture `victim_card_id` (a `state.objects` id) and `victim_entry_id` (the `StackObject`'s own
  id) and assert `victim_card_id != victim_entry_id` (non-vacuity anchor),
* cast Misdirection/Bolt Bend via a real `Command::CastSpell` announcing `victim_card_id` — **the
  assertion this batch exists to make pass**,
* resolve via real `Command::PassPriority` in APNAP order, assert `GameEvent::TargetsChanged`
  fires naming the STACK-ENTRY id,
* resolve the victim too and assert an end-to-end life-total change (p1 -3, p3 unchanged).

Both pass at the fixed HEAD (confirmed below they fail hard, even to the point of a compile error,
against the unfixed helper — the mandatory A/B).

---

## §4 Revert matrix (§7 of the plan) — every mutation EXECUTED, rebuild confirmed, restored

For each row: mutation applied → `touch` the file → rebuild via the relevant `cargo test`
invocation (confirms `Compiling mtg-engine` in the captured output, never a stale binary) → failure
text captured verbatim below → mutation reverted → re-checked with `cargo check -p mtg-engine
--tests` clean.

| # | Mutation | Test(s) | Result |
|---|---|---|---|
| V1 | delete the card-owning-kind clause from `stack_index_for_announced_target` | `t1_misdirection_announces_and_resolves`, `pb_ef11…::test_spell_single_target_accepts_single_target_spell` | **RED**, both. `t1`: `InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot")`. `pb_ef11`: same message. |
| V2 | delete the `!so.is_copy` guard | `t5_copy_is_not_announceable` | **RED**. `left: Some(0), right: None` — the copy became findable once it was the only entry left with a matching `card_in_stack_zone`. PB-DX25's own `pb_dx25_counterspell_stack_shapes` suite (6 tests, incl. `test_dx25_countering_a_copy_moves_no_card`) re-run under this mutation and **still pass** — expected: their scenarios keep the original present, so `position()`'s first-match-wins semantics land on the original regardless of the guard (documented in T5's own doc comment). |
| V3 | delete the `so.id == announced` clause | `t7_ward_still_finds_its_target` | **RED**. Ward's `CounterSpell` no longer found Doom Blade; no `SpellCountered` event in the resolve-events list (verbatim panic message captured the full event list). |
| V4 | restore C1's old `state.stack_objects.iter().find(\|so\| so.id == id)` | `t2_bolt_bend_announces_and_resolves` | **RED**. `InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot")` at the Bolt Bend cast. |
| V5 | restore C2's old `find` | in-src `test_target_spell_with_single_target_self_and_kind_check`, `pb_ef11…::test_spell_single_target_accepts_single_target_spell` | **RED**, both. **Correction to the plan's own row**: the plan named `test_target_spell_single_target_self_targeting_prevented` as the discriminating in-src test — that test exercises `TargetSpellOrAbilityWithSingleTarget` (C1), not `TargetSpellWithSingleTarget` (C2), and does NOT redden under V5 (confirmed: still green). The correct discriminator is the SIBLING test, `test_target_spell_with_single_target_self_and_kind_check`, which does redden: `"without self_id, single-target spell is a valid target: Err(InvalidTarget(\"stack object ObjectId(1) is not a spell...\"))"`. |
| V6 | delete the `is_spell` guard (`let is_spell = true;`) | in-src `test_target_spell_with_single_target_self_and_kind_check` | **RED**. Sub-case (ii) (collapsed ids, ActivatedAbility) now returns `Ok(())` instead of the expected rejection — `"DECOY: an ActivatedAbility with exactly 1 target must be REJECTED... got: Ok(())"`. |
| V7 | restore C3's old `is_on_stack` read via raw `so.id ==` | `t1_misdirection_announces_and_resolves`, `pb_ef11…::test_misdirection_retargets_single_target_spell` | **RED**, both. `t1`: "TargetsChanged event must be emitted when Misdirection resolves" (no event at all — the announced card id no longer resolved to any entry). `pb_ef11`: target stayed `Player(PlayerId(3))` instead of being redirected to `Player(PlayerId(1))`. |
| V8 | restore C3's old write via raw `iter_mut().find(\|s\| s.id == stack_obj_id)` (against the ANNOUNCED id) | `t1_misdirection_announces_and_resolves` | **RED**. `"the victim's StackObject must now target p1" — left: Player(PlayerId(3)), right: Player(PlayerId(1))`. Proves the event-only assertion is insufficient exactly as the plan predicted: `TargetsChanged` still fired (emission is unconditional once `changed`), but the mutation silently landed on nothing. |
| V9 | restore C4's old `is_on_stack` check + pass the announced id straight into `copy_spell_on_stack` | `t8_copy_spell_on_stack_finds_its_target` | **RED**. `left: 1, right: 2` — no copy was created (the announced card id was passed as `stack_object_id`, which `copy_spell_on_stack` could never find). |
| V10 | emit the announced card id instead of `real_stack_id` in `TargetsChanged` | `t1_misdirection_announces_and_resolves` | **RED**. `"TargetsChanged.stack_object_id must name the STACK-ENTRY id ... left: ObjectId(3), right: ObjectId(4)"`. |
| V11 | insert `state.stack_objects.iter().any(\|s\| s.id == ObjectId(0))` into the `ChangeTargets` arm | R4 gate | **RED**. `"must contain ZERO occurrences of stack_objects.iter()/iter_mut() ... left: 1, right: 0"`. |
| V12 | hide the required `stack_index_for_announced_target` call inside a `/* */` comment, with an inline reimplementation providing identical runtime behaviour (functionally a no-op mutation, textually evasive) | R4 gate | **RED**. `"must call stack_index_for_announced_target at least once ... got 0 calls"` — proves comment-stripping is load-bearing (an unstripped scanner would have seen the comment text and passed). |
| V13 | `def_contains` unconditionally returns `false` (walker broken) | R3's `draw_cards_control` assertion | **RED**. `"Effect::DrawCards must be found on at least one corpus def ... Got 0 names."` — proves the empty `CopySpellOnStack` roster is not indistinguishable from a broken walk. |
| V14 | R1's `expected` set missing "Untimely Malfunction" (one member short) | R1 | **RED**. `assertion left == right failed ... expected 2 names, got 3`. |
| G2-1 | delete the `card_in_stack_zone` call in the zone-move classification (re-inline a per-kind `match`) | re-aimed G2 (`card_in_stack_zone_calls >= 1`) | **RED**. `"Expected >= 1 call to card_in_stack_zone ... got 0"`. |
| G2-2 | restore the CounterSpell arm's lookup to a raw `so.id == id` scan | re-aimed G2 (`stack_index_for_announced_target_calls >= 1`) | **RED**. `"Expected >= 1 call in the Effect::CounterSpell arm, got 0"`. |

**Every revert restored; `git status --short` confirmed the file set matches only the intended
production/test diff (no leftover `REVERT` markers — grepped `git diff ... | grep REVERT` returned
empty) before proceeding to the next mutation.**

**No revert failed to discriminate.** Every mutation in the table reddened its named test(s) exactly
as designed; V2's note above documents an EXPECTED non-discrimination in a DIFFERENT test suite
(PB-DX25's own), not a failure of the revert itself.

---

## §5 G2 re-aim (plan §8 R6) — executed as a deliberate edit, not silently

`pb_dx25_stack_registry_roster.rs::g2_counter_spell_arm_does_not_reclassify_by_kind` went RED
immediately after routing `Effect::CounterSpell`'s lookup through the shared helper (count of
`card_in_stack_zone` occurrences in the arm body dropped from 2 to 1 — the lookup's own
`card_in_stack_zone` call now lives INSIDE `state/stack_registry.rs`, not in the arm). Confirmed
red before any fix:
```
Expected >= 2 calls to card_in_stack_zone (lookup + move) in the Effect::CounterSpell arm, got 1
```
Re-aimed to two separate assertions: `card_in_stack_zone_calls >= 1` (the zone-move classification,
unchanged in the arm) AND `stack_index_for_announced_target_calls >= 1` (the new shared lookup).
Both conjuncts individually proven load-bearing by executing the two reverts G2-1/G2-2 above. The
forbidden-literal assertions (`StackObjectKind::Spell {`, `K::Spell {`, …) are byte-unchanged.

---

## §6 Mandatory A/B (git stash) — executed

```
git stash push -m "pb-dx25b full batch" -- crates/engine/src/effects/mod.rs \
  crates/engine/src/rules/casting.rs crates/engine/src/state/stack_registry.rs
cargo test -p mtg-engine --test primitives pb_dx25b
```
Result: **compile failure** — `cannot find function 'stack_index_for_announced_target' in module
'mtg_engine::state::stack_registry'` (2 errors, both in `t5_copy_is_not_announceable`, since it is
the only test calling the helper directly). This is the strongest possible form of "zero of the
positive probes pass at HEAD": at the pre-batch commit, the new primitive does not exist at all, so
the entire `pb_dx25b_announced_stack_target_space.rs` file fails to compile. `git stash pop` restored
the batch; `cargo check -p mtg-engine --tests` confirmed clean afterward.

---

## §7 Gates — all executed, none predicted

* `cargo test -p mtg-engine --test core hash_schema` → **21/21 pass**, incl.
  `hash_schema_version_sentinel` (`HASH_SCHEMA_VERSION == 73`). **HASH 73 unmoved.**
* `cargo test -p mtg-engine --test core protocol_schema` → **17/17 pass**, incl.
  `protocol_version_sentinel` (`PROTOCOL_VERSION == 35`) and
  `protocol_schema_fingerprint_is_pinned`. **PROTOCOL 35 unmoved.**
* `cargo test -p play-server` → **80/80 pass**.
* `git diff main..HEAD --numstat -- crates/simulator/ crates/view-model/ crates/card-types/ tools/`
  → **empty**.
* `git status --short -- crates/card-defs/` → **empty**.
* `cargo test --workspace --no-fail-fast` (to a file, never `| tail`) → **4,465 passed / 0 failed /
  5 ignored** (45 binaries). Delta over the 4,452 baseline: **+13**. Residual failure list: empty
  (`grep -E "^test .* FAILED$"` and `grep -E "^error"` both zero hits).
* `cargo clippy --workspace --all-targets -- -D warnings` → clean, zero warnings.
* `cargo fmt --check` → one file needed formatting on first run
  (`pb_dx25b_announced_stack_target_space.rs`); `cargo fmt` applied; re-ran `cargo fmt --check` →
  clean. Full workspace test suite re-run after fmt: **4,465 / 0 / 5**, unchanged.
* `tools/check-defs-fmt.sh` → `card-defs fmt gate: 1803 defs checked / clean`.
* `cargo build --workspace` → clean (catches replay-viewer/TUI match-arm gaps; none here, since no
  `StackObjectKind`/`KeywordAbility` variant was added).
* Coverage: `python3 tools/authoring-report.py` regenerated `docs/authoring-status.md` — the
  substantive line (`1,803 files | clean 1,133 (62.8%) | todo 519 | empty 151`) is **byte-identical**
  to the pre-batch state; only the self-dating timestamp/git-head banner and the trailing git-log
  window differed (expected — the doc is self-dating and this branch has commits ahead of the
  doc's last-generated commit). Regenerated files reverted with `git checkout --` afterward (no
  substantive change to commit).
* Benches: `cargo bench -p mtg-engine --bench engine_perf -- "full_turn_4p|priority_cycle_4p"` →
  `priority_cycle_4p` 23.7-24.2 µs, `full_turn_4p` 212.9-213.8 µs. Within noise of the PB-DX25 close
  pin (214-215 µs) — the helper runs once per announced target resolution, not in a hot loop.

---

## §8 Delta summary

**Test count**: 4,452 → **4,465** (**+13**), not the plan's "+18 to +25" estimate. **Correction to
the plan**: the delta is exactly the count of NEW test FUNCTIONS (8 in
`pb_dx25b_announced_stack_target_space.rs` + 5 in `pb_dx25b_announced_target_roster.rs` = 13); the
plan's own text acknowledged the repaired EXISTING tests are "modifications, not additions" but its
headline range implied a higher new-test count than the design ultimately needed (T1-T8 rather than
T1-T7+synthetic-T8, R1-R5 rather than R1-R5 with extra sub-tests). +13 is correct and complete —
every named test in plan §5.1/§5.3 exists and passes; no test was dropped to hit the number.

**PROTOCOL**: **35** (unmoved), gate-executed.
**HASH**: **73** (unmoved), gate-executed.
**Coverage**: **1,133/1,803 = 62.8%** (unmoved), proven by regeneration then discarded (no
substantive diff to keep).

---

## §9 Corrections to the plan (found during execution, not merely predicted)

1. **§4.4/Test 4 citation fix landed as documented** — no surprises there.
2. **§5.2 row 2's V5 test name is wrong.** The plan's revert-matrix table (§7, row V5) and its own
   §5.2 table both cite `test_target_spell_single_target_self_targeting_prevented` as (one of) the
   test(s) that discriminates C2's (TargetSpellWithSingleTarget) lookup repair. That test exercises
   `TargetSpellOrAbilityWithSingleTarget` (C1), not `TargetSpellWithSingleTarget` (C2) — confirmed
   by executing V5 against it (stays green). The correct discriminator, confirmed by execution, is
   the SIBLING in-source test, `test_target_spell_with_single_target_self_and_kind_check`. This is
   a naming slip in the plan, not a defect in the shipped fix — but it would have let a reviewer
   trust an assertion that was never actually tested if not caught by execution.
3. **T3's error-variant prediction was wrong.** §5.1 T3 and §8 R1 both predict a cast naming the
   ability's stack-entry id fails with `GameStateError::ObjectNotFound`. In fact
   `validate_object_satisfies_requirement`'s `ObjectNotFound` never reaches the
   `Command::CastSpell` caller: the bipartite target/slot matcher in `casting.rs`
   (`target_satisfies`, `casting.rs:6089-6098`) swallows any `Err` into a bare `.is_ok() == false`,
   and when no requirement slot matches, the caller sees the GENERIC "declared N target(s) but N
   could not be matched to a requirement slot" `InvalidTarget`. The underlying mechanism (the
   lookup can never find the id) is exactly as the plan describes; only the wire-level error
   VARIANT differs from the prediction. T3 was rewritten to assert `InvalidTarget` with an
   explanatory doc comment rather than the originally-predicted `ObjectNotFound`.
4. **T4's `SpellFizzled.source_object_id` does not name the fizzled spell's OLD card id.** The plan
   (§5.1 T4) predicts asserting `GameEvent::SpellFizzled` "naming Misdirection" via that field.
   `resolution.rs:247-249` moves the fizzling card to the graveyard as PART of constructing the
   fizzle event (CR 400.7 mints a new ObjectId before the event exists), so `source_object_id`
   names the NEW graveyard object, an id the test cannot predict in advance. The stable identifier
   across that zone move is `stack_object_id` (the misdirection stack-ENTRY id, distinct from its
   card id per this batch's own non-collapse discipline). T4 was rewritten to assert on
   `stack_object_id` instead, with the mechanism documented in the test's own comment.
5. **R5's non-vacuity floor of "at least 50 .rs files" was wrong** — the measured count under
   `crates/engine/src/` is 43. Floor lowered to 40 with the measured value noted in the message
   (the PB-DX24 R2 non-vacuity-floor discipline, applied to a directory-walk count rather than a
   card count).
6. **R5's first draft was a false positive on `resolution.rs::counter_stack_object`** — a legitimate,
   deliberately-NOT-unified site (plan §2.3/§3.4) that contains BOTH a `so.id ==` lookup and a
   SEPARATE, later `card_in_stack_zone` classification call in the same function, but NOT joined by
   `||` in one expression. A naive 400-byte proximity window flagged this as a "second open-coded
   copy" of the announced-target rule. Tightened to require the `||` token AND the absence of a `;`
   between the two literals (i.e. same statement, matching the actual shape
   `stack_index_for_announced_target`'s own body has) — this correctly excludes
   `counter_stack_object` while still catching a genuine re-open-coding (confirmed: R5 passes
   clean at HEAD with the tightened check, and the earlier draft's false positive is documented
   here rather than silently fixed).

None of these corrections changed the SHIP decision or required a production-code change beyond
what the plan already specified — all six are test-construction/prediction corrections, caught
by executing the plan rather than trusting its narrative, which is exactly the discipline the plan
itself asked for.

---

## §10 Candidate new seeds (NOT filed — reported for the grep-first filing protocol)

1. **`OOS-DX25b-1`** — Bolt Bend's "or ability" half is still unreachable. An activated (or
   triggered) ability's stack entry is minted (`abilities.rs:1381`) and never added to
   `state.objects`, so (a) `queries::legal_targets_per_slot` cannot enumerate it and (b) a cast
   naming it is rejected (mechanism: `validate_object_satisfies_requirement`'s opening
   `state.objects.get(&id).ok_or(ObjectNotFound)?` can never find it; observed wire error is the
   generic `InvalidTarget` from the bipartite slot matcher, not `ObjectNotFound` — see §9.3).
   Closing requires a new target id space (`Target::StackObject`, a wire/HASH/PROTOCOL change) —
   out of this batch's scope. Pinned wrong-way-round by `t3_ability_half_is_still_unreachable`.
   Evidence: `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_space.rs::t3_*`.
2. **`OOS-DX25b-2`** — a copy of a spell (CR 707.10) is not an announceable Misdirection/Bolt Bend
   target: the `!so.is_copy` guard is required for disambiguation and excludes copies by
   construction. Consequence: copies are unreachable as retarget targets. Compounds with
   `OOS-DX25-2` (copies raise no `PermanentTargeted`). Pinned by `t5_copy_is_not_announceable`
   (mechanism directly exercised via `stack_index_for_announced_target`, plus the real
   `rules::copy::copy_spell_on_stack` engine function).
3. **`OOS-DX25b-3`** — `effects/mod.rs:7591-7626`'s object-target redirect (the `ChangeTargets`
   arm's `Target::Object` branch) picks the smallest `ObjectId` in the recorded `zone_at_cast` with
   NO check that the new object satisfies the original spell's `TargetRequirement` (CR 115.7a's
   "another LEGAL target"). Self-documented as a KNOWN LIMITATION in the source at the same lines
   (unchanged by this batch). This was UNREACHABLE before this batch (nothing could announce a
   target at all); this batch makes it reachable for the FIRST time. Object-target redirect
   legality is untested by T1-T8 (both use PLAYER targets, the CR-correct branch, deliberately —
   plan §8 R2). No dedicated probe was added in this batch for the illegal-redirect behavior itself
   (the plan scoped this as future work, not this batch's test surface) — recommend the filing
   include a note that a probe for the actual wrong-answer behavior does not yet exist in the tree.
4. **`OOS-DX25b-4`** — `deflecting_swat` (CR 115.7d "choose new targets", `must_change: false`)
   gains nothing from this batch: `effects/mod.rs`'s `!must_change` branch deterministically leaves
   all targets unchanged (`continue`), so the card announces its target correctly (post this batch)
   but the retarget itself is still a no-op — pre-existing (M9.4 "interactive choice deferred"), not
   caused by this batch. `r2_change_targets_roster_is_pinned` documents this in its own message
   ("membership here does not mean 'works'") but does not itself constitute a filed seed.

All four IDs were grepped against `docs/audits/decision-point-audit.md` and the `memory/` tree
before this report — **none exist yet** (the only occurrences found were inside
`memory/primitives/pb-plan-DX25b.md` itself, the plan document that names them prospectively).

---

## §11 What was NOT done (explicitly out of my task's instructions)

* `OOS-DX25-3` was **not** marked CLOSED in `docs/audits/decision-point-audit.md` — the seed-filing
  protocol is explicitly reserved to the coordinator/owner per this task's instructions.
* `OOS-DX25b-1..4` were **not** filed into any registry — reported as candidates above only, per
  explicit instruction.
* `CLAUDE.md`'s "Current State" delta, the v3 queue row 7b strike, and
  `memory/workstream-state.md`'s handoff entry were **not** written — not named in this task's
  explicit instructions, and left to the coordinator to avoid guessing at cross-batch bookkeeping
  conventions under a grep-first protocol I was told not to run myself.
