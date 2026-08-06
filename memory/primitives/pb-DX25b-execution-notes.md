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
   legality is untested by T1-T2 (both use PLAYER targets, the CR-correct branch, deliberately —
   plan §8 R2).
   **CORRECTED at the fix cycle (PB-DX25b review Finding E1, HIGH)**: the paragraph that used to
   sit here said "the plan scoped this as future work, not this batch's test surface" — **that is
   false as to the plan's own text**. Plan §8 R2 option (iii) is explicit: *"One additional probe
   asserts the object-target branch's illegal redirect **as the current behaviour**, cites CR
   115.7a, names `OOS-DX25b-3`, and tells the successor batch to invert it."* The plan scoped the
   **fix** as future work, not the **probe** — the probe was this batch's deliverable from the
   start, and it was missing from the implement phase. It has since been added:
   `pb_dx25b_announced_stack_target_space.rs::t9_object_target_redirect_ignores_the_original_requirement`,
   which casts a "destroy target creature" spell, redirects it via Misdirection onto a bystander
   land (lowest `ObjectId` on the battlefield other than the original target), and asserts the LAND
   is destroyed while the original creature survives — pinned wrong-way-round, citing CR 115.7a,
   naming `OOS-DX25b-3`, with an explicit instruction that the successor batch implementing
   object-target legality must invert the assertion. See §12 below for the completeness-decision
   record this finding also required.
4. **`OOS-DX25b-4`** — `deflecting_swat` (CR 115.7d "choose new targets", `must_change: false`)
   gains nothing from this batch: `effects/mod.rs`'s `!must_change` branch deterministically leaves
   all targets unchanged (`continue`), so the card announces its target correctly (post this batch)
   but the retarget itself is still a no-op — pre-existing (M9.4 "interactive choice deferred"), not
   caused by this batch. `r2_change_targets_roster_is_pinned` documents this in its own message
   ("membership here does not mean 'works'") but does not itself constitute a filed seed.
5. **`OOS-DX25b-5`** (added at the fix cycle, review Finding C3) — `deflecting_swat.rs` declares
   `TargetRequirement::TargetSpell` (spell-only) against a printed "target spell **or ability**".
   Blocked from being widened by the same missing ability id space `OOS-DX25b-1` names (an
   activated/triggered ability's stack entry is never added to `state.objects`, so it is
   unannounceable either way), and would change nothing observable even if widened
   (`must_change: false` makes the effect a deterministic no-op regardless of the requirement).
   The def's own comment previously asserted the OPPOSITE of what its code declares ("can target
   ANY spell or ability") — that contradiction is fixed in place (comment-only).

All five IDs were grepped against `docs/audits/decision-point-audit.md` and the `memory/` tree
before this report — **none exist yet** (the only occurrences found were inside
`memory/primitives/pb-plan-DX25b.md` itself, the plan document that names them prospectively, and
`memory/primitives/pb-review-DX25b.md`, which discusses `OOS-DX25b-5` as a candidate without filing
it).

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

---

## §12 Fix cycle (`memory/primitives/pb-review-DX25b.md`, 1 HIGH / 5 MEDIUM / 6 LOW, all 12 taken)

