# Primitive WIP — PB-RS4 (OOS-RS-3 / OOS-OS4-2 residuals) · PLAN

<!-- last_updated: 2026-07-26 -->

- **PB**: PB-RS4 — face-aware residuals: close the 3 surviving CR 712.8d/e deviations
- **Task**: `scutemob-146`
- **Branch**: `feat/pb-rs4-face-aware-residuals-close-the-3-surviving-cr-7128de-`
- **Class**: CORRECTNESS (latent — unreachable on today's roster, guaranteed to bite the
  first DFC with a back-face ETB replacement)
- **Phase**: CLOSED (steps 0-10 complete — implement, review, fix cycle, and close-out all done)
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §2.4 (chain notes) + §3 (R4 row)
- **Plan file**: `memory/primitives/pb-plan-RS4.md`
- **Review file**: `memory/primitives/pb-review-RS4.md`
- **Wire expectation**: **NO PROTOCOL bump, NO HASH bump** (PROTOCOL 27 / HASH 63 unchanged).
  This PB changes *which face's* ability list two existing gathering loops read, and extends an
  existing deregistration helper. It introduces no `Command` field, no `Effect` variant, no
  `HashInto`-reachable shape change. **If the planner finds a schema fingerprint must be
  re-pinned, STOP and re-scope** (explicit task-description directive) — do not silently re-pin.

## The three deviations (from triage §2.4 — verify each in-source before acting)

1. **`rules/replacement.rs:1160-1191`** — `apply_self_etb_from_definition` iterates
   `&def.abilities` (FRONT face) unconditionally instead of
   `def.effective_abilities(is_transformed)`. The comment at :1180-1191 self-labels this
   "PB-OS4b limitation (OOS-OS4-2)". Affects enter-transformed paths (craft / disturb /
   exile-return): a permanent entering back-face-up gathers the FRONT face's self-ETB
   replacements (enters-tapped / enters-with-counters), contra CR 712.8d/e.
2. **`rules/replacement.rs:1892-1913`** — `register_permanent_replacement_abilities`, same
   front-face read for non-self permanent replacement abilities; comment at :1907-1912.
3. **`rules/face.rs:104-171`** — `deregister_face_statics` handles only
   `AbilityDefinition::Static`. **Nine** other families registered by
   `register_static_continuous_effects` are never deregistered on transform:
   `TriggerDoubling`, `SuppressCreatureETBTriggers`, `StaticRestriction`,
   `CdaPowerToughness`, `CdaModifyPowerToughness` (up to TWO entries per ability),
   `AdditionalLandPlays`, `StaticFlashGrant`, `StaticPlayFromGraveyard`, `StaticPlayFromTop`.
   This was a **deliberate, well-argued deferral**, not an oversight (see the existing doc
   comment) — the plan must either extend symmetrically to all nine or justify a subset and
   re-file the remainder as an explicit seed (AC 5457 permits either).

## Known mechanism anchors (verify, don't trust)

- `register_static_continuous_effects` (`replacement.rs:2074`) already takes an explicit
  `is_transformed: bool` parameter — the established threading pattern for face awareness.
  `apply_face_change` (`face.rs:96-102`) passes it. The two replacement.rs sites take
  `new_id: ObjectId` + `card_id` + `registry` but no face signal.
- `def.effective_abilities(is_transformed)` is the face-selection accessor.
- **Index-parity hazard (PB-OS4b)**: any producer/consumer pair that keys on an ability index
  must agree on which ability list the index is relative to. Check whether either replacement
  site's output is index-keyed before changing the iteration source.

## Scope boundaries (task description, binding)

- **NOT in scope**: OOS-RS3-1 (queue-time intervening-if) and OOS-RS2-1 (TurnFaceUp cost).
  Separately rankable — **file seeds if touched, do not widen**.
- Expected: **0 coverage flips**. Honor `feedback_pb_yield_calibration` — do not inflate.

## Steps

