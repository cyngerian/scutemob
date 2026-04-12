# LOW Issues Remediation Plan

> Catalog of all ~68 OPEN + 5 DEFERRED LOW-severity issues from milestone reviews,
> grouped by regression risk, with phased implementation guidance.
>
> **Guiding principle**: The engine has 1,033 passing tests and is at a stable
> checkpoint (M9.5 complete). Every change below is evaluated against the risk of
> breaking that baseline. Changes are grouped into tiers by how likely they are to
> introduce regressions, not by how important the underlying issue is.

*Generated: 2026-02-28*

---

## Risk Assessment Framework

Each fix is classified into one of four risk tiers:

| Tier | Risk Profile | Examples | Gate |
|------|-------------|----------|------|
| **T1 — Zero risk** | Additive only: new tests, new assertions, dead code removal, comments. Cannot change runtime behavior. | Add a missing test, delete unused variant, add `debug_assert!` | `cargo test --all` passes |
| **T2 — Cosmetic risk** | Touches runtime code but in paths that are not reachable in normal game flow, OR changes are purely mechanical (rename, type swap). | Replace `unwrap_or` with `.ok_or()`, rename error variant, swap `HashMap` for `BTreeMap` where only point-lookups occur | `cargo test --all` + `cargo clippy` pass |
| **T3 — Behavioral risk** | Changes observable behavior or touches hot paths (SBA loop, effect execution, combat damage). Correct changes could still expose latent bugs elsewhere. | Add `ManaPool::spend()` method and migrate callers, change zone `contains` implementation, add shuffle to replacement effect | `cargo test --all` + manual review of all call sites + targeted new tests first |
| **T4 — Architectural** | Requires new types, new enum variants, or cross-cutting changes to multiple modules. | `is_copy` field on `GameObject`, `ShuffleIntoLibrary` replacement variant, `AddSupertypes` layer modification | Design review + implementation plan + dedicated test suite |

---

## Tier 1 — Zero Risk (add tests, assertions, comments, delete dead code)

These changes are purely additive or subtractive. They cannot alter runtime behavior.
**Implement anytime** — individually or in small batches. Each is independently safe.

### New tests (fill coverage gaps)

> **[DONE — Phase 0 complete 2026-03-03]** All 10 tests below were added as part of the W3 T1
> remediation pass (Phase 0). Tests are in `crates/engine/tests/`.

| ID | File | What to add | Why it matters |
|----|------|-------------|----------------|
| MR-M1-19 | `tests/` | Test same-zone move (battlefield → battlefield) produces new ObjectId | Protects CR 400.7 invariant — the most critical rule in the engine |
| MR-M1-20 | `tests/` | Test valid object moved to invalid destination zone returns error | Documents error behavior; currently untested error path |
| MR-M2-08 | `tests/concede.rs` | Test concede when active player + all others passed | Exercises a complex code path (engine.rs:302-307) with zero coverage |
| MR-M2-17 | `tests/concede.rs` | Test concede during active combat phase | Would likely expose stale `combat` state issues |
| MR-M4-13 | `tests/sba.rs` | Test aura whose target left the battlefield (attached_to points to object in graveyard) | Exercises the `target.zone != Battlefield` SBA branch |
| MR-M6-08 / MR-M7-16 | `test-data/generated-scripts/combat/` | Add at least one combat game script through the replay harness | Combat is the most complex subsystem with zero script coverage |
| MR-M8-15 | `tests/` | Test multiple ETB replacements on same permanent (self-ETB + global ETB) | Validates CR 614.15 ordering — currently untested |
| MR-M9-14 | `tests/commander.rs` | Test 3+ mulligans (London mulligan escalation) | Validates escalating bottom-of-library count |
| MR-M9-15 | `tests/commander.rs` | Test `BringCompanion` rejected with non-empty stack | Exercises companion timing validation |
| MR-M9.4-15 | `tests/card_def_fixes.rs` | Add counter-assertion: players without Thought Vessel DO discard | Strengthens an existing weak test |

**Estimated effort**: ~2 hours for all 10. Each test is independent; any subset is safe.

### Strengthen existing tests

> **[DONE — Phase 0 complete 2026-03-03]** All 4 test improvements below were applied.

