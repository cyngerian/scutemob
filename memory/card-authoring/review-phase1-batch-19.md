# Card Review: Phase 1 Batch 19 (Body-only MDFCs)

**Reviewed**: 2026-03-10
**Cards**: 5
**Findings**: 0 HIGH, 3 MEDIUM, 0 LOW

## Card 1: Cragcrown Pathway // Timbercrown Pathway
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None

Front face oracle text "{T}: Add {R}." is correct. Land type, no mana cost, `abilities: vec![]` consistent with other pathway cards in the codebase. No `back_face` -- consistent with established pathway pattern (see batch-wide note below).

## Card 2: Darkbore Pathway // Slitherbore Pathway
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (None for land)
- **DSL correctness**: YES
- **Findings**: None

Front face oracle text "{T}: Add {B}." is correct. Same pattern as Card 1.

## Card 3: Hydroelectric Specimen // Hydroelectric Laboratory
- **Oracle match**: YES
- **Types match**: YES (Creature -- Weird)
- **Mana cost match**: YES ({2}{U} = generic 2, blue 1)
- **P/T match**: YES (1/4)
- **DSL correctness**: YES
- **Findings**:
  - F1 (MEDIUM): Missing `back_face` for Hydroelectric Laboratory. Unlike pathway lands (both sides are simple tap-for-mana lands), this card's back face is a Land with a distinct ability. The front face is a creature; the back is a land -- this is a proper MDFC that should have `back_face: Some(CardFace { ... })` defined. Without it, the engine cannot know about the land side. Consistent with how `delver_of_secrets.rs` and other DFCs define their back faces.

## Card 4: Kabira Takedown // Kabira Plateau
- **Oracle match**: YES
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({1}{W} = generic 1, white 1)
- **DSL correctness**: YES
- **Findings**:
  - F2 (MEDIUM): Missing `back_face` for Kabira Plateau. The back face is a Land that enters tapped. Without `back_face`, the engine has no way to let a player play this as a land. Same structural issue as Card 3.

## Card 5: Marang River Regent // Coil and Catch
- **Oracle match**: YES
- **Types match**: YES (Creature -- Dragon)
- **Mana cost match**: YES ({4}{U}{U} = generic 4, blue 2)
- **P/T match**: YES (6/7)
- **DSL correctness**: YES
- **Findings**:
  - F3 (MEDIUM): Missing `back_face` for Coil and Catch. The back face is an Instant -- Omen with mana cost {3}{U}. This is a spell // spell MDFC where both sides are castable. Without `back_face`, only the front face creature is available. Same structural issue as Cards 3 and 4.

## Batch-Wide Note: Missing back_face on MDFCs

All 5 cards are MDFCs (Modal Double-Faced Cards) but none define a `back_face`. For the two pathway lands (Cards 1-2), this is consistent with all other pathway cards already in the codebase (brightclimb, blightstep, clearwater, etc.) -- both sides are simple "{T}: Add {C}." lands, and this appears to be an accepted simplification.

However, Cards 3-5 have meaningfully different back faces (land, land, instant respectively) and would benefit from `back_face` definitions. Without them, the engine treats these as single-faced cards. This is rated MEDIUM rather than HIGH because the front faces are all correct and castable, and MDFC modal casting infrastructure may not yet exist in the engine.

## Summary
- Cards with issues: Hydroelectric Specimen (F1), Kabira Takedown (F2), Marang River Regent (F3)
- Clean cards: Cragcrown Pathway, Darkbore Pathway
