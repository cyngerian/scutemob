# Card Review: Phase 1 Batch 05 — ETB Tapped Lands

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Sunken Hollow
- **Oracle match**: YES
- **Types match**: YES — `Land -- Island Swamp` subtypes correctly present
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you control two or more basic lands") implemented as unconditional EntersTapped with TODO documenting the DSL gap. Accepted per review instructions. Mana: U=pos1, B=pos2 correct.

## Card 2: Sulfur Falls
- **Oracle match**: YES
- **Types match**: YES — `Land` with no subtypes (correct, no land subtypes in oracle)
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you control an Island or a Mountain") implemented as unconditional EntersTapped with TODO documenting the DSL gap. Accepted per review instructions. Mana: U=pos1, R=pos3 correct.

## Card 3: Opulent Palace
- **Oracle match**: YES
- **Types match**: YES — `Land` with no subtypes (correct)
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Unconditional EntersTapped is fully correct here (oracle says "This land enters tapped." with no condition). Tri-land with 3 choices: B=pos2, G=pos4, U=pos1 all correct.

## Card 4: Training Center
- **Oracle match**: YES
- **Types match**: YES — `Land` with no subtypes (correct)
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: YES
- **Findings**: None
- **Notes**: Conditional ETB ("unless you have two or more opponents") implemented as unconditional EntersTapped with TODO documenting the DSL gap. Accepted per review instructions. Mana: U=pos1, R=pos3 correct. Note: in Commander (4-player), this land almost always enters untapped — the unconditional tapped is a significant gameplay deviation but acceptable given the DSL gap.

## Card 5: Necroblossom Snarl
- **Oracle match**: YES
- **Types match**: YES — `Land` with no subtypes (correct)
- **Mana cost match**: YES (no mana cost, land)
- **DSL correctness**: NO
- **Findings**:
  - F1 (MEDIUM): Missing TODO for conditional ETB. The comment says `// Enters tapped (CR 614.1c)` as if this is an unconditionally tapped land, but the oracle text is "As this land enters, you may reveal a Swamp or Forest card from your hand. If you don't, this land enters tapped." This is a conditional ETB (reveal-or-tapped) that should have a TODO comment matching the pattern used by the other conditional lands in this batch: `// TODO: Conditional ETB — may reveal a Swamp or Forest card, enters tapped if you don't / DSL gap: ReplacementModification::EntersTapped has no condition field`. The current comment is misleading because it suggests the land always enters tapped, which is incorrect. (Line 12, KI-9)

## Summary
- Cards with issues: Necroblossom Snarl (1 MEDIUM — misleading comment on conditional ETB, missing TODO)
- Clean cards: Sunken Hollow, Sulfur Falls, Opulent Palace, Training Center