| ID | File | What to change | Notes |
|----|------|----------------|-------|
| MR-M2-07 | `tests/turn_invariants.rs` | Add 10+ library cards per player in `run_pass_sequence` proptest | More turn cycles exercised before empty-library loss |
| MR-M5-08 | `tests/layers.rs` | Add test with two effects in same sublayer, one CDA and one not | Tests the `is_cda` partition logic in `resolve_layer_order` |
| MR-M9.4-13 | `tests/loop_detection.rs` | Replace tautological assertion with meaningful check | Current assertion is `is_none() || is_some()` — always true |
| MR-M9.4-14 | `tests/trigger_doubling.rs` | Add integration test for full ETB-to-doubler-registration pipeline | Current tests manually inject doublers, bypassing registration |

**Estimated effort**: ~1 hour. Each is isolated.

### Dead code and comments

| ID | File | What to do | Notes |
|----|------|------------|-------|
| ~~MR-M1-14~~ | `state/error.rs` | ~~Remove `InvalidZoneTransition` variant~~ | DONE — variant deleted |
| ~~MR-M9.4-11~~ | `casting.rs:232-234` | ~~Add comment explaining `spells_cast_this_turn` increment ordering dependency~~ | DONE — comment added |
| ~~MR-M9.5-08~~ | `Counter.svelte` | ~~Delete unused scaffold component~~ | DONE — file deleted |
| ~~MR-M1-06~~ | ~~`structural_sharing.rs`~~ | ~~Consider deleting test file (real structural sharing tested in `snapshot_perf.rs`)~~ | DONE — file deleted 2026-03-03 |

**Estimated effort**: 15 minutes.

---

## Tier 2 — Cosmetic Risk (mechanical code changes, no behavior change)

> **[DONE — Phase 3 T2 complete 2026-03-03]** All T2 items below were applied in commit
> `08c7b32` (`W3: apply T1+T2 LOW remediation`). 1421 tests pass. Clippy passes.
>
> **Bugs found by T2 work**: The `debug_assert!` additions (MR-M1-16, MR-M1-17) caught
> 5 real test construction bugs in `targeting.rs` — tests re-adding players already added
> by `GameStateBuilder::four_player()`. Fixed by replacing `add_player_with(p1, ...)` +
> `add_player(p(2/3/4))` with `.player_mana(p1, ManaPool { ... })`. This is the intended
> behavior of T2: treat debug_assert failures as bugs found, not regressions.
>
> **New LOW found**: `overlord_of_the_hauntwoods.rs:83` has a pre-existing clippy warning
> ("struct update has no effect — all fields already specified"). Confirmed pre-existing via
> `git stash`. Tracked as `MR-W3-01` in `docs/mtg-engine-milestone-reviews.md`.

These touch runtime code but in paths that are either unreachable in normal play
or where the change is purely mechanical. **Implement between milestones** when the
test suite is green and you can run the full suite after each change.

### `debug_assert!` additions (catch configuration bugs in tests, no-op in release)

| ID | File | Change | Regression concern |
|----|------|--------|-------------------|
| MR-M1-16 | `builder.rs` | Add `debug_assert!(found)` after player setter loops | Only fires in debug/test builds. If any existing test has a wrong PlayerId, the assert would expose it — which is the point, but could initially look like a regression. Run `cargo test --all` immediately after and fix any exposed issues. |
| MR-M1-17 | `builder.rs` | Add `debug_assert!` checking for duplicate PlayerIds in `add_player` | Same as above. No production code affected. |
| MR-M4-11 | `sba.rs:281` | Change `unwrap_or(1)` to `unwrap_or(0)` for planeswalker loyalty | Zero-loyalty planeswalker would die to SBA (correct behavior). `unwrap_or(1)` lets incorrectly-constructed planeswalkers survive. If any test has a badly-built planeswalker, this would expose it. |

**Strategy**: Add all three, run tests. If a test fails, the test was hiding a real bug.

### Error name corrections

| ID | File | Change | Regression concern |
|----|------|--------|-------------------|
| MR-M3-12 | `lands.rs:83-87` | Return `InvalidCommand("card not owned by player")` instead of `NotController` | Any test that `assert_eq!` on the specific error variant would need updating. Grep for `NotController` in tests first. No behavior change — same error path, different message. |

### Silent-default hardening

