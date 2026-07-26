# Primitive Batch Review: PB-RS4 — Face-Aware Residuals (close the 3 surviving CR 712.8d/e deviations)

<!-- last_updated: 2026-07-26 -->

**Date**: 2026-07-26
**Reviewer**: primitive-impl-reviewer (Opus)
**Task**: `scutemob-146` · Branch `feat/pb-rs4-face-aware-residuals-close-the-3-surviving-cr-7128de-`
**CR Rules verified via MCP**: 712.8 (a–g), 712.18, 614.12 (+a/b/c), 604.1, 603.2 (a–h), 613.4 (a–d),
714.2 (a–e), 714.3 (a/b), 601.3 (a–f), 305.1, 305.2, 614.16
**Engine files reviewed**: `crates/engine/src/rules/face.rs`, `crates/engine/src/rules/replacement.rs`,
`crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/state/ability_definition_registry.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_rs4_face_aware_residuals.rs` (17 tests),
`crates/engine/tests/core/face_dereg_parity.rs` (2 tests)
**Card defs reviewed**: 0 modified (verified); 15-file DFC roster cross-checked for behavioral flips
**Gates**: not re-run (independently verified green per task brief); PROTOCOL 27 / HASH 63 confirmed
unchanged by inspection (`rules/protocol.rs`, `state/hash.rs` untouched; no `Effect`/`Command`/
`GameEvent`/`AbilityDefinition` shape change).

## Verdict: needs-fix (documentation + test-coverage only — **the correctness core is clean**)

**The substantive engine work is correct.** I walked `remove_one_registration`'s ten arms field-by-field
against `register_static_continuous_effects`'s ten arms and found **zero** predicate mismatches: every
compared field is written identically by the registration side, the `Option<ObjectId>` vs `ObjectId`
split is handled correctly per family (0/4/5/7 use `Some(obj_id)`, the rest bare), the
`EffectFilter::Source -> SingleObject` resolution is applied to exactly the `Static` arm (the two CDA
arms register `SingleObject(new_id)` directly and are compared as such), the `CdaPowerToughness`
`SetPtDynamic` boxing matches, the `CdaModifyPowerToughness` `negate: false` / power-then-toughness
ordering matches, `condition.as_ref().map(|c| *c.clone())` reproduces the registration expression
verbatim, and per-family entry counts are exactly 1 (or 0/1/2 for `CdaModifyPowerToughness`). The
`_ => {}` catch-all is guarded by a parity gate that I traced end-to-end and confirmed **would** fail
on a new registration arm (`registered.difference(&deregistered)` → `assert!(...is_empty())`), not
merely exists. The live-read face signal is safe: `is_transformed` is written in exactly two engine
places (`resolution.rs:665`, `face.rs:92`), both strictly before every consumer, and I verified the two
enter-transformed call sites directly (disturb `:665` → `:1673`; stack craft `apply_face_change` `:7276`
→ `:7279`). `fizzle_object` is the correct SR-4 side-taking and adds zero bare lookups. Producer/consumer
index parity for `fire_saga_chapter_triggers` holds against **every** consumer in the tree (eight sites,
not the three the doc names — all use `effective_abilities(obj.is_transformed)`). Deviation #4 was
legitimately in scope, the `starting_loyalty` fence held, and OOS-RS3-1 / OOS-RS2-1 are untouched. Both
new seeds are accurate; I independently confirmed OOS-RS4-2's four-`Complete`-MDFC-lands claim and
OOS-RS4-1's missing-call-site claim.

