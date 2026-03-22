# Card Review: Removal/Destroy Batch 3

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 2 HIGH, 1 MEDIUM, 1 LOW

## Card 1: Culling Ritual
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): Missing second half of effect — "Add {B} or {G} for each permanent destroyed this way." The TODO documents this as a DSL gap (per-destroyed-count mana generation with color choice). This appears to be a genuine gap — `EffectAmount::LastEffectCount` exists but per-instance B-or-G player choice does not. The TODO is valid. However, under W5 policy, the card currently destroys permanents but does NOT generate mana, which is a partial implementation producing wrong game state (free wrath without the mana payoff benefits the caster less than it should, but the destroy effect alone is the more impactful half). This is borderline W5 — the missing mana is upside the caster loses, not a penalty they avoid. Flagging as MEDIUM rather than HIGH since the missing part benefits the caster (not a "pain land giving free mana" situation).

## Card 2: Rapid Hybridization
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): Token goes to wrong player. Oracle says "That creature's controller creates a 3/3 green Frog Lizard creature token." The DSL `CreateToken` always creates the token under `ctx.controller` (the spell's caster). In multiplayer, if you target an opponent's creature, your opponent should get the 3/3 token, not you. The current implementation gives the token to the caster. This is a W5 wrong-game-state violation — the card is a removal spell that incorrectly rewards the caster with a 3/3 instead of giving the replacement token to the victim. Should be `abilities: vec![]` with TODO until `CreateToken` supports a player target field (e.g., `player: PlayerTarget::TargetController { index: 0 }`).
  - F2 (HIGH): Missing `cant_be_regenerated` on DestroyPermanent. Oracle says "It can't be regenerated." The `DestroyPermanent` effect does not have a `cant_be_regenerated` field (only `DestroyAll` has it). If the engine honors regeneration shields on targeted destroy, this is functionally wrong. If `DestroyPermanent` ignores regeneration entirely, document with a TODO. Either way, the oracle text explicitly states "It can't be regenerated" and the def does not express this.

## Card 3: Broken Bond
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (LOW): Missing second effect documented with valid TODO. The TODO correctly identifies that "put a land card from your hand onto the battlefield" requires a primitive that does not exist. The destroy-target-artifact-or-enchantment half works correctly. Since the missing effect is optional ("You may") and benefits the caster, this is not a W5 violation — the card functions as a worse version of itself (Naturalize), not a card that produces wrong game state. The `has_card_types` filter on `TargetPermanentWithFilter` for "artifact or enchantment" looks correct (OR semantics for type matching).

## Card 4: Golgari Charm
- **Oracle match**: YES (minor formatting: oracle uses bullet points with newlines, def matches)
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: PARTIAL
- **Findings**:
  - F4 (MEDIUM): Mode 2 (Regenerate each creature you control) is a no-op `Effect::Sequence(vec![])`. The TODO claims `RegenerateAll` does not exist. This appears to be a genuine DSL gap — no bulk regenerate effect exists. However, W5 policy: the card is modal, and the player can choose modes 0 or 1 which work correctly. Mode 2 being non-functional means a player who specifically needs regeneration gets nothing. This is a partial implementation that produces wrong game state for mode 2 specifically. Flagging as MEDIUM because the card is castable and 2 of 3 modes work; mode 2 is a shield effect, not a destructive one — choosing it and getting nothing is bad but not game-breaking in the same way as a pain land giving free mana.

## Card 5: Casualties of War
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - No issues found. Modal structure is correct: min_modes=1, max_modes=5, each mode targets the right permanent type and uses the correct DeclaredTarget index. The five targets are declared up-front with appropriate TargetRequirement variants (TargetArtifact, TargetCreature, TargetEnchantment, TargetLand, TargetPlaneswalker). The comment about per-mode target lists is an accurate observation about the DSL's current approach.

## Summary
- Cards with issues: Culling Ritual (1 MEDIUM), Rapid Hybridization (2 HIGH), Golgari Charm (1 MEDIUM), Broken Bond (1 LOW)
- Clean cards: Casualties of War

### Issue Index

| ID | Card | Severity | Pattern | Description |
|----|------|----------|---------|-------------|
| F1 | Culling Ritual | MEDIUM | KI-2 (borderline) | Missing mana generation (valid TODO, caster loses upside) |
| F2 | Rapid Hybridization | HIGH | KI-2 / KI-11 | Token created for caster instead of target's controller — W5 violation, should be `vec![]` |
| F3 | Rapid Hybridization | HIGH | — | "Can't be regenerated" not expressible on DestroyPermanent — missing from def |
| F4 | Broken Bond | LOW | KI-19 | Valid TODO for missing PutLandFromHand primitive |
| F5 | Golgari Charm | MEDIUM | KI-2 | Mode 2 (Regenerate all your creatures) is no-op — valid TODO for missing RegenerateAll |