All twelve findings were applied. Three had a coordinator-directed scope decision (C1, C3, and
E1's completeness half); the other nine took the reviewer's own stated Fix directive. None
declined.

**E1 (HIGH), three parts, all landed in one set:**
1. `pb_dx25b_announced_stack_target_space.rs::t9_object_target_redirect_ignores_the_original_
   requirement` — a real 2-player cast chain (Destroy Creature → Misdirection targeting it),
   resolved via real `PassPriority`. Confirms, wrong-way-round: the redirect lands on a
   `TargetCreature`-violating LAND (lowest `ObjectId` on the battlefield other than the current
   target) and destroys it, while the original creature survives. Cites CR 115.7a, names
   `OOS-DX25b-3`, states the successor batch must invert the assertion.
2. Execution notes §10.3 (this file) corrected in place — the plan required the PROBE (§8 R2
   option (iii)), not the fix; the old parenthetical wrongly said the plan scoped the probe as
   future work.
3. Completeness decided explicitly for BOTH cards, not by omission: `bolt_bend.rs` and
   `misdirection.rs` each gained a comment-only block naming `OOS-DX25b-3` (and, for Bolt Bend,
   `OOS-DX25b-1` too), stating the reasoning (`completeness` measures fidelity to the PRINTED
   card; both defs translate it faithfully; the gap is in the SHARED `Effect::ChangeTargets`
   resolution logic, reachable from every card using it) and pointing at the wrong-way-round
   pins that must be inverted and revisited together. **Coordinator-directed**: neither def is
   demoted — see the coordinator's brief for the `OOS-DX20-10` precedent and the blast-radius
   argument (`CORPUS_COMPLETE = 1133` gate in `pb_dx32_fuzz_output.rs`, `OOS-CARDS2-3`'s
   re-roll-every-seed consequence). Verified, not merely trusted: `pb_dx32_fuzz_output.rs`'s
   `test_dx32_fuzz_deck_pool_size_is_pinned` stayed green throughout this fix cycle (unmoved,
   `CORPUS_COMPLETE` still 1,133 — confirmed via the full workspace run, §14 below).

**E2 (MEDIUM), preferred fix + hardening, both done — see §13 for the three-defeat re-execution.**
New `r6_casting_c1_c2_arms_use_the_shared_helper` (`pb_dx25b_announced_target_roster.rs`) mirrors
R4's per-arm structural check (helper called ≥1, zero `stack_objects.iter()`/`iter_mut()`) over
`casting.rs`'s two `if matches!(req, TargetRequirement::X) { .. }` blocks (a new
`extract_if_block_body` helper, since these are not `match` arms). R5 rewritten:
fixed-backward-window/`;`-only heuristic replaced with a symmetric, statement-boundary-aware scan
over the SPAN STRICTLY BETWEEN the nearest `card_in_stack_zone(`/`so.id ==` pair (`||` required
present, `;`/`}`/`let ` required absent in that span). R4's own doc corrected to stop calling R5
"the closest thing to a wide net" and states the true residual: no gate in this tree detects a
brand-new FIFTH site that never calls `card_in_stack_zone` at all.

**E3 (MEDIUM)**: R1/R2/R3's hand-written structural walkers (`is_single_target_spell_requirement`,
`effect_contains_change_targets`, `effect_contains_copy_spell_on_stack`, `effect_contains_draw_
cards`, `ability_contains`, `def_contains`, plus their `AbilityDefinition`/`TargetRequirement`
match arms) all DELETED, replaced by `sanitized_debug` — a `{:?}`-formatted, sanitized clone
(`oracle_text` cleared on all three faces, `completeness` normalized to `Complete`) scanned for
the needle string. Total over the whole struct by construction (derive-`Debug`), so immune to
`LoyaltyAbility`/`SagaChapter`/`ClassLevel`/`Forecast` and `Repeat`/`MayPayOrElse`/
`MayPayThenEffect`/`CoinFlip` — the exact blind spots the review measured. All three rosters
(`{Bolt Bend, Misdirection, Untimely Malfunction}` for R1, `+ Deflecting Swat` for R2, empty for
R3) reproduce IDENTICALLY under the new mechanism — re-measured, not assumed. **Load-bearing
proof, executed**: new `r3_sanitization_is_load_bearing` runs BOTH the sanitized scan (must NOT
flag Plumb the Forbidden) and the raw unsanitized `format!("{:?}", def)` (MUST flag it, on the
literal string inside its own `Completeness::partial(...)` prose) — both assertions pass, proving
sanitization is doing real work, not decoration. The module doc states the new mechanism's own
residual: a FUTURE free-text field this batch's sanitization doesn't know about is still a gap.

**E4 (LOW)**: `pb_dx25_stack_registry_roster.rs`'s G2 gains the same zero-`stack_objects.iter()`/
`iter_mut()` conjunct R4 holds its two arms to. Revert executed: inserted a bare
`state.stack_objects.iter().any(|s| s.id == id)` alongside the helper call in the
`Effect::CounterSpell` arm — reddened (`left: 1, right: 0`); restored, confirmed clean.

**E5 (LOW)**: all three sites (`effects/mod.rs:7570`, `pb_ef11_spell_single_target.rs:537`,
`pb_dx25b_announced_stack_target_space.rs:288`) re-justified from `GameEvent::TargetsChanged`'s
OWN doc comment (`rules/events.rs:1421-1422`) plus the true fact (`event_view.rs:927` discards the
field via `..`, so no consumer reads it today) — the fabricated "view-model/replay consumers read
it as one" claim removed everywhere it appeared. Doc-only, no revert needed.

**E6 (LOW)**: `casting.rs:8258`'s "C2" → "C1" — matches the plan's own census (C1 =
`TargetSpellOrAbilityWithSingleTarget`, `casting.rs:6476`-era). Doc-only.

