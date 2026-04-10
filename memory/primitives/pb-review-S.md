# Primitive Batch Review: PB-S — GrantActivatedAbility

**Date**: 2026-04-11
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: `9dc9331a` (base: `b212c100`)
**CR Rules**: 112.3, 602, 605.1a, 611.3a, 613.1f, 613.5, 613.7, 302.1/302.6, 400.7, 706, 708.2
**Engine files reviewed**:
- `crates/engine/src/state/continuous_effect.rs` (two new LayerModification variants)
- `crates/engine/src/state/hash.rs` (discriminants 23/24)
- `crates/engine/src/rules/layers.rs` (apply_modification arms, face-down override ordering)
- `crates/engine/src/rules/mana.rs` (handle_tap_for_mana calc-chars path)
- `crates/simulator/src/legal_actions.rs` (mana/activated ability loops calc-chars)
- `crates/engine/src/cards/helpers.rs` (ActivatedAbility export)
**Card defs reviewed**: 5 full (cryptolith_rite, chromatic_lantern, citanul_hierophants, paradise_mantle, enduring_vitality) + 2 partial TODO updates (song_of_freyalise, umbral_mantle)
**Tests reviewed**: `crates/engine/tests/grant_activated_ability.rs` (10 tests, 684 lines)

## Verdict: needs-fix

One HIGH finding (pre-existing but now promoted in importance by PB-S): `ActivatedAbility`'s
`HashInto` impl is missing the `once_per_turn` field, which means two granted activated
abilities differing only by `once_per_turn` hash identically. This is a replay-determinism
risk that PB-S propagates into Layer 6 via the new `AddActivatedAbility(Box<ActivatedAbility>)`
variant, and it also affects GameObject hashing via `chars.activated_abilities`. The engine
implementation is otherwise clean: oracle text matches for all 5 unblocked cards, filters
are battlefield-gated (no zone leakage), Humility timestamp ordering is correctly asserted,
the face-down test genuinely exercises the layer-6-over-override path, and the W3-LC
deferred `handle_tap_for_mana` path is now using `calculate_characteristics` correctly.

## Verification Checklist Results

### 1. Hash determinism for granted abilities (replay-critical)
- `HashInto for ManaAbility` (hash.rs:887-895) — CLEAN: hashes `produces`, `requires_tap`, `sacrifice_self`, `any_color`, `damage_to_controller`. All 5 struct fields covered.
- `HashInto for ActivatedAbility` (hash.rs:1985-1998) — **INCOMPLETE**: hashes `cost`, `description`, `effect`, `sorcery_speed`, `targets`, `activation_condition`, `activation_zone` — **missing `once_per_turn: bool`** (game_object.rs:283-289). Two grants differing only in `once_per_turn` collide. See Finding H1.
- `HashInto for ActivationCost` (hash.rs:1973-1983) — CLEAN: 8 fields, all hashed.
- The new `LayerModification::AddActivatedAbility(ab)` arm (hash.rs:1381-1384) correctly does `ab.hash_into(hasher)` via `impl<T: HashInto> HashInto for Box<T>` at hash.rs:150 — Box delegates to inner T, so `ab` is hashed, not just the discriminant.
- `AddManaAbility` arm (hash.rs:1387-1390) likewise hashes `ab` correctly.

**Result**: the dispatch is correct but the underlying `ActivatedAbility` hash impl is incomplete. File as HIGH (H1).

### 2. Face-down test exercises override→layer-6-readd path
Verified in `tests/grant_activated_ability.rs:607-684`:
- Sets `status.face_down = true` AND `face_down_as = Some(FaceDownKind::Morph)` (lines 642-648) — matches the guard at layers.rs:216 (`obj.status.face_down && obj.face_down_as.is_some()`). The override executes.
- Asserts (a) face-up: grant present + Flying present (pre-condition established).
- Asserts (b) face-down: grant **still present** in `mana_abilities` (proves layer-6 re-adds AFTER the face-down clear) AND Flying is **gone** (proves the override ran — otherwise Flying would still be there). This is the exact assertion pair that proves the override→layer-6 path executes.
- Asserts (c) face-up again: both grant and Flying restored.

All three checkpoints are real state assertions, not trivially-true. ✓ CLEAN.