Findings are one MEDIUM test-coverage gap and eight LOWs (mostly CR-citation / doc-count accuracy).
No HIGH. Nothing here blocks the merge on correctness grounds.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `rules/face.rs:201` | **CR 614.16a does not exist.** Miscitation copied from the registration side into the new deregistration arm and the new test. **Fix:** replace with CR 604.1 + 603.2 (a static ability generating a continuous effect that stops ETB abilities triggering); optionally correct `replacement.rs:2158` in the same pass. |
| 2 | LOW | `rules/face.rs:13-17` | **"the nine sibling collections" lists seven.** Nine is the *family* count, not the collection count. **Fix:** reword to "the seven sibling collections" (or "the nine sibling families across seven collections"). |
| 3 | LOW | `rules/replacement.rs:1291-1296` | **`fire_saga_chapter_triggers`'s new contract doc under-enumerates the consumers** ("the three consumers ... `resolution.rs:1996`/`:2028`, `sba.rs:889`). There are eight index-into-CardDef consumers. **Fix:** say "every consumer resolves the index against `effective_abilities(obj.is_transformed)`" and list the SagaChapter-specific ones as examples rather than as the complete set. |
| 4 | LOW | `rules/replacement.rs:1291-1296` (contract, not code) | **Residual "is_transformed at consume time" hazard survives PB-RS4** (pre-existing, PB-OS4b-established, not introduced here). **Fix:** file a seed; do not fix in RS4. |
| 5 | LOW | `rules/face.rs:173-187`, `:287-295` | **Two removal predicates are narrower than the registration write.** `Static` omits `is_cda` and `condition`; `StaticFlashGrant` omits `duration`. Both are unreachable-to-mismatch on today's producers. **Fix:** add `&& !e.is_cda && e.condition == continuous_effect.condition` to the `Static` arm and `&& f.duration == EffectDuration::WhileSourceOnBattlefield` to the flash arm — cheap, and makes the arms literal inverses. |
| 6 | LOW | `tests/core/face_dereg_parity.rs:44-70` | **Parity gate is brace-matched over un-stripped source** and cannot see a family moved into a registration helper, nor an arm that merely names a family without removing anything. **Fix:** strip comments *before* brace-matching, and note the two blind spots in the module doc. |
| 7 | LOW | `rules/face.rs:322` | `pm.on_cast_effect == on_cast_effect.clone()` allocates a clone per comparison inside a `position()` scan. **Fix:** compare `pm.on_cast_effect == *on_cast_effect`. |
| 8 | LOW | `rules/replacement.rs:1178-1181` vs `:1240`/`:1260`/`:1278` | **SR-4 classification is internally inconsistent inside one function.** The new top-of-function read is a `fizzle_object`; three later reads at strictly *less* certain points are `expect_object_mut`. **Fix:** none required for RS4 (`fizzle_object` is the correct choice); optionally note the asymmetry or downgrade the later reads in a follow-up. |

## Test Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 9 | **MEDIUM** | `tests/primitives/pb_rs4_face_aware_residuals.rs:955-1041` (and the other family probes) | **Every per-family probe uses degenerate values for the fields the removal predicate compares.** A future edit that drops or wrongly compares `condition` / `on_cast_effect` / `look_at_top` / `reveal_top` / `pay_life_instead` would not be caught. **Fix:** give `test_transform_deregisters_play_from_top` non-default values and `test_transform_deregisters_play_from_graveyard` a `Some(condition)`. |
| 10 | LOW | `:542-578` | `test_saga_chapter_trigger_index_matches_effective_face` proves *face-awareness*, not *index parity*. **Fix:** add a fixture whose back face has `SagaChapter` at a different position than the front and assert the produced `ability_index` resolves to the back face's chapter. |
| 11 | LOW | `:800-807` | Assertion message claims "back face's printed power is 2", but `apply_face_change` never rewrites base P/T. **Fix:** reword to "the object's base 2/2 is restored once the CDA is deregistered". |
| 12 | LOW | `:465-468` | `assert!(state.stack_objects().is_empty())` is a weak/possibly-vacuous secondary assertion. **Fix:** assert on `state.pending_triggers()` (filtered to `source == fable_id`) instead of / in addition to the stack. |

---

## Finding Details

### Finding 1: CR 614.16a does not exist

