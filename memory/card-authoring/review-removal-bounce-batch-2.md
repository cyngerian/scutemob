# Card Review: Removal-Bounce Batch 2

**Reviewed**: 2026-03-22
**Cards**: 4
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

## Card 1: Snap

- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES ({1}{U} = generic 1, blue 1)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): TODO for "Untap up to two lands" is valid — no DSL primitive for untapping N target lands with "up to" choice. The bounce half is correctly implemented with `MoveZone` to `Hand { owner: OwnerOf }`. Card is partially implemented but the bounce-to-hand is the primary effect and is correct, so this is acceptable under current policy (the untap is a secondary effect that doesn't produce wrong game state — it just does less than oracle).

## Card 2: Mistblade Shinobi

- **Oracle match**: YES
- **Types match**: YES (Creature — Human Ninja)
- **Mana cost match**: YES ({2}{U} = generic 2, blue 1)
- **P/T match**: YES (1/1)
- **DSL correctness**: PARTIAL
- **Findings**:
  - F1 (MEDIUM): The combat-damage trigger is entirely missing (only a TODO comment). The card has Ninjutsu keyword + Ninjutsu cost ability (correct dual-def per KI-6), but the creature's main triggered ability ("Whenever this creature deals combat damage to a player, you may return target creature that player controls to its owner's hand") is absent. The TODO claims "that player controls" filter is not expressible — this is likely valid since `TargetRequirement` doesn't have a "controlled by damaged player" variant. However, this means the card enters the battlefield but never does anything beyond being a 1/1 body. Per W5 policy, leaving abilities as `vec![]` for unexpressible cards is acceptable, but here the Ninjutsu IS expressed while the triggered ability is not. The Ninjutsu portion alone is fine (paying {U} to sneak in a 1/1 is not "wrong game state" — it just does less). Acceptable but worth noting.
  - F2 (LOW): The TODO correctly identifies the DSL gap. "That player controls" targeting (where "that player" refers to the player dealt damage) requires a dynamic target filter not currently in the DSL.

## Card 3: Sigil of Sleep

- **Oracle match**: YES
- **Types match**: YES (Enchantment — Aura)
- **Mana cost match**: YES ({U} = blue 1)
- **DSL correctness**: YES (empty abilities beyond Enchant keyword — correct for unexpressible trigger)
- **Findings**:
  - F1 (LOW): TODO is valid — "whenever enchanted creature deals damage to a player" requires an attached-creature-damage trigger pattern plus "that player controls" dynamic targeting, neither of which exists in the DSL. The Enchant keyword is correctly present. No power/toughness (correct for non-creature).

## Card 4: Hullbreaker Horror

- **Oracle match**: YES
- **Types match**: YES (Creature — Kraken Horror)
- **Mana cost match**: YES ({5}{U}{U} = generic 5, blue 2)
- **P/T match**: YES (7/8)
- **DSL correctness**: YES (Flash keyword present; abilities empty with valid TODOs)
- **Findings**:
  - No issues. Both TODOs are valid:
    1. `cant_be_countered` for permanent spells is a confirmed DSL gap (only exists on `AbilityDefinition::Spell`, not on `CardDefinition` — same gap as Niv-Mizzet, Parun).
    2. Modal triggered ability with per-mode targets and "up to one" (zero-mode option) is not expressible in the current DSL.
  - W5 policy assessment: The card enters as a 7/8 Flash creature without its cant-be-countered or triggered bounce. A 7/8 Flash body with no abilities is a meaningful understatement but doesn't produce *wrong* game state (it does less, not something incorrect). Acceptable as-is.

## Summary

- **Cards with issues**: Mistblade Shinobi (1 MEDIUM — partial implementation with Ninjutsu but no trigger)
- **Clean cards**: Snap, Sigil of Sleep, Hullbreaker Horror
- **All TODOs are valid** — no stale gap claims (KI-3 clean)
- **No KI-5, KI-7, KI-8, KI-12, KI-13, KI-14 issues found**
- **No legal-but-wrong multiplayer issues** — all bounce effects correctly use `OwnerOf` for "its owner's hand" (where implemented)
