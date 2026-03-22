# Primitive Batch Review: PB-22 S5 -- Copy/Clone Primitives

**Date**: 2026-03-21
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 707.2, 707.3, 508.4, 111.10, 700.2 (Delirium card types)
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/rules/events.rs`, `crates/engine/src/state/hash.rs`
**Card defs reviewed**: 4 (Thespian's Stage, Shifting Woodland, Thousand-Faced Shadow, Scion of the Ur-Dragon)

## Verdict: needs-fix

Two MEDIUM findings (Shifting Woodland missing permanent-card filter, Thousand-Faced Shadow
missing "another" in target filter) and three LOW findings. No HIGH findings. Engine
primitives are correct and well-tested. Card defs have documented TODOs that are honest about
DSL gaps.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:3797` | **CreateTokenCopy attack target defaults to ctx.source attacker.** CR 508.4 says controller chooses the defending player. Current code inherits from `ctx.source`'s attack target, which is correct for the primary use case (Ninjutsu) but not general. **Fix:** Add a comment noting this is a simplification; full interactive choice deferred to M10 Command infrastructure. No code change needed now. |
| 2 | LOW | `effects/mod.rs:3813-3817` | **CreateTokenCopy populates base_chars from source.** The token is created with `base_chars` from the source, then a CopyOf continuous effect is layered on top. This is redundant -- the CopyOf CE will overwrite characteristics at Layer 1 anyway. The base_chars pre-population is harmless but unnecessary. **Fix:** Consider using `Characteristics::default()` instead, or add a comment explaining it's a defensive fallback. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `shifting_woodland.rs:53-55` | **Target filter too permissive.** Oracle says "target permanent card in your graveyard" but `TargetFilter::default()` allows targeting instants and sorceries. The TargetFilter struct lacks `is_permanent: bool` or equivalent. **Fix:** Add `non_creature: false` doesn't help here. Either (a) add a `permanent_only: bool` field to TargetFilter and set it true, or (b) document the gap as a TODO with "permanent card filter not expressible in current TargetFilter DSL." |
| 4 | MEDIUM | `thousand_faced_shadow.rs:40-43` | **Target filter missing "another" and "attacking" constraints.** Oracle says "another target attacking creature" but the filter is `TargetPermanentWithFilter(TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() })` -- it allows targeting itself and non-attacking creatures. The TargetFilter DSL lacks `is_attacking` and `is_another` fields. **Fix:** Document the gap as a TODO: "TargetFilter lacks is_attacking and exclude_source constraints; currently allows illegal targeting of self and non-attacking creatures." |
| 5 | LOW | `scion_of_the_ur_dragon.rs:42-44` | **Search filter missing "permanent card" constraint.** Oracle says "Dragon permanent card" but the filter only checks for Dragon subtype. In practice this is harmless since all Dragon cards in MTG are permanents (creatures/artifacts), but it's technically imprecise. **Fix:** No code change needed; the existing TODO (line 6-8) correctly documents the real gap (BecomeCopyOf after search). |

### Finding Details

#### Finding 3: Shifting Woodland target filter too permissive

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/shifting_woodland.rs:53-55`
**Oracle**: "target permanent card in your graveyard"
**Issue**: The `TargetCardInYourGraveyard(TargetFilter::default())` matches any card in the graveyard, including instants and sorceries. Oracle text specifies "permanent card" which excludes instants, sorceries, and conspiracies. This allows an illegal target (e.g., targeting a Lightning Bolt in the graveyard, which would make the land a copy of an instant -- nonsensical).
**Fix**: Add a TODO comment documenting this gap: `// TODO: TargetFilter lacks "permanent card" constraint. Currently allows instants/sorceries as targets. Needs TargetFilter.permanent_only or similar.` Alternatively, if a `permanent_only` field can be added to TargetFilter cheaply, add it and set it to true.

#### Finding 4: Thousand-Faced Shadow target missing constraints

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/thousand_faced_shadow.rs:40-43`
**Oracle**: "another target attacking creature"
**Issue**: Two constraints are missing: (1) "another" -- the target cannot be the source creature itself; (2) "attacking" -- the target must be an attacking creature. The TargetFilter only checks `has_card_type: Some(CardType::Creature)`. If the player targets a non-attacking creature or the Shadow itself, the game state would be wrong. The existing TODO (line 6-9) documents the trigger condition gap but not the target constraint gap.
**Fix**: Add a TODO comment on the target requirement: `// TODO: TargetFilter lacks is_attacking and exclude_source constraints. Oracle says "another target attacking creature" but filter allows self-targeting and non-attacking creatures.`

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 707.2 (copiable values) | Yes | Yes | test_effect_become_copy_of, test_copy_does_not_copy_counters_or_status |
| 707.3 (clone chain) | Yes (pre-existing) | Yes | test_clone_copies_clone_chain |
| 508.4 (entering tapped+attacking) | Yes | Yes | test_effect_create_token_copy_tapped_attacking |
| 111.10 (token creation) | Yes | Yes | test_effect_create_token_copy |
| 700.2 (Delirium card types) | Yes | Yes | test_delirium_condition_evaluation |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Thespian's Stage | Partial | 1 | Mostly | TODO: "except it has this ability" not expressible (retained ability after copy). Documented. |
| Shifting Woodland | Partial | 0 | Mostly | Missing "permanent card" target filter (Finding 3). Missing TODO. |
| Thousand-Faced Shadow | Partial | 1 | Mostly | ETB trigger condition gap documented. Missing target constraint TODOs (Finding 4). |
| Scion of the Ur-Dragon | Partial | 1 | Partial | Search works, copy needs LastSearchResult. Well-documented TODO. |

## Test Summary

5 new tests added in `copy_effects.rs`, all well-structured:
- `test_effect_become_copy_of` -- BecomeCopyOf via execute_effect, verifies event + layer resolution
- `test_effect_become_copy_reverts_at_eot` -- UntilEndOfTurn copy reverts after cleanup
- `test_effect_create_token_copy` -- token creation with copied characteristics
- `test_effect_create_token_copy_tapped_attacking` -- token enters tapped+attacking in combat
- `test_delirium_condition_evaluation` -- CardTypesInGraveyardAtLeast true/false boundary checks

Missing test coverage:
- No test for BecomeCopyOf when copier is not on the battlefield (early return path)
- No test for CreateTokenCopy when source is gone (early return path)
- No test for CreateTokenCopy with token doubling (replacement effect path)

## Hash Discriminant Verification

All discriminants are unique within their respective enum namespaces:
- **Effect**: 64 (BecomeCopyOf), 65 (CreateTokenCopy) -- no collision with 31 (SacrificePermanents) or others
- **GameEvent**: 123 (BecameCopyOf) -- no collision (123 in KeywordAbility is Bloodthirst, different enum)
- **Condition**: 31 (CardTypesInGraveyardAtLeast) -- sequential after 30 (IsFirstCombatPhase), no collision