### 3. Granted-ability zone leakage
Verified in `crates/engine/src/rules/layers.rs`:
- `CreaturesYouControl` (line 649): `if obj_zone != ZoneId::Battlefield || !chars.card_types.contains(&CardType::Creature) { return false; }` — battlefield gate is explicit.
- `LandsYouControl` (line 891): `if obj_zone != ZoneId::Battlefield { return false; }` — battlefield gate is explicit.
- `AttachedCreature` (line 594): `if obj_zone != ZoneId::Battlefield { return false; }` + also requires `effect.source.attached_to == object_id` — double-gated.
- `OtherCreaturesYouControl` (line 663): battlefield-gated.

A creature spell on the stack, a card in hand, or a card in graveyard will fail the battlefield check before the filter ever looks at controller. No leakage. ✓ CLEAN.

### 4. Layer 6 dependency metadata
- Grants are self-contained (`{T}: Add mana`, no reference to counters or layer-3 text changes) — no new dependency edges needed.
- `test_humility_removes_granted_mana_ability` (lines 564-596) sets Rite at ts=10 and Humility at ts=20, then asserts `mana_abilities.is_empty()`. This is the **correct** "Humility later → wipes grant" assertion. The effect sort (`toposort_with_timestamp_fallback` at layers.rs:1157) ensures ts=10 `AddManaAbility` runs before ts=20 `RemoveAllAbilities`, so the grant is appended and then wiped. ✓ CLEAN.
- Note: the inverse case (Humility earlier → Rite re-adds on top) is not tested, but is correct-by-construction (timestamp sort + append semantics) and is low-value to cover (symmetric). Flagging only as LOW L1 for awareness, not a required fix.

### 5. Card def oracle-text accuracy
Looked up each via `mcp__mtg-rules__lookup_card`:

| Card | Oracle | Def match |
|------|--------|-----------|
| Cryptolith Rite | `{1}{G}` Enchantment; `Creatures you control have "{T}: Add one mana of any color."` | ✓ — filter=CreaturesYouControl, any_color=true, requires_tap=true |
| Chromatic Lantern | `{3}` Artifact; `Lands you control have "{T}: Add one mana of any color."` + self `{T}: Add one mana of any color.` | ✓ — both abilities present, grant is additive, self-ability uses existing Effect::AddManaAnyColor path |
| Citanul Hierophants | `{3}{G}` Creature 3/2; `Creatures you control have "{T}: Add {G}."` | ✓ — uses `ManaAbility::tap_for(ManaColor::Green)`; the source IS a creature, so the filter including self is correct (Hierophants itself gains the ability — matches oracle). |
| Paradise Mantle | `{0}` Equipment; `Equipped creature has "{T}: Add one mana of any color." / Equip {1}` | ✓ — Equip keyword present, AttachedCreature filter, any_color grant |
| Enduring Vitality | `{1}{G}{G}` Enchantment Creature 3/3; Vigilance + creatures-you-control grant + die-return-as-enchantment | ✓ for Vigilance and grant; die-return is documented TODO (explicitly deferred in plan scope) |

All oracle matches are correct. ✓ CLEAN.

