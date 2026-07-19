# Primitive Batch Review: PB-OS9 — Lieutenant / "you control your commander" condition (OOS-EF3b-1)

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 903.3 / 903.3d / 903.3e (controlling "your" commander), 603.4 (intervening-if re-check at resolution), 604.2 (conditional static abilities), 611.2a/613.7 (control layer), 702.121 (Melee), 118.9 (the "a commander" contrast trap)
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/rules/protocol.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/layers.rs`, `crates/engine/src/rules/abilities.rs`
**Card defs reviewed**: 3 (`skyhunter_strike_force.rs`, `loyal_apprentice.rs`, `siege_gang_lieutenant.rs`) + `legion_lieutenant.rs` (confirmed OUT, untouched)
**Tests reviewed**: `crates/engine/tests/primitives/pb_os9_lieutenant_commander_control.rs` (15 tests)

## Verdict: clean

Zero HIGH, zero MEDIUM. The new `Condition::YouControlYourCommander` evaluator is CR-903.3d-correct in every case checked (owned-vs-controlled, stolen, stole-back, control-opponent's-only, command-zone, phased-out, multi-commander). The wire re-pin is append-only and internally consistent (PROTOCOL 24 fingerprint pinned = history row = live const; HASH 61 discriminant 51, prior max was 50; no stale sentinels remain). The DEVIATION (keeping `loyal_apprentice` and `siege_gang_lieutenant` `partial`) is the correct engineering call — the `AtBeginningOfCombat` card-def sweep genuinely does not exist, verified independently below. `skyhunter_strike_force` is genuinely Complete. One optional LOW (a process recommendation the runner already flagged) is recorded. The batch is ship-ready; no fix cycle required.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | none | — | No engine defects found. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | none | — | No card-def defects found. |

## Process Findings

| # | Severity | Item | Description |
|---|----------|------|-------------|
| 1 | LOW | (workstream) | **File the `begin_combat()` card-def sweep seed.** The runner discovered a genuine pre-existing gap (see Deviation Verdict). It should be filed as a queue seed so `loyal_apprentice`/`siege_gang_lieutenant`/`legion_warboss`/`goblin_rabblemaster`/`mirage_phalanx`/`helm_of_the_host` can be advanced later. Correctly NOT fixed here (out of PB-OS9 scope). **Fix:** add the seed to the OOS retriage queue at close-out. |

## Verification Detail

### Change 1-2: `Condition::YouControlYourCommander` + `check_condition` arm — CORRECT

`crates/engine/src/effects/mod.rs:9304-9319`. The arm reads `state.expect_player(ctx.controller)` (SR-4 compliant, an improvement over the plan's bare `.get`), then `state.objects.values().any(|obj| obj.zone == Battlefield && obj.is_phased_in() && obj.controller == ctx.controller && obj.card_id ∈ ps.commander_ids)`.

Confirmed against CR 903.3d ("a permanent on the battlefield that is a commander") + "your commander" (owned):
- **Per-owner ownership encoding**: `commander_ids: Vector<CardId>` is a field on `PlayerState` (`crates/card-types/src/state/player.rs:297`), keyed per owner. Membership in the *controller's own* `commander_ids` inherently encodes ownership. Confirmed.
- **Stolen (opponent controls your commander) → OFF**: `obj.controller != ctx.controller` for the owner → false; and for the thief, the card is in the owner's `commander_ids`, not the thief's → false. Both branches asserted in `test_stolen_commander_decoy_lieutenant_off`. Correct.
- **Stole back → ON**: same fixture, control restored → true. Correct.
- **Control opponent's commander only → OFF**: `test_control_opponents_commander_only_still_off` pins the CR-118.9 divergence — this scenario WOULD satisfy the `casting.rs` CommanderFreeCast "a commander" predicate, but `YouControlYourCommander` correctly returns false. The plan's #1 correctness trap is defended. Correct.
- **Real-gameplay control changes are honored, not just builder-set controllers**: control-changing effects apply through Layer 2 `SetController` which writes back to `state.objects[id].controller` (`layers.rs:1863-1868`, `recompute_object_controller` at 1844-1870). So reading `obj.controller` directly in the arm reflects live Control-Magic-style effects, not stale base control. The stolen/stole-back cases therefore hold in production, not only under `.controlled_by()` test fixtures.
- **Zone/phase scoping**: `obj.zone == ZoneId::Battlefield` excludes command zone / graveyard / exile (`test_you_control_your_commander_false_in_command_zone`, `..._drops_when_commander_dies`); `is_phased_in()` (`game_object.rs:1360`) excludes phased-out. CR 400.7 handled implicitly — a destroyed commander becomes a graveyard object with the same `card_id` but `zone != Battlefield`, so the scan correctly drops it.
- **Multi-commander**: `.any()` — controlling one of two owned commanders suffices (`test_multiple_commanders_control_one_suffices`). Matches the partner ruling.

### Change 3: static path (`check_static_condition` `_ =>` fallback) — CORRECT

`crates/engine/src/effects/mod.rs:9400`. No dedicated arm; the fallback builds a minimal `EffectContext { controller, source, .. }` and delegates to `check_condition`. Since the arm reads only `ctx.controller` + `state`, the fallback evaluates it correctly. Proven directly by `test_check_static_condition_fallback_routes_you_control_your_commander` and end-to-end by the three Skyhunter tests. **Re-entrancy**: unlike `YouControlNOrMoreWithFilter`, this arm does NOT call `calculate_characteristics` — it reads base `obj.zone/controller/card_id` only, scanning OTHER objects, so it is strictly re-entrant-safe within the layer recompute. Confirmed.

### Change 4-5: hash + wire re-pin — CORRECT (append-only)

- Hash discriminant **51** (`hash.rs:5994`), prior max was **50** (`YouAttackedWithNOrMore`, line 5989). Correct, no collision.
- `HASH_SCHEMA_VERSION = 61` (`hash.rs:550`); `HashSchemaEpoch { version: 61, .. }` **appended** at line 825 with v60 preserved. Doc History line `- 61:` appended (543-549).
- `PROTOCOL_VERSION = 24` (`protocol.rs:225`); `PROTOCOL_SCHEMA_FINGERPRINT` (line 243) = the v24 `ProtocolEpoch` fingerprint (line 437) — lockstep. v22/v23 rows preserved unedited; v24 **appended** (433-438).
- **No stale sentinels**: grep for `HASH_SCHEMA_VERSION, 60` / `PROTOCOL_VERSION, 23` across `crates/engine/tests` → 0 matches. New `HASH_SCHEMA_VERSION, 61` / `PROTOCOL_VERSION, 24` present in 43 files. Bulk update complete; PB-OS9 test file carries both sentinels (`test_pb_os9_version_sentinels`).

### Intervening-if resolution path — CORRECT (uses `check_condition`, not the 2-variant enum)

Worth flagging that resolution.rs has TWO intervening-if sites: the runtime "Characteristics path" (line 2199) delegates to `abilities::check_intervening_if` (a 2-variant `InterveningIf` enum that does NOT know `Condition`), while the **CardDefETB path** (line 2125-2135) evaluates the DSL `intervening_if: Option<Condition>` via `crate::effects::check_condition`. The Lieutenant triggers are card-def triggers and resolve through the CardDefETB path, so `YouControlYourCommander` is evaluated correctly at resolution (CR 603.4). This is exactly the path the two Siege-Gang tests exercise, and they discriminate the resolution-time re-check (2 tokens when controlled at resolution; 0 tokens when the commander is removed in response). No bug.

## Deviation Verdict — CORRECT engineering call

The runner independently discovered (and I independently re-confirmed against source) that **`begin_combat()` in `crates/engine/src/rules/turn_actions.rs:1684-1703` sweeps ONLY emblem triggers** (`collect_emblem_triggers_for_event`, line 1693) and returns `Vec::new()`. There is NO card-def scan for `AbilityDefinition::Triggered { trigger_condition: AtBeginningOfCombat, .. }`, unlike the upkeep sweep at lines 258-320 which does scan battlefield objects' effective abilities. Therefore `loyal_apprentice` and `siege_gang_lieutenant`'s Lieutenant triggers never queue in real gameplay.

Keeping both `Completeness::partial` is correct under the no-wrong-game-state / no-gated-stub-as-Complete guardrail: their Lieutenant DSL is CR-correct (right `TriggerCondition::AtBeginningOfCombat`, right `intervening_if: Some(Condition::YouControlYourCommander)`, right token effect), but the ability is currently **inert** (never queued) — incomplete, not wrong. The `partial` notes are honest and accurate. This is the textbook correct application of implement-phase default-to-defer: the missing sweep is new engine surface beyond PB-OS9's declared scope, so it was flagged, not silently added.

**Faithfulness of the test isolation**: the two Siege-Gang tests and the Loyal Apprentice test queue the exact `PendingTrigger { ability_index, ..PendingTrigger::blank(obj, controller, PendingTriggerKind::CardDefETB) }` that a future combat sweep would produce. I verified the *existing* upkeep sweep produces precisely this shape (`turn_actions.rs:315-318`), so the tests faithfully predict real behavior once the sweep is added — they prove the primitive without masking the gap. The file-level doc comment is explicit that the trigger does NOT fire via a real `BeginningOfCombat` transition. This is the legitimate way to prove the primitive; no test falsely claims end-to-end firing.

## skyhunter_strike_force — genuinely Complete

`crates/card-defs/src/defs/skyhunter_strike_force.rs`. Oracle fully modeled: Flying (keyword), printed Melee (keyword), and the Lieutenant anthem as `AbilityDefinition::Static(ContinuousEffectDef { layer: EffectLayer::Ability, modification: AddKeyword(Melee), filter: OtherCreaturesYouControl, duration: WhileSourceOnBattlefield, condition: Some(YouControlYourCommander) })`. The layer/duration/filter triple matches the established reference pattern (`samut_voice_of_dissent.rs:36-39` grants Haste to `OtherCreaturesYouControl` with the identical `Ability`/`WhileSourceOnBattlefield` shape). No TODO / ENGINE-BLOCKED text remains; no gated stub. The continuous-grant path has no dependency on the missing combat sweep. `test_skyhunter_grant_active_and_melee_trigger_fires` proves the grant applies to another creature (Melee in calculated keywords) AND that the PB-EF3b-synthesized attack trigger actually fires (Buddy 2→3 after attacking one opponent); `..._drops_when_commander_leaves` and `..._off_when_commander_stolen` prove the layer re-eval drops the grant. Genuinely Complete.

## Test Integrity (SR-34/36) — all 15 non-vacuous

| Test | Discriminates | Non-vacuous? |
|------|---------------|--------------|
| true_on_battlefield | baseline ON | yes (direct true assertion) |
| false_in_command_zone | zone scoping | yes (ordinary battlefield creature present so scan non-empty) |
| drops_when_commander_dies | SBA/zone drop | yes (asserts true precondition, then false) |
| stolen_commander_decoy | owned-vs-controlled (both p1 and p2 OFF) | yes (stole-back sub-case reads true on identical card) |
| control_opponents_commander_only | CR 118.9 divergence | yes (the exact scenario that WOULD pass free-cast) |
| multiple_commanders_control_one | partner `.any()` | yes |
| skyhunter grant active + Melee trigger | static path + PB-EF3b dep | yes (keyword present AND power 2→3) |
| skyhunter grant drops on leave | layer re-eval | yes (asserts precondition then drop) |
| skyhunter grant off when stolen | control check in static path | yes (relative to the active test) |
| siege_gang creates tokens | intervening-if TRUE at resolution | yes (paired with the fail test) |
| siege_gang fails when removed | CR 603.4 resolution re-check | yes (0 tokens vs 2) |
| loyal_apprentice token shape | Thopter 1/1 flying artifact + haste | yes |
| cards_registered | roster smoke | n/a (registration) |
| static_condition_fallback route | Change 3 proof | yes |
| version_sentinels | wire pin | yes (strict eq) |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 903.3d (control your commander on battlefield) | Yes | Yes | `check_condition` arm; true/command-zone/dies tests |
| 903.3 (designation is a per-card attribute) | Yes | Yes | keyed off `card_id ∈ commander_ids`; stolen/opponent-only tests |
| "your" (owned) vs 118.9 "a" commander | Yes | Yes | `test_control_opponents_commander_only_still_off` |
| 611.2a/613.7 (control layer honored) | Yes | Yes (implicitly) | reads eager `obj.controller` written by Layer-2 SetController |
| 603.4 (intervening-if re-check at resolution) | Yes | Yes | CardDefETB path uses `check_condition`; siege-gang pair |
| 604.2 (conditional static) | Yes | Yes | skyhunter grant active/drop tests |
| 702.121 (Melee) granted trigger fires | Yes (via PB-EF3b) | Yes | Buddy 2→3 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| skyhunter_strike_force | Yes | 0 | Yes | Complete; layer/duration/filter match reference |
| loyal_apprentice | Yes (DSL) | 0 (honest partial note) | Yes (inert, not wrong) | partial — blocked on missing combat sweep |
| siege_gang_lieutenant | Yes (DSL; activated ability fully functional) | 0 (honest partial note) | Yes (Lieutenant inert; activated works) | partial — blocked on missing combat sweep |
| legion_lieutenant | n/a (name-only) | 0 | n/a | confirmed OUT, untouched (oracle: "Other Vampires you control get +1/+1") |

## Ship Decision

**Ship-ready.** No fix cycle needed. (a) The deviation — 2 cards staying `partial` — is the correct engineering call. (b) skyhunter_strike_force is truly Complete. The only follow-up is the optional LOW: file the `begin_combat()` card-def sweep seed at close-out so the two partials (and four out-of-scope siblings) can advance later.
