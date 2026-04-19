# Primitive Batch Review: PB-P — EffectAmount::PowerOfSacrificedCreature (LKI)

**Date**: 2026-04-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 608.2b (LKI), 701.16 (Sacrifice), 400.7 (object identity), 117.1f / 601.2g (cost ordering), 118.8 (additional costs), 602.2 (activated ability cost payment), 613.1 (layers)
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (EffectAmount variant)
- `crates/engine/src/state/types.rs` (AdditionalCost reshape)
- `crates/engine/src/state/stack.rs` (StackObject field)
- `crates/engine/src/state/hash.rs` (sentinel + hash arms)
- `crates/engine/src/effects/mod.rs` (EffectContext field, resolve_amount arm, EffectContext literal sites)
- `crates/engine/src/rules/casting.rs` (spell LKI capture site, AdditionalCost readers)
- `crates/engine/src/rules/abilities.rs` (activated LKI capture site, EffectContext literal site)
- `crates/engine/src/rules/resolution.rs` (spell-path propagation, activated-path propagation)
- `crates/engine/src/rules/copy.rs` (StackObject literal sites)
- `crates/engine/src/rules/engine.rs` (StackObject literal sites)
- `crates/engine/src/testing/replay_harness.rs` (AdditionalCost construction sites)
- `crates/engine/src/lib.rs` (HASH_SCHEMA_VERSION re-export)
**Card defs reviewed**: 3 — `altar_of_dementia.rs`, `greater_good.rs`, `lifes_legacy.rs`
**Tests reviewed**: `crates/engine/tests/pbp_power_of_sacrificed_creature.rs` (M1-M8 + O3 active, O1 `#[ignore]`)
**Hash sentinel update sites**: `pbd_damaged_player_filter.rs`, `pbn_subtype_filtered_triggers.rs`, `pbp_power_of_sacrificed_creature.rs` (all assert `6u8`)

## Verdict: PASS-WITH-NOTES

PB-P is structurally sound. The dispatch verdict (PASS-AS-NEW-EFFECT-AMOUNT-VARIANT) and capture-point decision (CAPTURE-BY-VALUE) are correctly implemented. LKI capture happens BEFORE `move_object_to_zone` at both cost-payment sites (verified at `casting.rs:3838-3848` and `abilities.rs:721-734`). EffectAmount discriminant 15 is unique (no collision with neighbors 4-14). Hash sentinel correctly bumped to 6 with strict `assert_eq!` at all three test-file consumers. The 3 card defs faithfully implement oracle text per MCP lookup. M4 (load-bearing LKI test) genuinely discriminates capture-by-value from capture-by-ID by exercising an anthem-boosted creature: capture-by-ID would mill 2 (graveyard base power), capture-by-value mills 3 (LKI battlefield value); test asserts 3. **No HIGH or MEDIUM findings; 5 LOW findings noted, all out-of-scope or opportunistic.** Test-validity check passes — every numbered MANDATORY test exercises the LKI primitive end-to-end with discriminating assertions, and the M5 zero-power test correctly verifies the defensive `unwrap_or(0)` path.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| L1 | LOW | `crates/engine/src/cards/defs/warstorm_surge.rs:4-9` | **Stale TODO comment not cleaned.** Plan called this opportunistic; not done. **Fix:** Replace stale TODO with one-line CR citation if next reviewer has spare cycles. |
| L2 | LOW | `crates/engine/src/rules/abilities.rs:587-621` | **`sacrifice_self` cost path does not capture LKI power.** PB-P scope is narrow (no card uses sacrifice-self with PowerOfSacrificedCreature), but a future card requiring "Sacrifice ~: do X equal to its power" would silently get 0. **Fix:** For a future PB, capture power from `source` BEFORE `move_object_to_zone(source, ZoneId::Graveyard(owner))` at line 601 and append to `sacrificed_lki_powers`. Out of PB-P scope. |
| L3 | LOW | `crates/engine/src/rules/resolution.rs:204` (fused-spell path), `:437` (splice path), `:1826` (loyalty), `:1847` (forecast), `:2018/2035/2111` (triggered abilities), `:4942` (haunt), `:6836/6908/7008` (other resolution sites) | **`sacrificed_creature_powers` not propagated at non-spell, non-activated-ability resolution sites.** None of these sites have sacrifice as a cost, so the field would be unused. Documented for completeness; no real-world impact under current MTG cards. **Fix:** If a future card combines triggered/loyalty/forecast/haunt with sacrifice cost, add propagation. Out of PB-P scope. |
| L4 | LOW | `crates/engine/src/effects/mod.rs:526` (DiscardCards), `:533` (MillCards) | **Negative `i32 as usize` wraparound: pre-existing engine bug.** `Effect::DrawCards` clamps with `.max(0) as usize` (line 516); MillCards and DiscardCards do NOT. PB-P does not introduce this regression — the existing `EffectAmount::PowerOf` path can already produce negative i32. The O1 `#[ignore]` test would expose this; its docstring's "CR 107.1b: clamp negative to 0" claim is aspirational, not actual engine behavior. **Fix:** Pre-existing — out of PB-P scope. Track as engine LOW (parity with MR-M7-05 fix to DrawCards). |
| L5 | LOW | `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:947` (O1 docstring) | **O1 docstring asserts CR 107.1b clamping, but engine doesn't actually clamp at MillCards.** The `#[ignore]` reason cited is "priority loop in pass_all helper", but the deeper issue is L4 above. The docstring is aspirationally-correct (per `memory/conventions.md` aspirational-comment hazard rule). **Fix:** Either fix MillCards (out of PB-P scope) or update O1 docstring to honestly say "blocked on pre-existing MR-M7-05-style cast issue at effects/mod.rs:533, not just the priority loop". Recommended: amend docstring in a follow-up; not a PB-P merge blocker. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | — | — | All 3 card defs match MCP-verified oracle text exactly. No remaining TODOs in any of the affected card defs. |