| ID | File | Change | Regression concern |
|----|------|--------|-------------------|
| MR-M2-09 | `turn_actions.rs:142` | `unwrap_or(7)` → `.ok_or(GameStateError::PlayerNotFound(active))?` | Requires corrupted state to trigger. No legitimate code path hits this. Zero risk of behavioral change in any test. |
| MR-M3-11 | `abilities.rs:435` | `unwrap_or(0)` → `.ok_or(GameStateError::...)?` | Same — requires active player missing from turn_order. Unreachable in practice. |
| MR-M9-17 | `commander.rs:63-78` | Add check for empty `commander_card_ids` | New validation that rejects an invalid input. Could only break a test that passes 0 commanders — which would be a malformed test. |

### HashMap replacements (determinism)

| ID | File | Current usage | Iterated? | Fix | Risk |
|----|------|--------------|-----------|-----|------|
| — | `effects/mod.rs` (`target_remaps`) | `HashMap<usize, ObjectId>` | No — only `.get()` and `.insert()` | **No fix needed.** Point lookups only; iteration order irrelevant. | None |
| — | `rules/sba.rs` (`chars_map`) | `HashMap<ObjectId, Characteristics>` | No — only `.get()` | **No fix needed.** Lookup table, never iterated. | None |
| — | `rules/combat.rs` (`blocker_count_for_attacker`) | `HashMap<ObjectId, usize>` | No — only `.entry()` and `.get()` | **No fix needed.** Counter map, never iterated. | None |
| — | `cards/registry.rs` (`definitions`) | `HashMap<CardId, CardDefinition>` | No — only `.get()` and `.from()` | **No fix needed.** Static registry, only point lookups. | None |
| — | `testing/replay_harness.rs` | `HashMap<String, ...>` | Yes — but test-only code, not engine behavior | **Low priority.** Non-determinism here affects test output ordering, not game correctness. | Negligible |
| MR-M9.5-11 | `replay-viewer/api.rs` | `player_map.keys().cloned().collect()` | **Yes — iterated** | Sort keys before collecting. 1-line fix. | Negligible (cosmetic API ordering) |

**Conclusion**: The `HashMap` non-determinism concern is **less severe than it appeared**. All engine-internal usages are point-lookup only. The only iterated HashMap is in the replay viewer's API response (cosmetic). No engine-level determinism fix is required.

### Performance micro-optimizations

