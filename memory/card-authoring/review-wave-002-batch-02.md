# Card Review: Wave 2 Batch 2

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 2 LOW

---

## Card 1: Gingerbrute

- **Oracle match**: YES
- **Types match**: YES (Artifact Creature -- Food Golem)
- **Mana cost match**: YES ({1})
- **P/T match**: YES (1/1)
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): Haste keyword is implemented but the two activated abilities are left as TODO. The TODOs accurately describe DSL gaps (filtered evasion for the {1} ability, sacrifice-as-cost for the {2},{T},Sacrifice ability). Acceptable per W5 policy.

## Card 2: Akroma's Will

- **Oracle match**: YES (unicode bullet characters used correctly)
- **Types match**: YES (Instant)
- **Mana cost match**: YES ({3}{W})
- **DSL correctness**: YES
- **Findings**:
  - F2 (LOW): All abilities left as `vec![]` with TODO. The TODO accurately identifies the gaps: conditional modal choice based on commander control, mass keyword grants until end of turn, and protection from each color. These are genuine DSL gaps. Correct per W5 policy.

## Card 3: Thousand-Year Elixir

- **Oracle match**: YES
- **Types match**: YES (Artifact)
- **Mana cost match**: YES ({3})
- **DSL correctness**: PARTIAL
- **Findings**:
  - F3 (MEDIUM): The activated ability `{1}, {T}: Untap target creature` is implemented but lacks a target declaration. The ability uses `EffectTarget::DeclaredTarget { index: 0 }` but the card definition has no `targets` field on the `Activated` variant to declare what index 0 refers to. If the engine requires explicit target declarations for activated abilities (similar to how spells declare targets), this will silently fail to find a target at resolution. Verify whether `AbilityDefinition::Activated` has a `targets` field -- if so, it needs `targets: vec![TargetSpec::Creature]` or equivalent. If the engine infers targets from the effect, this is fine. Either way, the static "as though they had haste" ability is correctly marked as a TODO.

## Card 4: Basilisk Collar

- **Oracle match**: YES (includes reminder text)
- **Types match**: YES (Artifact -- Equipment)
- **Mana cost match**: YES ({1})
- **DSL correctness**: YES
- **Findings**: None. Clean implementation. Static ability correctly uses Layer 6 (Ability) with `AddKeywords` for Deathtouch and Lifelink, filtered to `AttachedCreature`. Equip {2} correctly uses `SorcerySpeed` timing restriction and `AttachEquipment` effect.

## Card 5: Destiny Spinner

- **Oracle match**: YES
- **Types match**: YES (Enchantment Creature -- Human)
- **Mana cost match**: YES ({1}{G})
- **P/T match**: YES (2/3)
- **DSL correctness**: YES
- **Findings**: None beyond the TODOs. Both abilities are left unimplemented with `vec![]` and accurate TODO comments. The "can't be countered" static for specific spell types and the land animation with dynamic X are genuine DSL gaps. Correct per W5 policy.

---

## Summary

- **Cards with issues**: Thousand-Year Elixir (1 MEDIUM -- possible missing target declaration on activated ability)
- **Clean cards**: Gingerbrute, Akroma's Will, Basilisk Collar, Destiny Spinner
- **Notes**: All 5 cards have accurate oracle text matching Scryfall. All TODOs accurately describe real DSL gaps. No instances of KI-1 through KI-10 patterns found. No overbroad triggers, no GainLife(0) placeholders, no incorrect PlayerTarget usage.