**Severity**: LOW
**File**: `crates/engine/src/rules/face.rs:201` (also `tests/primitives/pb_rs4_face_aware_residuals.rs:694,726`)
**CR check**: `get_rule("614.16a")` → *"Rule '614.16a' not found."* `get_rule("614.16")` returns the
token/counter-creation replacement rule, which has nothing to do with ETB-trigger suppression.
**Issue**: The new `SuppressCreatureETBTriggers` deregistration arm is annotated
`// CR 614.16a: a Torpor Orb-style ETB trigger suppressor.` The citation was copied from the
registration side (`replacement.rs:2158`, pre-existing) and is now in three more places. Per Invariant #8
("tests cite their rules source"), a citation that resolves to nothing is worse than none.
**Failure scenario**: a future maintainer follows the citation, finds an unrelated rule about token
replacement effects, and reasons wrongly about the suppressor's semantics.
**Fix**: replace all four occurrences with `CR 604.1 / 603.2` (Torpor Orb is a static ability generating
a continuous effect; there is no dedicated subrule) and fix `replacement.rs:2158` while you are there.

### Finding 4: residual index-namespace hazard (contract-level, pre-existing)

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1291-1326` (producer);
`resolution.rs:1996`/`:2028`/`:2066`, `sba.rs:889`, `abilities.rs:7004`/`:7082`/`:7210`/`:8379` (consumers)
**CR Rule**: 113.7a — an activated or triggered ability on the stack "exists independently of its source";
712.18 — a transform does not create a new object.
**Issue**: PB-RS4 correctly aligns the producer to the PB-OS4b "`is_transformed` **at consume time**"
contract, so producer and consumer now agree at queue time. But the contract itself re-derives the index
against whatever face is up *at resolution*. If a DFC Saga transforms in place between the chapter
trigger being queued and it resolving, the consumer indexes the other face's list — resolving to the
wrong `SagaChapter` (or to `None`, silently doing nothing). Per CR 113.7a the queued ability should keep
its own effect regardless of what the source does afterwards.
**Reachability today**: none. It requires a DFC Saga *plus* an in-place `TransformSelf`-class flip while
a chapter trigger is pending. `fable_of_the_mirror_breaker` is the only roster Saga DFC and it is
`Completeness::partial` (rejected by `validate_deck`); its own chapter III uses
`ExileSourceAndReturnTransformed`, which is a CR 400.7 new object and therefore fizzles the pending
trigger rather than mis-indexing it.
**Not introduced by RS4** — the contract predates it, and pre-RS4 the producer disagreed with the
consumers *unconditionally* for a transformed permanent, which was strictly worse.
**Fix**: file a seed (e.g. OOS-RS4-4) proposing that `PendingTrigger`/`StackObjectKind::TriggeredAbility`
carry the resolved face (or the resolved `Effect`) so the index namespace is frozen at queue time per
CR 113.7a. Do not fix in RS4.

### Finding 5: two removal predicates are narrower than the registration write

**Severity**: LOW
**File**: `crates/engine/src/rules/face.rs:173-187` (`Static`), `:287-295` (`StaticFlashGrant`)
**Issue**: Registration writes `is_cda: false` and `condition: continuous_effect.condition.clone()` for
`Static` (`replacement.rs:2132-2142`) and `duration: WhileSourceOnBattlefield` for `StaticFlashGrant`
(`:2283-2291`); neither field is compared on removal.
**Failure scenario (constructed, not currently reachable)**: a face declaring both a
`Static { layer: PtCda, modification: SetPtDynamic{..} }` and a `CdaPowerToughness` with the same
amounts. The `Static` arm runs first and, ignoring `is_cda`, removes the *CDA* entry; the
`CdaPowerToughness` arm then finds nothing (it requires `is_cda`), and the non-CDA `Static` entry
survives the transform — exactly the stale-registration class this PB exists to kill. No roster card
constructs this, and the `Static` arm's shape is pre-existing PB-OS4b code, so this is LOW.
**Fix**: add `&& !e.is_cda && e.condition == continuous_effect.condition` to the `Static` arm and
`&& f.duration == crate::state::continuous_effect::EffectDuration::WhileSourceOnBattlefield` to the
flash-grant arm.

### Finding 9: per-family probes cannot catch a field-comparison regression

**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_rs4_face_aware_residuals.rs:955-1041`, and the same pattern
in tests 7/8/9/12a/12b/12c
**Issue**: `test_transform_deregisters_play_from_top` registers
`{ filter: All, look_at_top: false, reveal_top: false, pay_life_instead: false, condition: None,
on_cast_effect: None }` and `test_transform_deregisters_play_from_graveyard` registers
`{ filter: All, condition: None }`. Because every discriminating field is at its default, the tests pass
whether or not `remove_one_registration` compares those fields at all. I verified by inspection that all
of them *are* compared correctly today, so this is a coverage gap and not a live defect — but the
highest-risk part of this PB (field-by-field predicate fidelity) has no automated guard against a
future edit. The `face_dereg_parity` gate checks family *names* only, not predicate shape.
**Failure scenario**: someone simplifies the `StaticPlayFromTop` arm to compare only `source` and
`filter`. Suite stays green. A card with two `StaticPlayFromTop` abilities differing only in
`pay_life_instead` (Bolas's Citadel vs Future Sight shapes on two faces) then leaks a registration
across a transform.
**Fix**: change the `StaticPlayFromTop` fixture to `look_at_top: true, reveal_top: true,
pay_life_instead: true, condition: Some(<some Condition>), on_cast_effect: Some(<some Effect>)`, and the
`StaticPlayFromGraveyard` fixture to `condition: Some(...)`. Optionally add one negative case: register
two same-family entries differing only in one compared field, deregister one, assert the other survives.

---

## Priority-Question Answers (as briefed)

### 1. `remove_one_registration` arm-by-arm fidelity — CLEAN

| # | Family | Reg. site | Compared fields on removal | Verdict |
|---|--------|-----------|----------------------------|---------|
| 0 | `Static` | `replacement.rs:2120-2143` | `source==Some(obj_id)`, `layer`, `duration`, `modification`, resolved `filter` | match; `is_cda`/`condition` omitted (Finding 5) |
| 1 | `TriggerDoubling` | `:2145-2157` | `source==obj_id`, `filter`, `additional_triggers` | exact (`controller` derived, correctly not compared) |
| 2 | `SuppressCreatureETBTriggers` | `:2159-2166` | `source`, `filter` | exact (all fields) |
| 3 | `StaticRestriction` | `:2168-2176` | `source`, `restriction` | exact (`controller` derived) |
| 4 | `CdaPowerToughness` | `:2180-2202` | `source==Some`, `is_cda`, `layer==PtCda`, `duration`, `filter==SingleObject(obj_id)`, `modification==SetPtDynamic{Box::new(power.clone()),Box::new(toughness.clone())}` | exact — boxing and field order identical |
| 5 | `CdaModifyPowerToughness` | `:2218-2267` | same set, `layer==PtModify`, one removal per element of an identically-built `modifications` vec (`ModifyPowerDynamic{negate:false}` then `ModifyToughnessDynamic{negate:false}`) | exact — 0/1/2 entries, correct `negate`, correct ordering |
| 6 | `AdditionalLandPlays` | `:2271-2279` | `source`, `count` | exact (`controller` derived) |
| 7 | `StaticFlashGrant` | `:2282-2292` | `source==Some(obj_id)`, `filter` | `duration` omitted (Finding 5) |
| 8 | `StaticPlayFromGraveyard` | `:2294-2303` | `source`, `filter`, `condition.as_ref().map(|c| *c.clone())` | exact — condition expression is verbatim-identical to registration |
| 9 | `StaticPlayFromTop` | `:2305-2325` | `source`, `filter`, `look_at_top`, `reveal_top`, `pay_life_instead`, `condition`, `on_cast_effect` | exact (all fields) |

Source-field types line up per family (`Option<ObjectId>` for 0/4/5/7, bare `ObjectId` for 1/2/3/6/8/9).
`EffectFilter::Source -> SingleObject(obj_id)` is applied to the `Static` arm only — correct, because the
CDA arms register `SingleObject(new_id)` unconditionally and the removal compares that literal.
No `retain`-by-source purge anywhere; every arm is `position()` + `remove(pos)`.

**Catch-all coverage — gate verified, not assumed.** `face_dereg_parity.rs:111-134` builds
`BTreeSet<String>` from both bodies and asserts `registered.difference(&deregistered).is_empty()`. A new
`AbilityDefinition::Foo` arm added to `register_static_continuous_effects` puts `"Foo"` in `registered`
and not in `deregistered`, so the first `assert!` fires with a message naming the family. The non-vacuity
test additionally floors both sets at 10 and anchors on the `Static`/`StaticRestriction` prefix-collision
pair plus `CdaModifyPowerToughness`. The `use crate::cards::card_definition::AbilityDefinition;` inside
the registration body does **not** pollute the scan (needle requires a trailing `::`). Blind spots are
Finding 6.

### 2. Live-read face signal — SAFE

`is_transformed` is written by the engine in exactly two places (grep over `crates/` for
`is_transformed\s*[:=]\s*(true|new_is_transformed|!)` returns `resolution.rs:665` and `face.rs:92` only;
every other hit is a test or a doc comment). Both are strictly before the RS4 reads:

- **Disturb** — `resolution.rs:665` sets `is_transformed = true` inside the stack-object transfer block;
  `apply_self_etb_from_definition` is at `:1673` and `register_permanent_replacement_abilities` at
  `:1688`, ~1000 lines later in the same match arm. `apply_etb_replacements` at `:1682` does not touch
  the flag. The runner's Step-0 `eprintln!` probe (`Some(true)`) corroborates.
- **Stack craft** — `apply_face_change(state, new_id, true)` at `resolution.rs:7276`, three lines before
  `apply_self_etb_from_definition` at `:7279`.
- **`move_object_to_zone`** resets the flag to `false` (`state/mod.rs:1397-1399`, CR 712.8a/400.7). Every
  other call site therefore correctly reads `false`. Confirmed for `handle_craft` (`engine.rs:1432-1453`)
  and `ExileSourceAndReturnTransformed` (`effects/mod.rs:4321-4337`) — both call `apply_face_change`
  *before* any ETB-chain call they make (and neither calls the two RS4 functions at all — that is
  OOS-RS4-1, correctly seeded rather than widened into).
- **No "enters transformed" replacement modification exists** (`ReplacementModification` has 25 variants;
  none flips a face), so the single top-of-function snapshot cannot go stale relative to
  `fire_saga_chapter_triggers`'s independent live read later in the same function.
- **MDFC back-face play is genuinely unreachable** — `grep -c 'ModalDfc|modal_dfc|MDFC|Mdfc'` over
  `crates/engine/src` = **0**. So `lands.rs`'s `false` is not a latent wrong read.

**SR-4 / SR-25**: `fizzle_object` (`state/diagnostics.rs:373`) is the documented "a `None` here is legal
game state" lookup (CR 400.7 / 608.2b / 113.7a) and is the right side to take — a permanent that an
earlier same-batch ETB replacement removed genuinely has no face, and `expect_object`'s `debug_assert!`
would panic the ~16 direct `apply_self_etb_from_definition` test callers. It is not a bare
`.objects.get(`, so the pinned `bare_lookup_ratchet` ceilings for `replacement.rs` and `turn_actions.rs`
are untouched; `turn_actions.rs`'s Saga sweep adds **no** lookup at all (it reuses the closure's bound
`obj`). Minor internal inconsistency noted as Finding 8.