### Per-card verification

#### Altar of Dementia (`altar_of_dementia.rs`)

**Oracle (MCP)**: "Sacrifice a creature: Target player mills cards equal to the sacrificed creature's power."
**Implementation**:
- Mana cost `{2}`, type Artifact ✓
- `Cost::Sacrifice(TargetFilter { has_card_type: Some(CardType::Creature), .. })` ✓
- `Effect::MillCards { player: PlayerTarget::DeclaredTarget { index: 0 }, count: EffectAmount::PowerOfSacrificedCreature }` ✓
- `targets: vec![TargetRequirement::TargetPlayer]` ✓
- `Cost::Sacrifice(TargetFilter)` correctly maps to `SacrificeFilter::Creature` via `replay_harness.rs:3140` ✓
- TODO comment block stripped ✓

#### Greater Good (`greater_good.rs`)

**Oracle (MCP)**: "Sacrifice a creature: Draw cards equal to the sacrificed creature's power, then discard three cards."
**Implementation**:
- Mana cost `{2}{G}{G}`, type Enchantment ✓
- `Cost::Sacrifice` filtered to creature ✓
- `Effect::Sequence(vec![DrawCards(PowerOfSacrificedCreature), DiscardCards(Fixed(3))])` ✓ — sequencing correct ("then discard" = post-draw)
- DiscardCards uses `EffectAmount::Fixed(3)` (the field is `EffectAmount`, not bare `u32` — runner correctly used `Fixed(3)`) ✓
- Stale "Effect::DiscardCards not in DSL" TODO cleaned ✓

#### Life's Legacy (`lifes_legacy.rs`)

**Oracle (MCP)**: "As an additional cost to cast this spell, sacrifice a creature.\nDraw cards equal to the sacrificed creature's power."
**Implementation**:
- Mana cost `{1}{G}`, type Sorcery ✓
- `spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature]` retained ✓
- `Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::PowerOfSacrificedCreature }` (replaces Fixed(1) placeholder) ✓
- Stale TODO comment blocks at lines 5-9 and 21-23 cleaned ✓

