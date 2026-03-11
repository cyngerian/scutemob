# Card Review: Phase 1 Batch 01 — ETB Tapped Dual/Multiplayer Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 5 MEDIUM, 0 LOW

---

## Card 1: Woodland Cemetery
- **File**: `crates/engine/src/cards/defs/woodland_cemetery.rs`
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **Mana pool args**: YES — B=pos2, G=pos4
- **P/T**: N/A
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM/KI-9): ETB replacement is unconditional — always enters tapped. Oracle says "unless you control a Swamp or a Forest." The `ReplacementModification::EntersTapped` has no condition field to check for controlled land subtypes. No TODO comment documents this DSL gap. The card will always enter tapped even when the controller has a Swamp or Forest.

## Card 2: Undergrowth Stadium
- **File**: `crates/engine/src/cards/defs/undergrowth_stadium.rs`
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **Mana pool args**: YES — B=pos2, G=pos4
- **P/T**: N/A
- **DSL correctness**: NO
- **Findings**:
  - F2 (MEDIUM/KI-9): ETB replacement is unconditional — always enters tapped. Oracle says "unless you have two or more opponents." No condition on the replacement and no TODO comment. In Commander (the target format), this land should almost always enter untapped. The missing condition makes this card strictly worse than intended.

## Card 3: Vault of Champions
- **File**: `crates/engine/src/cards/defs/vault_of_champions.rs`
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **Mana pool args**: YES — W=pos0, B=pos2
- **P/T**: N/A
- **DSL correctness**: NO
- **Findings**:
  - F3 (MEDIUM/KI-9): Same unconditional ETB tapped issue as Undergrowth Stadium. Oracle says "unless you have two or more opponents." No TODO comment.

## Card 4: Luxury Suite
- **File**: `crates/engine/src/cards/defs/luxury_suite.rs`
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **Mana pool args**: YES — B=pos2, R=pos3
- **P/T**: N/A
- **DSL correctness**: NO
- **Findings**:
  - F4 (MEDIUM/KI-9): Same unconditional ETB tapped issue as Undergrowth Stadium. Oracle says "unless you have two or more opponents." No TODO comment.

## Card 5: Dragonskull Summit
- **File**: `crates/engine/src/cards/defs/dragonskull_summit.rs`
- **Oracle match**: YES
- **Types match**: YES (Land, no subtypes)
- **Mana cost match**: YES (none)
- **Mana pool args**: YES — B=pos2, R=pos3
- **P/T**: N/A
- **DSL correctness**: NO
- **Findings**:
  - F5 (MEDIUM/KI-9): Same unconditional ETB tapped issue as Woodland Cemetery. Oracle says "unless you control a Swamp or a Mountain." No TODO comment.

---

## Summary

- **Cards with issues**: Woodland Cemetery, Undergrowth Stadium, Vault of Champions, Luxury Suite, Dragonskull Summit (all 5)
- **Clean cards**: (none)

### Systematic Issue

All 5 cards share the same defect: the `EntersTapped` replacement effect is applied unconditionally, ignoring the "unless" condition from oracle text. There are two distinct condition types across these cards:

1. **Check-a-land-subtype** (Woodland Cemetery, Dragonskull Summit): "unless you control a Swamp or a [Forest/Mountain]"
2. **Opponent-count** (Undergrowth Stadium, Vault of Champions, Luxury Suite): "unless you have two or more opponents"

Neither condition is expressible in the current DSL. The `ReplacementModification::EntersTapped` variant has no associated condition/predicate field.

**Recommendation**: Each card should have a TODO comment on the replacement ability documenting the missing condition, e.g.:
```rust
// TODO: DSL gap — EntersTapped has no condition field.
// Should only enter tapped if controller does NOT control a Swamp or Forest.
```

This is a W5 policy concern: per the W5 "no simplifications" rule, an unconditional enters-tapped replacement is an incorrect simplification that changes gameplay. However, since the mana ability portion is correct and the ETB-tapped condition is a widespread DSL gap affecting many lands, these cards are reasonable to keep with proper TODO documentation rather than leaving abilities empty.

### Mana Color Verification (all correct)

| Card | Colors | pos0(W) | pos1(U) | pos2(B) | pos3(R) | pos4(G) | pos5(C) |
|------|--------|---------|---------|---------|---------|---------|---------|
| Woodland Cemetery | B/G | 0 | 0 | 1 | 0 | 0 | 0 / 0 | 0 | 0 | 0 | 1 | 0 |
| Undergrowth Stadium | B/G | 0 | 0 | 1 | 0 | 0 | 0 / 0 | 0 | 0 | 0 | 1 | 0 |
| Vault of Champions | W/B | 1 | 0 | 0 | 0 | 0 | 0 / 0 | 0 | 1 | 0 | 0 | 0 |
| Luxury Suite | B/R | 0 | 0 | 1 | 0 | 0 | 0 / 0 | 0 | 0 | 1 | 0 | 0 |
| Dragonskull Summit | B/R | 0 | 0 | 1 | 0 | 0 | 0 / 0 | 0 | 0 | 1 | 0 | 0 |