### 3. `fire_saga_chapter_triggers` index parity — HOLDS; consumer set is larger than the plan claimed

Every site in the engine that resolves a CardDef ability index against a `CardDefinition` uses
`effective_abilities(obj.is_transformed)`:

| Consumer | Purpose |
|---|---|
| `resolution.rs:1996` | non-CardDefETB registry fallback → SagaChapter effect |
| `resolution.rs:2028` | CardDefETB path → SagaChapter effect |
| `resolution.rs:2066` | modal-trigger `modes` lookup |
| `sba.rs:889` | CR 714.4 "chapter still on the stack" guard |
| `abilities.rs:7004` | `once_per_turn` registry fallback |
| `abilities.rs:7082` | `has_ability_targets` (CardDefETB branch) |
| `abilities.rs:7210` | `ability_targets` (CardDefETB branch) |
| `abilities.rs:8379` | flush-time ability lookup |

So there is no fourth consumer on a *different* convention — the producer is now in the same namespace
as all eight. The plan's "three consumers" is an undercount of the enumeration, not a correctness error
(Finding 3). The `sba.rs:843-853` "not a Saga if no chapter abilities" derivation is CR-correct against
714.2d + 714.4 ("each Saga ... **with one or more chapter abilities**"), and `turn_actions.rs:380-386`
now matches it — which was the disagreement RS4 set out to close.

