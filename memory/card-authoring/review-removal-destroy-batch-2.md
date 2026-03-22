# Card Review: Removal/Destroy Batch 2

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 0 LOW

## Card 1: Infernal Grasp
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. Sequence of DestroyPermanent + LoseLife correctly models the oracle text. TargetCreature is correct. PlayerTarget::Controller is correct here (caster loses life, not the creature's controller).

## Card 2: Mortify
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. Uses `TargetPermanentWithFilter` with `has_card_types: vec![Creature, Enchantment]` (OR semantics) which correctly targets "creature or enchantment".

## Card 3: Abrupt Decay
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. `cant_be_countered: true` correctly models the first line. `non_land: true` + `max_cmc: Some(3)` correctly models the target restriction.

## Card 4: Pongify
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH / KI-3 + KI-2): TODO claims `PlayerTarget::TargetController` is needed but `PlayerTarget::ControllerOf(Box<EffectTarget>)` already exists in the DSL. The CreateToken currently uses an implicit `PlayerTarget::Controller` (the caster), which is wrong in multiplayer -- when destroying an opponent's creature, the token should go to that opponent, not the caster. The card should use `player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` on the CreateToken effect (or wrap the token creation to target the correct player). This is both a stale TODO (KI-3) and a wrong-game-state issue (KI-2): the caster gets a free 3/3 instead of the destroyed creature's controller getting it.
  - F2 (MEDIUM): "It can't be regenerated" is not modeled. Regeneration prevention is a niche effect in Commander (regeneration is rarely relevant), but the oracle text explicitly states it. Should have a TODO noting this gap if the DSL lacks a regeneration-prevention flag on DestroyPermanent.

## Card 5: Cankerbloom
- **Oracle match**: YES
- **Types match**: YES (Creature -- Phyrexian Fungus, P/T 3/2)
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: None. Clean card. The modal activated ability with sacrifice cost is well-documented. The pre-declaration of all targets (even for Proliferate mode) is noted as a known DSL limitation and matches the Abzan Charm pattern. The `Choose` effect correctly selects one mode.

## Summary
- Cards with issues: Pongify (1 HIGH, 1 MEDIUM)
- Clean cards: Infernal Grasp, Mortify, Abrupt Decay, Cankerbloom
