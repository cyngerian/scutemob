# Card Review: F-4 Session 1 Batch B

**Reviewed**: 2026-03-22
**Cards**: 6
**Findings**: 1 HIGH, 3 MEDIUM, 1 LOW

---

## Card 1: Strip Mine
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**: None. Clean implementation. Colorless mana tap + sacrifice-to-destroy-target-land are both correct. `Cost::Sequence([Tap, SacrificeSelf])` and `TargetRequirement::TargetLand` match oracle exactly ("target land" with no further restriction).

## Card 2: Wasteland
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO (overbroad target)
- **Findings**:
  - F1 (MEDIUM): Target is `TargetLand` but oracle says "target nonbasic land." The TODO on line 30-31 correctly documents this as a DSL gap -- `TargetFilter` has `basic: bool` (must BE basic) but no `non_basic` exclusion field. The TODO is **valid** (not stale). The current implementation allows targeting basic lands, which is overbroad. This is a real DSL gap, not a KI-3 stale TODO. The approximation produces wrong game state (can destroy basic lands), but the TODO is properly documented and the alternative (empty `vec![]`) would make the card completely non-functional, which is arguably worse for a utility land. Acceptable as documented approximation.

## Card 3: Deserted Temple
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: YES
- **Findings**: None. Clean implementation. `Cost::Sequence([Mana({1}), Tap])` correctly encodes `{1}, {T}`. `Effect::UntapPermanent` with `TargetRequirement::TargetLand` matches "Untap target land."

## Card 4: Halimar Depths
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: NO (wrong approximation)
- **Findings**:
  - F2 (HIGH / KI-2): ETB effect uses `Effect::Scry { count: 3 }` but oracle says "look at the top three cards of your library, then put them back in any order." This is NOT Scry. Scry 3 allows putting any of the 3 cards on the bottom of the library, while the actual ability only allows reordering the top 3 (all must stay on top). Using Scry gives the player strictly more power than the real card -- they can bottom cards that should remain on top. This produces **wrong game state** per W5 policy. The comment says "Approximated as Scry 3" but this approximation is gameplay-altering. Should either (a) use `vec![]` with a TODO noting the DSL lacks a "look and reorder top N" effect, or (b) accept as a known approximation with an explicit W5 waiver comment.
  - F3 (LOW): The comment "Approximated as Scry 3" acknowledges the gap but doesn't use the standard TODO format, making it invisible to automated TODO scanners.

## Card 5: Mortuary Mire
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES (none, land)
- **DSL correctness**: PARTIAL (missing "you may")
- **Findings**:
  - F4 (MEDIUM): Oracle says "you may put target creature card from your graveyard on top of your library." The "you may" means the controller can decline the effect even after it resolves with a valid target. The current implementation unconditionally moves the target card. Since the ability has a target (`TargetCardInYourGraveyard`), the controller can avoid the effect by not having a valid target, and the trigger won't go on the stack at all if there's no legal target. However, if there IS a creature card in the graveyard, the controller is forced to move it. In practice this is a minor issue (you rarely wouldn't want to use it), but it technically produces wrong game state. The ETB-tapped replacement and mana ability are correct. The `TargetFilter { has_card_type: Some(CardType::Creature) }` correctly restricts to creature cards.

## Card 6: Torch Courier
- **Oracle match**: YES
- **Types match**: YES (Creature -- Goblin)
- **Mana cost match**: YES ({R})
- **P/T match**: YES (1/1)
- **DSL correctness**: PARTIAL (target allows self)
- **Findings**:
  - F5 (MEDIUM): Oracle says "Another target creature gains haste until end of turn." The word "another" means Torch Courier cannot target itself. The implementation uses `TargetRequirement::TargetCreature` which would allow self-targeting. In practice, self-targeting is pointless (the creature is sacrificed as a cost before the ability resolves, so the ability would fizzle), but the DSL lacks an "exclude self" field on `TargetFilter`/`TargetRequirement` to properly encode "another." This is a legitimate DSL gap. Functional impact is low -- a player who self-targets just wastes the activation.

---

## Summary
- **Cards with issues**: Wasteland (F1 MEDIUM -- overbroad target, valid TODO), Halimar Depths (F2 HIGH -- Scry approximation is wrong game state; F3 LOW -- non-standard TODO), Mortuary Mire (F4 MEDIUM -- missing "you may"), Torch Courier (F5 MEDIUM -- "another" not enforced)
- **Clean cards**: Strip Mine, Deserted Temple

### DSL gaps identified (not stale -- genuinely missing):
1. `TargetFilter` lacks `non_basic: bool` for "nonbasic land" targeting (Wasteland)
2. No "look at top N and reorder" effect distinct from Scry (Halimar Depths)
3. No "may" wrapper on triggered ability effects (Mortuary Mire)
4. No "exclude self" on `TargetRequirement` for "another target creature" (Torch Courier)