### 6. W3-LC deferred item closure
- `handle_tap_for_mana` (mana.rs:122-134) now resolves abilities via `crate::rules::layers::calculate_characteristics(state, source)` with `unwrap_or_else` fallback to base chars. This closes the W3-LC deferred MEDIUM noted in the plan ("abilities.rs:222,246 — activated ability access bypasses Humility" — the mana-ability leg).
- Summoning-sickness block (lines 146-156) also uses calc'd chars.
- Sacrifice path (lines 167-195) also uses calc'd chars.
- Legacy grants like Treasure tokens (sacrifice_self) still work because the calc'd chars path still returns the same `ManaAbility` struct with `sacrifice_self = true` for baked-in abilities. No regression risk.
- ✓ CLOSED. Note in review: the fix-session runner should update the W3-LC tracker entry when closing PB-S (no action in this review file — that's tracking, not review scope).

### 7. Standard PB review checklist
- Plan items from the Verification Checklist at `pb-plan-S.md:427-439`: all 11 items implemented per `primitive-wip.md` step checklist. ✓
- No new TODOs in modified engine files; 5 TODOs removed (in the 5 fully-authored cards); 2 TODO texts refined (song_of_freyalise, umbral_mantle). The updated TODOs are correct and include clear remaining-blocker info. ✓
- `grep` for `match .*LayerModification` in `tools/` returned no matches — neither `tools/tui/src/play/panels/stack_view.rs` nor `tools/replay-viewer/src/view_model.rs` matches on `LayerModification`. No exhaustive-match regressions. ✓
- Clippy: WIP reports 0 warnings. Spot-checked: `Box<ActivatedAbility>` avoids `large_enum_variant`, `(**ability).clone()` double-deref is idiomatic. ✓
- Card defs all use `..Default::default()` shorthand; no explicit-struct-construction gotcha. ✓
- Test count: 2589 → 2599 = +10, matches the 10 new tests in `grant_activated_ability.rs`. ✓

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| H1 | **HIGH** | `crates/engine/src/state/hash.rs:1985-1998` | **`ActivatedAbility::hash_into` misses `once_per_turn` field.** Two `ActivatedAbility` structs differing only in `once_per_turn: bool` will hash identically, producing duplicate game-state hashes for non-equal states. Pre-existing issue (applies to `chars.activated_abilities` in GameObject) but PB-S now promotes its blast radius by adding `LayerModification::AddActivatedAbility(Box<ActivatedAbility>)`, which participates in hashing of the continuous-effect registry — a collision here silently corrupts replay determinism. **Fix:** Add `self.once_per_turn.hash_into(hasher);` after `self.activation_zone.hash_into(hasher);` in `impl HashInto for ActivatedAbility`. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | — | No findings. All 5 card defs match oracle text and use the new primitive correctly. |

## Partial/Deferred Card Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | — | `song_of_freyalise` and `umbral_mantle` retain intentional TODOs with clearly-scoped blockers (Saga framework; `{Q}` untap symbol). Both are documented in the plan as out-of-scope for PB-S. No findings. |

## Test Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| L1 | LOW | `tests/grant_activated_ability.rs:564-596` | **Humility inverse-order case not tested.** Only "Humility later → wipes grant" is covered. The symmetric "Humility earlier → Rite re-adds grant on top" case is correct by construction (timestamp sort + append semantics) and low-value, but a ~15-line test would be cheap insurance against future regressions in the layer dispatch path. **Fix:** optional — add `test_cryptolith_rite_overrides_earlier_humility` with Humility ts=5 and Rite ts=10, asserting the creature has the granted mana ability. |
| L2 | LOW | `tests/grant_activated_ability.rs` (whole file) | **No test for the granted-ability path via `handle_activate_ability` (non-mana side).** All 10 tests exercise `AddManaAbility`; none exercise `AddActivatedAbility`. Reason is valid: no card in PB-S uses `AddActivatedAbility` (Umbral Mantle blocked on `{Q}`). But the primitive ships without a positive test for its activated-ability leg — a future PB that uses `AddActivatedAbility` will be the first real exercise. **Fix:** optional — add a synthetic test that builds a `ContinuousEffect` with `AddActivatedAbility(Box::new(ActivatedAbility { ... }))`, appends to state, and asserts the filtered creature's `calculate_characteristics().activated_abilities` contains the granted ability. 1 test, ~40 lines. |

## Other / Informational Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| L3 | LOW | `crates/simulator/src/mana_solver.rs:35` | **Deferred per plan; still reads base chars.** Bot mana-source enumeration still uses `obj.characteristics.mana_abilities` instead of calc'd chars, so bots will undervalue granted mana sources from Cryptolith Rite / Chromatic Lantern / Citanul Hierophants etc. This was explicitly deferred in `pb-plan-S.md:385` as a follow-up; not a PB-S blocker (bot non-correctness). **Fix:** track as LOW follow-up in workstream-state. No action required in PB-S fix cycle. |

### Finding Details

#### Finding H1: `ActivatedAbility::hash_into` missing `once_per_turn`

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1985-1998`
**CR**: 613.5 (identical-but-distinct grants), 611.3a (continuous effect identity) — plus replay-determinism invariant from CLAUDE.md.
**Struct**: `ActivatedAbility` at `crates/engine/src/state/game_object.rs:254-290` has 8 fields: `cost`, `description`, `effect`, `sorcery_speed`, `targets`, `activation_condition`, `activation_zone`, `once_per_turn`.
**Hash impl**: hashes the first 7 fields only. `once_per_turn: bool` (lines 283-289) is **not** fed into the hasher.
**Impact**:
1. Two granted activated abilities that differ only in `once_per_turn` produce the same `LayerModification::AddActivatedAbility(Box<ActivatedAbility>)` hash — registration and dedup in the effect registry could silently unify them.
2. More directly: two `GameObject`s whose `chars.activated_abilities` differ only by this bit hash identically. Any replay determinism check that relies on game-state hashes treats these states as equivalent.
3. PB-S changes the blast radius: previously the collision was limited to GameObject storage. Now any continuous effect in the registry that uses `AddActivatedAbility` is also subject to the collision.
**Runner's own plan said this**: `pb-plan-S.md:211` explicitly listed `once_per_turn` as a field to verify in the `ActivatedAbility` hash impl. The runner concluded "impls already present" and did not verify completeness against the struct definition.
**Fix**: In `crates/engine/src/state/hash.rs` after line 1996 (`self.activation_zone.hash_into(hasher);`), add:
```rust
// CR 602.5b: "activate only once each turn" restriction
self.once_per_turn.hash_into(hasher);
```
No field order is load-bearing here (each field is hashed with its own type-sensitive serialization) but placing it after `activation_zone` matches the struct declaration order, which is the convention in this file.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 112.3 (continuous effects modify characteristics) | Yes | Yes | test_cryptolith_rite_grants_mana_ability_to_creatures |
| 605.1a (granted no-target tap mana ability is a mana ability) | Yes | Yes | test_granted_mana_ability_taps_and_produces_mana |
| 611.3a (continuous effect from static ability applies while source is in play) | Yes | Yes | test_cryptolith_rite_grant_ends_when_source_leaves |
| 613.1f (Layer 6 ability-adding) | Yes | Yes | All 10 tests; filter gating verified |
| 613.5 (two separate grant sources → two ability instances) | Yes | Yes | test_two_cryptolith_rites_grant_two_abilities_but_one_tap |
| 613.7 (timestamp order within a layer) | Yes | Yes | test_humility_removes_granted_mana_ability (ts=10 vs ts=20) |
| 302.6 (summoning sickness with granted mana abilities) | Yes | Yes | test_granted_mana_ability_respects_summoning_sickness |
| 400.7 (source leaves battlefield → effect ends) | Yes | Yes | test_cryptolith_rite_grant_ends_when_source_leaves |
| 708.2 (face-down printed chars stripped; external effects still apply) | Yes | Yes | test_face_down_creature_inherits_granted_mana_ability |
| 706 (copies) | Correct by construction | No | Documented as "no test needed" in plan; copy path re-derives grant from layer loop |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| cryptolith_rite | Yes | 0 | Yes | Clean |
| chromatic_lantern | Yes | 0 | Yes | Both self-ability and grant present |
| citanul_hierophants | Yes | 0 | Yes | Self-inclusive filter matches oracle (source IS a creature) |
| paradise_mantle | Yes | 0 | Yes | Equip + AttachedCreature grant |
| enduring_vitality | Yes | 1 (die-return-as-enchantment, explicitly deferred) | Partial (Vigilance + grant correct; die-return missing per plan scope) | Acceptable per plan |
| song_of_freyalise | N/A (Saga blocked) | 2 (Saga chapter framework) | Unchanged (still stub) | TODO text refined |
| umbral_mantle | N/A ({Q} blocked) | 1 (AddActivatedAbility + {Q}) | Unchanged (Equip only) | TODO text refined |

## Summary

**1 HIGH / 0 MEDIUM / 3 LOW**

**Recommendation**: **fix-cycle required** to address H1 (hash completeness). The fix is 2 lines. All other findings are LOW and optional.

## Specific Callouts for Oversight

1. **H1 is pre-existing but PB-S escalates the blast radius.** The runner's plan at pb-plan-S.md:211 said "if impls don't exist, add them — hashing `once_per_turn` included" — the runner found impls already exist and did not verify field completeness against the current struct. Recommend the fix-session runner:
   - Add the missing `self.once_per_turn.hash_into(hasher);` line.
   - Also audit `HashInto for Characteristics` at hash.rs:896-916 to confirm it hashes `activated_abilities` (line 909) — it does, so the fix in `ActivatedAbility::hash_into` automatically propagates to all consumers.
   - Consider a one-shot grep for other structs where `HashInto` may have drifted from field count: any struct with `#[serde(default)]` fields added after the `HashInto` impl was written is at risk. This is a LOW-priority follow-up beyond PB-S scope.

2. **L2 (no AddActivatedAbility test)** is a legitimate gap but not a blocker. The variant is shipped "for when Umbral Mantle lands" and has a clean code path (same pattern as AddManaAbility, just a different vec). Recommend deferring the test to the first PB that consumes `AddActivatedAbility` (likely PB that introduces `{Q}` cost).

3. **W3-LC closure tracking**: the fix-session runner should update the W3-LC deferred item entry in `memory/w3-layer-audit.md` (or wherever it lives) to note PB-S closed the `handle_tap_for_mana` leg. The `mana_solver.rs` leg remains deferred as an explicit LOW follow-up.

4. **No action required on `umbral_mantle` or `song_of_freyalise`** beyond what's in the commit — the refined TODO texts clearly scope the remaining blockers.
