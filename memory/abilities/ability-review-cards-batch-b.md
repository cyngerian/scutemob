# Card Review: Batch B

**Date**: 2026-03-10
**Reviewer**: ability-impl-reviewer (Opus)
**Files reviewed**: 5 card definitions

## Card 1: Crop Rotation

**File**: `crates/engine/src/cards/defs/crop_rotation.rs`

- **Oracle match**: YES -- mana cost ({G}), type (Instant), oracle text all correct.
- **DSL correctness**: NO -- two issues found.
- **Findings**:

| # | Severity | Description |
|---|----------|-------------|
| F1 | **MEDIUM** | **Search effect is expressible but not used.** The TODO claims both "sacrifice a land" and "search for a land" are DSL gaps, but `Effect::SearchLibrary` exists with `TargetFilter { has_card_type: Some(CardType::Land) }` and `destination: ZoneTarget::Battlefield`. Only the sacrifice-a-land additional cost is a genuine DSL gap. The search+put-onto-battlefield effect should be implemented. |
| F2 | **MEDIUM** | **Placeholder no-op effect.** The spell effect is `Effect::GainLife { amount: Fixed(0) }` which is a meaningless no-op. Even with the additional cost gap, the search portion should be the spell's effect. If the card must remain incomplete, the TODO should explicitly state the placeholder is temporary. **Fix:** Replace the GainLife(0) placeholder with `Effect::SearchLibrary { player: PlayerTarget::Controller, filter: TargetFilter { has_card_type: Some(CardType::Land), ..Default::default() }, reveal: false, destination: ZoneTarget::Battlefield }`. Update the TODO to note only the sacrifice-a-land additional cost is the remaining gap. |

## Card 2: Deadly Dispute

**File**: `crates/engine/src/cards/defs/deadly_dispute.rs`

- **Oracle match**: YES -- mana cost ({1}{B}), type (Instant), oracle text all correct.
- **DSL correctness**: PARTIAL -- the draw + treasure effects are correct; sacrifice additional cost is a documented DSL gap.
- **Findings**:

| # | Severity | Description |
|---|----------|-------------|
| F3 | LOW | **Missing additional cost enforcement.** The sacrifice-an-artifact-or-creature additional cost is not enforced. This is correctly documented as a TODO/DSL gap. The draw and treasure creation effects are properly sequenced and use correct DSL constructs (`treasure_token_spec(1)`, `EffectAmount::Fixed(2)`). No action needed until the DSL supports typed sacrifice costs. |

## Card 3: Fyndhorn Elves

**File**: `crates/engine/src/cards/defs/fyndhorn_elves.rs`

- **Oracle match**: YES -- mana cost ({G}), type (Creature -- Elf Druid), P/T (1/1), oracle text ("{T}: Add {G}.") all correct.
- **DSL correctness**: YES -- identical structure to Llanowar Elves and Elvish Mystic. `Cost::Tap` + `Effect::AddMana` with `mana_pool(0,0,0,0,1,0)` (1 green) is correct. No timing_restriction is correct for mana abilities.
- **Findings**: None.

## Card 4: Isolated Chapel

**File**: `crates/engine/src/cards/defs/isolated_chapel.rs`

- **Oracle match**: YES -- no mana cost (land), type (Land), oracle text correct.
- **DSL correctness**: PARTIAL -- mana ability is correct; ETB condition is a documented DSL gap with a safe conservative fallback.
- **Findings**:

| # | Severity | Description |
|---|----------|-------------|
| F4 | LOW | **Always-tapped conservative fallback.** The conditional "unless you control a Plains or a Swamp" check is not implemented -- the land always enters tapped. This is documented as a TODO/DSL gap. The conservative fallback (always tapped) is safe -- it never produces an illegal game state, only a suboptimal one. The mana ability uses `Effect::Choose` with two `Effect::AddMana` variants for {W}/{B}, which is the correct DSL pattern. No action needed until the DSL supports subtype-based ETB conditions. |
| F5 | LOW | **Missing subtypes.** Isolated Chapel has no land subtypes (it is not a Plains or Swamp), so the lack of subtypes in the TypeLine is correct. Noting for completeness. |

## Card 5: Putrefy

**File**: `crates/engine/src/cards/defs/putrefy.rs`

- **Oracle match**: YES -- mana cost ({1}{B}{G}), type (Instant), oracle text correct.
- **DSL correctness**: NO -- target restriction is too narrow.
- **Findings**:

| # | Severity | Description |
|---|----------|-------------|
| F6 | **MEDIUM** | **Target allows only creatures, not artifacts.** Putrefy says "Destroy target artifact or creature" but the definition uses `TargetRequirement::TargetCreature`. There is no `TargetArtifactOrCreature` variant (documented DSL gap). However, the current target restriction is actively wrong -- it prevents the spell from targeting artifacts at all, which contradicts the oracle text. **Fix:** Use `TargetRequirement::TargetPermanent` as a broader fallback (matches the pattern used by Krosan Grip and Gemrazer for similar gaps), and update the TODO comment to note this is overly permissive but safer than being overly restrictive. Alternatively, `TargetPermanentWithFilter` could work but there is no OR-type filter for card types. |
| F7 | LOW | **"Can't be regenerated" clause not modeled.** Documented as a TODO. Regeneration is a deprecated mechanic and rarely relevant. No action needed. |

## Summary

| Card | Status | Issues |
|------|--------|--------|
| Crop Rotation | Needs fix | F1 (MEDIUM), F2 (MEDIUM) |
| Deadly Dispute | Acceptable | F3 (LOW, documented gap) |
| Fyndhorn Elves | Clean | None |
| Isolated Chapel | Acceptable | F4 (LOW, documented gap), F5 (LOW, non-issue) |
| Putrefy | Needs fix | F6 (MEDIUM), F7 (LOW, documented gap) |

**Overall**: 3 MEDIUM findings across 2 cards require fixes. Crop Rotation has an expressible effect that was not implemented (SearchLibrary exists in the DSL). Putrefy uses an overly restrictive target that contradicts oracle text.