**Adversarial case (transform between queue and resolution)**: reachable in principle, unreachable on
today's roster, and pre-existing. Written up as Finding 4 with a seed recommendation.

**One pre-existing namespace wart, out of scope**: for `PendingTriggerKind::Normal` triggers,
`abilities.rs:6988-6995` and `:7070-7074` consult `characteristics.triggered_abilities[ability_index]`
*first*, and a Saga chapter's index is into the CardDef effective list, not that runtime vector. Today
`build_face_ability_vectors` does not lower `SagaChapter` into `triggered_abilities`, so a pure Saga
falls through to the correct registry path; a card mixing `SagaChapter` and `Triggered` abilities could
read the wrong `once_per_turn`/`targets`. Predates PB-OS4b, untouched by RS4, no roster instance. Not
filed as an RS4 finding.

### 4. Scope discipline — HELD; deviation #4 was legitimately in scope

- **Deviation #4 is not scope creep.** Part 2 (`fire_saga_chapter_triggers`) is called from
  `apply_self_etb_from_definition`'s own body (`replacement.rs:1266`), immediately after the
  `has_saga_chapters` scan that deviation #1 necessarily makes face-aware — leaving the producer
  front-indexed while its caller became back-aware would have manufactured a *new* mismatch inside the
  very function this PB fixes. Part 1 (`turn_actions.rs:380-386`) is a different file, but it is the same
  CR (714.3b + 712.8e), the same one-expression mechanism (`effective_abilities`), and it was already in
  disagreement with `sba.rs:843` — shipping part 2 without part 1 would produce a transformed Fable that
  accrues lore counters forever while producing no chapter triggers, a *worse* state than before. The
  plan flagged it, argued it, and bounded it to exactly two expressions (risk item 8). Correct call.
