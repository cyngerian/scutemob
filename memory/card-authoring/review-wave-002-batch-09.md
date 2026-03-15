# Card Review: Wave 2, Batch 9

**Reviewed**: 2026-03-13
**Cards**: 5
**Findings**: 0 HIGH, 1 MEDIUM, 3 LOW

## Card 1: Mossborn Hydra
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - F1 (LOW): "This creature enters with a +1/+1 counter on it" is modeled as a WhenEntersBattlefield triggered ability, but the oracle text describes a replacement effect (it enters *with* the counter, not "when it enters, put a counter on it"). This is a known engine-wide simplification and is consistent with other cards (e.g., Golgari Grave-Troll, Simic Initiate via Graft). No action needed unless the engine adds replacement-based ETB counters.
  - F2 (LOW): Landfall trigger omitted with TODO. The TODO accurately describes the DSL gap (no Landfall TriggerCondition, no counter-doubling effect). Correct per W5 policy.

## Card 2: Grateful Apparition
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: MINOR ISSUE
- **Findings**:
  - F3 (MEDIUM): Oracle text says "deals combat damage to a player **or planeswalker**" but the trigger uses `WhenDealsCombatDamageToPlayer`, which does not cover planeswalker damage. This is a known DSL limitation (same pattern used by Thrummingbird, Bloated Contaminator). Should have a comment noting the planeswalker gap. However, since this is an engine-wide limitation and not a card-specific authoring error, marking as MEDIUM rather than HIGH.

## Card 3: Mothdust Changeling
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - No issues. Changeling keyword is implemented. The activated ability ("Tap an untapped creature you control: gains flying") is correctly omitted with an accurate TODO describing the DSL gap (no Cost variant for tapping another creature). Correct per W5 policy.

## Card 4: Niv-Mizzet, Visionary
- **Oracle match**: YES
- **Types match**: YES (Legendary supertype, Dragon Wizard subtypes)
- **Mana cost match**: YES ({4}{U}{R} = generic 4, blue 1, red 1)
- **DSL correctness**: YES
- **Findings**:
  - No issues. Both abilities (no maximum hand size, noncombat damage draw trigger) are correctly omitted with accurate TODOs describing the DSL gaps. Correct per W5 policy.

## Card 5: Serra Ascendant
- **Oracle match**: YES
- **Types match**: YES
- **Mana cost match**: YES
- **DSL correctness**: YES
- **Findings**:
  - F4 (LOW): The conditional static ability ("as long as you have 30 or more life, gets +5/+5 and has flying") is correctly omitted with an accurate TODO. The TODO correctly identifies the missing EffectDuration variant for life-total-conditional static effects. Correct per W5 policy.

## Summary
- Cards with issues: Grateful Apparition (1 MEDIUM -- planeswalker damage gap in trigger)
- Clean cards: Mossborn Hydra, Mothdust Changeling, Niv-Mizzet Visionary, Serra Ascendant
- Notes: All 5 cards have correct oracle text, mana costs, types, and P/T. All TODOs accurately describe DSL gaps. No KI-pattern violations found (no KI-1 through KI-10 matches). The batch is well-authored with appropriate use of W5 empty-abilities policy where abilities cannot be expressed.