**E7 (LOW)**: T5's doc gained a paragraph stating it is the ONLY test in the tree discriminating
the shared `!so.is_copy` guard, and that PB-DX25's own `pb_dx25_counterspell_stack_shapes` suite
does NOT discriminate it (re-confirmed: all six of that suite's tests stayed green under V2 during
the implement phase's own revert matrix — cited, not re-executed, since V2 was already executed
once and restoring/re-breaking it a second time for a doc-only fix would be pure overhead). A guard
shared by five consumers resting on one synthetic assertion is now a STATED residual, not an
implicit one. Doc-only.

**E8 (LOW)**: `casting.rs:8367-8368`'s misapplied "CR 115.7a's 'another legal target' excludes a
target that was never the spell's own target to begin with" sentence removed; the argument now
rests on CR 601.2a/601.2c alone, as the review specified. Checked (per the Fix directive) whether
`pb_ef11_spell_single_target.rs`'s module doc carries the same sentence — it does NOT (its own
citation-correction paragraph never had the misapplied clause), so no second edit was needed there.
Doc-only.

**C1 (MEDIUM, coordinator-decided)**: `bolt_bend.rs` stays `Complete` — see E1 part 3 above (same
comment block covers both C1's ability-half concern and E1's object-target-redirect concern).

**C2 (MEDIUM)**: new `t10_untimely_malfunction_mode1_target_index` probe. **First attempt (1
declared target) failed outright** — `InvalidTarget("expected 3..=3 target(s) but got 1")` — a
DIFFERENT, more fundamental mechanism than the plan anticipated: `Untimely Malfunction` uses
`mode_targets: None` (the flat/pooled scheme), so `casting.rs::target_count_range` demands a
target for ALL THREE pooled slots regardless of which single mode is chosen — CR 700.2c/700.2f's
per-mode-only targeting (the `mode_targets: Some(...)` fix PB-AC4 built) does not apply to this
def at all. Rebuilt declaring all three pooled targets in slot order (artifact, spell, creature) —
`validate_mapped_targets`'s own doc (`casting.rs:6226-6227`) states the returned targets preserve
DECLARATION order, not slot order, so declaration order must match pooled-slot order for
`DeclaredTarget{index: 1}` to land correctly. **Result: mode 1 DOES redirect correctly** —
`TargetsChanged` fires naming the victim's stack-entry id, and the victim's target becomes the
caster. The card def's own comment-only note (a `//` comment, not the `Completeness::partial(...)`
string itself — the string's TEXT was deliberately left byte-unchanged after an initial attempt to
edit it violated the "card-defs must be comment-only" scope rule, caught and reverted before this
report) now cites the probe as evidence for "Modes 0 and 1 are complete." R1's roster-test comment
softened per the review's instruction — no longer bare "unrelated", now "measured, not assumed",
citing T10.

**C3 (MEDIUM, coordinator-decided)**: `deflecting_swat.rs`'s contradictory `:32` comment ("can
target ANY spell or ability") corrected in place (removed — it directly contradicted the
`TargetRequirement::TargetSpell` line one below it); the requirement itself is UNWIDENED per
explicit coordinator instruction. A comment-only block records the mismatch as candidate
`OOS-DX25b-5`, notes it is blocked by the same missing ability id space `OOS-DX25b-1` names, and
states plainly that widening would change nothing observable (`must_change: false` is a
deterministic no-op regardless) and would misrepresent a no-op card as a completeness fix.