- **`starting_loyalty` fence held.** `replacement.rs:1237` still reads `def.starting_loyalty` with an
  explicit `// CR 306.5b: back-face starting loyalty is OOS-OS4-1 / rider-seed queue item R10 --
  deliberately front-only here (PB-RS4 does not widen into it).` pointer at `:1235-1236`. No widening.
- **OOS-RS3-1 untouched** — `precombat_main_actions` (`turn_actions.rs:363-419`) gained the face-aware
  filter and nothing else; no `check_intervening_if` was added to any `CardDefETB` sweep.
- **OOS-RS2-1 untouched** — `rules/engine.rs` is not in the change set; `handle_turn_face_up`'s raw
  `def.mana_cost` payment is unchanged.
- **`ability_definition_registry.rs`**: all nine families gained `"crates/engine/src/rules/face.rs"`
  (`:122-127`, `:128-133`, `:357-362`, `:363-368`, `:369-374`, `:375-381`, `:382-387`, `:388-393`,
  `:394-399`); `A::Static` already listed it (`:92-98`); `A::SagaChapter` (`:160-167`) correctly needed
  no change. No spurious additions.

### 5. Test quality — non-vacuous, with the gaps noted above

Sampled for discrimination rather than accepted from the wip record:

- **Tests 7–15 (per-family dereg)** register manually via
  `register_static_continuous_effects(.., false)`, assert presence (sanity), run `Effect::TransformSelf`
  → `transform_permanent_in_place` (`engine.rs:1216-1229`) → `apply_face_change`, assert absence. The
  back face in `mock_family_def` carries no abilities, so nothing is re-registered to mask the removal.
  Pre-fix `deregister_face_statics` handled only `Static`, so every one of these genuinely goes RED.
  The wip's recorded messages match the assertion strings in the file verbatim.
- **Tests 10/11** assert *behaviorally* through `calculate_characteristics` (5/5 → 2/2 and 5/4 → 2/2) in
  addition to a collection-level count. Good — this is the full-dispatch assertion `conventions.md` asks
  for.
- **Test 13 (there-and-back)** correctly counts 9 (not 10 — `CdaModifyPowerToughness { power: Some,
  toughness: None }` contributes exactly one entry) and asserts membership counts, not timestamps or
  `EffectId`s, respecting plan risk item 10.