| ID | File | Change | Regression concern |
|----|------|--------|-------------------|
| MR-M9.4-09 | `loop_detection.rs:117` | Remove unnecessary `Vec` collect + sort (im::OrdMap is already sorted) | Pure optimization — same result, fewer allocations. Could only regress if im::OrdMap iteration order assumption is wrong (it's documented as sorted). |
| MR-M4-10 | `sba.rs:391,449,453` | Cache `SubType("Aura".to_string())` etc. as `lazy_static` or module-level constants | Eliminates per-object-per-SBA-pass string allocations. Mechanical change. |
| MR-M5-06 | `layers.rs:417` | `Vec::remove(0)` → `VecDeque::pop_front()` in Kahn's algorithm | O(n) → O(1) per iteration. n is always ≤20 so impact is negligible but the fix is trivial and correct. |

**Strategy**: Apply these individually with a test run after each. They are the kind of change where "obviously correct" still deserves verification.

---

## Tier 3 — Behavioral Risk (changes to game-affecting code paths)

These modify code that directly affects game outcomes. Each should be preceded by
writing the test that validates the correct behavior *before* making the change.
**Implement only when deliberately focused on correctness work**, not as side-effects
of other tasks.

### ManaPool encapsulation (MR-M1-15)

| Aspect | Detail |
|--------|--------|
| **Issue** | `ManaPool` has no `spend()`/`pay()` method. Mana spending is done via raw field manipulation in `rules/mana.rs`. |
| **Risk** | Adding `spend()` and migrating callers is a refactor across `mana.rs`, `casting.rs`, and potentially `abilities.rs`. Any mistake in the migration could cause mana to not be deducted or to underflow. |
| **Prerequisite** | Write comprehensive `ManaPool` unit tests first (addresses MR-M1-07 simultaneously). Test: spend exact, spend insufficient (error), spend colored, spend generic with color priority. |
| **When** | M10 networking milestone. Mana payment correctness becomes critical when real players are involved. The refactor also makes the mana API cleaner for the network layer to reason about. |
| **Approach** | 1. Add `spend()` method. 2. Add tests for `spend()`. 3. Migrate ONE call site. 4. Run tests. 5. Migrate next call site. 6. Repeat. Never migrate all at once. |

### Darksteel Colossus shuffle (MR-M8-14)

| Aspect | Detail |
|--------|--------|
| **Issue** | "Shuffle into library" replacement uses `RedirectToZone(Library)` — moves to top without shuffling. |
| **Risk** | Adding a shuffle step after the zone move changes observable game state (library order). Any test that checks library order after a Darksteel Colossus would-die event would break. |
| **Prerequisite** | Verify no existing tests depend on Colossus going to library top. Write a test asserting the library IS shuffled. |
| **When** | When implementing more "shuffle into library" effects (Blightsteel Colossus, etc.) or when library-order-matters interactions are being tested. Not urgent as a standalone fix. |

### W3-LC residuals surfaced by PB-S fix cycle (PB-S-L02/L03/L04/L05)

| Aspect | Detail |
|--------|--------|
| **Context** | During PB-S review fix-cycle spot-check of `rules/abilities.rs::handle_activate_ability`, four sites were found that read base `obj.characteristics.*` where calculated characteristics should be used (CR 613.1f). Two were fixed in PB-S (HIGH: invisibility of granted activated abilities, and HIGH: summoning-sickness/haste check on tap-cost activations — sibling of the mana.rs fix). The remaining four are LOW because they are latent bugs not directly on the granted-ability invisibility path. Logged per oversight instruction: "If the spot-check finds base-reads outside abilities.rs, log them as new LOWs... move on. Don't fix them in this session." Scope interpretation extended to in-file-but-out-of-scope finds. |
| **Related** | PB-S (surfaced these), W3-LC (original layer-correctness audit, missed these) |

**PB-S-L02** — `crates/engine/src/rules/abilities.rs:159-165`. Channel/graveyard zone dispatch reads `obj.characteristics.activated_abilities.get(ability_index).map(|ab| (ab.cost.discard_self, ab.activation_zone.clone()))` from base. For PB-S grants, `.get()` returns `None` → falls through to the battlefield branch, which is correct-by-accident for all current Layer 6 grant patterns. Latent bug for future "grant a channel ability" or "grant a graveyard-activated ability" patterns. Fix: read from `calculate_characteristics`.

**PB-S-L03** — `crates/engine/src/rules/abilities.rs:583-585`. Sacrifice-self cost path reads `obj.characteristics.card_types.contains(Creature)` to decide whether to emit `CreatureDied` or `PermanentDestroyed`. If an animated creature (Layer 4 type-change) dies from a sac-self cost, base read says "not creature" and the `CreatureDied` event is skipped — "whenever a creature dies" triggers fail to fire. Not specific to granted abilities but on the activation-path. Fix: read from `calculate_characteristics`.

**PB-S-L04** — `crates/engine/src/rules/abilities.rs:685-695`. Same pattern as L03 for sacrifice-another-permanent cost. Animated creature sacrificed as a cost emits wrong event. Fix: read from calculated chars.

**PB-S-L05** — `crates/engine/src/rules/abilities.rs:519`. `get_self_activated_reduction(card_def, ability_index)` keys cost reductions by the card definition's native ability-index. For a granted ability (index beyond native range), returns `None` → no reduction applied. Correct-by-accident for current grants (none have card-def-specific cost reductions). Latent if a card-def author adds an indexed cost reduction that collides with a granted ability's index. Fix: either key reductions by stable identifier instead of numeric index, or document that grants always append past the native range.

**Missing test: Humility later than grant preserves grant** (PB-S-L06 / was L1 in PB-S review)

| Aspect | Detail |
|--------|--------|
| **Issue** | The PB-S test suite includes `test_humility_removes_granted_mana_ability` which asserts that a Humility timestamp LATER than Cryptolith Rite wipes the grant. The inverse case (Humility EARLIER than Cryptolith Rite → Humility strips base abilities, then Rite adds the grant on top, grant survives) is not tested. |
| **Related** | PB-S review, CR 613.1f layer ordering |
| **Risk** | Very low — the current timestamp-based layer ordering is well-exercised elsewhere. This is a coverage gap, not a known-broken path. |
| **Fix** | Add `test_humility_before_grant_preserves_grant` to `crates/engine/tests/grant_activated_ability.rs`. ~20 LOC. |

### Simulator mana_solver reads base characteristics (PB-S-L01)

| Aspect | Detail |
|--------|--------|
| **File** | `crates/simulator/src/mana_solver.rs:35` |
| **Issue** | `mana_solver` reads `obj.characteristics.mana_abilities` (base) instead of `calculate_characteristics(state, id).mana_abilities`. Granted mana abilities from Layer 6 (Cryptolith Rite, Chromatic Lantern, Citanul Hierophants, Paradise Mantle, Enduring Vitality) are invisible to the bot mana planner. |
| **Impact** | Bots undervalue mana sources granted by Layer 6 effects. E.g., a creature under Cryptolith Rite will not be counted as a mana source when the bot plans payment. Bot plays suboptimally — NOT a correctness issue; real game rules still enforce the granted abilities through `handle_tap_for_mana` (which PB-S fixed to read calculated chars). |
| **Risk** | Low. `calculate_characteristics` is already called from `legal_actions.rs` after PB-S; adding it to `mana_solver` uses the same pattern. Main risk is cost: `calculate_characteristics` allocates, and the mana solver iterates every mana source per payment plan. May need caching if hot. |
| **Related** | PB-S (surfaced this), W3-LC (original layer-correctness audit closed 2026-03-19, missed `mana_solver` because it's in the simulator crate, not engine) |
| **Prerequisite** | None. Bench before/after if the solver is in a hot path during simulator runs. |
| **When** | Opportunistic. Whenever someone touches `mana_solver.rs`, or when bot behavior in commander playtesting reveals the gap. Not urgent for pre-alpha since bot quality is not blocking path-to-playable. |

### Server bind address (MR-M9.5-06)

| Aspect | Detail |
|--------|--------|
| **Issue** | Replay viewer binds to `0.0.0.0` instead of `127.0.0.1`. |
| **Risk** | Minimal code risk, but could break the `--host 0.0.0.0` workflow documented in MEMORY.md if the default changes. |
| **When** | When doing any replay-viewer work. Add a `--bind` CLI flag defaulting to `127.0.0.1`, keeping `--host` as an alias for the old behavior. |

### apply_combat_damage refactoring (MR-M6-06)

| Aspect | Detail |
|--------|--------|
| **Issue** | `apply_combat_damage` is 312 lines. Could be split into helper functions. |
| **Risk** | Refactoring combat damage is high-risk. The function handles first strike, double strike, trample, deathtouch, lifelink, and infect in interleaving phases. Extracting helpers could subtly change evaluation order. |
| **Prerequisite** | Write the combat game scripts (MR-M6-08) FIRST. Get golden test coverage for all combat keyword combinations. Then refactor with the scripts as a safety net. |
| **When** | Only when combat needs new features (e.g., new combat keywords) and the complexity makes the current function unmaintainable. Not worth doing as a standalone cleanup. |

---

## Tier 4 — Architectural (new types, cross-cutting changes)

These require design decisions that affect multiple modules. **Do not implement
opportunistically.** Each should be a deliberate decision with a plan.

### `is_copy` field on GameObject (MR-M4-07)

| Aspect | Detail |
|--------|--------|
| **Issue** | CR 704.5e: "If a copy of a spell is in a zone other than the stack, it ceases to exist." No `is_copy` field exists. |
| **Current state** | Copy effects exist (storm, cascade) but copies are only on the stack. When they resolve, they go to graveyard as normal objects and persist incorrectly. |
| **Risk** | Adding `is_copy` to `GameObject` touches state serialization, hashing, builder, zone-change logic, and SBA checks. Wide blast radius. |
| **When** | When copy effects become testable with real game scripts that exercise the stack-to-graveyard transition. Currently copies are exercised in storm/cascade tests but the post-resolution SBA is not checked. |
| **Approach** | Add the field, default to `false`, set it when creating copies in `copy.rs`. Add SBA check. Write test for copy ceasing to exist in graveyard. |

### `AddSupertypes`/`RemoveSupertypes` layer modifications (MR-M5-07)

| Aspect | Detail |
|--------|--------|
| **Issue** | No way to add individual supertypes via continuous effects (e.g., "becomes legendary"). |
| **When** | Only when a card that adds supertypes is needed in the card pool. Very few Commander-relevant cards do this. |

### World rule SBA (MR-M4-08)

| Aspect | Detail |
|--------|--------|
| **Issue** | CR 704.5k (world rule) not implemented. |
| **When** | Indefinitely deferred. The `World` supertype appears on ~30 cards from pre-2000 sets, none of which are Commander staples. |

---

## Implementation Schedule

Based on the project state (M9.5 complete, M10 networking layer ahead), here is the
recommended order of operations:

### Phase 1 — Immediate (before M10 work begins) — **DONE 2026-03-03**

**Goal**: Expand test coverage without touching runtime code. Zero regression risk.

1. ~~Write all Tier 1 tests (10 new tests + 4 test improvements)~~ ✓
2. ~~Delete dead code (MR-M1-06, MR-M1-14, MR-M9.5-08)~~ ✓
3. ~~Add comments (MR-M9.4-11)~~ ✓
4. ~~Run full suite: `cargo test --all && cargo clippy -- -D warnings`~~ ✓
5. Committed as part of W3 T1+T2 pass.

**Result**: 19 LOW issues closed. 1421 tests passing.

### Phase 2 — Early M10 (while setting up network crate) — **DONE 2026-03-03**

**Goal**: Harden defensive checks. These are the `debug_assert!` and `unwrap_or` → `.ok_or()`
changes that make bugs loud instead of silent.

1. ~~Add all `debug_assert!` additions (MR-M1-16, MR-M1-17)~~ ✓
2. ~~Fix silent defaults (MR-M2-09, MR-M3-11, MR-M9-17)~~ ✓
3. ~~Fix error name (MR-M3-12)~~ ✓
4. ~~Fix planeswalker loyalty default (MR-M4-11)~~ ✓
5. ~~Apply performance micro-optimizations (MR-M9.4-09, MR-M4-10, MR-M5-06)~~ ✓
6. ~~Sort replay-viewer player keys (MR-M9.5-11)~~ ✓
7. ~~Run full suite after each individual change~~ ✓
8. Committed as `W3: apply T1+T2 LOW remediation` (commit `08c7b32`).

**Result**: 11 LOW issues closed. 5 real targeting.rs bugs found and fixed.
**Note**: MR-M2-09 and MR-M3-11 used `debug_assert!` instead of `.ok_or()?` — the functions
return `Vec<_>` not `Result`, so the approach was adjusted. Same protective effect.

### Phase 3 — Mid-M10 (with networking context)

**Goal**: ManaPool encapsulation. The network layer needs a clean mana API.

1. Write `ManaPool` unit tests (addresses MR-M1-07)
2. Add `ManaPool::spend()` method
3. Migrate callers one at a time with tests after each
4. Commit: `refactor: encapsulate mana spending in ManaPool::spend()`

**Estimated effort**: 2-3 hours.
**Regression risk**: Medium. Requires careful call-site migration. The prerequisite tests
are the safety net.

### Phase 4 — Opportunistic (no deadline)

Address when touching the relevant subsystem for other reasons:

| Issue | Trigger |
|-------|---------|
| MR-B9-01 (generic CardDef upkeep triggers) | Before scripting any B10+ card that has an upkeep, draw-step, or combat trigger in its CardDefinition. Fix: add a general CardDef trigger sweep in `upkeep_actions()` after the keyword-specific block. See `docs/mtg-engine-milestone-reviews.md` W1-B9 section for full description. |
| MR-M6-06 (combat refactor) | Next time combat needs new keyword support |
| MR-M8-14 (Colossus shuffle) | Next "shuffle into library" card |
| MR-M9.5-06 (server bind) | Next replay-viewer feature work |
| MR-M9.5-07 (blocking I/O) | If replay-viewer becomes multi-user |
| MR-M4-07 (`is_copy`) | When copy effects get game-script coverage |
| Card-db schema fixes (MR-M0-08, M0-09, M0-10, M0-15, M0-16) | When card-pipeline is actively used for deck building |
| MR-M9.4-12 (loop_detection mutability) | Next loop-detection work |

### Permanently deferred

| Issue | Reason |
|-------|--------|
| MR-M4-08 (world rule) | ~30 cards from pre-2000 sets, zero Commander relevance |
| MR-M5-07 (AddSupertypes) | Wait for a card that needs it |
| MR-M1-18 (zone O(n) contains) | Not a bottleneck — profiling would need to show otherwise |
| MR-M6-14 (blockers_for rebuild) | ≤10 blockers in practice, negligible |
| MR-M9.5-13 (PlayerId as cast) | Consistent with M1, not a real risk |
| MR-M9.4-10 (linear keyword scan) | <20 keywords per object, OrdSet range query would be more complex than the savings |

---

## Summary

| Tier | Issues | Risk | Status |
|------|--------|------|--------|
| T1 — Zero risk | 28 | None | **DONE** (2026-03-03) — 19 unique IDs closed |
| T2 — Cosmetic | 17 | Low | **DONE** (2026-03-03) — 11 unique IDs closed |
| T3 — Behavioral | 4 | Medium | Pending — ManaPool::spend() is next |
| T4 — Architectural | 3 | High | Pending — deliberate planning required |
| Permanently deferred | 6 | — | Never |
| Opportunistic | 10 | Varies | Address when touching subsystem |

**Current status (2026-03-03)**: 30 issues closed by W3 T1+T2 pass (commit `08c7b32`).
**LOW OPEN**: 39 (was 68) — 17 pre-M8 + 5 M8 + 6 M9 + 2 M9.4 + 1 CKP + 7 M9.5 + 1 W3
**LOW CLOSED**: 36 (was 6) — 30 new closures + 6 pre-existing
**Bonus**: 5 real targeting.rs bugs found by debug_assert. 1 new LOW (MR-W3-01) found.
**Next**: T3 (ManaPool::spend) — defer until M10 networking context; T4 opportunistic.

---

## P1 Ability Sanity Reviews (2026-03-09)

Findings from post-Morph sanity reviews of early P1 abilities. HIGHs and MEDIUMs were fixed immediately. LOWs deferred here.

### Trample (combat.rs)

| ID | Severity | Description | Location |
|----|----------|-------------|----------|
| SR-TRM-01 | LOW | Planeswalker combat damage marks damage on PW object instead of removing loyalty counters (CR 120.3c). Pre-existing, not trample-specific. | `rules/combat.rs:1527-1531` |
| SR-TRM-02 | LOW | Dead code removed by the `blocked_attackers` fix — old `is_blocked()` scan branch should be cleaned up if any residual dead branches remain. | `rules/combat.rs` |

### Protection (protection.rs / casting.rs)

| ID | Severity | Description | Location |
|----|----------|-------------|----------|
| SR-PRO-01 | LOW | `ProtectionQuality` missing `FromSuperType` and `FromName` variants (e.g. "protection from Wizards", "protection from Nicol Bolas"). No current cards need these. | `state/types.rs` |
| SR-PRO-02 | LOW | No `FromPlayer` variant for CR 702.16k (protection from a player — rare but rules-legal). | `state/types.rs` |
| SR-PRO-03 | LOW | No test for protection vs. multicolor source (source must share *any* color). | `tests/protection.rs` |
| SR-PRO-04 | LOW | No test for subtype-based protection (e.g. "protection from Goblins"). | `tests/protection.rs` |

### First Strike / Double Strike (combat.rs)

| ID | Severity | Description | Location |
|----|----------|-------------|----------|
| SR-FS-01 | LOW | `first_strike_damage_resolved` field on `CombatState` is written but never read — dead field left over from an incomplete snapshot plan. | `state/combat.rs` |
| SR-FS-02 | LOW | No test for a creature gaining first strike between the two combat damage steps (CR 702.7c). Low impact today — no cards trigger this — but structural gap. | `tests/combat.rs` |
| SR-FS-03 | LOW | No test for first-strike attacker vs. first-strike blocker (both deal damage simultaneously in first-strike step; neither should appear in regular step). | `tests/combat.rs` |

### PB-Q4 (Enchant target filter) — 2026-04-12

| ID | Severity | Description | Location |
|----|----------|-------------|----------|
| PB-Q4-M01 | MEDIUM | `EnchantFilter` (6 fields) duplicates the enchant-relevant subset of `TargetFilter` (24 fields). The two will diverge over time. Root cause: `cards/card_definition.rs` imports from `state::*` so `state/types.rs::EnchantTarget` cannot reference `TargetFilter` without a cycle. Fix options: (a) relocate `TargetFilter` to `state/`, then collapse `EnchantFilter` into `Filtered(Box<TargetFilter>)`; (b) document the 18 non-supported `TargetFilter` fields on `EnchantFilter` as deliberate. Decide when authoring the next non-land enchant target. | `state/types.rs:286`, `cards/card_definition.rs` |
| PB-Q4-L01 | LOW | `matches_enchant_target` defensive `.unwrap_or(aura_ctrl)` masks regressions if a target object lookup ever returns `None`. Replace with explicit error or `debug_assert!`. | `rules/sba.rs:1067-1071` |