**T1 (LOW)**: `copy_redirect.rs` gained a module-doc paragraph naming the fiction (its
`push_spell_targeting_player` helper returns the `StackObject`'s OWN id, so every test using it
exercises only the direct-id clause) and pointing at the real-cast coverage
(`pb_dx25b_announced_stack_target_space.rs::t1`/`t2`). `test_bolt_bend_redirects_single_target_
spell` renamed to `test_change_targets_redirects_single_target_spell_by_stack_entry_id`, with a
doc note explaining the rename and crediting T2 as the real Bolt Bend integration test. Grepped
for the old name elsewhere in the tree — the only surviving occurrence is inside
`pb-review-DX25b.md` itself (the review's own historical record, left untouched).

---

## §13 The reviewer's three R5 defeats, re-executed against the hardened gate

Each defeat re-created verbatim as an orphan scratch `.rs` file dropped directly under
`crates/engine/src/` (R5's own directory walk reads file CONTENTS via `std::fs::read_dir` +
`read_to_string`, not the Rust module graph, so an orphan file needs no `mod` declaration and
never enters compilation — confirmed: `cargo test` for these runs did not print `Compiling
mtg-engine`, since no *compiled* code changed). Each file deleted immediately after its run;
`find crates/engine/src -name "_pb_dx25b*"` empty before moving to the next.

* **Defeat (a)** — a genuinely new lookup with NO `card_in_stack_zone` call anywhere
  (`state.stack_objects.iter().position(|so| so.id == id)` alone). **Still NOT caught** — R5
  stayed green. This is the STATED residual (R4's doc, R5's own doc): R5 anchors its scan on
  `card_in_stack_zone(` occurrences, so a lookup that never calls it has nothing for R5 to find.
  No gate in this tree catches this shape; only a much larger change (e.g. making
  `state.stack_objects` inaccessible outside a small set of sanctioned callers) could, and that
  is out of this fix cycle's scope.
* **Defeat (b)** — the preceding-statement `;` (`let announced = id; let pos = ...position(|so|
  { so.id == announced || (!so.is_copy && card_in_stack_zone(&so.kind) == Some(announced)) });`).
  **NOW CAUGHT**: `"found a second open-coded copy in: [\"_pb_dx25b_r5_defeat_b.rs (card_in_stack_
  zone at byte offset 268, id-eq at byte offset 188)\"]"`. The symmetric span-between-literals
  check no longer includes the unrelated preceding statements the old fixed backward window did.
* **Defeat (c)** — the reversed clause order (`card_in_stack_zone(...) == Some(id) || so.id ==
  id`). **NOW CAUGHT**: `"found a second open-coded copy in: [\"_pb_dx25b_r5_defeat_c.rs
  (card_in_stack_zone at byte offset 200, id-eq at byte offset 257)\"]"`. The new scan searches
  both directions from each `card_in_stack_zone(` occurrence, so which literal comes first no
  longer matters.

R5 re-confirmed green (no residue, no false positive on the real `resolution.rs::
counter_stack_object` site) after each defeat file was deleted.

**Net**: 2 of 3 named defeats closed; defeat (a) is a structural limit of a shape-based scan and
is now stated as such at both R4's and R5's own doc comments, rather than implied away by calling
R5 "the closest thing to a wide net."

---

## §14 Final gates, fix cycle — all EXECUTED, none predicted

* `cargo test --workspace --no-fail-fast` (to a file, never `| tail`) — **4,469 / 0 / 5**
  (+4 over the implement-phase pin of 4,465: `t9`, `t10`,
  `r6_casting_c1_c2_arms_use_the_shared_helper`, `r3_sanitization_is_load_bearing`). Residual
  list empty.
* `cargo test -p mtg-engine --test core hash_schema` — 21/21, `HASH_SCHEMA_VERSION == 73`
  unmoved.
* `cargo test -p mtg-engine --test core protocol_schema` — 17/17, `PROTOCOL_VERSION == 35` and
  the fingerprint gate unmoved.
* `cargo test -p play-server` — 80/80, unmoved.
* `cargo fmt --check` — clean (ran `cargo fmt` once for a rustfmt-driven rewrap in the new T9/R6
  test code).
* `tools/check-defs-fmt.sh` — 1,803 defs, clean (ran `--fix` once after the `untimely_
  malfunction.rs` comment addition).
* `cargo clippy --workspace --all-targets -- -D warnings` — clean, zero warnings.
* `cargo build --workspace` — clean (catches replay-viewer/TUI match-arm gaps; none here, no
  wire-type change in this cycle).
* Scope: `git diff main --numstat -- crates/simulator/ crates/view-model/ crates/card-types/
  tools/` — **EMPTY**. `git diff main -- crates/card-defs/` — every changed line across all four
  touched files (`bolt_bend.rs`, `deflecting_swat.rs`, `misdirection.rs`, `untimely_
  malfunction.rs`) is a `//` comment; the `Completeness::partial(...)` STRING in `untimely_
  malfunction.rs` was deliberately left byte-unchanged after an initial edit to it was caught and
  reverted for violating this exact rule (see §12 C2).
* Coverage: `tools/authoring-report.py` regenerated — substantive line byte-identical
  (`1,803 files | clean 1,133 (62.8%) | todo 519 | empty 151`), only the self-dating
  timestamp/git-head banner and recent-commits window differed; regeneration churn reverted with
  `git checkout --` (nothing substantive to commit).
* One real finding fixed along the way, NOT in the review: `bolt_bend.rs`/`misdirection.rs`'s new
  comment blocks tripped `completeness_deviation_scan::deviation_language_requires_a_marker_or_
  allowlist` (SR-12) — both contained the literal word "deviation". Reworded to "ENGINE-layer gap"
  throughout (meaning unchanged) rather than adding either card to that gate's `ALLOWLIST`, since
  `ALLOWLIST` is reserved for entries whose language describes FAITHFUL MODELING, not a real
  behavioural gap — these two genuinely have one, so allowlisting them would have been the wrong
  kind of exemption for what they are.