- [x] 1. Plan phase (`primitive-impl-planner`) → `memory/primitives/pb-plan-RS4.md`.
      Plan file was already present at session start (detailed, step-numbered,
      file:line targets verified accurate against source during implementation —
      no line-number drift found in replacement.rs/face.rs/turn_actions.rs/
      ability_definition_registry.rs).
- [x] 2. Probe tests written FIRST and verified FAILING against pre-fix HEAD where the defect
      is reachable (AC 5458 explicitly requires fail-before/pass-after).
      **Step 0 orientation probe** (plan §6 Step 0): added a temporary `eprintln!` at
      the top of `apply_self_etb_from_definition` reading
      `state.objects.get(&new_id).map(|o| o.is_transformed)`, ran
      `cargo test -p mtg-engine --test mechanics_a_d disturb::test_disturb_enters_transformed -- --nocapture`.
      Observed `PB-RS4 STEP0 PROBE: is_transformed = Some(true)` — confirmed the
      plan's "already true at the call" claim empirically. Reverted the probe
      before any production edit.
      **17 probe tests** written in
      `crates/engine/tests/primitives/pb_rs4_face_aware_residuals.rs` (registered
      via `mod pb_rs4_face_aware_residuals;` in `tests/primitives/main.rs`), run
      against pre-fix HEAD with `cargo test -p mtg-engine --test primitives pb_rs4 -- --nocapture`.
      **All 17 failed** (verbatim panic messages, one per test):
      1. `test_disturb_back_face_self_etb_replacement_applies` — `"back face's
         self-ETB 'enters tapped' replacement must be gathered and applied when
         the permanent enters back-face-up (CR 614.12 / 712.8e)"`
      2. `test_disturb_front_face_self_etb_replacement_does_not_apply` —
         `assertion `left == right` failed: the FRONT face's 'enters with
         counters' self-ETB replacement must NOT apply once the permanent enters
         back-face-up (CR 712.8e); got 2 counters — left: 2, right: 0`
      3. `test_disturb_back_face_permanent_replacement_is_registered` —
         `assertion `left == right` failed: the back face's non-self permanent
         replacement ability must be registered (CR 614, 712.8e); found 0
         matching entries — left: 0, right: 1`
      4. `test_disturb_front_face_permanent_replacement_is_not_registered` —
         `assertion `left == right` failed: the FRONT face's non-self permanent
         replacement ability must NOT be registered once the permanent enters
         back-face-up (CR 712.8e); found 1 matching entries — left: 1, right: 0`
      5. `test_transformed_saga_stops_accruing_lore_counters` —
         `assertion `left == right` failed: a transformed Saga with no back-face
         SagaChapter abilities must not accrue another lore counter at precombat
         main (CR 714.3b / 712.8e) — left: 1, right: 0` (first attempt hit a
         test-setup bug, not the defect: "no priority holder" at the Draw step
         because the mock player's library was empty — fixed by adding two
         filler library cards, matching the pb_os4b test file's own pattern; the
         fixed version reproduces the intended CR 714.3b defect cleanly)
      6. `test_saga_chapter_trigger_index_matches_effective_face` — pending
         trigger produced when none should be: `pending_triggers: [PendingTrigger
         { source: ObjectId(1), ability_index: 1, ... }]` (assertion
         `state.pending_triggers().is_empty()` failed)
      7. `test_transform_deregisters_trigger_doubling` — `"the front's
         TriggerDoubling must be deregistered once transformed away from it (CR
         603.2d / 604.1)"`
      8. `test_transform_deregisters_etb_suppressor` — `"the front's
         SuppressCreatureETBTriggers must be deregistered once transformed away
         from it (CR 614.16a)"`
      9. `test_transform_deregisters_static_restriction` — `"the front's
         StaticRestriction must be deregistered once transformed away from it
         (CR 604.1)"`
      10. `test_transform_deregisters_cda_power_toughness` — `assertion `left ==
          right` failed: ... back face's printed power is 2 — left: Some(5),
          right: Some(2)`
      11. `test_transform_deregisters_cda_modify_both_entries` — `assertion
          `left == right` failed: both CdaModifyPowerToughness entries must be
          deregistered (power) — left: Some(5), right: Some(2)`
      12. `test_transform_deregisters_additional_land_plays` — `"the front's
          AdditionalLandPlays must be deregistered once transformed away from it
          (CR 305.2)"`
      13. `test_transform_deregisters_static_flash_grant` — `"the front's
          StaticFlashGrant must be deregistered once transformed away from it
          (CR 601.3b)"`
      14. `test_transform_deregisters_play_from_graveyard` — `"the front's
          StaticPlayFromGraveyard must be deregistered once transformed away
          from it (CR 601.3 / 305.1)"`
      15. `test_transform_deregisters_play_from_top` — `"the front's
          StaticPlayFromTop must be deregistered once transformed away from it
          (CR 601.3)"`
      16. `test_transform_there_and_back_restores_all_nine_families` —
          `assertion `left == right` failed: all nine families must be
          deregistered on the way out — left: 9, right: 0`
      17. `test_transform_does_not_remove_other_sources_registrations` —
          `assertion `left == right` failed: exactly one entry (the front's own
          count:1) must be removed, leaving the same-source count:5 entry alone;
          found [.. count: 1, .. count: 5] — left: 2, right: 1`. Note (deviation
          from the plan's prediction): plan §7.3 predicted this regression guard
          would "trivially pass" pre-fix; it did not — pre-fix
          `deregister_face_statics` removes nothing at all, so BOTH the front's
          own entry AND the injected same-source entry survive (2, not the
          asserted 1). That is real information consistent with — not
          contradictory to — deviation #3 (nothing is removed pre-fix), so the
          test was kept as written and its doc comment corrected to describe
          the actual (also-a-probe) pre-fix behavior rather than the predicted
          trivial-pass.
      No probe came up unexpectedly GREEN pre-fix.
- [x] 3. `apply_self_etb_from_definition` made face-aware; call sites threaded; limitation
      comment removed/updated.
      `crates/engine/src/rules/replacement.rs`: added a live `fizzle_object`
      read of `entering_is_transformed` right after the `def` guard; swapped
      `&def.abilities` → `def.effective_abilities(entering_is_transformed)` for
      the self-ETB-replacement gather loop, the `has_saga_chapters` scan, and
      the `has_class_levels` scan; replaced the stale "PB-OS4b limitation
      (OOS-OS4-2)" comment with an accurate PB-RS4 one; added a one-line pointer
      on `starting_loyalty` recording it stays front-only (OOS-OS4-1 / R10,
      deliberately out of scope). No call-site edits needed (plan verified
      both call sites — `resolution.rs:1673` disturb, `resolution.rs:7279`
      craft — already hold `is_transformed == true` before the call; confirmed
      by the Step 0 probe and by `cargo check` compiling clean with no other
      call sites touched).
- [x] 4. `register_permanent_replacement_abilities` made face-aware; call sites threaded;
      limitation comment removed/updated.
      `crates/engine/src/rules/replacement.rs`: same live-read pattern; swapped
      `&def.abilities` → `def.effective_abilities(entering_is_transformed)`;
      replaced the "PB-OS4b limitation (OOS-OS4-2)" comment with an accurate
      PB-RS4 one. Also (plan §6 Step 3) added the deviation-#4.2 fix inside
      `fire_saga_chapter_triggers`: added a live `fizzle_object` read of
      `is_transformed` on `saga_id` (no parameter — the fn is `pub` and has an
      external caller in `turn_actions.rs` plus a direct test caller in
      `saga_class.rs`, both already hold a live object) and swapped
      `def.abilities.iter().enumerate()` →
      `def.effective_abilities(is_transformed).iter().enumerate()`, plus a doc
      addition recording the index-namespace contract the three consumers
      (`resolution.rs:1996`/`:2028`, `sba.rs:889`) already rely on.
- [x] 5. `deregister_face_statics` extended per plan (all nine, or justified subset + seed);
      doc comment rewritten to match reality.
      `crates/engine/src/rules/turn_actions.rs`: `precombat_main_actions`'s CR
      714.3b Saga sweep swapped `def.abilities.iter().any(..)` →
      `def.effective_abilities(obj.is_transformed).iter().any(..)` (no new state
      lookup — `obj` is already the closure's bound `&GameObject`; SR-25
      ratchet for this file unaffected, confirmed by `bare_lookup_ratchet`
      staying green at its existing ceiling of 7). Added the CR 714.3b/712.8e
      citation inline.
      `crates/engine/src/rules/face.rs`: fully rewrote `deregister_face_statics`
      (now a thin loop) plus a new private `remove_one_registration` covering
      all TEN families (`Static` + the nine PB-RS4 additions:
      `TriggerDoubling`, `SuppressCreatureETBTriggers`, `StaticRestriction`,
      `CdaPowerToughness`, `CdaModifyPowerToughness` [up to two entries, built
      the same `modifications` vec the registration side builds],
      `AdditionalLandPlays`, `StaticFlashGrant`, `StaticPlayFromGraveyard`,
      `StaticPlayFromTop`), each arm doing first-`position()`-match +
      `remove()` (never a bulk `retain`-by-source purge). Fully-qualified
      `AbilityDefinition::X` patterns throughout (no aliasing import) per the
      SR-15 scanner requirement. Rewrote both the module-level doc comment and
      `deregister_face_statics`'s own doc comment to state the CR 604.1/613/
      712.8e/712.18 basis, the "remove at most the registered count" rule, the
      three same-source-but-safe registrants (Class level-up, emblem
      permission, `GrantFlash`), and pointed at the new parity gate — no
      surviving PB-OS4b deferral prose. `deregister_face_statics`'s exported
      name and call site (`face.rs:73` inside `apply_face_change`) unchanged.
      **Parity gate** written at `crates/engine/tests/core/face_dereg_parity.rs`
      (registered via `mod face_dereg_parity;` in `tests/core/main.rs`,
      alphabetically between `emblem_tests` and `hash_schema` — the plan's
      "after :10" line reference didn't match the file's actual current
      layout, so I inserted by alphabetical rule instead, consistent with the
      surrounding list's ordering convention). Brace-matches
      `register_static_continuous_effects`'s body out of `replacement.rs` and
      `remove_one_registration`'s body out of `face.rs`, strips `//` comments
      (same technique as `bare_lookup_ratchet.rs`), collects every
      `AbilityDefinition::<Name>` token with a word-boundary check (same
      technique as `ability_definition_registry.rs`), and asserts the two
      `BTreeSet<String>`s are equal plus a non-vacuity floor (`>= 10` names in
      each). Both `registration_and_deregistration_cover_the_same_ability_families`
      and `parity_scan_is_not_vacuous` pass.
- [x] 6. Regression tests: back-face ETB replacement gathered correctly after transform;
      one deregistration test per newly handled family.
      Covered by the 17-test file from step 2 (all now GREEN post-fix): 4
      disturb-path tests for deviations #1/#2 (back-face self-ETB replacement
      applies / front-face does not; back-face permanent replacement
      registers / front-face does not), 1 Saga lore-counter test + 1 index-
      parity test for deviation #4, 9 deregistration tests (one per family,
      `test_transform_deregisters_<family>`), 1 there-and-back round-trip test
      covering all nine at once, and 1 regression guard proving deregistration
      removes exactly the registered count (not a bulk source purge, not zero).