### Finding Details (none above LOW)

No HIGH or MEDIUM findings. Five LOWs documented in the engine table above; all are either out-of-PB-P-scope or opportunistic.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 608.2b (LKI) | Yes | Yes (M4 load-bearing) | Capture-by-value at sacrifice site; M4 anthem-boosted bear correctly mills 3 not 2 |
| 701.16 (Sacrifice) | Yes | Yes (M1-M5) | All 4 cards exercise sacrifice-as-cost; both spell additional cost and activated ability paths |
| 400.7 (object identity) | Yes (capture-by-value avoids dead-ID lookup) | Yes (M4 explicitly verifies graveyard object's base power = 2 vs LKI = 3) | OLD ObjectId is dead post-move; PB-P never reads through it |
| 117.1f / 601.2g (cost ordering) | Yes | Yes (M3) | Spell additional cost paid before stack push; activated cost paid before stack push |
| 118.8 (additional costs) | Yes | Yes (M3) | `SpellAdditionalCost::SacrificeCreature` plumbed via `additional_costs.Sacrifice.lki_powers` |
| 602.2 (activated ability cost) | Yes | Yes (M1, M2) | `Cost::Sacrifice(TargetFilter)` resolves via `ability_cost.sacrifice_filter` and captures LKI before zone move |
| 613.1 (layers) | Yes | Yes (M4) | LKI capture uses `calculate_characteristics` for layer-resolved power |
| 107.1b (negative quantities) | Partial (DrawCards clamps; MillCards/DiscardCards don't) | O1 ignored | L4/L5 documented; pre-existing engine gap |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| altar_of_dementia | Yes | 0 | Yes | Mill = sac creature's LKI power (M1, M4, M5 verify) |
| greater_good | Yes | 0 | Yes | Draw = sac creature's LKI power; discard 3 sequenced after (M2 verifies) |
| lifes_legacy | Yes | 0 | Yes | Draw = sac creature's LKI power via spell additional cost path (M3, O3 verify) |

## Test Plan Compliance

| Test | Mandatory? | Status | Discriminator | CR Cited |
|------|-----------|--------|---------------|----------|
| M1 `test_altar_of_dementia_mills_by_sacrificed_power` | M | PASS | Mill = 5 (5/5 Goblin) | 608.2b, 701.16, 602.2 |
| M2 `test_greater_good_draws_by_sacrificed_power_then_discards_three` | M | PASS | Net hand +1 (drew 4, discarded 3); library -4 | 608.2b, 701.16, 602.2, 701.7 |
| M3 `test_lifes_legacy_draws_by_sacrificed_power_on_resolve` | M | PASS | Library -6 (6/6 Beast as additional cost) | 608.2b, 118.8, 117.1f |
| M4 `test_lki_correctness_anthem_boosted_creature_sacrifice` | M | PASS | Mill = 3 (LKI under anthem), NOT 2 (graveyard base) | 608.2b, 613.1 — **load-bearing** |
| M5 `test_zero_power_creature_sacrifice_mills_zero` | M | PASS | Mill = 0 (no library change for 0/4 Wall) | 608.2b, 701.16 |
| M6 `test_sacrifice_no_capture_returns_zero_defensive` | M | PASS | Empty `sacrificed_creature_powers` → resolve_amount returns 0; no panic | N/A defensive |
| M7 `test_hash_parity_power_of_sacrificed_creature_distinct` | M | PASS | All 5 hashes distinct; `assert_eq!(HASH_SCHEMA_VERSION, 6u8)` | N/A hash infra |
| M8 `test_backward_compat_existing_powerof_cards_still_work` | M | PASS | Swords to Plowshares: P2 gains 5 life from 5/5 (existing PowerOf path) | N/A regression |
| O1 `test_sacrifice_negative_power_creature_mills_zero` | O | `#[ignore]` (priority loop) | Documented in test docstring; see L5 for partial-aspirational concern | 608.2b, 107.1b |
| O2 `test_multi_sacrifice_carries_only_first_power` | O | Not present (single-creature sufficient per plan) | — | N/A |
| O3 `test_lifes_legacy_with_zero_power_creature_draws_zero` | O | PASS | Cast Life's Legacy on 0/4 Wall: drew 0 cards | 608.2b |

## Hash Sentinel Verification

| Site | Value | Form | Verified |
|------|-------|------|----------|
| `crates/engine/src/state/hash.rs:36` | `pub const HASH_SCHEMA_VERSION: u8 = 6` | const definition | ✓ |
| `crates/engine/src/state/hash.rs:6168` | `HASH_SCHEMA_VERSION.hash_into(&mut hasher)` | written into stream | ✓ |
| `crates/engine/src/state/hash.rs:4406` | `EffectAmount::PowerOfSacrificedCreature => 15u8.hash_into(hasher)` | discriminant (unique 4-15) | ✓ |
| `crates/engine/src/state/hash.rs:3002` | `AdditionalCost::Sacrifice { ids, lki_powers } => { ... }` | both fields hashed | ✓ |
| `crates/engine/src/state/hash.rs:2928-2931` | `(self.sacrificed_creature_powers.len() as u64).hash_into; for p in ...` | StackObject field hashed | ✓ |
| `crates/engine/src/lib.rs:30` | `pub use state::hash::HASH_SCHEMA_VERSION` | re-exported for tests | ✓ |
| `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:781` | `assert_eq!(HASH_SCHEMA_VERSION, 6u8, ...)` | strict equality | ✓ |
| `crates/engine/tests/pbd_damaged_player_filter.rs:596` | `assert_eq!(HASH_SCHEMA_VERSION, 6u8, ...)` | strict equality (updated 5→6) | ✓ |
| `crates/engine/tests/pbn_subtype_filtered_triggers.rs:547` | `assert_eq!(HASH_SCHEMA_VERSION, 6u8, ...)` | strict equality (updated 5→6) | ✓ |
| History comment (`hash.rs:23-35`) | Includes "6: PB-P (2026-04-19)" entry | history preserved | ✓ |

**Discriminant uniqueness check**: EffectAmount discriminants are 0=Fixed, 1=XValue, 2=PowerOf, 3=ToughnessOf, 4=AffinityCount, 5=CostReduction (`{ zone, player, filter }`), 6=PermanentCount, 7=DevotionTo, 8=CounterCount, 9=LastEffectCount, 10=LastDiceRoll, 11=Sum, 12=CombatDamageDealt, 13=ChosenTypeCreatureCount, 14=DomainCount, **15=PowerOfSacrificedCreature** (new). No collision.

## LKI-Correctness Deep Dive (M4)

M4 is the load-bearing test that discriminates capture-by-value from capture-by-ID. Setup:

1. P1 controls Altar of Dementia (battlefield).
2. P1 controls a 2/2 Bear (base power 2 stored in `obj.characteristics.power`).
3. A continuous effect with `EffectFilter::AllCreatures` + `LayerModification::ModifyPower(1)` makes the Bear 3/3 on the battlefield (verified at lines 541-547 via `calculate_characteristics`).
4. P1 activates Altar, sacrificing the 3/3 anthem-boosted Bear, targeting P2.

Discriminating assertions:

- **Line 578-582**: `graveyard_bear.characteristics.power == Some(2)` — confirms the graveyard object's BASE characteristics retain the printed value (2), not the LKI (3). This is the BASELINE-LKI-01 condition: `calculate_characteristics` re-runs filters against the graveyard object and would also return 2 because the anthem's `EffectFilter::AllCreatures` no longer applies to a non-battlefield object.
- **Line 593-598**: `milled == 3` — the actual game-state assertion. If the engine had implemented capture-by-ID (read graveyard chars at resolution time), this would be 2 and the test would fail.

The test correctly walks the full dispatch chain: `Cost::Sacrifice(filter)` → `replay_harness::flatten_cost_into` → `SacrificeFilter::Creature` → `ability_cost.sacrifice_filter` → `abilities.rs:721-734` (capture before `move_object_to_zone`) → `stack_obj.sacrificed_creature_powers` → `resolution.rs:1810` (propagate to ctx) → `effects/mod.rs:6136` (read first via `unwrap_or(0)`) → `Effect::MillCards { count }` resolves to 3 → P2 mills 3 cards.

**Verdict**: M4 is a sound discriminator. Capture happens BEFORE move_object_to_zone (line 733 captures, line 735 moves). The capture uses `calculate_characteristics(state, sac_id)` while sac_id is still on the battlefield, so layer effects (the anthem) still apply. Per CR 608.2b, this is the correct LKI value.

## AdditionalCost Reshape Validation

The reshape from `AdditionalCost::Sacrifice(Vec<ObjectId>)` (tuple) to `AdditionalCost::Sacrifice { ids: Vec<ObjectId>, lki_powers: Vec<i32> }` (struct) was applied at all consumer sites:

| File:Line | Pattern | Behavior |
|-----------|---------|----------|
| `casting.rs:137` | `Sacrifice { .. } => {}` | match arm in additional_cost iter |
| `casting.rs:176` | `Sacrifice { ids, .. }` | extract sacrifice IDs (ignores lki) |
| `casting.rs:3852` | `Sacrifice { ids, lki_powers }` | LKI population (writes lki_powers) |
| `casting.rs:3989` | `Sacrifice { ids, .. }` | devour sacrifice ID extraction |
| `resolution.rs:392` | `Sacrifice { lki_powers, .. }` | propagate to ctx |
| `resolution.rs:1120` | `Sacrifice { ids, .. }` | devour-on-stack ID extraction |
| `replay_harness.rs:344, 1133, 1197, 1276, 1526` | `Sacrifice { ids: vec![...], lki_powers: vec![] }` | constructions; engine fills lki_powers |
| `state/hash.rs:3002` | `Sacrifice { ids, lki_powers }` | hash both fields |

**Devour/Casualty/Bargain/Emerge non-LKI consumers**: All use `Sacrifice { ids, .. }` ignoring `lki_powers`. They were never expected to use LKI; the field is benignly empty for them. ✓

**Test files**: Per the runner's checklist line 210, 43 test sites converted. Spot-checked `bargain.rs`, `casualty.rs`, `cost_primitives.rs`, `devour.rs`, `emerge.rs` via grep — all use `Sacrifice { ids: vec![...], lki_powers: vec![] }` form. ✓

**Replay-fixture compatibility**: Old serialized JSON with `{"Sacrifice":[1,2,3]}` (tuple form) will fail to deserialize against the new struct form. Hash sentinel bump 5→6 ensures stale replays would fail validation anyway, so this is acceptable per the plan's R1 risk acknowledgment.

## EffectContext Propagation Audit

All `EffectContext` literal/new sites populate `sacrificed_creature_powers`:

| File:Line | Form | Field set | Verified |
|-----------|------|-----------|----------|
| `effects/mod.rs:161` | `EffectContext::new` | `vec![]` default | ✓ |
| `effects/mod.rs:193` | `EffectContext::new_with_kicker` | `vec![]` default | ✓ |
| `effects/mod.rs:2398` | inner_ctx for ForEach (player) | `ctx.sacrificed_creature_powers.clone()` | ✓ |
| `effects/mod.rs:2432` | inner_ctx for ForEach (object) | `ctx.sacrificed_creature_powers.clone()` | ✓ |
| `effects/mod.rs:7085` | minimal-ctx for static condition check | `vec![]` (correct — no spell context) | ✓ |
| `abilities.rs:270` | activation-condition ctx | `vec![]` (correct — no sacrifice yet) | ✓ |

**Resolution propagation sites** (where StackObject → ctx propagation matters):

| File:Line | Site | Propagation present? | Verified |
|-----------|------|-----------------------|----------|
| `resolution.rs:359` (spell, non-fuse) | `EffectContext::new_with_kicker(...)` then `ctx.sacrificed_creature_powers = lki_powers_from_ac` at :398 | YES (additional_costs path) | ✓ |
| `resolution.rs:1800` (activated ability) | `EffectContext::new(...)` then `ctx.sacrificed_creature_powers = stack_obj.sacrificed_creature_powers.clone()` at :1810 | YES (StackObject path) | ✓ |
| `resolution.rs:204` (fuse spells) | No PB-P propagation | NO (LOW L3 — split spells lack sacrifice cost) | informational |
| `resolution.rs:437` (splice ctx) | No PB-P propagation | NO (LOW L3 — splice cards are Arcane only) | informational |
| `resolution.rs:1826` (loyalty), `:1847` (forecast), `:2018/2035/2111` (triggered), `:4942` (haunt), `:6836/6908/7008` (others) | No PB-P propagation | NO (none of these have sacrifice cost) | informational |

## Stale-TODO Sweep

| Card def | TODO state | Action taken |
|----------|-----------|--------------|
| `altar_of_dementia.rs` | Was `abilities: vec![]` with TODO | Authored ability + cleaned TODO ✓ |
| `greater_good.rs` | Was `abilities: vec![]` with stale "Effect::DiscardCards missing" TODO | Authored ability + cleaned TODO ✓ |
| `lifes_legacy.rs` | Had `Fixed(1)` placeholder + TODO at lines 5-9 and 21-23 | Replaced placeholder + cleaned TODOs ✓ |
| `warstorm_surge.rs` | Stale "Effect approximated as Nothing" comment (already implemented via PowerOf(TriggeringCreature)) | NOT cleaned (out of PB-P scope per plan; documented as L1) |

## Previous Findings (re-review only)

(None — first review of PB-P.)

## Risk Assessment Summary

- **R1 (AdditionalCost shape change blast radius)**: All 10+ engine sites + 43 test sites converted cleanly per runner checklist; no JSON fixture breakage observed (hash sentinel bump 5→6 makes any stale-replay deserialization fail loud anyway).
- **R2 (Capture-by-value vs Capture-by-ID)**: Capture-by-value chosen and correctly implemented. M4 directly tests the structural soundness vs capture-by-ID alternative.
- **R3 (Spell-resolution dispatch site identification)**: Resolved at `resolution.rs:359` (the main non-fuse path). Other spell paths (fuse, splice) don't apply per LOW L3 analysis.
- **R4 (Hash bump replay-fixture rebakes)**: Hash sentinel bumped consistently across 3 test files; assertions strict-equal `6u8`. Old replays will fail loud (intended).
- **R5 (BASELINE-LKI-01 reach check)**: Verified safe — capture happens BEFORE move_object_to_zone, so `calculate_characteristics` runs against the still-on-battlefield object and applies all layer effects.
- **R6 (Effect ordering inside Greater Good Sequence)**: Verified — `Effect::Sequence(vec![DrawCards(PowerOfSacrificedCreature), DiscardCards(Fixed(3))])` with M2 net hand +1 assertion confirms draw-then-discard ordering.
- **R7 (Mill 0 / Draw 0 / Discard with empty hand)**: M5 + O3 + M2 cover the 0-power and overdraw cases; no panics observed.
- **R8 (Replay harness wire-format compatibility)**: replay_harness.rs constructs new struct form directly via `Sacrifice { ids, lki_powers: vec![] }`; engine fills lki_powers at cost-payment time per the design.
- **R9 (Yield 23%)**: Acknowledged narrow; 3/3 confirmed cards correctly authored.
- **R10 (BASELINE-CLIPPY-01..06)**: Per runner checklist line 208, no new clippy warnings introduced.

## Summary

PB-P is a clean implementation of a structurally-correct LKI primitive. The dispatch verdict and capture-point decision both pass independent verification. All 8 mandatory tests are present and discriminating; M4 is genuinely load-bearing. The 3 card defs match MCP-verified oracle text exactly. Hash sentinel plumbing is consistent and strict. The 5 LOW findings are all out-of-scope or opportunistic (no merge blockers).

**Recommendation**: Merge.
