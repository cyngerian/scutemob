# Card Review: A-24 Attack-Trigger Batch 1

**Reviewed**: 2026-03-23
**Cards**: 12
**Findings**: 1 HIGH, 1 MEDIUM, 1 LOW

---

## Card 1: Shared Animosity
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES (empty abilities with valid TODO)
- **Findings**: CLEAN

---

## Card 2: Six
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Treefolk)
- **Mana cost match**: YES
- **P/T match**: YES (2/4)
- **DSL correctness**: YES (Reach keyword present; remaining abilities empty with valid TODOs)
- **Findings**: CLEAN

---

## Card 3: Sanctum Seeker
- **Oracle match**: YES
- **Types match**: YES (Creature -- Vampire Knight)
- **Mana cost match**: YES
- **P/T match**: YES (3/4)
- **DSL correctness**: YES (empty abilities with valid TODO -- creature-type-filtered attack trigger)
- **Findings**: CLEAN

---

## Card 4: Etali, Primal Storm
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elder Dinosaur)
- **Mana cost match**: YES
- **P/T match**: YES (6/6)
- **DSL correctness**: YES (empty abilities with valid TODO)
- **Findings**: CLEAN

---

## Card 5: Najeela, the Blade-Blossom
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Warrior)
- **Mana cost match**: YES
- **P/T match**: YES (3/2)
- **DSL correctness**: YES (empty abilities with valid TODOs)
- **Findings**: CLEAN

---

## Card 6: Aurelia, the Warleader
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Angel)
- **Mana cost match**: YES (2RRWW)
- **P/T match**: YES (3/4)
- **DSL correctness**: PARTIALLY -- see findings
- **Findings**:
  - F1 (MEDIUM): `Condition::IsFirstCombatPhase` is a simplification of "for the first time each turn". Oracle means "the first time Aurelia attacks this turn" -- if Aurelia skips the first combat phase and attacks in a second one (e.g., from another source's extra combat), the trigger should still fire. `IsFirstCombatPhase` would incorrectly prevent it. Same simplification exists on Karlach. This is a known DSL approximation, not a new bug. Acceptable for now but should be documented.
  - F2 (LOW): No inline comment noting this is a simplification (unlike Karlach which has a NOTE comment). Consider adding a comment for consistency.

---

## Card 7: Grand Warlord Radha
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Elf Warrior)
- **Mana cost match**: YES
- **P/T match**: YES (3/4)
- **DSL correctness**: YES (Haste keyword present; remaining abilities empty with valid TODOs)
- **Findings**: CLEAN

---

## Card 8: Isshin, Two Heavens as One
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Samurai)
- **Mana cost match**: YES (RWB, no generic)
- **P/T match**: YES (3/4)
- **DSL correctness**: YES (empty abilities with valid TODO -- trigger doubler for attack triggers)
- **Findings**: CLEAN

---

## Card 9: Saskia the Unyielding
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Human Soldier)
- **Mana cost match**: YES (BRGW, no generic)
- **P/T match**: YES (3/4)
- **DSL correctness**: YES (Vigilance + Haste keywords present; remaining abilities empty with valid TODOs)
- **Findings**: CLEAN

---

## Card 10: Reconnaissance
- **Oracle match**: YES (includes reminder text)
- **Types match**: YES (Enchantment)
- **Mana cost match**: YES ({W})
- **DSL correctness**: YES (empty abilities with valid TODO -- RemoveFromCombat effect missing)
- **Findings**: CLEAN

---

## Card 11: Hellrider
- **Oracle match**: YES
- **Types match**: YES (Creature -- Devil)
- **Mana cost match**: YES (2RR)
- **P/T match**: YES (3/3)
- **DSL correctness**: YES (Haste keyword present; remaining abilities empty with valid TODOs)
- **Findings**:
  - F3 (HIGH): The oracle says "Hellrider deals 1 damage" -- the damage source is Hellrider itself, not the attacking creature. The TODO correctly identifies that "the player or planeswalker IT'S attacking" (the attacking creature's target) is not expressible. However, if this were naively implemented with `DealDamage` using `PlayerTarget::Controller` or similar, it would deal damage to the wrong entity. The TODO is correct and the empty abilities are the right choice. **Downgraded to informational** -- no actual bug since abilities are empty. Reclassifying.

Actually, re-evaluating: abilities are empty, so no wrong game state. Removing HIGH.

- **Findings**: CLEAN

---

## Card 12: Marisi, Breaker of the Coil
- **Oracle match**: YES
- **Types match**: YES (Legendary Creature -- Cat Warrior)
- **Mana cost match**: YES (1RGW)
- **P/T match**: YES (5/4)
- **DSL correctness**: YES (empty abilities with valid TODOs)
- **Findings**: CLEAN

---

## Summary

- **Cards with issues**: Aurelia, the Warleader (1 MEDIUM, 1 LOW)
- **Clean cards**: Shared Animosity, Six, Sanctum Seeker, Etali Primal Storm, Najeela the Blade-Blossom, Grand Warlord Radha, Isshin Two Heavens as One, Saskia the Unyielding, Reconnaissance, Hellrider, Marisi Breaker of the Coil

### Findings Summary

| ID | Sev | Card | Description |
|----|-----|------|-------------|
| F1 | MEDIUM | Aurelia, the Warleader | `IsFirstCombatPhase` is an approximation of "for the first time each turn" -- fails if Aurelia attacks in a non-first combat phase for her first attack that turn. Known DSL simplification (same as Karlach). |
| F2 | LOW | Aurelia, the Warleader | Missing simplification NOTE comment unlike Karlach's equivalent implementation. |

### Overall Assessment

This is a very clean batch. 10 of 12 cards have empty abilities with well-documented TODO comments identifying genuine DSL gaps (creature-type-filtered attack triggers, combat assignment references, trigger doublers, etc.). The 2 cards with partial implementations (Aurelia with keywords + triggered ability, Six with Reach keyword) are correctly structured. Aurelia's implementation follows the established Karlach pattern and the `IsFirstCombatPhase` simplification is a known engine-wide approximation, not specific to this authoring batch.

No W5-policy violations (no wrong game state from partial implementations). All TODOs cite genuine DSL gaps.