- [x] 7. PROTOCOL 27 / HASH 63 confirmed unchanged.
      `grep "pub const PROTOCOL_VERSION"` → `rules/protocol.rs:260` = `27`.
      `grep "pub const HASH_SCHEMA_VERSION"` → `state/hash.rs:578` = `63`. No
      `Command`/`GameEvent`/`Effect`/`AbilityDefinition`/struct-field shape
      changed — only which entries are pushed to/removed from existing
      collections, and which ability list two existing loops iterate. No gate
      forced a bump; nothing was silently re-pinned.
- [x] 8. Full gates: `cargo test --all`, `cargo clippy --all-targets -- -D warnings`,
      `cargo fmt --check`, `tools/check-defs-fmt.sh`.
      `cargo build --workspace` clean (no `view_model.rs`/`stack_view.rs` match-
      arm gaps — expected, no new `StackObjectKind`/`KeywordAbility` variant).
      `cargo test --all`: 29/29 test binaries `test result: ok`, 3696 total
      tests run, 0 failures (includes the new 17 + 2 parity-gate tests).
      `cargo clippy --all-targets -- -D warnings`: clean, zero warnings.
      `cargo fmt --check`: 7 formatting diffs surfaced after the engine edits
      (line-length wraps in `face.rs` and the new test file); ran `cargo fmt`,
      re-checked clean (exit 0).
      `tools/check-defs-fmt.sh`: `card-defs fmt gate: 1804 defs checked / clean`
      (0 card defs touched by this PB, as predicted).
      `python3 tools/authoring-report.py`: coverage unchanged at
      `1,139/1,804 = 63.1%` (0 flips, matching the plan's yield prediction);
      reverted the regenerated `docs/authoring-status*` files afterward since
      they only carry a self-dating timestamp/git-head bump, not a substantive
      change, to keep the diff scoped to PB-RS4's actual work.
- [x] 9. `primitive-impl-reviewer` pass with every finding dispositioned. Verdict: needs-fix
      (0 HIGH, 1 MEDIUM, 11 LOW). `memory/primitives/pb-review-RS4.md`.

## Fix cycle (post-review-9, all 12 findings dispositioned)

primitive-impl-reviewer verdict was needs-fix with 0 HIGH, 1 MEDIUM, 11 LOW — reviewer
called the correctness core clean and every finding documentation/test-coverage only. All
12 applied; none declined outright, though Finding 8 was confirmed no-action (matching the
review's own explicit recommendation) and Finding 4 was seeded rather than fixed (matching
the review's own explicit "seed, do not fix" directive).

- **Finding 1 (LOW, `rules/face.rs:201` + 2× test file + `replacement.rs:2158` — CR
  614.16a does not exist)** — APPLIED. Independently confirmed via the rules MCP-backed CR
  text file (`.scryfall-cache/MagicCompRules.txt`): `614.16` exists and governs
  token/counter-creation replacement effects, has no `a` subrule, and is unrelated to
  ETB-trigger suppression. Replaced all four occurrences (`face.rs:201`'s dereg arm,
  `replacement.rs:2158`'s registration arm, and both mentions in
  `pb_rs4_face_aware_residuals.rs`, doc comment + assertion message) with `CR 604.1 /
  603.2` plus an inline note that there is no dedicated CR subrule for the Torpor Orb
  pattern and that CR 614.16 should not be cited. Did NOT touch the other pre-existing
  `614.16a` mentions the review did not name (`state/mod.rs:159`, `replacement.rs:1340`/
  `:1406`, `state/hash.rs:7425`/`:7715`, `tests/rules/etb_trigger_suppression.rs`) — out of
  the four-occurrence scope the finding's Fix directive specified.
- **Finding 2 (LOW, `face.rs:13-17` — "nine sibling collections" lists seven)** —
  APPLIED. Reworded to "seven sibling collections" plus a parenthetical explaining that
  nine of the ten `AbilityDefinition` families this PB deregisters are non-`Static`, and
  two of those nine (`CdaPowerToughness`, `CdaModifyPowerToughness`) write into
  `continuous_effects` alongside `Static` rather than into a collection of their own — so
  nine families span exactly seven sibling collections plus `continuous_effects`.
- **Finding 3 (LOW, `replacement.rs:1291-1296` + test file — under-enumerated
  consumers)** — APPLIED. Rewrote `fire_saga_chapter_triggers`'s doc comment to state
  "every consumer that resolves a CardDef ability index resolves it against
  `effective_abilities(obj.is_transformed)`" and listed all eight sites
  (`resolution.rs:1996`/`:2028`/`:2066`, `sba.rs:889`, `abilities.rs:7004`/`:7082`/
  `:7210`/`:8379`), naming the SagaChapter-specific ones as examples rather than the
  complete set. Corrected the same "three consumers" phrase in
  `test_saga_chapter_trigger_index_matches_effective_face`'s doc comment (the only other
  place it appeared, per grep). Did not edit `memory/primitive-wip.md`'s own historical
  step-4 entry — that is a log of what was done at the time, not a live doc claim.
- **Finding 4 (LOW, `replacement.rs:1291-1326` — residual CR 113.7a index-namespace
  hazard, contract-level, pre-existing)** — SEEDED, not fixed, per the review's own
  explicit disposition ("Seed, do not fix"). Filed **OOS-RS4-4** in
  `memory/primitives/rider-seed-triage-2026-07-19.md` §1c, same row format as OOS-RS4-1/2:
  a transform between a Saga chapter trigger being queued and it resolving can desync the
  CardDef-ability-index namespace `fire_saga_chapter_triggers` writes into, contradicting
  CR 113.7a's "ability exists independently of its source once triggered"; unreachable
  today (the only roster Saga DFC, `fable_of_the_mirror_breaker`, is `partial` and its own
  chapter III uses the new-object-producing `ExileSourceAndReturnTransformed`, not an
  in-place transform); pre-existing PB-OS4b contract, not introduced by PB-RS4. No engine
  change made.
- **Finding 5 (LOW→treated as load-bearing, `face.rs:173-187`/`:287-295` — `Static` and
  `StaticFlashGrant` removal predicates narrower than registration)** — APPLIED. Added
  `&& !e.is_cda && e.condition == continuous_effect.condition` to the `Static` arm and
  `&& f.duration == EffectDuration::WhileSourceOnBattlefield` to the `StaticFlashGrant`
  arm. Added a regression test, `test_static_removal_does_not_match_cda_shaped_entry`:
  seeds a phantom `is_cda: true` continuous-effect entry (structurally identical to a
  `Static` ability's own registration in every field except `is_cda`) EARLIER in
  `state.continuous_effects` than the real `Static` registration, then asserts the real
  (non-CDA) entry is the one removed on transform and the phantom survives. **Discrimination
  verified**: temporarily dropped `!e.is_cda` from the `Static` arm's predicate and re-ran
  — failed with `assertion `left == right` failed: the real (is_cda: false) Static entry
  must be the one removed on transform` / `left: 1` / `right: 0` (verbatim). Reverted;
  re-confirmed `test_static_removal_does_not_match_cda_shaped_entry ... ok`.
- **Finding 6 (LOW, `tests/core/face_dereg_parity.rs:44-70` — brace-matched over
  un-stripped source)** — APPLIED. Reordered the pipeline in both
  `registration_families()`/`deregistration_families()` to strip `//` comments from the
  WHOLE source file FIRST, then brace-match the target function's body out of the
  already-stripped text (previously: brace-match first, strip second) — so a `//` comment
  containing a stray `{`/`}` cannot desync `extract_fn_body`'s depth count. Documented both
  named blind spots (a family moved into a registration helper; an arm that merely
  *names* a family without removing/registering anything) directly in the module doc,
  per the finding's Fix directive. Re-ran `registration_and_deregistration_cover_the_same_ability_families`
  and `parity_scan_is_not_vacuous` — both still pass post-reorder.
- **Finding 7 (LOW, `face.rs:322` — `pm.on_cast_effect == on_cast_effect.clone()`
  allocates a clone per comparison)** — APPLIED. Changed to
  `pm.on_cast_effect == *on_cast_effect` (dereference instead of clone).
- **Finding 8 (LOW, `replacement.rs:1178-1181` vs `:1240`/`:1260`/`:1278` — SR-4
  classification internally inconsistent, `fizzle_object` vs `expect_object_mut`)** — NO
  ACTION, matching the review's own explicit "No action" disposition: the finding itself
  states `fizzle_object` at the top of the function is the CORRECT SR-4 choice and no fix
  is required for RS4; the noted asymmetry with the three later `expect_object_mut` reads
  is flagged only as an optional future follow-up, not a defect this PB introduced or must
  fix.
- **Finding 9 (MEDIUM, `pb_rs4_face_aware_residuals.rs:955-1041` + siblings — degenerate
  per-family probe values can't catch a field-comparison regression)** — APPLIED, and
  strengthened beyond the finding's literal Fix directive after verification showed the
  literal directive was insufficient. First applied the directive as written (non-default
  `look_at_top`/`reveal_top`/`pay_life_instead`/`condition`/`on_cast_effect` on
  `StaticPlayFromTop`, `Some(condition)` on `StaticPlayFromGraveyard`) and ran the
  discrimination check the fix-cycle brief requires (temporarily drop a field comparison,
  confirm failure) — **both tests still PASSED** with the comparison dropped, because with
  only ONE entry sharing that `source` in the collection, `position()` finds it regardless
  of which fields the predicate compares; non-default values alone prove the entry was
  *registered* correctly, not that *removal* discriminates on those fields. Rebuilt both
  tests around the review's own "optional" negative case (made mandatory here, since it is
  the only shape that can actually fail): seed a phantom same-source entry with the OLD
  degenerate/default field values EARLIER in the collection than the real (non-default)
  registration, then assert the real entry is removed and the phantom survives.
  **Discrimination verified for both**:
  - `test_transform_deregisters_play_from_graveyard` — dropped the `condition` comparison
    from the `StaticPlayFromGraveyard` arm; failed with `assertion `left == right` failed:
    the front's StaticPlayFromGraveyard (condition: Some(ControllerLifeAtLeast(7))) must be
    deregistered once transformed away from it (CR 601.3 / 305.1)` / `left: 1` / `right: 0`
    (verbatim). Reverted; re-confirmed passing.
  - `test_transform_deregisters_play_from_top` — a single dropped field
    (`pay_life_instead`) did NOT fail it (the phantom differs on four other fields too, so
    those remain discriminating on their own); collapsing the whole `StaticPlayFromTop`
    predicate to `source`/`filter` only (the review's own named regression scenario, "someone
    simplifies the arm to compare only source and filter") DID fail, with `assertion `left
    == right` failed: the front's StaticPlayFromTop (non-default field values) must be
    deregistered once transformed away from it (CR 601.3)` / `left: 1` / `right: 0`
    (verbatim). Reverted; re-confirmed passing. Noted here rather than silently: an
    individual-field isolation break was not performed for each of the five
    `StaticPlayFromTop` fields separately (the phantom-vs-real fixture differs on all five
    at once, so a single dropped comparison is not guaranteed to be individually
    load-bearing) — the verified guarantee is against the review's actual named failure
    mode, not against every possible partial regression.
- **Finding 10 (LOW, `:542-578` — index-matches test proves face-awareness, not index
  parity)** — APPLIED. Added `mock_saga_index_parity_back_chapter_def` (front:
  `SagaChapter{chapter:1}` at index 1; back: a non-`SagaChapter` filler Triggered ability
  at index 0, `Transform` keyword at index 1, THEN `SagaChapter{chapter:1}` at index 2 —
  the back's chapter sits at a different position than the front's) and
  `test_saga_chapter_trigger_index_resolves_to_back_faces_chapter_position`, which asserts
  the produced `PendingTrigger.ability_index == 2` (the back's real position), not 1 (the
  front's) or any other value — proving the producer resolves to the correct position, not
  merely that it detects presence/absence of a `SagaChapter` on the back face.
- **Finding 11 (LOW, `:800-807` — assertion message wrongly claims `apply_face_change`
  rewrites base P/T)** — APPLIED. Reworded to "the object's base 2/2 is restored once the
  CDA is deregistered."
- **Finding 12 (LOW, `:465-468` — weak/possibly-vacuous `stack_objects().is_empty()`
  secondary assertion)** — APPLIED. Added a second assertion filtering
  `state.pending_triggers()` to `source == fable_id`, asserting none are pending for the
  transformed Fable, in addition to (not replacing) the existing stack-emptiness check.

**Gates re-run after all 12 dispositions**: `cargo build --workspace` clean; `cargo test
--all` 0 failed across all 29 suites (`primitives` suite: 734 passed, 1 ignored — up from
732 pre-fix-cycle with the 2 new tests); `cargo clippy --all-targets -- -D warnings` clean;
`cargo fmt --check` needed one `cargo fmt` pass (the new/rewritten test-file blocks),
re-verified clean and full suite re-run green after; `tools/check-defs-fmt.sh` clean (1804
defs, 0 card defs touched, as predicted). **PROTOCOL_VERSION == 27, HASH_SCHEMA_VERSION ==
63 — both unchanged** (grep-verified against `rules/protocol.rs:260` /
`state/hash.rs:578`), as expected for a fix cycle that touched only comments, one CR
citation family, two removal-predicate field lists, and test files.

No finding was declined outright; Finding 4 was seeded per its own explicit directive and
Finding 8 confirmed no-action per its own explicit directive.

- [x] 10. Close-out DONE (2026-07-26): OOS-OS4-2 flipped to **fully closed** in `CLAUDE.md`
      (Current State PB-OS4b parenthetical + RS-chain entry + "Last Updated" line),
      `memory/primitives/oos-retriage-plan-2026-07-18.md` (the PB-OS4b table row at :593 and a
      new ✅ RESOLVED block appended to the `scutemob-142` correction note), the
      `rider-seed-triage-2026-07-19.md` §5 banner (PB-RS4 SHIPPED row; next dispatch R5), and
      `memory/workstream-state.md` (PB-OS4b line corrected + a full PB-RS4 entry + next-session
      pointer). Original step text follows:
      Close-out: flip OOS-OS4-2 to fully closed in `CLAUDE.md`,
      `memory/primitives/oos-retriage-plan-2026-07-18.md`,
      `memory/primitives/rider-seed-triage-2026-07-19.md` §5 banner, and
      `memory/workstream-state.md`; file any new seeds.
      Seeds OOS-RS4-1 (stack craft / ExileSourceAndReturnTransformed
      never register permanent replacement abilities or queue ETB triggers) and
      OOS-RS4-2 (MDFC back faces are unplayable — 4 `Complete` MDFC lands have
      an unreachable back-face replacement) filed in
      `memory/primitives/rider-seed-triage-2026-07-19.md` §1c per the task
      instructions; OOS-RS4-3 cross-referenced to OOS-OS4-1/R10 rather than
      duplicated. **OOS-RS4-4** (transform between trigger queue and resolution
      can desync a CardDef ability index, CR 113.7a) filed in the fix cycle per
      review finding #4.

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`). PB-RS2 SHIPPED (`scutemob-144`, merge
`86176ff7`; PROTOCOL 26→27). PB-RS3 SHIPPED (`scutemob-145`, merge `b1c21909`; 3 flips +
helm_of_the_host integrity repair; seeds OOS-RS3-1..4 filed). Queue was PAUSED after R3 by the
user on 2026-07-20 and resumed here at R4. The R1..R11 ranked queue lives in
`memory/primitives/rider-seed-triage-2026-07-19.md` §3.
