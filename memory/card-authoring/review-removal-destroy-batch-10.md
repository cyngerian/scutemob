# Card Review: Removal/Destroy Batch 10

**Reviewed**: 2026-03-22
**Cards**: 5
**Findings**: 1 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Ravenous Chupacabra
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. ETB trigger correctly targets creature an opponent controls using `TargetCreatureWithFilter` with `controller: TargetController::Opponent`.

## Card 2: Aura Shards
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**: Clean. Trigger correctly uses `WheneverCreatureEntersBattlefield` with `controller: You` filter. Target filter uses `has_card_types: vec![CardType::Artifact, CardType::Enchantment]` for "artifact or enchantment" which matches OR semantics. Note: oracle says "you may destroy" (optional), but the DSL lacks a "may" modifier on the effect -- this is a minor DSL gap, not a card def error.

## Card 3: Argentum Armor
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: NO
- **Findings**:
  - F1 (HIGH): **W5 policy violation (KI-2)** -- The attack trigger ("Whenever equipped creature attacks, destroy target permanent") is left as a TODO, but the static +6/+6 buff and Equip ability are implemented. This means the card is castable and equippable, granting +6/+6 without its significant attack trigger ever firing. A creature wearing Argentum Armor gets free stats without the signature ability. The TODO comment claims `WhenEquippedCreatureAttacks` is a DSL gap, which is valid -- that trigger condition does not exist in the engine. However, per W5 policy, a partial implementation that produces wrong game state (free +6/+6 without the balancing attack trigger) should use `abilities: vec![]` to prevent the card from being played at all. As-is, the card is overpowered (free stats, no trigger).
  - F2 (LOW): Equip ability uses `targets: vec![]` -- this is consistent with the codebase convention for Equip (engine handles targeting internally), so not a bug.

## Card 4: Kogla, the Titan Ape
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype present)
- **Mana cost match**: YES
- **DSL correctness**: YES (with noted approximations)
- **Findings**:
  - F3 (MEDIUM): **"Up to one" target not modeled** -- Oracle says "it fights up to one target creature you don't control." The "up to one" means the controller can choose zero targets (optional targeting). The current implementation uses a mandatory `TargetRequirement::TargetCreatureWithFilter` which requires exactly one target. If no opponent creature exists on the battlefield, the trigger fizzles (correct), but if an opponent has creatures, the controller cannot decline to target (incorrect). The DSL may lack an `UpToOne` wrapper for target requirements.
  - F4 (LOW): "defending player controls" approximated as `TargetController::Opponent` on the attack trigger. This is documented in the code comment and is a valid DSL gap (no `TargetController::DefendingPlayer` variant). In most multiplayer games this is functionally correct since you typically attack an opponent, but technically in some edge cases (e.g., planeswalker attacks) the defending player might differ from who you'd choose as "opponent." Acceptable approximation with documentation.

## Card 5: Parapet Thrasher
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: N/A (abilities left as TODO)
- **Findings**: The card has Flying implemented and all other abilities left as TODO with `abilities: vec![]` (just the Flying keyword). The TODO claims three DSL gaps: (1) no subtype-filtered combat-damage-to-player trigger for "Dragons you control," (2) no "choose one that hasn't been chosen this turn" modal constraint, and (3) PlayExiledCard needs play-from-exile with end-of-turn expiry. Gap 1 is valid -- `WhenDealsCombatDamageToPlayer` is self-referential only. Gap 2 is valid -- mode tracking per turn is not in the DSL. Gap 3 is partially valid -- `PlayExiledCard` exists as an Effect variant but the "until end of turn" expiry tracking may not be fully wired. Overall the TODO is legitimate and the card correctly avoids partial implementation per W5 policy.

## Summary
- **Cards with issues**: Argentum Armor (1 HIGH -- W5 partial impl), Kogla (1 MEDIUM -- up-to-one targeting)
- **Clean cards**: Ravenous Chupacabra, Aura Shards, Parapet Thrasher