- **Test 5** uses the shipped `fable_of_the_mirror_breaker` def; the lore-counter equality is the real
  discriminator (RED pre-fix at 1 vs 0). Secondary stack assertion is weak (Finding 12).
- **Test 6** genuinely discriminates (pre-fix the front's 4-entry list yields chapter 1 at index 1;
  post-fix the back's 1-entry list has no `SagaChapter`), but proves face-awareness rather than index
  parity (Finding 10).
- **Test 17** — the runner's report that it failed pre-fix *contrary to the plan's prediction* is
  correct and its post-fix pass is meaningful: pre-fix nothing is removed so 2 same-source entries
  survive; post-fix exactly the structurally-matching `count: 1` entry is removed and the injected
  `count: 5` entry plus both of `other_id`'s entries survive. That is a real assertion about
  "remove at most the registered count, first structural match", not a trivial pass. The runner's
  correction of the test's doc comment to describe actual rather than predicted behavior is the right
  handling.
- **SR-9a**: both new files are group modules (`tests/primitives/main.rs:49`,
  `tests/core/main.rs:22`); `crates/engine/tests/*.rs` contains only `no_stray_test_binaries.rs`.
- **Fixtures are test-local**: all mocks are `CardDefinition` literals inside the test file with
  `mock-rs4-*` card ids; the only real card used is the already-shipped `fable_of_the_mirror_breaker`
  via `all_cards()`. Nothing speculative was added to `crates/card-defs/`.

### 6. Seeds — both accurate and correctly scoped

- **OOS-RS4-1** (`rider-seed-triage-2026-07-19.md:93`) — **verified**. `resolution.rs:7241-7296` calls
  `apply_self_etb_from_definition` + `apply_etb_replacements` but not
  `register_permanent_replacement_abilities`; `effects/mod.rs:4273-4351` calls neither, and calls
  `queue_carddef_etb_triggers` but not the other two; `engine.rs handle_craft:1432-1453` calls none of
  the three (it does call `apply_face_change`, which the seed does not claim otherwise). Correctly
  classed "correctness, latent" — no roster `Complete` craft/exile-return card has a back face needing
  either.
- **OOS-RS4-2** (`:94`) — **verified**. All four of `bridgeworks_battle` (`:68`/`:81`/`:112`),
  `sea_gate_restoration` (`:52`/`:67`/`:98`), `revitalizing_repast` (`:56`/`:69`/`:100`) and
  `disciple_of_freyalise` (`:61`/`:76`/`:107`) are `Completeness::Complete` with a `back_face` carrying
  an `EntersTapped`/`EntersTappedUnlessPayLife(3)` self-replacement, and `grep -c 'ModalDfc|modal_dfc|
  MDFC|Mdfc'` over `crates/engine/src` = 0. The plan's charge that the PB-OS4b comment's *"No roster
  DFC/craft/disturb back face declares a `WouldEnterBattlefield` self-replacement"* was factually false
  is correct, and the replacement comment at `replacement.rs:1188-1192` no longer makes any roster claim
  at all — it states only the CR basis and the two reachable enter-transformed paths. That is the
  narrow-truth outcome the plan asked for.
- **OOS-RS4-3** (`:95`) — correctly cross-referenced to OOS-OS4-1 / R10 rather than duplicated.

---

## CR Coverage Check

| CR Rule | Verified text matches plan? | Implemented? | Tested? | Notes |
|---------|---------------------------|--------------|---------|-------|
| 712.8d | Yes | Yes | Yes | `test_disturb_front_face_*` (front-face abilities must stop applying) |
| 712.8e | Yes | Yes | Yes | `test_disturb_back_face_*`; all nine dereg probes |
| 712.8a | Yes | Yes (pre-existing) | indirectly | `move_object_to_zone` reset, `state/mod.rs:1397` |
| 712.18 | Yes | Yes | Yes | `test_transform_there_and_back_restores_all_nine_families` |
| 614.12 | Yes ("as it would exist on the battlefield") | Yes | Yes | tests 1/2 |
| 614.1c | Yes | Yes (pre-existing) | Yes | enters-tapped / enters-with-counters fixtures |
| 604.1 | Yes | Yes | Yes | dereg arms 0/1/2/3/6/7/8/9 |
| 604.3 | Yes | Yes | Yes | CDA arms 4/5, asserted through `calculate_characteristics` |
| 613.4a / 613.4c | Yes | Yes | Yes | tests 10/11 |
| 603.2d | Yes | Yes | Yes | `test_transform_deregisters_trigger_doubling` |
| 714.2b | Yes | Yes | Yes | test 6 |
| 714.2d | Yes (not cited, but relied on by `sba.rs:851`) | Yes | indirectly | transformed Fable has final chapter `None` → not a Saga |
| 714.3a | Yes | Yes | no direct probe | `has_saga_chapters` swap at `replacement.rs:1254`; no back-face-Saga fixture exists |
| 714.3b | Yes ("each Saga they control with one or more chapter abilities") | Yes | Yes | test 5 |
| 601.3 / 601.3b | Yes | Yes | Yes | flash-grant + play-permission arms |
| 305.1 / 305.2 | Yes | Yes | Yes | `test_transform_deregisters_additional_land_plays` |
| 306.5b | n/a — deliberately out of scope | No (front-only) | n/a | fenced with a pointer comment; OOS-OS4-1 / R10 |
| **614.16a** | **DOES NOT EXIST** | — | — | **Finding 1** |
| 113.7a | Yes | Partially (see Finding 4) | No | residual "consume-time" contract hazard |

---

## Card Def Summary

**0 card definitions modified.** Verified independently rather than accepted: the DFC roster is 15 files
(`grep "back_face: Some"` over `crates/card-defs/src/defs`), and scanning all 15 for the ten
`AbilityDefinition` families RS4 touches returns only `AbilityDefinition::Static` in `bloodline_keeper.rs:94`
and `docent_of_perfection.rs:101/112/123` — the one family PB-OS4b already deregistered. No roster card
declares `TriggerDoubling` / `SuppressCreatureETBTriggers` / `StaticRestriction` / `CdaPowerToughness` /
`CdaModifyPowerToughness` / `AdditionalLandPlays` / `StaticFlashGrant` / `StaticPlayFromGraveyard` /
`StaticPlayFromTop` on either face. **0 coverage flips is the correct, honest number**, and the two
"integrity repairs" (`fable_of_the_mirror_breaker` deviation #4, and the four MDFC lands documented as a
seed rather than claimed as fixed) are correctly reported as repairs, not flips —
`feedback_pb_yield_calibration` honored.

| Card | Touched | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|---------|-------------|-----------------|-------------------|-------|
| (none) | — | — | — | — | 1804 defs `Fresh`; SR-6 held (no card-def crate change) |
| `fable_of_the_mirror_breaker` | no | n/a | unchanged | **improved** | stays `partial` on its two unrelated blockers; deviation #4 repair only |
| `bridgeworks_battle`, `sea_gate_restoration`, `revitalizing_repast`, `disciple_of_freyalise` | no | yes | 0 | **half-inert** | back-face land unreachable — OOS-RS4-2, correctly seeded not silently fixed |

---

## Recommended Disposition

All twelve findings are non-blocking. Suggested handling:

1. **Fix now (cheap, in-file)**: Findings 1, 2, 3, 7, 11 — pure text/one-token edits.
2. **Fix now (small)**: Findings 5 and 9 — two extra predicate clauses and two richer test fixtures;
   together they close the only real regression-coverage hole in the batch.
3. **Fix opportunistically**: Findings 6, 10, 12.
4. **Seed, do not fix**: Finding 4 (CR 113.7a consume-time contract) — file as OOS-RS4-4 alongside
   OOS-RS4-1/-2 in `rider-seed-triage-2026-07-19.md` §1c.
5. **No action**: Finding 8.

With Findings 1–3, 5, 7, 9, 11 applied, PB-RS4 is clean and **OOS-OS4-2 can be declared fully closed**:
all three briefed deviations plus the fourth found in planning are fixed, all ten registration families
are symmetrically deregistered, a drift gate keeps them that way, and every CR claim in the change set
verifies against the Comprehensive Rules — with the single exception of the non-existent 614.16a.
